use anyhow::Result;
use brane_log::ingestion;
use cassandra_cpp::Cluster;
use clap::Clap;
use dotenv::dotenv;
use log::LevelFilter;

#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
    /// Kafka brokers
    #[clap(short, long, default_value = "localhost:9092", env = "BROKERS")]
    brokers: String,
    /// Print debug info
    #[clap(short, long, env = "DEBUG", takes_value = false)]
    debug: bool,
    /// Topic to receive events from
    #[clap(short, long = "evt-topic", env = "EVENT_TOPIC")]
    event_topic: String,
    /// Consumer group id
    #[clap(short, long, default_value = "brane-log", env = "GROUP_ID")]
    group_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let opts = Opts::parse();

    // Configure logger.
    let mut logger = env_logger::builder();
    logger.format_module_path(false);

    if opts.debug {
        logger.filter_level(LevelFilter::Debug).init();
    } else {
        logger.filter_level(LevelFilter::Info).init();
    }

    // Configure Cassandra cluster connection.
    let mut cassanda_cluster = Cluster::default();
    cassanda_cluster.set_load_balance_round_robin();
    cassanda_cluster
        .set_contact_points("127.0.0.1")
        .map_err(|_| anyhow::anyhow!("Failed to append Cassandra contact point."))?;

    let cassandra_session = cassanda_cluster
        .connect_async()
        .await
        .map_err(|_| anyhow::anyhow!("Failed to create Cassandra session connection."))?;

    // Ensure Cassandra keyspace and table.
    ingestion::ensure_db_keyspace(&cassandra_session).await?;
    ingestion::ensure_db_tables(&cassandra_session).await?;

    // Spawn a single event ingestion worker.
    let handle = tokio::spawn(ingestion::start_worker(
        opts.brokers.clone(),
        opts.group_id.clone(),
        opts.event_topic.clone(),
        cassandra_session,
    ));

    handle.await?
}

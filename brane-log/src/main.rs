use anyhow::Result;
use futures::FutureExt;
use brane_log::ingestion;
use brane_log::{Context, Schema};
use brane_log::schema::{Query, Event, Subscription};
use cassandra_cpp::Cluster;
use clap::Clap;
use dotenv::dotenv;
use juniper::{self, EmptyMutation};
use juniper_graphql_ws::ConnectionConfig;
use juniper_warp::{playground_filter, subscriptions::serve_graphql_ws};
use log::LevelFilter;
use std::sync::Arc;
use tokio::sync::watch;
use warp::Filter;
use warp::ws::Ws;
use std::sync::RwLock;

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

    // Configure internal event watcher (used for subscriptions).
    let (events_tx, events_rx) = watch::channel(Event::default());

    // Configure Cassandra cluster connection.
    let mut cassanda_cluster = Cluster::default();
    cassanda_cluster.set_load_balance_round_robin();
    cassanda_cluster
        .set_contact_points("127.0.0.1")
        .map_err(|_| anyhow::anyhow!("Failed to append Cassandra contact point."))?;

    let cassandra_session = cassanda_cluster
        .connect_async()
        .await
        .map(RwLock::new)
        .map(Arc::new)
        .map_err(|_| anyhow::anyhow!("Failed to create Cassandra session connection."))?;

    // Ensure Cassandra keyspace and table.
    ingestion::ensure_db_keyspace(&cassandra_session).await?;
    ingestion::ensure_db_tables(&cassandra_session).await?;

    // Spawn a single event ingestion worker.
    tokio::spawn(ingestion::start_worker(
        opts.brokers.clone(),
        opts.group_id.clone(),
        opts.event_topic.clone(),
        events_tx,
        cassandra_session.clone(),
    ));

    let cassandra_arc = cassandra_session.clone();
    let events = events_rx.clone();

    // Configure Wrap web server.
    let log = warp::log("warp_server");
    let schema = Schema::new(Query {}, EmptyMutation::new(), Subscription {});

    let context_cassandra = cassandra_session.clone();
    let context_events_rx = events_rx.clone();
    let context = warp::any().map(move || Context {
        cassandra: context_cassandra.clone(),
        events_rx: context_events_rx.clone(),
    });

    let graphql_filter = juniper_warp::make_graphql_filter(schema, context.boxed());
    let root_node = Arc::new(Schema::new(Query {}, EmptyMutation::new(), Subscription {}));

    let routes = (warp::path("subscriptions")
        .and(warp::ws())
        .map(move |ws: Ws| {
            let root_node = root_node.clone();
            let events = events.clone();
            let cass = cassandra_arc.clone();

            ws.on_upgrade(move |websocket| async move {
                serve_graphql_ws(websocket, root_node, ConnectionConfig::new(
                    Context { cassandra: cass, events_rx: events }
                ))
                .map(|r| {
                    if let Err(e) = r {
                        println!("Websocket error: {}", e);
                    }
                })
                .await
            })
        }))
        .map(|reply| {
            // TODO#584: remove this workaround
            warp::reply::with_header(reply, "Sec-WebSocket-Protocol", "graphql-ws")
        })
        .or(warp::post()
            .and(warp::path("graphql"))
            .and(graphql_filter))
        .or(warp::get()
            .and(warp::path("playground"))
            .and(playground_filter("/graphql", Some("/subscriptions"))))
        .with(log);

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;

    Ok(())
}

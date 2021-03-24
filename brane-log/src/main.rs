use anyhow::Result;
use brane_log::{schema::Query, Context, Schema};
use clap::Clap;
use dotenv::dotenv;
use juniper::{self, EmptyMutation, EmptySubscription, Variables};
use log::LevelFilter;

#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
    /// Kafka brokers
    #[clap(short, long, default_value = "localhost:9092", env = "BROKERS")]
    _brokers: String,
    /// Print debug info
    #[clap(short, long, env = "DEBUG", takes_value = false)]
    debug: bool,
    /// Topic to receive events from
    #[clap(short, long = "evt-topic", env = "EVENT_TOPIC")]
    _event_topic: String,
    /// Consumer group id
    #[clap(short, long, default_value = "brane-log", env = "GROUP_ID")]
    _group_id: String,
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

    let schema = Schema::new(Query {}, EmptyMutation::new(), EmptySubscription::new());
    let variables = Variables::new();
    let context = Context {
        name: String::from("TEST!"),
    };

    let (res, _) = juniper::execute("query { events { identifier } }", None, &schema, &variables, &context)
        .await
        .unwrap();
    println!("{:?}", res);

    Ok(())
}

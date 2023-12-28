use clap::Parser;
use eyre::Result;
use lapin::{options::BasicPublishOptions, BasicProperties, ConnectionProperties};
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// AMQP address to access message queue
    #[arg(env = "BUILDIT_AMQP_ADDR")]
    amqp_addr: String,

    #[arg(short, long)]
    json: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let Args { amqp_addr, json } = Args::parse();

    // initialize tracing
    let env_log = EnvFilter::try_from_default_env();

    if let Ok(filter) = env_log {
        tracing_subscriber::registry()
            .with(fmt::layer().with_filter(filter))
            .init();
    } else {
        tracing_subscriber::registry().with(fmt::layer()).init();
    }

    let conn = lapin::Connection::connect(&amqp_addr, ConnectionProperties::default()).await?;

    let channel = conn.create_channel().await?;

    channel
        .basic_publish(
            "",
            "github-webhooks",
            BasicPublishOptions::default(),
            json.as_bytes(),
            BasicProperties::default(),
        )
        .await?
        .await?;

    info!("Sent json: {json}");

    Ok(())
}

use std::{fs, path::PathBuf};

use clap::Parser;
use eyre::Result;
use lapin::{
    options::{BasicPublishOptions, QueueDeclareOptions},
    types::{AMQPValue, FieldTable},
    BasicProperties, Channel, ConnectionProperties, Queue,
};
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// AMQP address to access message queue
    #[arg(env = "BUILDIT_AMQP_ADDR")]
    amqp_addr: String,

    #[arg(short, long)]
    json: PathBuf,
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

    let file = fs::read_to_string(json)?;

    ensure_job_queue("github-webhooks", &channel).await?;

    channel
        .basic_publish(
            "",
            "github-webhooks",
            BasicPublishOptions::default(),
            file.as_bytes(),
            BasicProperties::default(),
        )
        .await?
        .await?;

    info!("Sent json: {file}");

    Ok(())
}

pub async fn ensure_job_queue(queue_name: &str, channel: &Channel) -> Result<Queue> {
    let mut arguments = FieldTable::default();
    // extend consumer timeout because we may have long running tasks
    arguments.insert(
        "x-consumer-timeout".into(),
        AMQPValue::LongInt(24 * 3600 * 1000),
    );
    Ok(channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions {
                durable: true,
                ..QueueDeclareOptions::default()
            },
            arguments,
        )
        .await?)
}

use log::info;

mod chat_gpt;
mod storages;
mod telegram;
mod utils;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    pretty_env_logger::init();
    info!("Starting ChatGPT telegram bot...");

    telegram::startup().await?;
    Ok(())
}

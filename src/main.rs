use log::info;

mod chat_gpt;
mod telegram;
mod utils;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    info!("Starting ChatGPT telegram bot...");

    telegram::startup().await;
}

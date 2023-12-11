#[macro_use]
extern crate lazy_static;

mod core;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    tracing::info!("Starting...");

    core::bot::run().await.unwrap();
}

#[macro_use]
extern crate lazy_static;

mod core;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    log::info!("Starting...");

    core::bot::run().await.unwrap();
}

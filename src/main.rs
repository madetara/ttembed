#[macro_use]
extern crate lazy_static;

mod core;

#[tokio::main]
async fn main() {
    let file_appender = tracing_appender::rolling::daily("/workload/logs", "hekapoo.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt().with_writer(non_blocking).init();

    tracing::info!("Starting...");

    core::bot::run().await.unwrap();
}

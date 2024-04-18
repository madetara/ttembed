use tracing_appender::rolling::{RollingFileAppender, Rotation};

#[macro_use]
extern crate lazy_static;

mod core;

#[tokio::main]
async fn main() {
    let file_appender = RollingFileAppender::builder()
        .filename_prefix("hekapoo.log")
        .rotation(Rotation::DAILY)
        .max_log_files(3)
        .build("/workload/logs")
        .expect("failed to initialize file logger");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt().with_writer(non_blocking).init();

    tracing::info!("Starting...");

    core::bot::run().await.unwrap();
}

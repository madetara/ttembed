use std::time::Duration;

use opentelemetry::trace::TraceError;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::resource::{
    EnvResourceDetector, SdkProvidedResourceDetector, TelemetryResourceDetector,
};
use opentelemetry_sdk::trace::Config;
use opentelemetry_sdk::Resource;
use tonic::metadata::MetadataMap;
use tonic::transport::ClientTlsConfig;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

#[macro_use]
extern crate lazy_static;

mod core;

#[tokio::main]
async fn main() {
    let dsn = std::env::var("UPTRACE_DSN").expect("UPTRACE_DSN not set");
    let tracer_provider = init_tracer(dsn.as_str()).expect("failed to initialize tracer");
    let tracer = tracer_provider.tracer("ttembed");

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let subscriber = Registry::default()
        .with(telemetry)
        .with(fmt::Layer::default());

    tracing::subscriber::set_global_default(subscriber).unwrap();

    tracing::info!("starting");

    core::bot::run().await.unwrap();
}

fn init_tracer(dsn: &str) -> Result<opentelemetry_sdk::trace::TracerProvider, TraceError> {
    let resource = Resource::from_detectors(
        Duration::from_secs(0),
        vec![
            Box::new(SdkProvidedResourceDetector),
            Box::new(EnvResourceDetector::new()),
            Box::new(TelemetryResourceDetector),
        ],
    );

    let mut metadata = MetadataMap::with_capacity(1);
    metadata.insert("uptrace-dsn", dsn.parse().unwrap());

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_tls_config(ClientTlsConfig::new())
                .with_endpoint("https://otlp.uptrace.dev:4317")
                .with_timeout(Duration::from_secs(5))
                .with_metadata(metadata),
        )
        .with_batch_config(
            opentelemetry_sdk::trace::BatchConfigBuilder::default()
                .with_max_queue_size(30000)
                .with_max_export_batch_size(10000)
                .with_scheduled_delay(Duration::from_millis(5000))
                .build(),
        )
        .with_trace_config(Config::default().with_resource(resource))
        .install_batch(opentelemetry_sdk::runtime::Tokio)
}

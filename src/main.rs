use std::time::Duration;

use opentelemetry::{
    logs::LogError,
    trace::{TraceError, TracerProvider as _},
};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    resource::{EnvResourceDetector, SdkProvidedResourceDetector, TelemetryResourceDetector},
    trace::{Config, RandomIdGenerator},
    Resource,
};
use tonic::{metadata::MetadataMap, transport::ClientTlsConfig};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, prelude::*, Registry};

#[macro_use]
extern crate lazy_static;

mod core;

#[tokio::main]
async fn main() {
    openssl_probe::init_ssl_cert_env_vars();

    let dsn = std::env::var("UPTRACE_DSN").expect("UPTRACE_DSN not set");
    let tracer_provider = init_tracer(dsn.as_str()).expect("failed to initialize tracer");
    let tracer = tracer_provider.tracer("ttembed");
    let logger_provider = init_logger(dsn.as_str()).expect("failed to initialize logger");

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let subscriber = Registry::default()
        .with(telemetry.with_filter(LevelFilter::INFO))
        .with(OpenTelemetryTracingBridge::new(&logger_provider).with_filter(LevelFilter::INFO))
        .with(fmt::Layer::default().with_filter(LevelFilter::DEBUG));

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
                .with_tls_config(ClientTlsConfig::new().with_native_roots())
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
        .with_trace_config(
            Config::default()
                .with_resource(resource)
                .with_id_generator(RandomIdGenerator::default()),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
}

fn init_logger(dsn: &str) -> Result<opentelemetry_sdk::logs::LoggerProvider, LogError> {
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
        .logging()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_tls_config(ClientTlsConfig::new().with_native_roots())
                .with_endpoint("https://otlp.uptrace.dev:4317")
                .with_timeout(Duration::from_secs(5))
                .with_metadata(metadata.clone()),
        )
        .with_batch_config(
            opentelemetry_sdk::logs::BatchConfigBuilder::default()
                .with_max_queue_size(30000)
                .with_max_export_batch_size(10000)
                .with_scheduled_delay(Duration::from_millis(5000))
                .build(),
        )
        .with_resource(resource)
        .install_batch(opentelemetry_sdk::runtime::Tokio)
}

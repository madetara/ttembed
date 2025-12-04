use std::time::Duration;

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{
    LogExporterBuilder, SpanExporterBuilder, WithExportConfig, WithTonicConfig,
};
use opentelemetry_sdk::{
    Resource,
    logs::SdkLoggerProvider,
    resource::{
        EnvResourceDetector, ResourceDetector, SdkProvidedResourceDetector,
        TelemetryResourceDetector,
    },
    trace::{RandomIdGenerator, SdkTracerProvider},
};
use tonic::{metadata::MetadataMap, transport::ClientTlsConfig};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{Registry, fmt, prelude::*};

mod core;

#[tokio::main]
async fn main() {
    unsafe {
        openssl_probe::init_openssl_env_vars();
    }

    let dsn = std::env::var("UPTRACE_DSN").expect("UPTRACE_DSN not set");
    let tracer_provider = init_tracer(dsn.as_str());
    let tracer = tracer_provider.tracer("ttembed");
    let logger_provider = init_logger(dsn.as_str());

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let subscriber = Registry::default()
        .with(telemetry.with_filter(LevelFilter::INFO))
        .with(OpenTelemetryTracingBridge::new(&logger_provider).with_filter(LevelFilter::INFO))
        .with(fmt::Layer::default().with_filter(LevelFilter::DEBUG));

    tracing::subscriber::set_global_default(subscriber).unwrap();

    tracing::info!("starting");

    core::bot::run().await.unwrap();
}

fn init_tracer(dsn: &str) -> SdkTracerProvider {
    let detectors: Vec<Box<dyn ResourceDetector>> = vec![
        Box::new(SdkProvidedResourceDetector),
        Box::new(EnvResourceDetector::new()),
        Box::new(TelemetryResourceDetector),
    ];
    let resource = Resource::builder().with_detectors(&detectors).build();

    let mut metadata = MetadataMap::with_capacity(1);
    metadata.insert("uptrace-dsn", dsn.parse().unwrap());

    SdkTracerProvider::builder()
        .with_batch_exporter(
            SpanExporterBuilder::new()
                .with_tonic()
                .with_tls_config(ClientTlsConfig::new().with_native_roots())
                .with_endpoint("https://otlp.uptrace.dev:4317")
                .with_timeout(Duration::from_secs(5))
                .with_metadata(metadata)
                .build()
                .unwrap(),
        )
        .with_resource(resource)
        .with_id_generator(RandomIdGenerator::default())
        .build()
}

fn init_logger(dsn: &str) -> SdkLoggerProvider {
    let detectors: Vec<Box<dyn ResourceDetector>> = vec![
        Box::new(SdkProvidedResourceDetector),
        Box::new(EnvResourceDetector::new()),
        Box::new(TelemetryResourceDetector),
    ];
    let resource = Resource::builder().with_detectors(&detectors).build();

    let mut metadata = MetadataMap::with_capacity(1);
    metadata.insert("uptrace-dsn", dsn.parse().unwrap());

    SdkLoggerProvider::builder()
        .with_batch_exporter(
            LogExporterBuilder::new()
                .with_tonic()
                .with_tls_config(ClientTlsConfig::new().with_native_roots())
                .with_endpoint("https://otlp.uptrace.dev:4317")
                .with_timeout(Duration::from_secs(5))
                .with_metadata(metadata.clone())
                .build()
                .unwrap(),
        )
        .with_resource(resource)
        .build()
}

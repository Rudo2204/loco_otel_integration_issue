use loco_rs::cli;
use loco_rs::config::Config;
use loco_rs::environment::{Environment, resolve_from_env};
use loco_otel_integration_issue::app::App;
use std::str::FromStr;

use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::LogExporter;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use std::sync::OnceLock;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, Layer};

const MODULE_WHITELIST: &[&str] = &[
    env!("CARGO_CRATE_NAME"),
    "loco_otel_integration_issue",
    "loco_rs",
    "sea_orm_migration",
    "tower_http",
    "sqlx::query",
    "sidekiq",
    "playground",
    "loco_gen",
];

fn get_resource() -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();
    RESOURCE
        .get_or_init(|| {
            Resource::builder()
                .with_service_name("loco_otel_integration_issue")
                .build()
        })
        .clone()
}

fn init_logs() -> SdkLoggerProvider {
    let exporter = LogExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create log exporter");

    SdkLoggerProvider::builder()
        .with_resource(get_resource())
        .with_batch_exporter(exporter)
        .build()
}

#[tokio::main]
#[tracing::instrument]
async fn main() -> loco_rs::Result<()> {
    let env = resolve_from_env();
    let loco_env = Environment::from_str(&env).expect("could not load environment");
    let config =
        Config::new(&loco_env).expect(&format!("could not load config for environment {env}"));
    let log_level = config.logger.level;
    let (tracer_layer, _guard) =
        init_tracing_opentelemetry::tracing_subscriber_ext::build_otel_layer()
            .expect("failed to initialize tracer");
    let log_filter = MODULE_WHITELIST
        .iter()
        .map(|m| format!("{m}={log_level}"))
        .collect::<Vec<_>>()
        .join(",");
    let logger_provider = init_logs();

    let otel_layer = OpenTelemetryTracingBridge::new(&logger_provider);

    let filter_otel = EnvFilter::new(&log_filter)
        // specifically don't send logs from these crates because they are spammy
        .add_directive("hyper=off".parse().unwrap())
        .add_directive("opentelemetry=off".parse().unwrap())
        .add_directive("tonic=off".parse().unwrap())
        .add_directive("h2=off".parse().unwrap())
        .add_directive("reqwest=off".parse().unwrap());
    let otel_layer = otel_layer.with_filter(filter_otel);

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_line_number(true)
        .with_thread_names(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_filter(
            EnvFilter::new(&log_filter)
            //     .add_directive(
            //     // middleware logger creates an ERROR span to propgate data therefore is super spammy
            //     "loco_rs::controller::middleware::logger=off"
            //         .parse()
            //         .unwrap(),
            // ),
        );

    let test_subscriber = tracing_subscriber::registry()
        .with(tracer_layer)
        .with(
            init_tracing_opentelemetry::tracing_subscriber_ext::build_level_filter_layer(
                &log_filter,
            )
            .unwrap(),
        )
        .with(otel_layer)
        .with(fmt_layer);
    tracing::subscriber::set_global_default(test_subscriber)
        .expect("could not set global subscriber");

    tracing::info!("logger (level: {log_level}) & tracer initialized");

    cli::main::<App>().await
}

use opentelemetry::KeyValue;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{ runtime, trace as sdktrace, Resource };
use tracing_subscriber::{ layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer };

pub fn init_tracing(service_name: &str, otlp_endpoint: Option<&str>) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());

    let fmt_layer = tracing_subscriber::fmt::layer().boxed();

    if let Some(endpoint) = otlp_endpoint {
        let exporter = opentelemetry_otlp::SpanExporter
            ::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .build()
            .expect("Failed to build OTLP exporter");

        let tracer_provider = sdktrace::TracerProvider
            ::builder()
            .with_resource(
                Resource::new(
                    vec![
                        KeyValue::new("service.name", service_name.to_string()),
                        KeyValue::new("service.version", env!("CARGO_PKG_VERSION"))
                    ]
                )
            )
            .with_batch_exporter(exporter, runtime::Tokio)
            .build();

        let tracer = tracer_provider.tracer("crypto-payment-gateway");
        opentelemetry::global::set_tracer_provider(tracer_provider);

        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer).boxed();

        tracing_subscriber::registry().with(env_filter).with(fmt_layer).with(otel_layer).init();
    } else {
        tracing_subscriber::registry().with(env_filter).with(fmt_layer).init();
    }
}

pub fn shutdown_tracer() {
    opentelemetry::global::shutdown_tracer_provider();
}

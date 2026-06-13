use axum::{ routing::get, Router };
use prometheus::{
    register_counter_vec,
    register_histogram_vec,
    register_int_gauge,
    CounterVec,
    Encoder,
    HistogramVec,
    IntGauge,
    TextEncoder,
};
use std::sync::OnceLock;

static HTTP_REQUESTS_TOTAL: OnceLock<CounterVec> = OnceLock::new();
static HTTP_REQUEST_DURATION: OnceLock<HistogramVec> = OnceLock::new();
static PAYMENTS_TOTAL: OnceLock<CounterVec> = OnceLock::new();
static ACTIVE_INVOICES: OnceLock<IntGauge> = OnceLock::new();

pub fn init_metrics() {
    HTTP_REQUESTS_TOTAL.get_or_init(|| {
        register_counter_vec!(
            "http_requests_total",
            "Total number of HTTP requests",
            &["method", "path", "status"]
        ).expect("Failed to register http_requests_total")
    });

    HTTP_REQUEST_DURATION.get_or_init(|| {
        register_histogram_vec!(
            "http_request_duration_seconds",
            "HTTP request duration in seconds",
            &["method", "path"],
            vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5]
        ).expect("Failed to register http_request_duration_seconds")
    });

    PAYMENTS_TOTAL.get_or_init(|| {
        register_counter_vec!(
            "payments_total",
            "Total number of payments processed",
            &["status"]
        ).expect("Failed to register payments_total")
    });

    ACTIVE_INVOICES.get_or_init(|| {
        register_int_gauge!(
            "active_invoices",
            "Number of currently active (pending) invoices"
        ).expect("Failed to register active_invoices")
    });
}

#[allow(dead_code)]
pub fn increment_payment(status: &str) {
    if let Some(counter) = PAYMENTS_TOTAL.get() {
        counter.with_label_values(&[status]).inc();
    }
}

#[allow(dead_code)]
pub fn record_http_request(method: &str, path: &str, status: &str, duration_secs: f64) {
    if let Some(counter) = HTTP_REQUESTS_TOTAL.get() {
        counter.with_label_values(&[method, path, status]).inc();
    }
    if let Some(histogram) = HTTP_REQUEST_DURATION.get() {
        histogram.with_label_values(&[method, path]).observe(duration_secs);
    }
}

async fn metrics_handler() -> impl axum::response::IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap_or_default();
    ([(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4")], buffer)
}

pub fn metrics_router() -> Router {
    Router::new().route("/metrics", get(metrics_handler))
}

use anyhow::Result;
use prometheus::{Encoder, HistogramOpts, HistogramVec, Registry};
use std::{convert::Infallible, net::SocketAddr, sync::Arc, time::Instant};
use tokio::task::{self, JoinHandle};
use warp::{reply::Response, Filter, Rejection, Reply};

pub struct Metrics {
    /// Incoming API request metrics
    api_requests: HistogramVec,
}

impl Metrics {
    pub fn new(registry: &Registry) -> Result<Self> {
        let opts = HistogramOpts::new(
            "gp_v2_data_api_requests",
            "API Request durations labelled by route and response status code",
        );
        let api_requests = HistogramVec::new(opts, &["response", "request_type"]).unwrap();
        registry.register(Box::new(api_requests.clone()))?;

        Ok(Self { api_requests })
    }
}

// Response wrapper needed because we cannot inspect the reply's status code without consuming it
struct MetricsReply {
    response: Response,
}

impl Reply for MetricsReply {
    fn into_response(self) -> Response {
        self.response
    }
}

// Wrapper struct to annotate a reply with a handler label for logging purposes
pub struct LabelledReply {
    inner: Box<dyn Reply>,
    label: &'static str,
}

impl LabelledReply {
    pub fn new(inner: impl Reply + 'static, label: &'static str) -> Self {
        Self {
            inner: Box::new(inner),
            label,
        }
    }
}

impl Reply for LabelledReply {
    fn into_response(self) -> Response {
        self.inner.into_response()
    }
}

pub fn start_request() -> impl Filter<Extract = (Instant,), Error = Infallible> + Clone {
    warp::any().map(Instant::now)
}

pub fn end_request(metrics: Arc<Metrics>, timer: Instant, reply: LabelledReply) -> impl Reply {
    let LabelledReply { inner, label } = reply;
    let response = inner.into_response();
    let elapsed = timer.elapsed().as_secs_f64();
    metrics
        .api_requests
        .with_label_values(&[response.status().as_str(), label])
        .observe(elapsed);
    MetricsReply { response }
}

pub const DEFAULT_METRICS_PORT: u16 = 9586;

#[async_trait::async_trait]
pub trait LivenessChecking: Send + Sync {
    async fn is_alive(&self) -> bool;
}

pub fn serve_metrics(registry: Registry, address: SocketAddr) -> JoinHandle<()> {
    let filter = handle_metrics(registry);
    tracing::info!(%address, "serving metrics");
    task::spawn(warp::serve(filter).bind(address))
}

// `/metrics` route exposing encoded prometheus data to monitoring system
pub fn handle_metrics(
    registry: Registry,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("metrics").map(move || {
        let encoder = prometheus::TextEncoder::new();
        let mut buffer = Vec::new();
        if let Err(e) = encoder.encode(&registry.gather(), &mut buffer) {
            tracing::error!("could not encode metrics: {}", e);
        };
        match String::from_utf8(buffer) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("metrics could not be from_utf8'd: {}", e);
                String::default()
            }
        }
    })
}

pub mod api;
pub mod date_de_serialization;
pub mod dune_data_loading;
pub mod h160_hexadecimal;
pub mod in_memory_maintainance;
pub mod metrics;
pub mod models;
pub mod referral_maintenance;

extern crate serde_derive;

use metrics::serve_metrics;
use metrics::{Metrics, DEFAULT_METRICS_PORT};
use prometheus::Registry;

use models::in_memory_database::InMemoryDatabase;
use std::{net::SocketAddr, sync::Arc};
use tokio::{task, task::JoinHandle};

pub fn serve_task(
    db: Arc<InMemoryDatabase>,
    address: SocketAddr,
    registry: Registry,
    metrics: Arc<Metrics>,
) -> JoinHandle<()> {
    let filter = api::handle_all_routes(db, metrics);
    let mut metrics_address = address;
    tracing::info!(%address, "serving data");
    task::spawn(warp::serve(filter).bind(address));

    tracing::info!(%metrics_address, "serving metrics");
    metrics_address.set_port(DEFAULT_METRICS_PORT);
    serve_metrics(registry, metrics_address)
}

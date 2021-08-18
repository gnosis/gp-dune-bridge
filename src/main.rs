extern crate serde_derive;
use gpdata::dune_data_loading::load_data_from_json_into_memory;
use gpdata::metrics::Metrics;
use prometheus::Registry;

use gpdata::serve_task;
use std::{net::SocketAddr, sync::Arc};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Arguments {
    #[structopt(long, env = "BIND_ADDRESS", default_value = "0.0.0.0:8080")]
    bind_address: SocketAddr,
    #[structopt(long, env = "DUNE_DATA_FILE", default_value = "./user_data.json")]
    dune_data_file: String,
    
}

#[tokio::main]
async fn main() {
    let args = Arguments::from_args();
    tracing::info!("running data-server with {:#?}", args);

    let registry = Registry::default();
    let metrics = Arc::new(Metrics::new(&registry).unwrap());

    let memory_database =
        Arc::new(load_data_from_json_into_memory(args.dune_data_file).expect("could not load data into memory"));

    let serve_task = serve_task(
        memory_database.clone(),
        args.bind_address,
        registry,
        metrics.clone(),
    );
    tokio::select! {
        result = serve_task => tracing::error!(?result, "serve task exited"),
    };
}

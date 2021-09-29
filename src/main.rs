extern crate serde_derive;
use gpdata::dune_data_loading::load_data_from_json_into_memory;
use gpdata::in_memory_db_maintainance::in_memory_database_maintaince;
use gpdata::metrics::Metrics;
use gpdata::models::in_memory_database::DatabaseStruct;
use gpdata::models::in_memory_database::InMemoryDatabase;
use gpdata::models::referral_store::ReferralStore;
use gpdata::referral_maintenance::referral_maintainance;
use gpdata::serve_task;
use gpdata::tracing::initialize;
use prometheus::Registry;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Arguments {
    #[structopt(long, env = "LOG_FILTER", default_value = "warn,debug,info")]
    pub log_filter: String,
    #[structopt(long, env = "BIND_ADDRESS", default_value = "0.0.0.0:8080")]
    bind_address: SocketAddr,
    #[structopt(long, env = "DUNE_DATA_FOLDER", default_value = "./data/dune_data/")]
    dune_data_folder: String,
    #[structopt(
        long,
        env = "REFERRAL_DATA_FOLDER",
        default_value = "./data/referral_data/"
    )]
    referral_data_folder: String,
}

#[tokio::main]
async fn main() {
    let args = Arguments::from_args();
    initialize(args.log_filter.as_str());
    tracing::info!("running data-server with {:#?}", args);

    let registry = Registry::default();
    let metrics = Arc::new(Metrics::new(&registry).unwrap());
    let dune_download_folder = args.dune_data_folder;
    let referral_data_folder = args.referral_data_folder;

    let dune_data =
        load_data_from_json_into_memory(String::from(dune_download_folder.clone() + "user_data/"))
            .unwrap_or(DatabaseStruct::default());
    let memory_database = Arc::new(InMemoryDatabase(Mutex::new(dune_data)));

    let referral_store = ReferralStore::new(Vec::new());
    let referral_maintance_task = tokio::task::spawn(referral_maintainance(
        Arc::new(referral_store),
        referral_data_folder,
        dune_download_folder.clone(),
    ));
    let serve_task = serve_task(
        memory_database.clone(),
        args.bind_address,
        registry,
        metrics.clone(),
    );
    let maintance_task = tokio::task::spawn(in_memory_database_maintaince(
        memory_database.clone(),
        dune_download_folder,
    ));
    tokio::select! {
        result = referral_maintance_task => tracing::error!(?result, "referral maintance task exited"),
        result = maintance_task => tracing::error!(?result, "db maintance task exited"),
        result = serve_task => tracing::error!(?result, "serve task exited"),
    };
}

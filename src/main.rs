use gpdata::health::HttpHealthEndpoint;
use gpdata::in_memory_maintenance::in_memory_database_maintaince;
use gpdata::models::in_memory_database::DatabaseStruct;
use gpdata::models::in_memory_database::InMemoryDatabase;
use gpdata::models::referral_store::ReferralStore;
use gpdata::referral_maintenance::referral_maintainance;
use gpdata::serve_task;
use gpdata::tracing_helper::initialize;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
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
    let dune_download_folder = args.dune_data_folder;
    let referral_data_folder = args.referral_data_folder;
    let memory_database = Arc::new(InMemoryDatabase(Mutex::new(DatabaseStruct::default())));
    let health = Arc::new(HttpHealthEndpoint::new());
    let serve_task = serve_task(memory_database.clone(), args.bind_address, health.clone());
    let maintance_task = tokio::task::spawn(in_memory_database_maintaince(
        memory_database.clone(),
        dune_download_folder.clone(),
        health,
    ));
    let referral_store = ReferralStore::new(Vec::new());
    let referral_maintance_task = tokio::task::spawn(referral_maintainance(
        Arc::new(referral_store),
        dune_download_folder.clone(),
        referral_data_folder,
    ));
    tokio::select! {
        result = referral_maintance_task => tracing::error!(?result, "referral maintance task exited"),
        result = maintance_task => tracing::error!(?result, "maintance task exited"),
        result = serve_task => tracing::error!(?result, "serve task exited"),
    };
}

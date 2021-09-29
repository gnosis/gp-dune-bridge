use crate::dune_data_loading::load_data_from_json_into_memory;
use crate::models::in_memory_database::InMemoryDatabase;
use std::sync::Arc;
use std::time::Duration;

const MAINTENANCE_INTERVAL: Duration = Duration::from_secs(30);

pub async fn in_memory_database_maintaince(
    memory_database: Arc<InMemoryDatabase>,
    dune_download_file: String,
) {
    let db = Arc::clone(&memory_database);
    loop {
        let new_data_mutex = load_data_from_json_into_memory(String::from(
            dune_download_file.clone() + "user_data/",
        ));
        match new_data_mutex {
            Ok(data) => {
                let mut guard = match db.0.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                *guard = data;
            }
            Err(err) => tracing::info!(
                "could not load query-data from json due to the error {:}",
                err
            ),
        };
        tokio::time::sleep(MAINTENANCE_INTERVAL).await;
    }
}

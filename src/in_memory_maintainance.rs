use crate::dune_data_loading::load_data_from_json_into_memory;
use crate::models::in_memory_database::InMemoryDatabase;
use std::sync::Arc;
use std::time::Duration;

const MAINTENANCE_INTERVAL: Duration = Duration::from_secs(3);

pub async fn in_memory_database_maintaince(
    memory_database: Arc<InMemoryDatabase>,
    dune_download_file: String,
) {
    let db = Arc::clone(&memory_database);
    loop {
        {
            let mut guard = match db.0.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            let new_data_mutex = load_data_from_json_into_memory(dune_download_file.clone())
                .expect("could not load data into memory");
            *guard = new_data_mutex;
        }
        tokio::time::sleep(MAINTENANCE_INTERVAL).await;
    }
}

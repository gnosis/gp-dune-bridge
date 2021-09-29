use crate::date_de_serialization::from_date;
use crate::models::app_data_from_json::AppData;
use crate::models::referral_store::ReferralStore;
use anyhow::{anyhow, Result};
use chrono::DateTime;
use chrono::Utc;
use cid::Cid;
use serde_json;
use std::convert::TryFrom;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use primitive_types::{H160, H256};
use serde::{Deserialize, Serialize};
use substring::Substring;

const MAINTENANCE_INTERVAL: Duration = Duration::from_secs(80);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DuneAppDataDownload {
    pub app_data: Vec<UserData>,
    #[serde(deserialize_with = "from_date")]
    pub time_of_download: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserData {
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Data {
    pub appdata: String,
}

fn read_dune_data_from_file<P: AsRef<Path>>(path: P) -> Result<DuneAppDataDownload> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}

pub fn load_distinct_app_data_from_json(dune_data_file: String) -> Result<Vec<H256>> {
    let dune_download = read_dune_data_from_file(dune_data_file)?;
    let app_data: Vec<H256> = dune_download
        .app_data
        .iter()
        .filter_map(|data_point| data_point.data.appdata.substring(3, 67).parse().ok())
        .collect();
    Ok(app_data)
}
pub async fn maintenaince_tasks(
    db: Arc<ReferralStore>,
    referral_data_folder: String,
    dune_data_folder: String,
) -> Result<()> {
    // 1st step: getting all possible app_data from file and store them in ReferralStore
    let vec_with_all_app_data = match load_distinct_app_data_from_json(String::from(
        dune_data_folder + "app_data/distinct_app_data.json",
    )) {
        Ok(vec) => vec,
        Err(err) => {
            tracing::info!("Could not load distinct app data, due to: {:?}", err);
            return Ok(());
        }
    };
    for app_data in vec_with_all_app_data {
        {
            let mut guard = match db.0.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            match guard.app_data.get(&app_data) {
                Some(_) => {}
                None => {
                    guard.app_data.insert(app_data, None);
                }
            };
        }
    }

    // 2st step: get all unintialized referrals
    let uninitialized_app_data_hashes: Vec<H256>;
    {
        let guard = match db.0.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        uninitialized_app_data_hashes = guard
            .app_data
            .clone()
            .iter()
            .filter(|(_, referral)| referral.is_none())
            .map(|(hash, _)| *hash)
            .collect();
    }
    // 3. try to retrieve all ipfs data for hashes and store them
    for hash in uninitialized_app_data_hashes.iter() {
        let cid_string = get_cid_from_app_data(hash.clone());
        if cid_string.is_ok() {
            let cid = cid_string.unwrap();
            tracing::info!("{:?}", cid);
            let referrer = match get_ipfs_file_and_read_referrer(cid.clone()).await {
                Ok(referrer) => referrer,
                Err(err) => {
                    tracing::info!(
                "Could not find referrer in cid {:?}, due to the error {:?}, setting referrer to zero address",
                cid, err
            );
                    H160::zero()
                }
            };
            {
                let mut guard = match db.0.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                guard.app_data.insert(*hash, Some(referrer));
            }
            tracing::info!("Adding the referrer {:?} for the hash {:?}", referrer, hash);
        } else {
            tracing::info!("For the app_data hash {:?}, there could not be found a unique referrer due to {:?}", hash, cid_string.as_ref().err());
        }
    }
    // 4. dump hashmap to json
    let mut file = File::create(referral_data_folder + "app_data_referral_relationship.json")?;
    {
        let guard = match db.0.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let file_content = serde_json::to_string(&*guard)?;
        file.write_all(&file_content.as_bytes())?;
    }
    Ok(())
}
pub async fn referral_maintainance(
    memory_database: Arc<ReferralStore>,
    referral_data_folder: String,
    dune_data_folder: String,
) {
    loop {
        match maintenaince_tasks(
            Arc::clone(&memory_database),
            referral_data_folder.clone(),
            dune_data_folder.clone(),
        )
        .await
        {
            Ok(_) => {}
            Err(err) => tracing::error!("Error during maintenaince_task for referral: {:?}", err),
        }
        tokio::time::sleep(MAINTENANCE_INTERVAL).await;
    }
}
async fn get_ipfs_file_and_read_referrer(cid: String) -> Result<H160> {
    let url = format!("https://gateway.pinata.cloud/ipfs/{:}", cid);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(1))
        .build()?;
    let body = client.get(url).send().await?.text().await?;
    tracing::info!("returned body {:?}", body);
    let json: AppData = serde_json::from_str(&body)?;
    if let Some(metadata) = json.clone().metadata {
        if let Some(referrer) = metadata.referrer {
            return Ok(referrer.address);
        }
    }
    tracing::info!(
        "Could not find referrer in cid {:?}, setting referrer to zero address",
        cid
    );
    Err(anyhow!(
        "Could not find referrer in json object: {:?}",
        json
    ))
}

fn get_cid_from_app_data(hash: H256) -> Result<String> {
    let cid_prefix = vec![1u8, 112u8, 18u8, 32u8];
    let cid = Cid::try_from([&cid_prefix, hash.as_bytes()].concat())?;
    Ok(cid.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_cid_from_app_data() {
        let test_app_data_hash: H256 =
            "3d876de8fcd70969349c92d731eeb0482fe8667ceca075592b8785081d630b9a"
                .parse()
                .unwrap();
        assert_eq!(
            get_cid_from_app_data(test_app_data_hash).unwrap(),
            String::from("bafybeib5q5w6r7gxbfutjhes24y65mcif7ugm7hmub2vsk4hqueb2yylti")
        );
    }
    #[test]
    fn test_get_cid_from_app_data_2() {
        let test_app_data_hash: H256 =
            "1FE7C5555B3F9C14FF7C60D90F15F1A5B11A0DA5B1E8AA043582A1B2E1058D0C"
                .parse()
                .unwrap();
        assert_eq!(
            get_cid_from_app_data(test_app_data_hash).unwrap(),
            String::from("bafybeia747cvkwz7tqkp67da3ehrl4nfwena3jnr5cvainmcugzocbmnbq")
        );
    }
    #[tokio::test]
    async fn test_fetching_ipfs() {
        let referral = get_ipfs_file_and_read_referrer(String::from(
            "bafybeib5q5w6r7gxbfutjhes24y65mcif7ugm7hmub2vsk4hqueb2yylti",
        ))
        .await
        .unwrap();
        let expected_referral: H160 = "0x424a46612794dbb8000194937834250Dc723fFa5"
            .parse()
            .unwrap();
        assert_eq!(referral, expected_referral);
    }
    #[tokio::test]
    async fn test_fetching_ipfs_2() {
        let referral = get_ipfs_file_and_read_referrer(String::from(
            "bafybeia747cvkwz7tqkp67da3ehrl4nfwena3jnr5cvainmcugzocbmnbq",
        ))
        .await
        .unwrap();
        let expected_referral: H160 = "0x8c35B7eE520277D14af5F6098835A584C337311b"
            .parse()
            .unwrap();
        assert_eq!(referral, expected_referral);
    }
    #[tokio::test]
    async fn test_maintenaince_tasks2() {
        let test_app_data_hash: H256 =
            "3d876de8fcd70969349c92d731eeb0482fe8667ceca075592b8785081d630b9a"
                .parse()
                .unwrap();
        let referral_store = ReferralStore::new(vec![test_app_data_hash]);
        let result = maintenaince_tasks(Arc::new(referral_store)).await;
        assert!(result.is_ok());
    }
}

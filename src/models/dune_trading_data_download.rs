extern crate serde_derive;
use crate::date_de_serialization::from_date;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DuneTradingDataDownload {
    pub user_data: Vec<UserData>,
    #[serde(deserialize_with = "from_date")]
    pub time_of_download: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserData {
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Data {
    pub cowswap_usd_volume: Option<f64>,
    pub day: String,
    pub nr_of_referrals: Option<u64>,
    pub number_of_trades: Option<u64>,
    pub owner: String,
    pub total_referred_volume: Option<f64>,
    pub usd_volume_all_exchanges: Option<f64>,
}

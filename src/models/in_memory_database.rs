
extern crate serde_derive;
use anyhow::Result;
use chrono::prelude::*;
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::dune_download::Data;

#[derive(Debug, Clone)]
pub struct InMemoryDatabase(pub HashMap<H160, Vec<Data>>, pub DateTime<Utc>);

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    total_trades: u64,
    total_referrals: u64,
    trade_volume_usd: f64,
    referral_volume_usd: f64,
    last_updated: Option<DateTime<Utc>>,
}

impl InMemoryDatabase {
    pub fn get_profile_from_raw_data(&self, user: H160) -> Result<Profile> {
        match self.0.get(&user) {
            Some(data) => {
                Ok(Profile {
                    total_trades: data
                        .iter()
                        .map(|data| data.number_of_trades.unwrap_or(0u64))
                        .sum(),
                    total_referrals: 0u64, // <-- dummy
                    trade_volume_usd: data
                        .iter()
                        .map(|data| data.cowswap_usd_volume.unwrap_or(0f64))
                        .sum(),
                    referral_volume_usd: 0f64,  // <-- dummy
                    last_updated: Some(self.1), // <-- dummy
                })
            }
            None => Ok(Default::default()),
        }
    }
}

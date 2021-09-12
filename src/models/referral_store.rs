extern crate serde;
extern crate serde_derive;
use primitive_types::{H160, H256};
use serde::ser::{Serialize, SerializeMap, Serializer};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug)]
pub struct AppDataStruct {
    pub app_data: HashMap<H256, Option<H160>>,
}
impl Serialize for AppDataStruct {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.app_data.keys().len()))?;
        for (k, value) in &self.app_data {
            match value {
                Some(v) => map.serialize_entry(&k.to_string(), &v.to_string())?,
                None => map.serialize_entry(&k.to_string(), &"null")?,
            }
        }
        map.end()
    }
}

#[derive(Debug)]
pub struct ReferralStore(pub Mutex<AppDataStruct>);

impl ReferralStore {
    pub fn new(app_data_hashes: Vec<H256>) -> Self {
        let mut hm = HashMap::new();
        for hash in app_data_hashes {
            hm.insert(hash, None);
        }
        let app_data_struct = AppDataStruct { app_data: hm };
        ReferralStore(Mutex::new(app_data_struct))
    }
}

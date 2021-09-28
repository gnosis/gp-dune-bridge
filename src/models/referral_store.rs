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
        for (hash, address) in &self.app_data {
            let mut bytes = [0u8; 2 + 32 * 2];
            bytes[..2].copy_from_slice(b"0x");
            // Can only fail if the buffer size does not match but we know it is correct.
            hex::encode_to_slice(hash, &mut bytes[2..]).unwrap();
            // Hex encoding is always valid utf8.
            let hash_serialized = std::str::from_utf8(&bytes).unwrap();

            match address {
                Some(v) => {
                    let mut bytes = [0u8; 2 + 20 * 2];
                    bytes[..2].copy_from_slice(b"0x");
                    // Can only fail if the buffer size does not match but we know it is correct.
                    hex::encode_to_slice(v, &mut bytes[2..]).unwrap();
                    let address_serialized = std::str::from_utf8(&bytes).unwrap();
                    map.serialize_entry(&hash_serialized, &address_serialized.to_string())?
                }
                None => map.serialize_entry(&hash_serialized, &"null")?,
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

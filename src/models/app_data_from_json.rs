//! Contains the app_data file structures, as they are stored in ipfs
//!
use crate::h160_hexadecimal;
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Eq, PartialEq, Clone, Debug, Deserialize, Serialize, Hash, Default)]
pub struct Referrer {
    #[serde(with = "h160_hexadecimal")]
    pub address: H160,
    pub version: String,
}

#[serde_as]
#[derive(Eq, PartialEq, Clone, Debug, Deserialize, Serialize, Hash, Default)]
pub struct Metadata {
    pub referrer: Option<Referrer>,
}

#[serde_as]
#[derive(Eq, PartialEq, Clone, Debug, Deserialize, Serialize, Hash, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppData {
    pub version: String,
    pub app_code: Option<String>,
    pub metadata: Option<Metadata>,
}

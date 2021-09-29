extern crate serde_derive;
use crate::models::in_memory_database::DatabaseStruct;
use anyhow::Result;
use chrono::prelude::*;
use primitive_types::H160;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use substring::Substring;

use crate::models::dune_trading_data_download::{Data, DuneTradingDataDownload};

fn read_dune_data_from_file<P: AsRef<Path>>(path: P) -> Result<DuneTradingDataDownload> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}

pub fn load_data_from_json_into_memory(dune_data_folder: String) -> Result<DatabaseStruct> {
    let mut memory_database: HashMap<H160, Vec<Data>> = HashMap::new();
    let mut date = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc);
    println!("{:?}", dune_data_folder);
    for entry in fs::read_dir(dune_data_folder)? {
        let entry = entry?;
        let dune_download =
            read_dune_data_from_file(entry.path()).expect("JSON was not well-formatted");
        for user_data in dune_download.user_data {
            let address: H160 = user_data
                .data
                .owner
                .substring(2, 160)
                .parse()
                .expect("JSON owner was not well-formatted");
            let vector_to_insert;
            if let Some(data_vector) = memory_database.get_mut(&address) {
                data_vector.push(user_data.data);
                vector_to_insert = data_vector.to_vec();
            } else {
                vector_to_insert = vec![user_data.data];
            }
            memory_database.insert(address, vector_to_insert);
        }
        date = dune_download.time_of_download;
    }
    Ok(DatabaseStruct {
        user_data: memory_database,
        updated: date,
    })
}

mod tests;

use crate::error::Error;
use crate::schema::kv;
use crate::schema::kv::dsl::*;

use log::debug;
use ::uuid::Uuid;
use diesel::prelude::*;
use diesel::result::Error::NotFound;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct KV {
    pub id: i32,
    pub uuid: Option<Uuid>,
    pub platform: String,
    pub identity: String,
    pub content: serde_json::Value,
}

#[derive(Insertable, Debug)]
#[table_name = "kv"]
pub struct NewKV {
    pub platform: String,
    pub identity: String,
}

/// Returns (KV, is_founded)
pub fn find_or_create(
    conn: &PgConnection,
    expected_platform: &str,
    expected_identity: &str,
) -> Result<(KV, bool), Error> {
    let found: Result<KV, _> = kv
        .filter(platform.eq(expected_platform))
        .filter(identity.eq(expected_identity))
        .first(conn);
    debug!("Found: {:?}", found.is_ok());
    // Found
    if let Ok(result) = found {
        return Ok((result, true));
    }
    // General DB error
    let err = found.unwrap_err();
    if err != NotFound {
        return Err(err.into());
    }
    // Create
    diesel::insert_into(kv::table)
        .values((
            platform.eq(expected_platform),
            identity.eq(expected_identity),
        ))
        .get_result(conn)
        .map(|created| (created, false))
        .map_err(|e| e.into())
}
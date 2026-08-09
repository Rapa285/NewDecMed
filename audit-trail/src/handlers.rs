use reqwest::{Client, IntoUrl};
use anyhow::Context;
use crate::{
    constants::{GAS_STATION_BASE_URL, IOTA_URL, IPFS_BASE_URL, IPFS_GATEWAY_BASE_URL},
    current_fn,
    audit_error::AuditError,
    // types::{ExecuteTxResponse, ReserveGasResponse, SuccessResponse, UtilIpfsAddResponse},
    types::{UtilIpfsAddResponse},
};
use uuid::Uuid;

pub struct Handlers {}

impl Handlers {

    pub async fn new_audit_record(data: String) -> Result<String, AuditError> {

        // berikan ID
        let uuid = Uuid::now_v7();

        // add ke ipfs
        let audit_record = AuditRecord {
            id: uuid,
            data,
        };

        let cid = Self::add_and_pin_to_ipfs(audit_record).await?;

        // add cid ke IOTA
        // uuid
        // ipfs cid
        

        // add indexing ke postgres
        // uuid
        // ts
        // event type
        // actor
        // ipfs cid
        // iota id

        Ok(String::from("fungsi berhasil"))
    }

    pub async fn delete_audit_record(cid: String) -> Result<String, AuditError> {

        Ok()
    }


}
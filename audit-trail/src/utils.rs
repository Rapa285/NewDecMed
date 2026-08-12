
use types::{AuditRecord, UtilIpfsAddResponse};

use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use reqwest::Body;

pub struct Utils {}

impl Utils {
    pub async fn add_and_pin_to_ipfs(data: AuditRecord) -> Result<String, AuditError> {
        let path_part = reqwest::multipart::Part::text(data);
        let form = reqwest::multipart::Form::new().part("path", path_part);
        let req_client = reqwest::Client::new();
        let res = req_client
            .post(format!("{}/add", IPFS_BASE_URL))
            .multipart(form)
            .send()
            .await
            .context(current_fn!())?;

        let res = res
            .json::<UtilIpfsAddResponse>()
            .await
            .context(current_fn!())?;

        Ok(res.cid)
    }


    pub async fn add_file_to_ipfs(file_path: &str) -> Result<String, AuditError> {

        let file = File::open(file_path)
            .await
            .context(current_fn!())?;

        let stream = FramedRead::new(file, BytesCodec::new());
        let body = Body::wrap_stream(stream);

        let file_part = reqwest::multipart::Part::stream(body)
            .file_name("audit_trail.log")
            .mime_str("text/plain")
            .context(current_fn!())?;

        let form = reqwest::multipart::Form::new().part("file", file_part);
        let req_client = reqwest::Client::new();
        
        let res = req_client
            .post(format!("{}/api/v0/add", IPFS_BASE_URL))
            .multipart(form)
            .send()
            .await
            .context(current_fn!())?;

        let res = res
            .json::<UtilIpfsAddResponse>()
            .await
            .context(current_fn!())?;

        Ok(res.cid)
    } 

}
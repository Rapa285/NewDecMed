
use types::{AuditRecord, UtilIpfsAddResponse};

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

}
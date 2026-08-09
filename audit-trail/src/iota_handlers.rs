
pub struct Iota{}

impl Iota {
    pub async fn get_iota_client() -> Result<iota_client::Client, AuditError> {
        let iota_client = iota_client::Client::builder()
            .with_node(IOTA_URL)
            .context(current_fn!())?
            .finish()
            .await
            .context(current_fn!())?;

        Ok(iota_client)
    }

    

}
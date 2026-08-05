// pub struct AuditService {
//     storage: Box<dyn AuditStorage>,
// }

// impl AuditService {
//     pub async fn record(
//         &self,
//         event: AuditEvent,
//     ) -> anyhow::Result<()> {
//         self.storage.store(event).await
//     }
// }

// #[async_trait]
// pub trait AuditStorage: Send + Sync {
//     async fn store(
//         &self,
//         event: AuditEvent,
//     ) -> anyhow::Result<()>;
// }
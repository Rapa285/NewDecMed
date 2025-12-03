// pub const IOTA_URL: &str = "https://live-sturgeon-needlessly.ngrok-free.app";
// pub const GAS_STATION_BASE_URL: &str = "https://ec7f-114-122-115-51.ngrok-free.app/v1";
pub const IOTA_URL: &str = "http://localhost:9000";
pub const GAS_STATION_BASE_URL: &str = "http://localhost:9527/v1";
pub const GAS_BUDGET: u64 = 10_000_000;
pub const _HASH_SALT: &str = "169224A2BE2B267684F93A9CE38080D359BD774741FD3AE738D09B657A1A8104";
pub const IPFS_BASE_URL: &str = "http://localhost:9094";
pub const IPFS_GATEWAY_BASE_URL: &str = "http://127.0.0.1:8080";
/// Duration: 3 minutes
pub const NONCE_EXP_DUR: u64 = 3 * 60;
/// Duration: 5 minutes
pub const ADMINISTRATIVE_KEYS_READ_DUR: u64 = 5 * 60;
/// Duration: 15 minutes
pub const MEDICAL_KEYS_READ_DUR: u64 = 15 * 60;
/// Duration: 2 hours
pub const MEDICAL_KEYS_UPDATE_DUR: u64 = 2 * 60 * 60;

pub const DECMED_MODULE_PROXY: &str = "proxy";
pub const DECMED_MODULE_SHARED: &str = "shared";

pub const DECMED_PACKAGE_ID: &str =
    "0x26f528960c395669259276ae295b3432b4e2440381bb5e3330b65453c78ac7fe";
pub const DECMED_MODULE_ADMIN: &str = "admin";

pub const DECMED_ADDRESS_ID_OBJECT_ID: &str =
    "0x940bcdd5f055f878206ab9b038e5243370df47bcfaa65337229c8d9ba13923d0";
pub const DECMED_ADDRESS_ID_OBJECT_VERSION: u64 = 3;
pub const DECMED_HOSPITAL_ID_METADATA_OBJECT_ID: &str =
    "0xe86e38e81645aabb12c7638628a44844b4c12bef9b6b366b09ae68221c5a0785";
pub const DECMED_HOSPITAL_ID_METADATA_OBJECT_VERSION: u64 = 3;
pub const DECMED_HOSPITAL_PERSONNEL_ID_ACCOUNT_OBJECT_ID: &str =
    "0xee0d2b09e92a1e535235cd1f3ddbf815696e80b125b39b8179ee93e8f9c9514f";
pub const DECMED_HOSPITAL_PERSONNEL_ID_ACCOUNT_OBJECT_VERSION: u64 = 3;
pub const DECMED_PATIENT_ID_ACCOUNT_OBJECT_ID: &str =
    "0x34eef713e618359780121bebc167b17cfb3259e2dcd231f9c3a921ca399a918a";
pub const DECMED_PATIENT_ID_ACCOUNT_OBJECT_VERSION: u64 = 3;

pub const DECMED_GLOBAL_ADMIN_CAP_ID: &str =
    "0xca2546e14bb5850ce513c9673ce855f929ce4aa8fb11091b4f009b2ea34cd917";

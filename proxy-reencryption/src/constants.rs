// pub const IOTA_URL: &str = "https://live-sturgeon-needlessly.ngrok-free.app";
// pub const GAS_STATION_BASE_URL: &str = "https://ec7f-114-122-115-51.ngrok-free.app/v1";
pub const IOTA_URL: &str = "http://localhost:9000";
pub const GAS_STATION_BASE_URL: &str = "http://localhost:9528/v1";
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
    "0x1da496ea0919fb7f6c297108ee904ae251f8e1cbf4b32ecfe000af46dbb1515d";
pub const DECMED_MODULE_ADMIN: &str = "admin";

pub const DECMED_ADDRESS_ID_OBJECT_ID: &str =
    "0x44d73fad6b544c61ecf8088e85100f05fd33e749c8c531a03558b7ec0edc4543";
pub const DECMED_ADDRESS_ID_OBJECT_VERSION: u64 = 3;
pub const DECMED_HOSPITAL_ID_METADATA_OBJECT_ID: &str =
    "0xf0bf3e8eb4920533e397e4ad72b5aacadab243e10ddf69cf6e9ced122464d90b";
pub const DECMED_HOSPITAL_ID_METADATA_OBJECT_VERSION: u64 = 3;
pub const DECMED_HOSPITAL_PERSONNEL_ID_ACCOUNT_OBJECT_ID: &str =
    "0x9228c84213a9a10b0f03b01980a9b5444631656ce9d576af60d8e340727cd812";
pub const DECMED_HOSPITAL_PERSONNEL_ID_ACCOUNT_OBJECT_VERSION: u64 = 3;
pub const DECMED_PATIENT_ID_ACCOUNT_OBJECT_ID: &str =
    "0xbd0ec784c2a52799a2cbfeecd71458f77cf3ffc9bf3f7fa7e2c33efa6630b679";
pub const DECMED_PATIENT_ID_ACCOUNT_OBJECT_VERSION: u64 = 3;

pub const DECMED_GLOBAL_ADMIN_CAP_ID: &str =
    "0xe664e86b9a82c7b98bbc1e69f4d377d7d610e0e3d6b8555453ace804b045f876";

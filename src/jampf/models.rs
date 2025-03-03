use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct JamfAuthReponse {
    pub(crate) token: String,
    pub(crate) expires: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct JamfGetComputersResponse {
    pub(crate) computers: Vec<JamfComputerMetadataSummary>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct JamfComputerMetadataSummary {
    pub(crate) name: String,
    pub(crate) managed: bool,
    pub(crate) username: String,
    pub(crate) model: String,
    pub(crate) id: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct JamfComputerDetailedResponse {
    pub(crate) computer: JamfComputerDetailedMetadata,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct JamfComputerDetailedMetadata {
    pub(crate) hardware: Option<ComputerHardwareMetadata>,
    pub(crate) security: Option<ComputerHardwareSecurityMetadata>,
    pub(crate) software: Option<ComputerSoftwareMetadata>,
    pub(crate) configuration_profiles: Option<Vec<ConfigurationProfile>>,
    #[serde(rename = "operatingSystem")]
    pub(crate) operating_system: Option<OperatingSystemMetadata>,
    pub(crate) general: Option<GeneralMetadata>,
    pub(crate) id: Option<String>,
    pub(crate) udid: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct OperatingSystemMetadata {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) build: String,
    #[serde(rename = "softwareUpdates")]
    pub(crate) software_updates: Option<Vec<SoftwareUpdate>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SoftwareUpdate {
    pub(crate) name: String,
    pub(crate) version: String,
    #[serde(rename = "packageName")]
    pub(crate) package_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GeneralMetadata {
    pub(crate) name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ComputerHardwareMetadata {
    pub(crate) make: String,
    pub(crate) model: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ComputerHardwareSecurityMetadata {
    pub(crate) activation_lock: Option<bool>,
    pub(crate) recovery_lock_enabled: Option<bool>,
    pub(crate) secure_boot_level: String,
    pub(crate) external_boot_level: String,
    pub(crate) firewall_enabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ComputerSoftwareMetadata {
    pub(crate) available_software_updates: Vec<String>,
    pub(crate) available_updates: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ConfigurationProfile {
    pub(crate) id: isize,
    pub(crate) name: String,
    pub(crate) uuid: String,
    pub(crate) is_removable: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ComputerInventoryResponse {
    #[serde(rename = "totalCount")]
    pub(crate) total_count: usize,
    pub(crate) results: Vec<JamfComputerDetailedMetadata>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct AvailableUpdates {
    #[serde(rename = "availableUpdates")]
    pub(crate) available_updates: AvailableUpdatesInner,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct AvailableUpdatesInner {
    #[serde(rename = "macOS")]
    pub(crate) mac_os: Vec<String>,
    #[serde(rename = "iOS")]
    pub(crate) ios: Vec<String>,
}

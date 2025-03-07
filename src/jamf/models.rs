use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct JamfAuthReponse {
    pub(crate) token: String,
    pub(crate) expires: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct JamfComputer {
    pub(crate) hardware: Option<JamfComputerHardware>,
    pub(crate) software: Option<JamfComputerSoftware>,
    #[serde(rename = "operatingSystem")]
    pub(crate) operating_system: Option<JamfComputerOperatingSystem>,
    pub(crate) general: Option<JamfComputerGeneral>,
    pub(crate) id: Option<String>,
    pub(crate) udid: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct JamfComputerOperatingSystem {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) build: String,
    #[serde(rename = "softwareUpdates")]
    pub(crate) software_updates: Option<Vec<JamfSoftwareUpdate>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct JamfSoftwareUpdate {
    pub(crate) name: String,
    pub(crate) version: String,
    #[serde(rename = "packageName")]
    pub(crate) package_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct JamfComputerGeneral {
    pub(crate) name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct JamfComputerHardware {
    pub(crate) make: String,
    pub(crate) model: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct JamfComputerSoftware {
    pub(crate) available_software_updates: Vec<String>,
    pub(crate) available_updates: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct JamfComputerInventoryResponse {
    #[serde(rename = "totalCount")]
    pub(crate) total_count: usize,
    pub(crate) results: Vec<JamfComputer>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct JamfAvailableUpdates {
    #[serde(rename = "availableUpdates")]
    pub(crate) available_updates: AvailableUpdates,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct AvailableUpdates {
    #[serde(rename = "macOS")]
    pub(crate) mac_os: Vec<String>,
    #[serde(rename = "iOS")]
    pub(crate) ios: Vec<String>,
}

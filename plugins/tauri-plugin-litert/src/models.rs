use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitModelRequest {
  pub model_path: String,
  pub accelerator: String,
  pub max_tokens: u32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitModelResponse {
  pub success: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckModelRequest {
  pub model_path: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckModelResponse {
  pub exists: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadModelRequest {
  pub model_path: String,
  pub token: Option<String>,
  pub download_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurgeModelRequest {
  pub model_path: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadModelResponse {
  pub success: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateChatRequest {
  pub prompt: String,
  #[serde(default)]
  pub reset: bool,
  pub audio_base64: Option<String>,
  pub image_uri: Option<String>,
  pub system_instruction: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateChatResponse {
  pub response: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseModelRequest {}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseModelResponse {
  pub success: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PickGalleryImageResponse {
  pub path: String,
  pub uri: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TakeCameraPhotoResponse {
  pub path: String,
  pub uri: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BleDeviceInfo {
  pub device_id: String,
  pub device_name: String,
  pub mac_address: String,
  pub rssi: i32,
  pub is_paired: bool,
  pub is_connected: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanBleDevicesResponse {
  pub devices: Vec<BleDeviceInfo>,
  pub status: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncBleDeviceRequest {
  pub mac_address: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub force_pull_all: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncBleDeviceResponse {
  pub success: bool,
  pub battery_level: Option<u8>,
  pub fit_files: Option<Vec<String>>,
  pub synced_activities_count: usize,
  pub status_message: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarPermissionResponse {
  pub granted: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCalendarEventsRequest {
  #[serde(default)]
  pub start_time_epoch_ms: i64,
  #[serde(default)]
  pub end_time_epoch_ms: i64,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeCalendarEvent {
  pub id: String,
  pub title: String,
  pub description: String,
  pub location: String,
  pub start_time: i64,
  pub end_time: i64,
  pub is_all_day: bool,
  pub calendar_name: String,
  pub event_color: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCalendarEventsResponse {
  pub events: Vec<NativeCalendarEvent>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechRecognitionResponse {
  pub transcription: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanUsbMtpResponse {
  pub is_attached: bool,
  pub device_name: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub vendor_id: Option<u16>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub product_id: Option<u16>,
  pub has_permission: bool,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub device_path: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub mount_point: Option<String>,
  pub status: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestUsbMtpPermissionResponse {
  pub granted: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncUsbMtpRequest {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub force_pull_all: Option<bool>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub existing_hashes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncUsbMtpResponse {
  pub success: bool,
  pub status_message: String,
  pub file_count: usize,
  pub total_bytes: u64,
  pub fit_files: Option<Vec<String>>,
  pub file_names: Option<Vec<String>>,
}


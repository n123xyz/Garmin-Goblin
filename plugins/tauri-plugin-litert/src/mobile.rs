use serde::de::DeserializeOwned;
use tauri::{
  plugin::{PluginApi, PluginHandle},
  AppHandle, Runtime,
};

use crate::models::*;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_litert);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
  _app: &AppHandle<R>,
  api: PluginApi<R, C>,
) -> crate::Result<Litert<R>> {
  #[cfg(target_os = "android")]
  let handle = api.register_android_plugin("com.plugin.litert", "LitertPlugin")?;
  #[cfg(target_os = "ios")]
  let handle = api.register_ios_plugin(init_plugin_litert)?;
  Ok(Litert(handle))
}

/// Access to the litert APIs.
pub struct Litert<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Litert<R> {
  pub fn init_model(&self, payload: InitModelRequest) -> crate::Result<InitModelResponse> {
    self
      .0
      .run_mobile_plugin("initModel", payload)
      .map_err(Into::into)
  }

  pub fn check_model_exists(&self, payload: CheckModelRequest) -> crate::Result<CheckModelResponse> {
    self
      .0
      .run_mobile_plugin("checkModelExists", payload)
      .map_err(Into::into)
  }

  pub fn download_model(&self, payload: DownloadModelRequest) -> crate::Result<DownloadModelResponse> {
    self
      .0
      .run_mobile_plugin("downloadModel", payload)
      .map_err(Into::into)
  }

  pub fn purge_model(&self, payload: PurgeModelRequest) -> crate::Result<()> {
    self
      .0
      .run_mobile_plugin("purgeModel", payload)
      .map_err(Into::into)
  }

  pub fn generate_chat(&self, payload: GenerateChatRequest) -> crate::Result<GenerateChatResponse> {
    self
      .0
      .run_mobile_plugin("generateChat", payload)
      .map_err(Into::into)
  }

  pub fn close_model(&self, payload: CloseModelRequest) -> crate::Result<CloseModelResponse> {
    self
      .0
      .run_mobile_plugin("closeModel", payload)
      .map_err(Into::into)
  }

  pub fn pick_gallery_image(&self) -> crate::Result<PickGalleryImageResponse> {
    self
      .0
      .run_mobile_plugin("pickGalleryImage", ())
      .map_err(Into::into)
  }

  pub fn take_camera_photo(&self) -> crate::Result<TakeCameraPhotoResponse> {
    self
      .0
      .run_mobile_plugin("takeCameraPhoto", ())
      .map_err(Into::into)
  }

  pub fn scan_ble_devices(&self) -> crate::Result<ScanBleDevicesResponse> {
    self
      .0
      .run_mobile_plugin("scanBleDevices", ())
      .map_err(Into::into)
  }

  pub fn sync_ble_device(&self, payload: SyncBleDeviceRequest) -> crate::Result<SyncBleDeviceResponse> {
    self
      .0
      .run_mobile_plugin("connectAndSyncGarminDevice", payload)
      .map_err(Into::into)
  }

  pub fn start_speech_recognition(&self) -> crate::Result<SpeechRecognitionResponse> {
    self
      .0
      .run_mobile_plugin("startSpeechRecognition", ())
      .map_err(Into::into)
  }

  pub fn stop_speech_recognition(&self) -> crate::Result<SpeechRecognitionResponse> {
    self
      .0
      .run_mobile_plugin("stopSpeechRecognition", ())
      .map_err(Into::into)
  }

  pub fn check_calendar_permission(&self) -> crate::Result<CalendarPermissionResponse> {
    self
      .0
      .run_mobile_plugin("checkCalendarPermission", ())
      .map_err(Into::into)
  }

  pub fn request_calendar_permission(&self) -> crate::Result<CalendarPermissionResponse> {
    self
      .0
      .run_mobile_plugin("requestCalendarPermission", ())
      .map_err(Into::into)
  }

  pub fn get_calendar_events(&self, payload: GetCalendarEventsRequest) -> crate::Result<GetCalendarEventsResponse> {
    self
      .0
      .run_mobile_plugin("getCalendarEvents", payload)
      .map_err(Into::into)
  }

  pub fn scan_usb_mtp_devices(&self) -> crate::Result<ScanUsbMtpResponse> {
    self
      .0
      .run_mobile_plugin("scanUsbMtpDevices", ())
      .map_err(Into::into)
  }

  pub fn request_usb_mtp_permission(&self) -> crate::Result<RequestUsbMtpPermissionResponse> {
    self
      .0
      .run_mobile_plugin("requestUsbMtpPermission", ())
      .map_err(Into::into)
  }

  pub fn sync_usb_mtp_device(&self, payload: SyncUsbMtpRequest) -> crate::Result<SyncUsbMtpResponse> {
    self
      .0
      .run_mobile_plugin("syncUsbMtpDevice", payload)
      .map_err(Into::into)
  }
}

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<Litert<R>> {
  Ok(Litert(app.clone()))
}

/// Access to the litert APIs.
pub struct Litert<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Litert<R> {
  pub fn init_model(&self, _payload: InitModelRequest) -> crate::Result<InitModelResponse> {
    Ok(InitModelResponse { success: false })
  }

  pub fn check_model_exists(&self, _payload: CheckModelRequest) -> crate::Result<CheckModelResponse> {
    Ok(CheckModelResponse { exists: false })
  }

  pub fn download_model(&self, _payload: DownloadModelRequest) -> crate::Result<DownloadModelResponse> {
    Ok(DownloadModelResponse { success: false })
  }

  pub fn purge_model(&self, _payload: PurgeModelRequest) -> crate::Result<()> {
    Ok(())
  }

  pub fn generate_chat(&self, _payload: GenerateChatRequest) -> crate::Result<GenerateChatResponse> {
    Ok(GenerateChatResponse { response: String::new() })
  }

  pub fn close_model(&self, _payload: CloseModelRequest) -> crate::Result<CloseModelResponse> {
    Ok(CloseModelResponse { success: false })
  }

  pub fn pick_gallery_image(&self) -> crate::Result<PickGalleryImageResponse> {
    Ok(PickGalleryImageResponse { path: String::new(), uri: None })
  }

  pub fn take_camera_photo(&self) -> crate::Result<TakeCameraPhotoResponse> {
    Ok(TakeCameraPhotoResponse { path: String::new(), uri: None })
  }

  pub fn scan_ble_devices(&self) -> crate::Result<ScanBleDevicesResponse> {
    Ok(ScanBleDevicesResponse {
      devices: vec![],
      status: "Desktop uses btleplug".to_string(),
    })
  }

  pub fn sync_ble_device(&self, _payload: SyncBleDeviceRequest) -> crate::Result<SyncBleDeviceResponse> {
    Ok(SyncBleDeviceResponse {
      success: true,
      battery_level: None,
      fit_files: None,
      synced_activities_count: 0,
      status_message: "Desktop sync completed".to_string(),
    })
  }

  pub fn start_speech_recognition(&self) -> crate::Result<SpeechRecognitionResponse> {
    Ok(SpeechRecognitionResponse { transcription: String::new() })
  }

  pub fn stop_speech_recognition(&self) -> crate::Result<SpeechRecognitionResponse> {
    Ok(SpeechRecognitionResponse { transcription: String::new() })
  }

  pub fn check_calendar_permission(&self) -> crate::Result<CalendarPermissionResponse> {
    Ok(CalendarPermissionResponse { granted: true })
  }

  pub fn request_calendar_permission(&self) -> crate::Result<CalendarPermissionResponse> {
    Ok(CalendarPermissionResponse { granted: true })
  }

  pub fn get_calendar_events(&self, _payload: GetCalendarEventsRequest) -> crate::Result<GetCalendarEventsResponse> {
    Ok(GetCalendarEventsResponse { events: vec![] })
  }

  pub fn scan_usb_mtp_devices(&self) -> crate::Result<ScanUsbMtpResponse> {
    let mounts = find_garmin_mounts();
    if let Some(m) = mounts.first() {
      Ok(ScanUsbMtpResponse {
        is_attached: true,
        device_name: "Garmin Watch (Mounted)".to_string(),
        vendor_id: Some(0x091e),
        product_id: None,
        has_permission: true,
        device_path: None,
        mount_point: Some(m.to_string_lossy().to_string()),
        status: format!("Garmin watch detected at {}", m.display()),
      })
    } else {
      Ok(ScanUsbMtpResponse {
        is_attached: false,
        device_name: String::new(),
        vendor_id: None,
        product_id: None,
        has_permission: false,
        device_path: None,
        mount_point: None,
        status: "No Garmin MTP device or filesystem mount detected".to_string(),
      })
    }
  }

  pub fn request_usb_mtp_permission(&self) -> crate::Result<RequestUsbMtpPermissionResponse> {
    Ok(RequestUsbMtpPermissionResponse { granted: true })
  }

  pub fn sync_usb_mtp_device(&self, payload: SyncUsbMtpRequest) -> crate::Result<SyncUsbMtpResponse> {
    use base64::Engine;
    use sha2::{Digest, Sha256};

    let mounts = find_garmin_mounts();
    if mounts.is_empty() {
      return Ok(SyncUsbMtpResponse {
        success: false,
        status_message: "No connected Garmin watch mount found. Ensure watch is connected via USB and unlocked.".to_string(),
        file_count: 0,
        total_bytes: 0,
        fit_files: None,
        file_names: None,
      });
    }

    let existing_hashes: std::collections::HashSet<String> = payload.existing_hashes
      .unwrap_or_default()
      .into_iter()
      .collect();

    let mut fit_files_b64 = Vec::new();
    let mut fit_file_names = Vec::new();
    let mut total_bytes = 0u64;

    for garmin_dir in mounts {
      for sub in &["ACTIVITY", "Activity", "activity", "MONITOR", "Monitor", "monitor", "SLEEP", "Sleep", "sleep"] {
        let dir = garmin_dir.join(sub);
        if !dir.is_dir() {
          continue;
        }
        if let Ok(entries) = std::fs::read_dir(&dir) {
          for entry in entries.flatten() {
            let path = entry.path();
            let is_fit = path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("fit"));
            if is_fit && path.is_file() {
              if let Ok(bytes) = std::fs::read(&path) {
                if bytes.is_empty() {
                  continue;
                }
                let mut hasher = Sha256::new();
                hasher.update(&bytes);
                let hash_str: String = hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect();
                if !existing_hashes.contains(&hash_str) {
                  let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                  let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("workout.fit").to_string();
                  total_bytes += bytes.len() as u64;
                  fit_files_b64.push(b64);
                  fit_file_names.push(name);
                }
              }
            }
          }
        }
      }
    }

    let count = fit_files_b64.len();
    Ok(SyncUsbMtpResponse {
      success: true,
      status_message: format!("Successfully synced {} new FIT file(s) via USB ({:.1} KB)", count, total_bytes as f64 / 1024.0),
      file_count: count,
      total_bytes,
      fit_files: Some(fit_files_b64),
      file_names: Some(fit_file_names),
    })
  }
}

fn find_garmin_mounts() -> Vec<std::path::PathBuf> {
  use std::path::{Path, PathBuf};
  let mut mounts = Vec::new();

  // 1. Linux GVFS MTP paths
  let uid = std::env::var("UID").unwrap_or_else(|_| "1000".to_string());
  let gvfs_dir = PathBuf::from(format!("/run/user/{}/gvfs", uid));
  if gvfs_dir.exists() {
    if let Ok(entries) = std::fs::read_dir(&gvfs_dir) {
      for entry in entries.flatten() {
        let p = entry.path();
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.starts_with("mtp:host=") {
          for sub in &["GARMIN", "Garmin", "Internal Storage/GARMIN", "Internal storage/GARMIN", "Primary/GARMIN"] {
            let check = p.join(sub);
            if check.is_dir() && !mounts.contains(&check) {
              mounts.push(check);
            }
          }
        }
      }
    }
  }

  // 2. Standard Linux / Unix mounts
  for root in &[Path::new("/media"), Path::new("/run/media"), Path::new("/mnt"), Path::new("/Volumes")] {
    if root.exists() {
      if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
          let p = entry.path();
          if p.is_dir() {
            for sub in &["GARMIN", "Garmin"] {
              let check = p.join(sub);
              if check.is_dir() && !mounts.contains(&check) {
                mounts.push(check);
              }
            }
          }
        }
      }
    }
  }

  // 3. Filter candidate paths to ensure they contain Garmin indicators
  mounts
    .into_iter()
    .filter(|dir| {
      dir.join("ACTIVITY").is_dir()
        || dir.join("Activity").is_dir()
        || dir.join("activity").is_dir()
        || dir.join("MONITOR").is_dir()
        || dir.join("GarminDevice.xml").is_file()
        || dir.join("garmindevice.xml").is_file()
        || dir.join("device.fit").is_file()
    })
    .collect()
}

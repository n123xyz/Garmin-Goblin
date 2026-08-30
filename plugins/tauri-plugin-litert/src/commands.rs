use tauri::{AppHandle, command, Runtime};

use crate::models::*;
use crate::Result;
use crate::LitertExt;

#[command]
pub(crate) async fn init_model<R: Runtime>(
    app: AppHandle<R>,
    payload: InitModelRequest,
) -> Result<InitModelResponse> {
    app.litert().init_model(payload)
}

#[command]
pub(crate) async fn check_model_exists<R: Runtime>(
    app: AppHandle<R>,
    payload: CheckModelRequest,
) -> Result<CheckModelResponse> {
    app.litert().check_model_exists(payload)
}

#[command]
pub(crate) async fn download_model<R: Runtime>(
    app: AppHandle<R>,
    payload: DownloadModelRequest,
) -> Result<DownloadModelResponse> {
    app.litert().download_model(payload)
}

#[command]
pub(crate) async fn purge_model<R: Runtime>(
    app: AppHandle<R>,
    payload: PurgeModelRequest,
) -> Result<()> {
    app.litert().purge_model(payload)
}

#[command]
pub(crate) async fn generate_chat<R: Runtime>(
    app: AppHandle<R>,
    payload: GenerateChatRequest,
) -> Result<GenerateChatResponse> {
    app.litert().generate_chat(payload)
}

#[command]
pub(crate) async fn close_model<R: Runtime>(
    app: AppHandle<R>,
    payload: CloseModelRequest,
) -> Result<CloseModelResponse> {
    app.litert().close_model(payload)
}

#[command]
pub(crate) async fn pick_gallery_image<R: Runtime>(
    app: AppHandle<R>,
) -> Result<PickGalleryImageResponse> {
    app.litert().pick_gallery_image()
}

#[command]
pub(crate) async fn take_camera_photo<R: Runtime>(
    app: AppHandle<R>,
) -> Result<TakeCameraPhotoResponse> {
    app.litert().take_camera_photo()
}

#[command]
pub(crate) async fn scan_ble_devices<R: Runtime>(
    app: AppHandle<R>,
) -> Result<ScanBleDevicesResponse> {
    app.litert().scan_ble_devices()
}

#[command]
pub(crate) async fn sync_ble_device<R: Runtime>(
    app: AppHandle<R>,
    payload: Option<SyncBleDeviceRequest>,
    mac_address: Option<String>,
) -> Result<SyncBleDeviceResponse> {
    let req = if let Some(p) = payload {
        p
    } else if let Some(mac) = mac_address {
        SyncBleDeviceRequest {
            mac_address: mac,
            force_pull_all: None,
        }
    } else {
        return Err(crate::Error::Generic("MAC address required".to_string()));
    };
    app.litert().sync_ble_device(req)
}

#[command]
pub(crate) async fn start_speech_recognition<R: Runtime>(
    app: AppHandle<R>,
) -> Result<SpeechRecognitionResponse> {
    app.litert().start_speech_recognition()
}

#[command]
pub(crate) async fn stop_speech_recognition<R: Runtime>(
    app: AppHandle<R>,
) -> Result<SpeechRecognitionResponse> {
    app.litert().stop_speech_recognition()
}

#[command]
pub(crate) async fn check_calendar_permission<R: Runtime>(
    app: AppHandle<R>,
) -> Result<CalendarPermissionResponse> {
    app.litert().check_calendar_permission()
}

#[command]
pub(crate) async fn request_calendar_permission<R: Runtime>(
    app: AppHandle<R>,
) -> Result<CalendarPermissionResponse> {
    app.litert().request_calendar_permission()
}

#[command]
pub(crate) async fn get_calendar_events<R: Runtime>(
    app: AppHandle<R>,
    payload: Option<GetCalendarEventsRequest>,
) -> Result<GetCalendarEventsResponse> {
    let req = payload.unwrap_or_default();
    app.litert().get_calendar_events(req)
}

#[command]
pub(crate) async fn scan_usb_mtp_devices<R: Runtime>(
    app: AppHandle<R>,
) -> Result<ScanUsbMtpResponse> {
    app.litert().scan_usb_mtp_devices()
}

#[command]
pub(crate) async fn request_usb_mtp_permission<R: Runtime>(
    app: AppHandle<R>,
) -> Result<RequestUsbMtpPermissionResponse> {
    app.litert().request_usb_mtp_permission()
}

#[command]
pub(crate) async fn sync_usb_mtp_device<R: Runtime>(
    app: AppHandle<R>,
    payload: Option<SyncUsbMtpRequest>,
) -> Result<SyncUsbMtpResponse> {
    let req = payload.unwrap_or_default();
    app.litert().sync_usb_mtp_device(req)
}

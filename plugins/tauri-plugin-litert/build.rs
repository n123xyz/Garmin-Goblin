const COMMANDS: &[&str] = &[
  "init_model", "generate_chat", "close_model", "check_model_exists", "download_model",
  "purge_model", "pick_gallery_image", "take_camera_photo", "scan_ble_devices", "sync_ble_device",
  "start_speech_recognition", "stop_speech_recognition", "check_calendar_permission",
  "request_calendar_permission", "get_calendar_events",
  "scan_usb_mtp_devices", "request_usb_mtp_permission", "sync_usb_mtp_device"
];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .android_path("android")
    .ios_path("ios")
    .build();
}

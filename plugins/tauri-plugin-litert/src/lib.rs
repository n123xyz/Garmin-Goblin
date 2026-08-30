use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::Litert;
#[cfg(mobile)]
use mobile::Litert;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the litert APIs.
pub trait LitertExt<R: Runtime> {
  fn litert(&self) -> &Litert<R>;
}

impl<R: Runtime, T: Manager<R>> crate::LitertExt<R> for T {
  fn litert(&self) -> &Litert<R> {
    self.state::<Litert<R>>().inner()
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("litert")
    .invoke_handler(tauri::generate_handler![
        commands::init_model,
        commands::check_model_exists,
        commands::download_model,
        commands::purge_model,
        commands::generate_chat,
        commands::close_model,
        commands::pick_gallery_image,
        commands::take_camera_photo,
        commands::scan_ble_devices,
        commands::sync_ble_device,
        commands::start_speech_recognition,
        commands::stop_speech_recognition,
        commands::check_calendar_permission,
        commands::request_calendar_permission,
        commands::get_calendar_events,
        commands::scan_usb_mtp_devices,
        commands::request_usb_mtp_permission,
        commands::sync_usb_mtp_device
    ])
    .setup(|app, api| {
      #[cfg(mobile)]
      let litert = mobile::init(app, api)?;
      #[cfg(desktop)]
      let litert = desktop::init(app, api)?;
      app.manage(litert);
      Ok(())
    })
    .build()
}

use crate::services::fit_parser::{ingest_parsed_fit, parse_fit_bytes};
use crate::services::garmin::GarminActivity;
use base64::prelude::*;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Runtime};
use tauri_plugin_litert::LitertExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarminUsbDeviceInfo {
    pub is_attached: bool,
    pub device_name: String,
    pub has_permission: bool,
    pub mount_point: Option<String>,
    pub status_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarminMtpSyncSummary {
    pub success: bool,
    pub device_name: String,
    pub synced_count: usize,
    pub total_bytes: u64,
    pub xp_earned: i64,
    pub gold_earned: i64,
    pub activities: Vec<GarminActivity>,
    pub status_message: String,
    pub last_sync_timestamp: String,
}

/// Scan for connected Garmin watch via USB MTP (Android USB Host or Desktop Mount)
pub async fn scan_usb_garmin<R: Runtime>(app: &AppHandle<R>) -> Result<GarminUsbDeviceInfo, String> {
    let res = app
        .litert()
        .scan_usb_mtp_devices()
        .map_err(|e| format!("USB MTP scan error: {}", e))?;

    Ok(GarminUsbDeviceInfo {
        is_attached: res.is_attached,
        device_name: if res.device_name.is_empty() {
            "Garmin Watch".to_string()
        } else {
            res.device_name
        },
        has_permission: res.has_permission,
        mount_point: res.mount_point,
        status_message: res.status,
    })
}

/// Request Android USB Host permission for Garmin device
pub async fn request_usb_permission<R: Runtime>(app: &AppHandle<R>) -> Result<bool, String> {
    let res = app
        .litert()
        .request_usb_mtp_permission()
        .map_err(|e| format!("USB permission error: {}", e))?;
    Ok(res.granted)
}

/// Perform Full Garmin USB MTP Sync & Offload:
/// 1. Queries watch internal storage for /GARMIN/ACTIVITY, /GARMIN/MONITOR, /GARMIN/SLEEP.
/// 2. Downloads all new .FIT binary files.
/// 3. Parses workout records, laps, GPS tracks, steps, and sleep architecture.
/// 4. Ingests all telemetry directly into the local SQLite database.
/// 5. Awards Goblin XP and Gold rewards.
/// 6. Updates device sync timestamp.
pub async fn sync_garmin_usb_mtp<R: Runtime>(
    app: &AppHandle<R>,
    db: Arc<Mutex<Connection>>,
    force_pull: Option<bool>,
) -> Result<GarminMtpSyncSummary, String> {
    // 1. Load known hashes from database to prevent redundant re-syncing
    let existing_hashes: Vec<String> = {
        let conn = db.lock().map_err(|_| "DB lock failed")?;
        let mut stmt = conn
            .prepare("SELECT file_hash FROM garmin_synced_files")
            .ok();
        if let Some(ref mut s) = stmt {
            s.query_map([], |row| row.get(0))
                .ok()
                .map(|mapped| mapped.filter_map(Result::ok).collect())
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    };

    // 2. Invoke MTP device sync
    let req = tauri_plugin_litert::SyncUsbMtpRequest {
        force_pull_all: force_pull,
        existing_hashes: Some(existing_hashes),
    };

    let sync_res = app
        .litert()
        .sync_usb_mtp_device(req)
        .map_err(|e| format!("MTP sync failed: {}", e))?;

    if !sync_res.success {
        return Err(sync_res.status_message);
    }

    let raw_fit_files = sync_res.fit_files.unwrap_or_default();
    let file_names = sync_res.file_names.unwrap_or_default();
    let mut synced_activities = Vec::new();
    let mut total_xp = 0i64;
    let mut total_gold = 0i64;
    let mut newly_ingested_count = 0usize;

    // 3. Ingest each transferred FIT file
    for (i, fit_b64) in raw_fit_files.iter().enumerate() {
        let cleaned = fit_b64
            .trim()
            .trim_start_matches("data:application/octet-stream;base64,")
            .trim_start_matches("data:;base64,");

        if let Ok(file_bytes) = BASE64_STANDARD.decode(cleaned) {
            if let Ok(mut parsed) = parse_fit_bytes(&file_bytes) {
                if let Some(name) = file_names.get(i) {
                    if !name.is_empty() && !name.ends_with(".fit") && !name.ends_with(".FIT") {
                        parsed.title = name.clone();
                    }
                }

                if let Ok(import_res) = ingest_parsed_fit(db.clone(), parsed, Some(&file_bytes)) {
                    if import_res.success {
                        newly_ingested_count += 1;
                        let act = import_res.activity;
                        if act.duration_sec > 0 || act.distance_meters > 0.0 {
                            total_xp += import_res.xp_earned;
                            total_gold += import_res.gold_earned;
                            synced_activities.push(act);
                        }
                    }
                }
            }
        }
    }

    // 4. Update last sync time in database
    {
        let conn = db.lock().map_err(|_| "DB lock failed")?;
        let _ = conn.execute(
            "UPDATE garmin_devices SET 
                last_sync_time = CURRENT_TIMESTAMP,
                pending_fit_files = 0
             WHERE is_paired = 1",
            [],
        );
    }

    let now_str = chrono_time_string();
    let status_msg = if newly_ingested_count > 0 {
        format!(
            "MTP Sync complete! Ingested {} file(s) ({:.1} KB) • +{} XP • +{} Gold",
            newly_ingested_count,
            sync_res.total_bytes as f64 / 1024.0,
            total_xp,
            total_gold
        )
    } else {
        format!(
            "Garmin watch is up to date! Scanned USB storage ({:.1} KB checked).",
            sync_res.total_bytes as f64 / 1024.0
        )
    };

    Ok(GarminMtpSyncSummary {
        success: true,
        device_name: "Garmin Watch (USB MTP)".to_string(),
        synced_count: newly_ingested_count,
        total_bytes: sync_res.total_bytes,
        xp_earned: total_xp,
        gold_earned: total_gold,
        activities: synced_activities,
        status_message: status_msg,
        last_sync_timestamp: now_str,
    })
}

fn chrono_time_string() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let secs_in_day = now % 86400;
    let hours = secs_in_day / 3600;
    let minutes = (secs_in_day % 3600) / 60;
    let seconds = secs_in_day % 60;
    format!("Just now ({:02}:{:02}:{:02})", hours, minutes, seconds)
}

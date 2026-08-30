pub mod ai;
pub mod db;
pub mod services;

#[cfg(target_os = "android")]
use ai::litert_adapter::LiteRtAdapter;
use ai::ollama_adapter::OllamaAdapter;
use ai::{ChatMessage, ChatResponse, LlmProvider, VisionAnalysisResponse};
use rusqlite::Connection;
use services::garmin::{GarminActivity, GarminBiometrics};
use services::goblin::{GoblinProfile, GoblinQuest};
use services::medgemma::HealthInsight;
use services::settings::AppSettings;
use std::sync::{Arc, Mutex};
use tauri::{Manager, State};

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub ai: Arc<dyn LlmProvider + Send + Sync>,
}

// ----------------------------------------------------
// Biometric Commands
// ----------------------------------------------------

#[tauri::command]
async fn get_latest_biometrics(
    state: State<'_, AppState>,
) -> Result<GarminBiometrics, String> {
    services::garmin::get_latest_biometrics(state.db.clone())
}

#[tauri::command]
async fn get_biometrics_history(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<GarminBiometrics>, String> {
    services::garmin::get_biometrics_history(state.db.clone(), limit.unwrap_or(30))
}

#[tauri::command]
async fn update_biometrics(
    state: State<'_, AppState>,
    biometrics: GarminBiometrics,
) -> Result<GarminBiometrics, String> {
    services::garmin::update_biometrics(state.db.clone(), biometrics)
}

#[tauri::command]
async fn get_activities(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<GarminActivity>, String> {
    services::garmin::get_activities(state.db.clone(), limit.unwrap_or(20))
}

#[tauri::command]
async fn log_activity(
    state: State<'_, AppState>,
    activity: GarminActivity,
) -> Result<GarminActivity, String> {
    services::garmin::log_activity(state.db.clone(), activity)
}

#[tauri::command]
async fn sync_garmin_activity(
    state: State<'_, AppState>,
    activity: GarminActivity,
) -> Result<services::garmin_ble::GarminActivitySyncResult, String> {
    services::garmin_ble::ingest_activity_sync(state.db.clone(), activity)
}

#[tauri::command]
async fn cloak_gps_track(
    coords: Vec<[f64; 2]>,
    privacy_radius_meters: Option<f64>,
    enable_dp_noise: Option<bool>,
) -> Result<services::privacy::PrivacyCloakedRoute, String> {
    Ok(services::privacy::cloak_gps_route(
        &coords,
        privacy_radius_meters.unwrap_or(300.0),
        enable_dp_noise.unwrap_or(true),
    ))
}

#[tauri::command]
async fn scan_garmin_devices(
    state: State<'_, AppState>,
) -> Result<Vec<services::garmin_ble::GarminDeviceInfo>, String> {
    services::garmin_ble::scan_garmin_ble_devices(state.db.clone()).await
}

#[tauri::command]
async fn pair_garmin_device(
    state: State<'_, AppState>,
    device_id: String,
    device_name: String,
    mac_address: String,
) -> Result<services::garmin_ble::GarminDeviceInfo, String> {
    services::garmin_ble::pair_garmin_ble_device(state.db.clone(), device_id, device_name, mac_address).await
}

#[tauri::command]
async fn unpair_garmin_device(
    state: State<'_, AppState>,
) -> Result<bool, String> {
    services::garmin_ble::unpair_garmin_ble_device(state.db.clone())
}

#[tauri::command]
async fn get_garmin_device_status(
    state: State<'_, AppState>,
) -> Result<Option<services::garmin_ble::GarminDeviceInfo>, String> {
    services::garmin_ble::get_garmin_device_status(state.db.clone())
}

#[tauri::command]
async fn sync_and_offload_garmin(
    state: State<'_, AppState>,
    battery_level: Option<u8>,
) -> Result<services::garmin_ble::GarminOffloadSummary, String> {
    services::garmin_ble::sync_and_offload_device(state.db.clone(), battery_level).await
}

#[tauri::command]
async fn scan_usb_mtp_status(
    app: tauri::AppHandle,
) -> Result<services::garmin_mtp::GarminUsbDeviceInfo, String> {
    services::garmin_mtp::scan_usb_garmin(&app).await
}

#[tauri::command]
async fn request_usb_mtp_permission(
    app: tauri::AppHandle,
) -> Result<bool, String> {
    services::garmin_mtp::request_usb_permission(&app).await
}

#[tauri::command]
async fn sync_garmin_mtp_device(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    force_pull: Option<bool>,
) -> Result<services::garmin_mtp::GarminMtpSyncSummary, String> {
    services::garmin_mtp::sync_garmin_usb_mtp(&app, state.db.clone(), force_pull).await
}

#[tauri::command]
async fn import_fit_file(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<services::fit_parser::FitImportResult, String> {
    let file_bytes = std::fs::read(&file_path)
        .map_err(|e| format!("Failed to read FIT file at {}: {}", file_path, e))?;
    let parsed = services::fit_parser::parse_fit_bytes(&file_bytes)?;
    services::fit_parser::ingest_parsed_fit(state.db.clone(), parsed, Some(&file_bytes))
}

#[tauri::command]
async fn import_fit_base64(
    state: State<'_, AppState>,
    base64_data: String,
    file_name: Option<String>,
) -> Result<services::fit_parser::FitImportResult, String> {
    use base64::prelude::*;
    let cleaned = base64_data.trim()
        .trim_start_matches("data:application/octet-stream;base64,")
        .trim_start_matches("data:;base64,");
    let file_bytes = BASE64_STANDARD.decode(cleaned)
        .map_err(|e| {
            let err = format!("Invalid base64 encoding: {}", e);
            eprintln!("[GarminGoblin] {}", err);
            err
        })?;
    
    eprintln!("[GarminGoblin] import_fit_base64: received {} bytes of FIT binary", file_bytes.len());
    let mut parsed = services::fit_parser::parse_fit_bytes(&file_bytes)
        .map_err(|e| {
            eprintln!("[GarminGoblin] parse_fit_bytes error: {}", e);
            e
        })?;
    
    eprintln!("[GarminGoblin] parse_fit_bytes succeeded: title='{}', sport='{}', dur={}s, dist={}m, kcal={}, steps={:?}",
        parsed.title, parsed.sport, parsed.duration_sec, parsed.distance_meters, parsed.calories, parsed.total_steps);

    if let Some(name) = file_name {
        if !name.is_empty() && !name.ends_with(".fit") && !name.ends_with(".FIT") {
            parsed.title = name;
        }
    }
    services::fit_parser::ingest_parsed_fit(state.db.clone(), parsed, Some(&file_bytes))
}

#[tauri::command]
async fn fetch_live_garmin_weather(
    lat: f64,
    lon: f64,
    location_name: Option<String>,
    privacy_cloak: Option<bool>,
) -> Result<services::garmin_ble::GarminWeatherSyncPacket, String> {
    services::garmin_ble::fetch_open_meteo_weather(
        lat,
        lon,
        location_name,
        privacy_cloak.unwrap_or(true),
    )
    .await
}

#[tauri::command]
async fn sync_garmin_weather(
    temp_c: f32,
    condition_code: Option<u8>,
    location_name: Option<String>,
    wind_speed_ms: Option<f32>,
    humidity: Option<u8>,
    privacy_cloak: Option<bool>,
) -> Result<services::garmin_ble::GarminWeatherSyncPacket, String> {
    let cond = match condition_code.unwrap_or(0) {
        0 => services::garmin_ble::GarminCondition::Clear,
        1 => services::garmin_ble::GarminCondition::PartlyCloudy,
        2 => services::garmin_ble::GarminCondition::MostlyCloudy,
        3 => services::garmin_ble::GarminCondition::Rain,
        4 => services::garmin_ble::GarminCondition::Snow,
        5 => services::garmin_ble::GarminCondition::Windy,
        6 => services::garmin_ble::GarminCondition::Thunderstorms,
        _ => services::garmin_ble::GarminCondition::Clear,
    };

    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let hourly: Vec<services::garmin_ble::GarminHourlyForecast> = (0..12)
        .map(|i| services::garmin_ble::GarminHourlyForecast {
            hour_offset: i as u8,
            timestamp: now_ts + (i as i64 * 3600),
            temp_c: temp_c + (i as f32 * 0.4) - 2.0,
            feels_like_c: temp_c + (i as f32 * 0.4) - 2.0,
            condition: cond,
            pop_percent: (i as u8 * 5).min(80),
            wind_speed_ms: wind_speed_ms.unwrap_or(4.2),
            wind_deg: 180.0,
            humidity: humidity.unwrap_or(55),
        })
        .collect();

    let daily = vec![
        services::garmin_ble::GarminDailyForecast {
            day_offset: 0,
            timestamp: now_ts,
            condition: cond,
            pop_percent: 10,
            temp_min: temp_c - 5.0,
            temp_max: temp_c + 3.0,
        },
        services::garmin_ble::GarminDailyForecast {
            day_offset: 1,
            timestamp: now_ts + 86400,
            condition: services::garmin_ble::GarminCondition::PartlyCloudy,
            pop_percent: 20,
            temp_min: temp_c - 4.0,
            temp_max: temp_c + 4.0,
        },
        services::garmin_ble::GarminDailyForecast {
            day_offset: 2,
            timestamp: now_ts + (86400 * 2),
            condition: services::garmin_ble::GarminCondition::Clear,
            pop_percent: 0,
            temp_min: temp_c - 3.0,
            temp_max: temp_c + 5.0,
        },
    ];

    let packet = services::garmin_ble::build_full_garmin_weather_fit(
        37.7749,
        -122.4194,
        &location_name.unwrap_or_else(|| "Goblin Peak".to_string()),
        privacy_cloak.unwrap_or(true),
        temp_c,
        temp_c,
        cond,
        wind_speed_ms.unwrap_or(4.2),
        180.0,
        humidity.unwrap_or(55),
        temp_c - 5.0,
        temp_c + 3.0,
        &hourly,
        &daily,
        Some(34),
        Some(8.2),
        Some(14.5),
    );

    Ok(packet)
}

// ----------------------------------------------------
// Goblin & Quest Commands
// ----------------------------------------------------

#[tauri::command]
async fn get_goblin_profile(state: State<'_, AppState>) -> Result<GoblinProfile, String> {
    services::goblin::get_goblin_profile(state.db.clone())
}

#[tauri::command]
async fn get_quests(state: State<'_, AppState>) -> Result<Vec<GoblinQuest>, String> {
    services::goblin::get_quests(state.db.clone())
}

#[tauri::command]
async fn claim_quest_reward(
    state: State<'_, AppState>,
    quest_id: i64,
) -> Result<GoblinProfile, String> {
    services::goblin::claim_quest_reward(state.db.clone(), quest_id)
}

#[tauri::command]
async fn chat_with_goblin(
    state: State<'_, AppState>,
    history: Vec<ChatMessage>,
    image_uri: Option<String>,
    audio_base64: Option<String>,
    mode: Option<String>,
) -> Result<ChatResponse, String> {
    let biometrics = services::garmin::get_latest_biometrics(state.db.clone())?;
    let goblin = services::goblin::get_goblin_profile(state.db.clone())?;

    if mode.as_deref() == Some("therapy") {
        let history_bio = services::garmin::get_biometrics_history(state.db.clone(), 7).unwrap_or_default();
        state.ai.generate_therapy_chat(history, biometrics, history_bio, goblin).await
    } else {
        state
            .ai
            .generate_goblin_chat(history, biometrics, goblin, image_uri, audio_base64)
            .await
    }
}

#[tauri::command]
async fn chat_with_therapy_goblin(
    state: State<'_, AppState>,
    history: Vec<ChatMessage>,
) -> Result<ChatResponse, String> {
    let biometrics = services::garmin::get_latest_biometrics(state.db.clone())?;
    let history_bio = services::garmin::get_biometrics_history(state.db.clone(), 7).unwrap_or_default();
    let goblin = services::goblin::get_goblin_profile(state.db.clone())?;

    state
        .ai
        .generate_therapy_chat(history, biometrics, history_bio, goblin)
        .await
}

#[tauri::command]
async fn reframe_cbt_thought(
    state: State<'_, AppState>,
    situation: String,
    automatic_thought: String,
    distortions: Vec<String>,
) -> Result<services::therapy::CbtReframeResult, String> {
    let biometrics = services::garmin::get_latest_biometrics(state.db.clone())?;
    let history_bio = services::garmin::get_biometrics_history(state.db.clone(), 7).unwrap_or_default();
    state
        .ai
        .generate_cbt_reframe(situation, automatic_thought, distortions, biometrics, history_bio)
        .await
}

#[tauri::command]
async fn save_cbt_thought_record(
    state: State<'_, AppState>,
    record: services::therapy::CbtThoughtRecordInput,
) -> Result<services::therapy::CbtThoughtRecord, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    services::therapy::TherapyService::save_cbt_record(&conn, record)
}

#[tauri::command]
async fn get_cbt_thought_records(
    state: State<'_, AppState>,
    date: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<services::therapy::CbtThoughtRecord>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    services::therapy::TherapyService::get_cbt_records(&conn, date, limit.unwrap_or(30))
}

#[tauri::command]
async fn delete_cbt_thought_record(
    state: State<'_, AppState>,
    id: i64,
) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    services::therapy::TherapyService::delete_cbt_record(&conn, id)
}

#[tauri::command]
async fn get_chat_history(
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().map_err(|_| "DB lock failed")?;
        let mut stmt = conn
            .prepare("SELECT role, content, goblin_mood, audio_base64 FROM goblin_chat_history ORDER BY id ASC")
            .map_err(|e| e.to_string())?;

        let iter = stmt
            .query_map([], |row| {
                let role: String = row.get(0)?;
                let content: String = row.get(1)?;
                let goblin_mood: Option<String> = row.get(2)?;
                let audio_base64: Option<String> = row.get(3)?;
                Ok((role, content, goblin_mood, audio_base64))
            })
            .map_err(|e| e.to_string())?;

        let mut history = Vec::new();
        for item in iter {
            let (role, content, goblin_mood, audio_base64) = item.map_err(|e| e.to_string())?;
            history.push(serde_json::json!({
                "role": role,
                "content": content,
                "goblinMood": goblin_mood,
                "audioBase64": audio_base64
            }));
        }
        Ok(history)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn save_chat_message(
    state: State<'_, AppState>,
    role: String,
    content: String,
    goblin_mood: Option<String>,
    audio_base64: Option<String>,
) -> Result<(), String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().map_err(|_| "DB lock failed")?;
        conn.execute(
            "INSERT INTO goblin_chat_history (role, content, goblin_mood, audio_base64) VALUES (?1, ?2, ?3, ?4)",
            (&role, &content, &goblin_mood, &audio_base64),
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn clear_chat_history(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().map_err(|_| "DB lock failed")?;
        conn.execute("DELETE FROM goblin_chat_history", [])
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

// ----------------------------------------------------
// MedGemma & Multimodal OCR Commands
// ----------------------------------------------------

#[tauri::command]
async fn generate_medgemma_health_insight(
    state: State<'_, AppState>,
) -> Result<HealthInsight, String> {
    let biometrics = services::garmin::get_latest_biometrics(state.db.clone())?;
    let activities = services::garmin::get_activities(state.db.clone(), 5)?;

    let insight = state
        .ai
        .generate_medgemma_health_insight(biometrics, activities)
        .await?;

    let _ = services::medgemma::save_health_insight(state.db.clone(), &insight);
    Ok(insight)
}

#[tauri::command]
async fn get_latest_health_insights(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<HealthInsight>, String> {
    services::medgemma::get_latest_health_insights(state.db.clone(), limit.unwrap_or(10))
}

#[tauri::command]
async fn analyze_food_ingestion(
    state: State<'_, AppState>,
    image_uri: Option<String>,
    user_notes: Option<String>,
    audio_base64: Option<String>,
) -> Result<VisionAnalysisResponse, String> {
    let result = state
        .ai
        .generate_food_ingestion_analysis(image_uri, user_notes, audio_base64)
        .await?;

    // If XP was awarded, update Goblin stats!
    if result.goblin_xp_awarded > 0 || result.goblin_gold_awarded > 0 {
        let _ = services::goblin::add_xp_and_gold(
            state.db.clone(),
            result.goblin_xp_awarded,
            result.goblin_gold_awarded,
        );
    }

    Ok(result)
}

#[tauri::command]
async fn analyze_vision_ocr(
    state: State<'_, AppState>,
    image_uri: Option<String>,
    _prompt_type: Option<String>,
    user_notes: Option<String>,
    audio_base64: Option<String>,
) -> Result<VisionAnalysisResponse, String> {
    analyze_food_ingestion(state, image_uri, user_notes, audio_base64).await
}

#[tauri::command]
async fn save_calendar_events(
    state: State<'_, AppState>,
    events: Vec<services::calendar::CalendarEventItem>,
) -> Result<usize, String> {
    let conn = state.db.lock().map_err(|_| "DB lock failed")?;
    services::calendar::CalendarService::save_calendar_events(&conn, &events)
}

#[tauri::command]
async fn get_calendar_events_for_date(
    state: State<'_, AppState>,
    date: String,
) -> Result<Vec<services::calendar::CalendarEventItem>, String> {
    let conn = state.db.lock().map_err(|_| "DB lock failed")?;
    services::calendar::CalendarService::get_events_for_date(&conn, &date)
}

#[tauri::command]
async fn get_calendar_events_for_range(
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<services::calendar::CalendarEventItem>, String> {
    let conn = state.db.lock().map_err(|_| "DB lock failed")?;
    services::calendar::CalendarService::get_events_for_range(&conn, &start_date, &end_date)
}

// ----------------------------------------------------
// Settings & System Commands
// ----------------------------------------------------

#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    services::settings::get_settings(state.db.clone())
}

#[tauri::command]
async fn update_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    services::settings::update_settings(state.db.clone(), settings)
}

#[tauri::command]
async fn check_ollama_health(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.ai.check_health().await)
}

#[tauri::command]
async fn list_ollama_models(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state.ai.list_models().await
}

#[tauri::command]
async fn check_tokenizer_exists(app_handle: tauri::AppHandle) -> Result<bool, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    let tokenizer_path = app_dir.join("tokenizer.json");
    let config_path = app_dir.join("tokenizer_config.json");

    Ok(tokenizer_path.exists() && config_path.exists())
}

#[tauri::command]
async fn download_tokenizer(app_handle: tauri::AppHandle) -> Result<(), String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

    let tokenizer_path = app_dir.join("tokenizer.json");
    let config_path = app_dir.join("tokenizer_config.json");

    let client = reqwest::Client::new();

    let res = client
        .get("https://huggingface.co/google/gemma-4-E2B/resolve/main/tokenizer.json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch tokenizer.json: {}", e))?;
    let bytes = res.bytes().await.map_err(|e| e.to_string())?;
    std::fs::write(&tokenizer_path, bytes).map_err(|e| e.to_string())?;

    let res = client
        .get("https://huggingface.co/google/gemma-4-E2B/resolve/main/tokenizer_config.json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch tokenizer_config.json: {}", e))?;
    let bytes = res.bytes().await.map_err(|e| e.to_string())?;
    std::fs::write(&config_path, bytes).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn delete_tokenizer(app_handle: tauri::AppHandle) -> Result<(), String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    let _ = std::fs::remove_file(app_dir.join("tokenizer.json"));
    let _ = std::fs::remove_file(app_dir.join("tokenizer_config.json"));

    Ok(())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn generate_supertonic_tts(
    app: tauri::AppHandle,
    state: tauri::State<'_, tauri_plugin_supertonic::commands::SupertonicState>,
    app_state: tauri::State<'_, AppState>,
    text: String,
    lang: Option<String>,
    speed: Option<f32>,
    steps: Option<u32>,
    voice_style: Option<String>,
) -> Result<tauri_plugin_supertonic::GenerateTtsResponse, String> {
    let settings = services::settings::get_settings(app_state.db.clone())
        .map_err(|e| e.to_string())?;
    let voice_style = voice_style.unwrap_or(settings.tts_voice_style);
    let lang = lang.unwrap_or_else(|| "en".to_string());
    let speed = speed.unwrap_or(1.0);
    let steps = steps.unwrap_or(4);

    tauri_plugin_supertonic::commands::generate_supertonic_tts(
        app,
        state,
        tauri_plugin_supertonic::GenerateTtsRequest {
            text,
            lang,
            speed,
            steps,
            voice_style,
        },
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_food_logs(
    state: State<'_, AppState>,
    date: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<services::journal::FoodLog>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    services::journal::JournalService::get_food_logs(&conn, date, limit).map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_food_log(
    state: State<'_, AppState>,
    log: services::journal::FoodLogInput,
) -> Result<services::journal::FoodLog, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    services::journal::JournalService::save_food_log(&conn, log).map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_food_log(
    state: State<'_, AppState>,
    id: i64,
) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    services::journal::JournalService::delete_food_log(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_daily_nutrition_summary(
    state: State<'_, AppState>,
    date: String,
) -> Result<services::journal::DailyNutritionSummary, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    services::journal::JournalService::get_daily_nutrition_summary(&conn, date).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_emotional_logs(
    state: State<'_, AppState>,
    date: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<services::journal::EmotionalLog>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    services::journal::JournalService::get_emotional_logs(&conn, date, start_date, end_date).map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_emotional_log(
    state: State<'_, AppState>,
    log: services::journal::EmotionalLogInput,
) -> Result<services::journal::EmotionalLog, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    services::journal::JournalService::save_emotional_log(&conn, log).map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_emotional_log(
    state: State<'_, AppState>,
    id: i64,
) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    services::journal::JournalService::delete_emotional_log(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_calendar_month(
    state: State<'_, AppState>,
    year: i32,
    month: u32,
) -> Result<services::journal::CalendarMonthResponse, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    services::journal::JournalService::get_calendar_month(&conn, year, month).map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_carit_session(
    state: State<'_, AppState>,
    session: services::carit::CaritSessionRecord,
) -> Result<services::carit::CaritSessionRecord, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    services::carit::CaritService::save_carit_session(&conn, session).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_carit_sessions(
    state: State<'_, AppState>,
    date: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<services::carit::CaritSessionRecord>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(d) = date {
        services::carit::CaritService::get_carit_sessions_for_date(&conn, &d).map_err(|e| e.to_string())
    } else {
        services::carit::CaritService::get_recent_carit_sessions(&conn, limit.unwrap_or(20)).map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn get_today_carit_summary(
    state: State<'_, AppState>,
    date: String,
) -> Result<services::carit::TodayCaritSummary, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    services::carit::CaritService::get_today_carit_summary(&conn, &date).map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_facename_session(
    state: State<'_, AppState>,
    session: services::facename::FaceNameSessionRecord,
) -> Result<services::facename::FaceNameSessionRecord, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    services::facename::FaceNameService::save_facename_session(&conn, session).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_facename_sessions(
    state: State<'_, AppState>,
    date: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<services::facename::FaceNameSessionRecord>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(d) = date {
        services::facename::FaceNameService::get_facename_sessions_for_date(&conn, &d).map_err(|e| e.to_string())
    } else {
        services::facename::FaceNameService::get_recent_facename_sessions(&conn, limit.unwrap_or(20)).map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn get_today_facename_summary(
    state: State<'_, AppState>,
    date: String,
) -> Result<services::facename::TodayFaceNameSummary, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    services::facename::FaceNameService::get_today_facename_summary(&conn, &date).map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_vismotor_session(
    state: State<'_, AppState>,
    session: services::vismotor::VismotorSessionRecord,
) -> Result<services::vismotor::VismotorSessionRecord, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    services::vismotor::VismotorService::save_vismotor_session(&conn, session).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_vismotor_sessions(
    state: State<'_, AppState>,
    date: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<services::vismotor::VismotorSessionRecord>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(d) = date {
        services::vismotor::VismotorService::get_vismotor_sessions_for_date(&conn, &d).map_err(|e| e.to_string())
    } else {
        services::vismotor::VismotorService::get_recent_vismotor_sessions(&conn, limit.unwrap_or(20)).map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn get_today_vismotor_summary(
    state: State<'_, AppState>,
    date: String,
) -> Result<services::vismotor::TodayVismotorSummary, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    services::vismotor::VismotorService::get_today_vismotor_summary(&conn, &date).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_litert::init())
        .plugin(tauri_plugin_supertonic::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let conn = db::init_db(app.handle()).map_err(|e| {
                eprintln!("❌ CRITICAL: Failed to initialize database: {}", e);
                e
            })?;
            let db_arc = Arc::new(Mutex::new(conn));

            #[cfg(target_os = "android")]
            let ai_adapter: Arc<dyn LlmProvider + Send + Sync> = {
                let settings = services::settings::get_settings(db_arc.clone())
                    .unwrap_or_else(|_| AppSettings {
                        active_model: "gemma-4-E2B-it.litertlm".to_string(),
                        medgemma_model: "medgemma-1.5-4b-it.litertlm".to_string(),
                        huggingface_token: None,
                        litert_accelerator: "Auto".to_string(),
                        litert_max_tokens: 5000,
                        ollama_server_url: "http://localhost:11434".to_string(),
                        tts_voice_style: "voice_styles/M1.json".to_string(),
                        use_simulated_biometrics: true,
                        goblin_name: "Gribble".to_string(),
                        goblin_personality: "Tough Love Trainer".to_string(),
                    });

                match LiteRtAdapter::new(
                    app.handle().clone(),
                    settings.active_model,
                    settings.litert_accelerator,
                    settings.litert_max_tokens,
                ) {
                    Ok(adapter) => Arc::new(adapter),
                    Err(e) => {
                        eprintln!("[Garmin Goblin LiteRT] Fallback to Ollama: {}", e);
                        Arc::new(OllamaAdapter::new(db_arc.clone()))
                    }
                }
            };

            #[cfg(not(target_os = "android"))]
            let ai_adapter: Arc<dyn LlmProvider + Send + Sync> =
                Arc::new(OllamaAdapter::new(db_arc.clone()));

            app.manage(AppState {
                db: db_arc,
                ai: ai_adapter,
            });

            #[cfg(all(target_os = "linux", not(target_os = "android")))]
            {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.with_webview(|webview| {
                        use webkit2gtk::PermissionRequestExt;
                        use webkit2gtk::WebViewExt;
                        webview.inner().connect_permission_request(move |_, request| {
                            request.allow();
                            true
                        });
                    });
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_latest_biometrics,
            get_biometrics_history,
            update_biometrics,
            get_activities,
            log_activity,
            sync_garmin_activity,
            sync_garmin_weather,
            fetch_live_garmin_weather,
            scan_garmin_devices,
            pair_garmin_device,
            unpair_garmin_device,
            get_garmin_device_status,
            sync_and_offload_garmin,
            scan_usb_mtp_status,
            request_usb_mtp_permission,
            sync_garmin_mtp_device,
            import_fit_file,
            import_fit_base64,
            cloak_gps_track,
            get_goblin_profile,
            get_quests,
            claim_quest_reward,
            chat_with_goblin,
            get_chat_history,
            save_chat_message,
            clear_chat_history,
            generate_medgemma_health_insight,
            get_latest_health_insights,
            analyze_food_ingestion,
            analyze_vision_ocr,
            get_settings,
            update_settings,
            check_ollama_health,
            list_ollama_models,
            check_tokenizer_exists,
            download_tokenizer,
            delete_tokenizer,
            generate_supertonic_tts,
            get_food_logs,
            save_food_log,
            delete_food_log,
            get_daily_nutrition_summary,
            get_emotional_logs,
            save_emotional_log,
            delete_emotional_log,
            get_calendar_month,
            save_calendar_events,
            get_calendar_events_for_date,
            get_calendar_events_for_range,
            save_carit_session,
            get_carit_sessions,
            get_today_carit_summary,
            save_facename_session,
            get_facename_sessions,
            get_today_facename_summary,
            save_vismotor_session,
            get_vismotor_sessions,
            get_today_vismotor_summary,
            chat_with_therapy_goblin,
            reframe_cbt_thought,
            save_cbt_thought_record,
            get_cbt_thought_records,
            delete_cbt_thought_record,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Garmin Goblin application");
}

#[cfg(target_os = "android")]
#[no_mangle]
pub unsafe extern "C" fn Java_com_user_garmin_1goblin_MainActivity_initBtleplug(
    env: jni::JNIEnv,
    _class: jni::objects::JClass,
) {
    let _ = btleplug::platform::init(&env);
}

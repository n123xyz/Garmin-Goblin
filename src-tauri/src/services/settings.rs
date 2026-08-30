use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppSettings {
    pub active_model: String,
    pub medgemma_model: String,
    pub huggingface_token: Option<String>,
    pub litert_accelerator: String,
    pub litert_max_tokens: u32,
    pub ollama_server_url: String,
    pub tts_voice_style: String,
    pub use_simulated_biometrics: bool,
    pub goblin_name: String,
    pub goblin_personality: String,
}

pub fn get_settings(db: Arc<Mutex<Connection>>) -> Result<AppSettings, String> {
    let conn = db.lock().map_err(|_| "DB lock failed")?;

    let mut stmt = conn
        .prepare(
            "SELECT active_model, medgemma_model, huggingface_token, litert_accelerator, 
                    litert_max_tokens, ollama_server_url, tts_voice_style, 
                    use_simulated_biometrics, goblin_name, goblin_personality 
             FROM settings WHERE id = 1",
        )
        .map_err(|e| e.to_string())?;

    let settings = stmt
        .query_row([], |row| {
            Ok(AppSettings {
                active_model: row.get(0)?,
                medgemma_model: row.get(1).unwrap_or_else(|_| "medgemma-1.5-4b-it.litertlm".to_string()),
                huggingface_token: row.get(2)?,
                litert_accelerator: row.get(3).unwrap_or_else(|_| "Auto".to_string()),
                litert_max_tokens: row.get(4).unwrap_or(3000),
                ollama_server_url: row.get(5).unwrap_or_else(|_| "http://localhost:11434".to_string()),
                tts_voice_style: row.get(6).unwrap_or_else(|_| "voice_styles/M1.json".to_string()),
                use_simulated_biometrics: row.get(7).unwrap_or(true),
                goblin_name: row.get(8).unwrap_or_else(|_| "Gribble".to_string()),
                goblin_personality: row.get(9).unwrap_or_else(|_| "Tough Love Trainer".to_string()),
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(settings)
}

pub fn update_settings(db: Arc<Mutex<Connection>>, settings: AppSettings) -> Result<(), String> {
    let conn = db.lock().map_err(|_| "DB lock failed")?;

    conn.execute(
        "UPDATE settings SET 
            active_model = ?1, 
            medgemma_model = ?2,
            huggingface_token = ?3, 
            litert_accelerator = ?4, 
            litert_max_tokens = ?5, 
            ollama_server_url = ?6, 
            tts_voice_style = ?7, 
            use_simulated_biometrics = ?8,
            goblin_name = ?9,
            goblin_personality = ?10
         WHERE id = 1",
        (
            &settings.active_model,
            &settings.medgemma_model,
            &settings.huggingface_token,
            &settings.litert_accelerator,
            &settings.litert_max_tokens,
            &settings.ollama_server_url,
            &settings.tts_voice_style,
            &settings.use_simulated_biometrics,
            &settings.goblin_name,
            &settings.goblin_personality,
        ),
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

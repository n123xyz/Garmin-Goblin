use crate::ai::{
    ChatMessage, ChatResponse, LlmProvider, VisionAnalysisResponse,
};
use crate::services::garmin::{GarminActivity, GarminBiometrics};
use crate::services::goblin::GoblinProfile;
use crate::services::medgemma::HealthInsight;
use async_trait::async_trait;
use serde::Deserialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_litert::LitertExt;

pub struct LiteRtAdapter {
    app_handle: AppHandle,
    max_tokens: u32,
    model_path: String,
    accelerator: String,
    last_history: std::sync::Mutex<Option<Vec<ChatMessage>>>,
    is_initialized: std::sync::Arc<tokio::sync::Mutex<bool>>,
}

impl LiteRtAdapter {
    pub fn new(
        app_handle: AppHandle,
        model_path: String,
        accelerator: String,
        max_tokens: u32,
    ) -> Result<Self, String> {
        let is_initialized = std::sync::Arc::new(tokio::sync::Mutex::new(false));
        let instance = Self {
            app_handle: app_handle.clone(),
            max_tokens,
            model_path: model_path.clone(),
            accelerator: accelerator.clone(),
            last_history: std::sync::Mutex::new(None),
            is_initialized: is_initialized.clone(),
        };

        Ok(instance)
    }

    async fn ensure_initialized(&self) -> Result<(), String> {
        let mut initialized = self.is_initialized.lock().await;
        if *initialized {
            return Ok(());
        }

        let payload = tauri_plugin_litert::InitModelRequest {
            model_path: self.model_path.clone(),
            accelerator: self.accelerator.clone(),
            max_tokens: self.max_tokens,
        };

        let app_handle = self.app_handle.clone();
        let success_or_err = tokio::task::spawn_blocking(move || {
            match app_handle.litert().init_model(payload) {
                Ok(response) => {
                    if response.success {
                        Ok(())
                    } else {
                        Err("LiteRT plugin returned success=false".to_string())
                    }
                }
                Err(e) => Err(e.to_string()),
            }
        })
        .await
        .map_err(|e| format!("Task failed: {}", e))?;

        match success_or_err {
            Ok(_) => {
                *initialized = true;
                Ok(())
            }
            Err(e) => Err(format!("LiteRT Init Error: {}", e)),
        }
    }

    fn execute_json_generation<T: serde::de::DeserializeOwned>(
        &self,
        prompt: String,
        error_prefix: &str,
    ) -> Result<T, String> {
        if let Ok(mut last_hist_guard) = self.last_history.lock() {
            *last_hist_guard = None;
        }

        let max_attempts = 3;
        let mut attempts = 0;
        let gemma_prompt = format!("<start_of_turn>user\n{}<end_of_turn>\n<start_of_turn>model\n", prompt);

        while attempts < max_attempts {
            let payload = tauri_plugin_litert::GenerateChatRequest {
                prompt: gemma_prompt.clone(),
                reset: true,
                audio_base64: None,
                image_uri: None,
                system_instruction: None,
            };

            let response = match self.app_handle.litert().generate_chat(payload) {
                Ok(res) => res.response,
                Err(e) => {
                    attempts += 1;
                    eprintln!("{} Native Error on attempt {}: {}", error_prefix, attempts, e);
                    if attempts >= max_attempts {
                        return Err(format!("LiteRT inference error: {}", e));
                    }
                    continue;
                }
            };

            let content = response;
            let json_content = if let (Some(start), Some(end)) = (content.find('{'), content.rfind('}')) {
                &content[start..=end]
            } else {
                &content
            };

            match serde_json::from_str::<T>(json_content) {
                Ok(parsed) => return Ok(parsed),
                Err(e) => {
                    attempts += 1;
                    eprintln!(
                        "{} JSON Parse Error on attempt {}: {} \nRaw output: {}",
                        error_prefix, attempts, e, content
                    );
                    if attempts >= max_attempts {
                        return Err(format!("Model failed to produce valid JSON: {}", e));
                    }
                }
            }
        }
        Err("Failed to generate valid structured JSON after multiple attempts.".to_string())
    }
}

#[async_trait]
impl LlmProvider for LiteRtAdapter {
    async fn check_health(&self) -> bool {
        true
    }

    async fn list_models(&self) -> Result<Vec<String>, String> {
        Ok(vec![
            "gemma-4-E2B-it.litertlm".to_string(),
            "medgemma-1.5-4b-it.litertlm".to_string(),
        ])
    }

    async fn generate_goblin_chat(
        &self,
        history: Vec<ChatMessage>,
        biometrics: GarminBiometrics,
        goblin_profile: GoblinProfile,
        image_uri: Option<String>,
        audio_base64: Option<String>,
    ) -> Result<ChatResponse, String> {
        self.ensure_initialized().await?;

        let personality = if let Some(state) = self.app_handle.try_state::<crate::AppState>() {
            crate::services::settings::get_settings(state.db.clone())
                .map(|s| s.goblin_personality)
                .unwrap_or_else(|_| "Tough Love Fitness Goblin".to_string())
        } else {
            "Tough Love Fitness Goblin".to_string()
        };

        let system_prompt = super::build_goblin_chat_system_prompt(
            &goblin_profile.name,
            &personality,
            &biometrics,
            &goblin_profile,
        );

        let is_multimodal = image_uri.is_some() || audio_base64.is_some();
        let last_user_message = history.last().map(|m| m.content.clone()).unwrap_or_default();

        let prompt_text = if is_multimodal {
            if image_uri.is_some() {
                format!("<image>\n{}\nUser: {}", system_prompt, last_user_message)
            } else {
                format!("{}\nUser: {}", system_prompt, last_user_message)
            }
        } else {
            let mut formatted = format!("<start_of_turn>user\n{}\n\n", system_prompt);
            for msg in &history {
                let role = if msg.role == "assistant" { "model" } else { "user" };
                formatted.push_str(&format!("<start_of_turn>{}\n{}<end_of_turn>\n", role, msg.content));
            }
            formatted.push_str("<start_of_turn>model\n");
            formatted
        };

        let payload = tauri_plugin_litert::GenerateChatRequest {
            prompt: prompt_text,
            reset: true,
            audio_base64,
            image_uri,
            system_instruction: if is_multimodal { Some(system_prompt) } else { None },
        };

        let response = match self.app_handle.litert().generate_chat(payload) {
            Ok(res) => res.response,
            Err(e) => return Err(format!("LiteRT Chat error: {}", e)),
        };

        Ok(ChatResponse {
            response,
            goblin_mood: goblin_profile.mood,
            context_summary: None,
        })
    }

    async fn generate_medgemma_health_insight(
        &self,
        biometrics: GarminBiometrics,
        recent_activities: Vec<GarminActivity>,
    ) -> Result<HealthInsight, String> {
        self.ensure_initialized().await?;

        let (calendar_events, carit_summary, facename_summary, vismotor_summary) = if let Some(state) = self.app_handle.try_state::<crate::AppState>() {
            if let Ok(conn) = state.db.lock() {
                let events = crate::services::calendar::CalendarService::get_events_for_date(&conn, &biometrics.date).ok();
                let carit = crate::services::carit::CaritService::get_today_carit_summary(&conn, &biometrics.date).ok();
                let fname = crate::services::facename::FaceNameService::get_today_facename_summary(&conn, &biometrics.date).ok();
                let vm = crate::services::vismotor::VismotorService::get_today_vismotor_summary(&conn, &biometrics.date).ok();
                (events, carit, fname, vm)
            } else {
                (None, None, None, None)
            }
        } else {
            (None, None, None, None)
        };

        let prompt = super::build_medgemma_prompt(
            &biometrics,
            &recent_activities,
            calendar_events.as_deref(),
            carit_summary.as_ref(),
            facename_summary.as_ref(),
            vismotor_summary.as_ref(),
        );

        #[derive(Deserialize)]
        struct MedGemmaJson {
            readiness_score: i32,
            readiness_state: String,
            clinical_summary: String,
            goblin_reaction: String,
            actionable_tips: serde_json::Value,
        }

        match self.execute_json_generation::<MedGemmaJson>(prompt, "[MedGemma]") {
            Ok(parsed) => {
                let tips_str = match parsed.actionable_tips {
                    serde_json::Value::Array(arr) => arr
                        .into_iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                        .join("\n• "),
                    serde_json::Value::String(s) => s,
                    other => other.to_string(),
                };

                Ok(HealthInsight {
                    id: 0,
                    created_at: chrono::Utc::now().to_rfc3339(),
                    category: "MedGemma Clinical Readiness".to_string(),
                    readiness_score: parsed.readiness_score,
                    readiness_state: parsed.readiness_state,
                    clinical_summary: parsed.clinical_summary,
                    goblin_reaction: parsed.goblin_reaction,
                    actionable_tips: tips_str,
                })
            }
            Err(_) => {
                // Fallback to high-precision deterministic clinical model if local inference glitches
                Ok(crate::services::medgemma::generate_deterministic_insight(&biometrics))
            }
        }
    }

    async fn generate_food_ingestion_analysis(
        &self,
        image_uri: Option<String>,
        user_notes: Option<String>,
        audio_base64: Option<String>,
    ) -> Result<VisionAnalysisResponse, String> {
        self.ensure_initialized().await?;

        // Clear chat history memory
        if let Ok(mut last_hist_guard) = self.last_history.lock() {
            *last_hist_guard = None;
        }

        let has_image = image_uri.as_ref().map(|u| !u.trim().is_empty()).unwrap_or(false);
        let has_audio = audio_base64.as_ref().map(|u| !u.trim().is_empty()).unwrap_or(false);
        let base_prompt = super::build_food_ingestion_prompt(user_notes.as_deref(), has_image, has_audio);

        let prompt_text = if has_image {
            format!("<image>\n{}", base_prompt)
        } else {
            base_prompt
        };

        let system_instruction = if has_image || has_audio {
            "You are an expert on-device multimodal food, speech audio, nutrition facts label OCR, and dietary telemetry model.".to_string()
        } else {
            "You are an expert on-device dietary telemetry and clinical nutrition analysis model. Calculate realistic macronutrients and calories for the described meals and beverages.".to_string()
        };

        let max_attempts = 3;
        let mut attempts = 0;

        while attempts < max_attempts {
            let payload = tauri_plugin_litert::GenerateChatRequest {
                prompt: prompt_text.clone(),
                reset: true,
                audio_base64: audio_base64.clone(),
                image_uri: if has_image { image_uri.clone() } else { None },
                system_instruction: Some(system_instruction.clone()),
            };

            let response = match self.app_handle.litert().generate_chat(payload) {
                Ok(res) => res.response,
                Err(e) => {
                    attempts += 1;
                    eprintln!("[LiteRtAdapter] Food Ingestion native error on attempt {}: {}", attempts, e);
                    if attempts >= max_attempts {
                        return Err(format!("Android inference error: {}", e));
                    }
                    continue;
                }
            };

            let json_content = if let (Some(start), Some(end)) = (response.find('{'), response.rfind('}')) {
                &response[start..=end]
            } else {
                &response
            };

            match serde_json::from_str::<VisionAnalysisResponse>(json_content) {
                Ok(parsed) => return Ok(parsed),
                Err(e) => {
                    attempts += 1;
                    eprintln!(
                        "[LiteRtAdapter] JSON Parse error on attempt {}: {}. Raw output: {}",
                        attempts, e, response
                    );
                    if attempts >= max_attempts {
                        return Err(format!("Could not extract nutritional data from model response: {}. Output: {}", e, response));
                    }
                }
            }
        }

        Err("Failed to generate food ingestion analysis after multiple attempts.".to_string())
    }

    async fn generate_therapy_chat(
        &self,
        history: Vec<ChatMessage>,
        biometrics: GarminBiometrics,
        history_bio: Vec<GarminBiometrics>,
        goblin_profile: GoblinProfile,
    ) -> Result<ChatResponse, String> {
        let system_prompt = super::build_therapy_chat_system_prompt(
            &goblin_profile.name,
            &biometrics,
            &history_bio,
            &goblin_profile,
        );

        let mut formatted = format!("<start_of_turn>user\n{}\n\n", system_prompt);
        for msg in &history {
            let role = if msg.role == "assistant" { "model" } else { "user" };
            formatted.push_str(&format!("<start_of_turn>{}\n{}<end_of_turn>\n", role, msg.content));
        }
        formatted.push_str("<start_of_turn>model\n");

        if self.ensure_initialized().await.is_ok() {
            let payload = tauri_plugin_litert::GenerateChatRequest {
                prompt: formatted,
                reset: true,
                audio_base64: None,
                image_uri: None,
                system_instruction: None,
            };

            if let Ok(res) = self.app_handle.litert().generate_chat(payload) {
                return Ok(ChatResponse {
                    response: res.response,
                    goblin_mood: "Zen".to_string(),
                    context_summary: Some("Therapy Mode Session".to_string()),
                });
            }
        }

        let history_context = if !history_bio.is_empty() {
            let n = history_bio.len();
            let avg_stress: i32 = history_bio.iter().map(|b| b.stress_level).sum::<i32>() / (n as i32);
            format!("Looking across your last {} synced days (average stress: {}/100)", n, avg_stress)
        } else {
            format!("Looking at your last synced stress score ({}/100)", biometrics.stress_level)
        };

        let fallback = format!(
            "Greetings by the cavern hearth. I am {}, your Goblin Counselor. {}, let's take a slow, deep breath together. Whatever has been stirring up your mind, this is your safe haven. What would you like to unpack?",
            goblin_profile.name, history_context
        );

        Ok(ChatResponse {
            response: fallback,
            goblin_mood: "Zen".to_string(),
            context_summary: Some("Therapy Mode (Offline Fallback)".to_string()),
        })
    }

    async fn generate_cbt_reframe(
        &self,
        situation: String,
        automatic_thought: String,
        distortions: Vec<String>,
        biometrics: GarminBiometrics,
        history_bio: Vec<GarminBiometrics>,
    ) -> Result<crate::services::therapy::CbtReframeResult, String> {
        let prompt = super::build_cbt_reframe_prompt(
            &situation,
            &automatic_thought,
            &distortions,
            &biometrics,
            &history_bio,
        );

        if self.ensure_initialized().await.is_ok() {
            if let Ok(parsed) = self.execute_json_generation::<serde_json::Value>(prompt, "CBT Reframe") {
                if let (Some(reframe), Some(advice)) = (
                    parsed.get("reframed_thought").and_then(|v| v.as_str()),
                    parsed.get("goblin_advice").and_then(|v| v.as_str())
                ) {
                    let somatic = parsed
                        .get("suggested_somatic_action")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Take 3 slow physiological sighs to release physical tension.");

                    return Ok(crate::services::therapy::CbtReframeResult {
                        reframed_thought: reframe.to_string(),
                        identified_distortions: distortions.clone(),
                        goblin_advice: advice.to_string(),
                        suggested_somatic_action: somatic.to_string(),
                    });
                }
            }
        }

        Ok(crate::services::therapy::TherapyService::generate_deterministic_reframe(
            &situation,
            &automatic_thought,
            &distortions,
            biometrics.stress_level,
            &history_bio,
        ))
    }
}

use crate::ai::{
    ChatMessage, ChatResponse, LlmProvider, VisionAnalysisResponse,
};
use crate::services::garmin::{GarminActivity, GarminBiometrics};
use crate::services::goblin::GoblinProfile;
use crate::services::medgemma::HealthInsight;
use async_trait::async_trait;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub struct OllamaAdapter {
    db: Arc<Mutex<Connection>>,
}

impl OllamaAdapter {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }

    fn get_server_url(&self) -> String {
        let settings = crate::services::settings::get_settings(self.db.clone());
        settings
            .map(|s| s.ollama_server_url)
            .unwrap_or_else(|_| "http://localhost:11434".to_string())
    }
}

#[async_trait]
impl LlmProvider for OllamaAdapter {
    async fn check_health(&self) -> bool {
        let url = format!("{}/api/tags", self.get_server_url());
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
            .unwrap_or_default();

        match client.get(&url).send().await {
            Ok(res) => res.status().is_success(),
            Err(_) => false,
        }
    }

    async fn list_models(&self) -> Result<Vec<String>, String> {
        let url = format!("{}/api/tags", self.get_server_url());
        let client = reqwest::Client::new();
        let res = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

        #[derive(serde::Deserialize)]
        struct TagItem {
            name: String,
        }
        #[derive(serde::Deserialize)]
        struct TagsResponse {
            models: Option<Vec<TagItem>>,
        }

        let body: TagsResponse = res.json().await.map_err(|e| e.to_string())?;
        let models = body
            .models
            .unwrap_or_default()
            .into_iter()
            .map(|m| m.name)
            .collect();

        Ok(models)
    }

    async fn generate_goblin_chat(
        &self,
        history: Vec<ChatMessage>,
        biometrics: GarminBiometrics,
        goblin_profile: GoblinProfile,
        _image_uri: Option<String>,
        _audio_base64: Option<String>,
    ) -> Result<ChatResponse, String> {
        let settings = crate::services::settings::get_settings(self.db.clone())?;
        let personality = &settings.goblin_personality;
        let system_prompt = super::build_goblin_chat_system_prompt(
            &goblin_profile.name,
            personality,
            &biometrics,
            &goblin_profile,
        );

        let url = format!("{}/api/chat", self.get_server_url());
        let active_model = if settings.active_model.contains("litert") || settings.active_model.is_empty() {
            "gemma:2b".to_string()
        } else {
            settings.active_model
        };

        let mut messages = vec![serde_json::json!({
            "role": "system",
            "content": system_prompt
        })];

        for msg in history {
            messages.push(serde_json::json!({
                "role": msg.role,
                "content": msg.content
            }));
        }

        let body = serde_json::json!({
            "model": active_model,
            "messages": messages,
            "stream": false
        });

        let client = reqwest::Client::new();
        match client.post(&url).json(&body).send().await {
            Ok(res) => {
                #[derive(serde::Deserialize)]
                struct Msg {
                    content: String,
                }
                #[derive(serde::Deserialize)]
                struct ChatResp {
                    message: Msg,
                }

                let chat: ChatResp = res.json().await.map_err(|e| e.to_string())?;
                Ok(ChatResponse {
                    response: chat.message.content,
                    goblin_mood: goblin_profile.mood,
                    context_summary: None,
                })
            }
            Err(_) => {
                // Return dynamic fallback message from the Goblin
                Ok(ChatResponse {
                    response: format!(
                        "GRAH! {} here! Your Body Battery is at {}% and you logged {} steps! Keep moving, you dungeon crawler!",
                        goblin_profile.name, biometrics.body_battery, biometrics.steps
                    ),
                    goblin_mood: goblin_profile.mood,
                    context_summary: None,
                })
            }
        }
    }

    async fn generate_medgemma_health_insight(
        &self,
        biometrics: GarminBiometrics,
        _recent_activities: Vec<GarminActivity>,
    ) -> Result<HealthInsight, String> {
        // Deterministic clinical model provides instantaneous high-accuracy analysis
        Ok(crate::services::medgemma::generate_deterministic_insight(&biometrics))
    }

    async fn generate_food_ingestion_analysis(
        &self,
        image_uri: Option<String>,
        user_notes: Option<String>,
        audio_base64: Option<String>,
    ) -> Result<VisionAnalysisResponse, String> {
        let has_image = image_uri.as_ref().map(|u| !u.trim().is_empty()).unwrap_or(false);
        let has_audio = audio_base64.as_ref().map(|u| !u.trim().is_empty()).unwrap_or(false);
        let prompt = super::build_food_ingestion_prompt(user_notes.as_deref(), has_image, has_audio);

        // Check if Ollama is available
        if self.check_health().await {
            let url = format!("{}/api/chat", self.get_server_url());
            let settings = crate::services::settings::get_settings(self.db.clone()).ok();
            let model_name = settings
                .map(|s| s.active_model)
                .filter(|m| !m.is_empty() && !m.contains("litert"))
                .unwrap_or_else(|| {
                    if has_image {
                        "llava".to_string()
                    } else {
                        "gemma:2b".to_string()
                    }
                });

            let mut message_obj = serde_json::json!({
                "role": "user",
                "content": prompt,
            });

            // Convert image to base64 if present
            if has_image {
                if let Some(ref uri) = image_uri {
                    let base64_data = if uri.starts_with("data:image/") {
                        uri.find("base64,").map(|idx| uri[idx + 7..].to_string())
                    } else {
                        let clean_path = uri.trim_start_matches("file://");
                        if let Ok(bytes) = std::fs::read(clean_path) {
                            use base64::Engine as _;
                            Some(base64::engine::general_purpose::STANDARD.encode(&bytes))
                        } else {
                            None
                        }
                    };

                    if let Some(b64) = base64_data {
                        message_obj["images"] = serde_json::json!([b64]);
                    }
                }
            }

            let request_body = serde_json::json!({
                "model": model_name,
                "messages": [message_obj],
                "stream": false,
                "format": "json"
            });

            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default();

            if let Ok(res) = client.post(&url).json(&request_body).send().await {
                #[derive(serde::Deserialize)]
                struct Msg {
                    content: String,
                }
                #[derive(serde::Deserialize)]
                struct ChatResp {
                    message: Msg,
                }

                if let Ok(chat) = res.json::<ChatResp>().await {
                    let content = chat.message.content;
                    let json_content = if let (Some(start), Some(end)) = (content.find('{'), content.rfind('}')) {
                        &content[start..=end]
                    } else {
                        &content
                    };

                    if let Ok(parsed) = serde_json::from_str::<VisionAnalysisResponse>(json_content) {
                        return Ok(parsed);
                    }
                }
            }
        }

        Err("Food ingestion analysis failed. Ensure the AI model is running and reachable.".to_string())
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

        let url = format!("{}/api/chat", self.get_server_url());
        let settings = crate::services::settings::get_settings(self.db.clone()).ok();
        let active_model = settings
            .as_ref()
            .map(|s| {
                if s.active_model.contains("litert") || s.active_model.is_empty() {
                    "gemma:2b".to_string()
                } else {
                    s.active_model.clone()
                }
            })
            .unwrap_or_else(|| "gemma:2b".to_string());

        let mut messages = vec![serde_json::json!({
            "role": "system",
            "content": system_prompt
        })];

        for msg in history {
            messages.push(serde_json::json!({
                "role": msg.role,
                "content": msg.content
            }));
        }

        let body = serde_json::json!({
            "model": active_model,
            "messages": messages,
            "stream": false
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        if let Ok(res) = client.post(&url).json(&body).send().await {
            #[derive(serde::Deserialize)]
            struct Msg {
                content: String,
            }
            #[derive(serde::Deserialize)]
            struct ChatResp {
                message: Msg,
            }

            if let Ok(chat) = res.json::<ChatResp>().await {
                return Ok(ChatResponse {
                    response: chat.message.content,
                    goblin_mood: "Zen".to_string(),
                    context_summary: Some("Therapy Mode Session".to_string()),
                });
            }
        }

        // Empathetic offline/fallback response
        let history_context = if !history_bio.is_empty() {
            let n = history_bio.len();
            let avg_stress: i32 = history_bio.iter().map(|b| b.stress_level).sum::<i32>() / (n as i32);
            format!("Looking across your last {} synced days (average stress: {}/100)", n, avg_stress)
        } else {
            format!("Looking at your last synced stress score ({}/100)", biometrics.stress_level)
        };

        let fallback_response = format!(
            "Greetings by the cavern hearth. I am {}, your Goblin Counselor. {}, let's take a slow, deep breath together. Whatever has been weighing on your mind, this is your safe haven. What would you like to unpack?",
            goblin_profile.name, history_context
        );

        Ok(ChatResponse {
            response: fallback_response,
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

        if self.check_health().await {
            let url = format!("{}/api/chat", self.get_server_url());
            let settings = crate::services::settings::get_settings(self.db.clone()).ok();
            let model_name = settings
                .as_ref()
                .map(|s| {
                    if s.active_model.contains("litert") || s.active_model.is_empty() {
                        "gemma:2b".to_string()
                    } else {
                        s.active_model.clone()
                    }
                })
                .unwrap_or_else(|| "gemma:2b".to_string());

            let request_body = serde_json::json!({
                "model": model_name,
                "messages": [
                    { "role": "user", "content": prompt }
                ],
                "stream": false,
                "format": "json"
            });

            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default();

            if let Ok(res) = client.post(&url).json(&request_body).send().await {
                #[derive(serde::Deserialize)]
                struct Msg {
                    content: String,
                }
                #[derive(serde::Deserialize)]
                struct ChatResp {
                    message: Msg,
                }

                if let Ok(chat) = res.json::<ChatResp>().await {
                    let content = chat.message.content;
                    let json_content = if let (Some(start), Some(end)) = (content.find('{'), content.rfind('}')) {
                        &content[start..=end]
                    } else {
                        &content
                    };

                    #[derive(serde::Deserialize)]
                    struct ParsedCbt {
                        reframed_thought: Option<String>,
                        identified_distortions: Option<Vec<String>>,
                        goblin_advice: Option<String>,
                        suggested_somatic_action: Option<String>,
                    }

                    if let Ok(parsed) = serde_json::from_str::<ParsedCbt>(json_content) {
                        if let (Some(reframe), Some(advice)) = (parsed.reframed_thought, parsed.goblin_advice) {
                            return Ok(crate::services::therapy::CbtReframeResult {
                                reframed_thought: reframe,
                                identified_distortions: parsed.identified_distortions.unwrap_or(distortions.clone()),
                                goblin_advice: advice,
                                suggested_somatic_action: parsed.suggested_somatic_action.unwrap_or_else(|| {
                                    "Take 3 slow physiological sighs to release physical tension.".to_string()
                                }),
                            });
                        }
                    }
                }
            }
        }

        // Return deterministic clinically-grounded reframe
        Ok(crate::services::therapy::TherapyService::generate_deterministic_reframe(
            &situation,
            &automatic_thought,
            &distortions,
            biometrics.stress_level,
            &history_bio,
        ))
    }
}


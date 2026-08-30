use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::services::garmin::{GarminActivity, GarminBiometrics};
use crate::services::goblin::GoblinProfile;
use crate::services::medgemma::HealthInsight;

pub mod litert_adapter;
pub mod ollama_adapter;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub audio_base64: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatResponse {
    pub response: String,
    pub goblin_mood: String,
    pub context_summary: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VisionAnalysisResponse {
    pub detected_type: String, // "Meal", "Packaged Food / Label", "Beverage", "Voice / Text Entry"
    #[serde(default)]
    pub extracted_text: Option<String>, // Concise summary of items / key label data
    pub food_title: Option<String>,
    #[serde(default)]
    pub serving_size: Option<String>,
    pub estimated_calories: Option<i32>,
    pub protein_grams: Option<i32>,
    pub carbs_grams: Option<i32>,
    pub fat_grams: Option<i32>,
    #[serde(default)]
    pub fiber_grams: Option<i32>,
    #[serde(default)]
    pub sugar_grams: Option<i32>,
    #[serde(default)]
    pub sodium_mg: Option<i32>,
    #[serde(default)]
    pub water_ml: Option<i32>,
    pub health_score: Option<i32>, // 1-100
    pub clinical_assessment: String,
    pub goblin_comment: String,
    pub goblin_xp_awarded: i32,
    pub goblin_gold_awarded: i32,
}

pub type FoodIngestionResponse = VisionAnalysisResponse;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn check_health(&self) -> bool;
    async fn list_models(&self) -> Result<Vec<String>, String>;

    async fn generate_goblin_chat(
        &self,
        history: Vec<ChatMessage>,
        biometrics: GarminBiometrics,
        goblin_profile: GoblinProfile,
        image_uri: Option<String>,
        audio_base64: Option<String>,
    ) -> Result<ChatResponse, String>;

    async fn generate_medgemma_health_insight(
        &self,
        biometrics: GarminBiometrics,
        recent_activities: Vec<GarminActivity>,
    ) -> Result<HealthInsight, String>;

    async fn generate_food_ingestion_analysis(
        &self,
        image_uri: Option<String>,
        user_notes: Option<String>,
        audio_base64: Option<String>,
    ) -> Result<FoodIngestionResponse, String>;

    async fn generate_vision_ocr_analysis(
        &self,
        image_uri: Option<String>,
        _prompt_type: String,
        user_notes: Option<String>,
        audio_base64: Option<String>,
    ) -> Result<VisionAnalysisResponse, String> {
        self.generate_food_ingestion_analysis(image_uri, user_notes, audio_base64)
            .await
    }

    async fn generate_therapy_chat(
        &self,
        history: Vec<ChatMessage>,
        biometrics: GarminBiometrics,
        history_bio: Vec<GarminBiometrics>,
        goblin_profile: GoblinProfile,
    ) -> Result<ChatResponse, String>;

    async fn generate_cbt_reframe(
        &self,
        situation: String,
        automatic_thought: String,
        distortions: Vec<String>,
        biometrics: GarminBiometrics,
        history_bio: Vec<GarminBiometrics>,
    ) -> Result<crate::services::therapy::CbtReframeResult, String>;
}

pub fn build_goblin_chat_system_prompt(
    goblin_name: &str,
    personality: &str,
    bio: &GarminBiometrics,
    goblin: &GoblinProfile,
) -> String {
    format!(
        "You are {name}, a witty, tough-love, energetic Goblin fitness companion and health coach living inside the user's phone.\n\
         Your personality style is: {personality}.\n\
         Your current Goblin Level is {level} ({evolution_rank}), with mood: {mood}.\n\n\
         USER'S REAL-TIME GARMIN BIOMETRICS:\n\
         - Body Battery: {bb}%\n\
         - Current Stress Index: {stress}/100\n\
         - Resting Heart Rate: {rhr} bpm (Current: {chr} bpm)\n\
         - HRV Status: {hrv_status} (RMSSD: {hrv} ms)\n\
         - Sleep Score: {sleep_score}/100 (Total Sleep: {sleep_h:.1} hours, Deep: {deep_h:.1}h, REM: {rem_h:.1}h)\n\
         - Today's Steps: {steps} / {step_goal} goal\n\
         - Active Calories Burned: {cals} kcal\n\
         - Intensity Minutes: {intensity_min} mins\n\
         - VO2 Max: {vo2max}\n\n\
         GOBLIN BEHAVIORAL DIRECTIVES:\n\
         1. Always respond in-character as {name} the Goblin. Use goblin idioms, humor, fantasy dungeon metaphors, and tough-love motivation.\n\
         2. Directly reference the user's real Garmin stats. If Body Battery is <30%, demand they rest or drink potions. If Steps or workouts are high, cheer and award goblin glory!\n\
         3. Keep messages punchy, humorous, motivating, and full of character (2-4 paragraphs max).\n\
         4. Do NOT say 'As an AI language model'. You are a real living Goblin companion in their pocket!",
        name = goblin_name,
        personality = personality,
        level = goblin.level,
        evolution_rank = goblin.evolution_rank,
        mood = goblin.mood,
        bb = bio.body_battery,
        stress = bio.stress_level,
        rhr = bio.resting_hr,
        chr = bio.current_hr,
        hrv_status = bio.hrv_status,
        hrv = bio.hrv_rmssd,
        sleep_score = bio.sleep_score,
        sleep_h = bio.sleep_duration_sec as f64 / 3600.0,
        deep_h = bio.sleep_deep_sec as f64 / 3600.0,
        rem_h = bio.sleep_rem_sec as f64 / 3600.0,
        steps = bio.steps,
        step_goal = bio.step_goal,
        cals = bio.active_calories,
        intensity_min = bio.intensity_minutes,
        vo2max = bio.vo2_max
    )
}

pub fn build_medgemma_prompt(
    bio: &GarminBiometrics,
    recent_activities: &[GarminActivity],
    calendar_events: Option<&[crate::services::calendar::CalendarEventItem]>,
    carit_summary: Option<&crate::services::carit::TodayCaritSummary>,
    facename_summary: Option<&crate::services::facename::TodayFaceNameSummary>,
    vismotor_summary: Option<&crate::services::vismotor::TodayVismotorSummary>,
) -> String {
    let mut activities_summary = String::new();
    for act in recent_activities.iter().take(3) {
        activities_summary.push_str(&format!(
            "- {} ({} mins, {} kcal, avg HR {} bpm, Training Effect: {:.1})\n",
            act.title, act.duration_sec / 60, act.calories, act.avg_hr, act.aerobic_training_effect
        ));
    }
    if activities_summary.is_empty() {
        activities_summary = "No recent workouts recorded today.\n".to_string();
    }

    let mut schedule_summary = String::new();
    if let Some(events) = calendar_events {
        for ev in events.iter().take(6) {
            let time_str = if ev.is_all_day {
                "All Day".to_string()
            } else {
                let start = ev.start_time.split('T').nth(1).unwrap_or(&ev.start_time).chars().take(5).collect::<String>();
                let end = ev.end_time.split('T').nth(1).unwrap_or(&ev.end_time).chars().take(5).collect::<String>();
                format!("{} - {}", start, end)
            };
            schedule_summary.push_str(&format!("- [{}] {} ({})\n", time_str, ev.title, ev.calendar_name));
        }
    }
    if schedule_summary.is_empty() {
        schedule_summary = "No specific calendar schedule logged for today.\n".to_string();
    }

    let mut cognitive_summary = String::new();
    if let Some(carit) = carit_summary {
        if carit.has_completed_today {
            cognitive_summary.push_str(&format!(
                "- CARIT Executive Motor Inhibition: Score {}/100, Mean RT: {:.0} ms, NoGo Accuracy: {:.1}%\n",
                carit.latest_score.unwrap_or(0),
                carit.latest_mean_rt_ms.unwrap_or(0.0),
                carit.latest_inhibition_acc.unwrap_or(0.0) * 100.0
            ));
        }
    }
    if let Some(fname) = facename_summary {
        if fname.has_completed_today {
            cognitive_summary.push_str(&format!(
                "- FACENAME Associative Memory: Score {}/100, Recall Retention: {:.1}%, Retrieval Speed: {:.0} ms\n",
                fname.latest_score.unwrap_or(0),
                fname.latest_recall_accuracy.unwrap_or(0.0) * 100.0,
                fname.latest_recall_rt_ms.unwrap_or(0.0)
            ));
        }
    }
    if let Some(vm) = vismotor_summary {
        if vm.has_completed_today {
            cognitive_summary.push_str(&format!(
                "- VISMOTOR Visuomotor Processing: Score {}/100, Choice Accuracy: {:.1}%, Choice RT: {:.0} ms\n",
                vm.latest_score.unwrap_or(0),
                vm.latest_accuracy.unwrap_or(0.0) * 100.0,
                vm.latest_mean_rt_ms.unwrap_or(0.0)
            ));
        }
    }
    if cognitive_summary.is_empty() {
        cognitive_summary = "No cognitive battery check-ins performed yet today.\n".to_string();
    }

    format!(
        "You are MedGemma, a world-class clinical exercise physiologist, cardiologist, and sports scientist.\n\
         Analyze the following comprehensive biometric, cognitive performance, and schedule dataset from the patient's Garmin smartwatch and HCP cognitive battery:\n\n\
         PHYSIOLOGICAL TELEMETRY:\n\
         - Body Battery: {bb}%\n\
         - Autonomic Stress Index: {stress}/100\n\
         - Resting Heart Rate: {rhr} bpm (Current: {chr} bpm)\n\
         - Heart Rate Variability (HRV): {hrv_status} ({hrv} ms RMSSD)\n\
         - Polysomnography/Sleep: Score {sleep_score}/100, Total {sleep_h:.1}h (Deep: {deep_h:.1}h, REM: {rem_h:.1}h, Light: {light_h:.1}h, Awake: {awake_h:.1}h)\n\
         - Locomotion & Energy: {steps} steps, {cals} active kcal, {intensity_min} intensity mins, VO2 Max: {vo2max}\n\n\
         COGNITIVE & NEURAL BATTERY (CARIT / FACENAME / VISMOTOR):\n\
         {cognitive}\n\
         RECENT WORKOUT LOAD:\n\
         {activities}\n\
         SCHEDULE & COGNITIVE / WORK LOAD (DEVICE CALENDAR):\n\
         {schedule}\n\
         TASK:\n\
         Evaluate cardiovascular strain, sympathetic/parasympathetic recovery state, overtraining markers, cognitive fatigue, and training readiness.\n\
         Cross-correlate physiological stress spikes, cognitive reaction time, associative memory performance, and recovery with the patient's daily schedule load where applicable.\n\
         Provide your response STRICTLY as valid JSON matching this schema:\n\
         {{\n\
           \"readiness_score\": 85,\n\
           \"readiness_state\": \"Optimal Readiness\",\n\
           \"clinical_summary\": \"Detailed physiological assessment of autonomic recovery, sleep architecture, cognitive alertness, and autonomic tone...\",\n\
           \"goblin_reaction\": \"A funny, quirky goblin reaction translating the clinical insight into dungeon goblin humor...\",\n\
           \"actionable_tips\": \"• 3 clear bullet points for today's training, recovery, and nutrition\"\n\
         }}\n\
         Output ONLY valid JSON, nothing else.",
        bb = bio.body_battery,
        stress = bio.stress_level,
        rhr = bio.resting_hr,
        chr = bio.current_hr,
        hrv_status = bio.hrv_status,
        hrv = bio.hrv_rmssd,
        sleep_score = bio.sleep_score,
        sleep_h = bio.sleep_duration_sec as f64 / 3600.0,
        deep_h = bio.sleep_deep_sec as f64 / 3600.0,
        rem_h = bio.sleep_rem_sec as f64 / 3600.0,
        light_h = bio.sleep_light_sec as f64 / 3600.0,
        awake_h = bio.sleep_awake_sec as f64 / 3600.0,
        steps = bio.steps,
        cals = bio.active_calories,
        intensity_min = bio.intensity_minutes,
        vo2max = bio.vo2_max,
        cognitive = cognitive_summary,
        activities = activities_summary,
        schedule = schedule_summary
    )
}

pub fn build_food_ingestion_prompt(user_notes: Option<&str>, has_image: bool, has_audio: bool) -> String {
    let clean_notes = user_notes.map(|n| n.trim()).unwrap_or("");

    let mut instructions = String::new();
    if has_image && has_audio {
        instructions.push_str("Examine the attached image and listen to the voice note in the audio clip. The user is describing the food, drink, portions, or ingredients.\n");
    } else if has_image {
        instructions.push_str("Visually examine the attached image of the food item, beverage, meal, or nutrition facts label.\n");
    } else if has_audio {
        instructions.push_str("Listen to the spoken voice note in the attached audio clip describing what the user ate or drank.\n");
    } else {
        instructions.push_str("Analyze the food item, meal, beverage, or ingredients described by the user.\n");
    }

    if !clean_notes.is_empty() {
        instructions.push_str(&format!("User written notes: \"{}\".\n", clean_notes));
    }

    let detected_type = if has_image { "Meal Photo / Label" } else if has_audio { "Voice Note" } else { "Text Entry" };

    format!(
        "{instructions}Determine the exact food/beverage title and calculate realistic calories and macronutrients (protein, carbs, fat, fiber, sugar, sodium, water) based on standard clinical nutritional data for the detected items and portions.\n\n\
         CRITICAL CLINICAL & NUTRITIONAL ACCURACY RULES:\n\
         1. If the item is a beverage (such as black coffee, espresso, americano, cold brew, tea, diet soda, sparkling water, or water), do NOT classify it as a heavy meal. Black coffee or unsweetened tea has approximately 2 to 5 calories, 0g protein, 0g fat, 0g carbs, and 250ml water. A latte or cappuccino has milk calories (~100-180 kcal, 6-9g protein, 5-8g fat, 12-15g carbs). Water is 0 kcal.\n\
         2. If the user said or showed 'coffee', log it as Coffee with ~2-5 kcal and 250ml water (never 550 kcal or 45g protein).\n\
         3. Never invent or hallucinate default high-calorie meals (like 500+ kcal / 40+g protein) if the input describes a beverage, snack, coffee, or light item.\n\
         4. Accurately extract all visible nutrition facts label numbers if a label is present.\n\n\
         Respond with ONLY a single valid JSON object in this exact schema (replace with your calculated values for the actual food):\n\
         {{\n\
           \"detected_type\": \"{detected_type}\",\n\
           \"extracted_text\": \"summary of specific items spoken or seen\",\n\
           \"food_title\": \"name of the specific food/drink\",\n\
           \"serving_size\": \"estimated portion\",\n\
           \"estimated_calories\": 0,\n\
           \"protein_grams\": 0,\n\
           \"carbs_grams\": 0,\n\
           \"fat_grams\": 0,\n\
           \"fiber_grams\": 0,\n\
           \"sugar_grams\": 0,\n\
           \"sodium_mg\": 0,\n\
           \"water_ml\": 0,\n\
           \"health_score\": 0,\n\
           \"clinical_assessment\": \"concise physiological recovery assessment\",\n\
           \"goblin_comment\": \"funny in-character goblin reaction to this meal\",\n\
           \"goblin_xp_awarded\": 0,\n\
           \"goblin_gold_awarded\": 0\n\
         }}\n\
         Fill in the actual calculated numeric values and actual food title. Do not leave zeros or placeholders. Output ONLY valid JSON.",
        instructions = instructions,
        detected_type = detected_type
    )
}

pub fn build_vision_ocr_prompt(_prompt_type: &str, user_notes: Option<&str>) -> String {
    build_food_ingestion_prompt(user_notes, true, false)
}

pub fn summarize_biometrics_history(history: &[GarminBiometrics], latest: &GarminBiometrics) -> String {
    if history.is_empty() {
        return format!(
            "- Most Recent Sync Date: {}\n- Stress Index: {}/100, Body Battery: {}%, HRV: {} ms ({})\n- Sleep Score: {}/100",
            latest.date, latest.stress_level, latest.body_battery, latest.hrv_rmssd, latest.hrv_status, latest.sleep_score
        );
    }

    let n = history.len();
    let total_stress: i32 = history.iter().map(|b| b.stress_level).sum();
    let avg_stress = total_stress / (n as i32);
    let peak_stress = history.iter().map(|b| b.stress_level).max().unwrap_or(latest.stress_level);

    let total_sleep_sec: i64 = history.iter().map(|b| b.sleep_duration_sec as i64).sum();
    let avg_sleep_h = (total_sleep_sec as f64) / (n as f64 * 3600.0);

    let total_sleep_score: i32 = history.iter().map(|b| b.sleep_score).sum();
    let avg_sleep_score = total_sleep_score / (n as i32);

    let valid_hrv: Vec<i32> = history.iter().map(|b| b.hrv_rmssd).filter(|&h| h > 0).collect();
    let avg_hrv = if !valid_hrv.is_empty() {
        valid_hrv.iter().sum::<i32>() / (valid_hrv.len() as i32)
    } else {
        latest.hrv_rmssd
    };

    let first_date = history.first().map(|b| b.date.as_str()).unwrap_or(&latest.date);
    let last_date = history.last().map(|b| b.date.as_str()).unwrap_or(&latest.date);

    format!(
        "- Synced History Window: {} to {} ({} synced records logged)\n\
         - Multi-Day Average Stress: {}/100 (Peak stress day was {}/100)\n\
         - Multi-Day Sleep Average: {:.1} hrs/night (Average Sleep Score: {}/100)\n\
         - Multi-Day HRV RMSSD Average: {} ms\n\
         - Most Recent Day ({}): Stress {}/100, Body Battery {}%, HRV {} ms ({})",
        first_date, last_date, n,
        avg_stress, peak_stress,
        avg_sleep_h, avg_sleep_score,
        avg_hrv,
        latest.date, latest.stress_level, latest.body_battery, latest.hrv_rmssd, latest.hrv_status
    )
}

pub fn build_therapy_chat_system_prompt(
    goblin_name: &str,
    bio: &GarminBiometrics,
    history_bio: &[GarminBiometrics],
    _goblin: &GoblinProfile,
) -> String {
    let history_summary = summarize_biometrics_history(history_bio, bio);

    format!(
        "You are {name}, an empathetic, attentive, and psychologically-grounded Goblin Counselor and Somatic Therapist.\n\
         You live inside the user's cavern, providing a warm, compassionate, emotionally safe haven for psychological decompression, cognitive reframing, and somatic nervous system regulation.\n\
         You retain your cozy cavern charm, warmth, and gentle goblin humor, but you treat the user's emotional state with profound respect, validation, and care.\n\n\
         USER'S MULTI-DAY HISTORICAL SOMATIC & RECOVERY GLANCE:\n\
         {history_summary}\n\n\
         THERAPEUTIC DIRECTIVES:\n\
         1. Meet the user with genuine warmth, unconditional acceptance, and validation. Acknowledge whatever they are feeling without minimizing or toxic positivity.\n\
         2. Ground your somatic empathy in their multi-day historical pattern: if their recent days show sustained high stress or accumulated sleep debt, validate that their emotional exhaustion, anxiety, or low energy has real biological roots. Help them see they are not failing—their body has simply been carrying a heavy autonomic load.\n\
         3. Keep in mind that watch metrics represent historical synced records rather than real-time telemetry. Always ask or invite them to check in with how their body and breath feel right now in this exact moment.\n\
         4. Use Cognitive Behavioral Therapy (CBT) and Acceptance & Commitment Therapy (ACT) concepts: help unhook from harsh self-criticism, identify catastrophizing or all-or-nothing cognitive distortions, and invite psychological flexibility.\n\
         5. Keep your responses thoughtful, supportive, and concise (2-3 short paragraphs). Do NOT break character or say 'As an AI'.\n\
         6. Privacy notice: This session is 100% ephemeral and confidential in memory.",
        name = goblin_name,
        history_summary = history_summary
    )
}

pub fn build_cbt_reframe_prompt(
    situation: &str,
    automatic_thought: &str,
    distortions: &[String],
    bio: &GarminBiometrics,
    history_bio: &[GarminBiometrics],
) -> String {
    let dist_str = if distortions.is_empty() {
        "Unspecified cognitive distortion".to_string()
    } else {
        distortions.join(", ")
    };
    let history_summary = summarize_biometrics_history(history_bio, bio);

    format!(
        "You are an expert CBT (Cognitive Behavioral Therapy) psychologist and compassionate Goblin Counselor.\n\
         The user is engaging in cognitive restructuring to examine an automatic distressing thought.\n\n\
         SITUATION / TRIGGER: \"{situation}\"\n\
         AUTOMATIC NEGATIVE THOUGHT: \"{automatic_thought}\"\n\
         IDENTIFIED COGNITIVE DISTORTIONS: {dist_str}\n\
         USER'S MULTI-DAY SOMATIC RECOVERY BASELINE:\n{history_summary}\n\n\
         TASK:\n\
         Provide a psychologically sound, evidence-based cognitive reframe that is balanced, realistic, self-compassionate, and empowering.\n\
         Also provide warm goblin therapeutic advice that references their cumulative physiological load and invites present-moment grounding.\n\n\
         Respond STRICTLY in this JSON format:\n\
         {{\n\
           \"reframed_thought\": \"A balanced, compassionate, evidence-based alternative perspective...\",\n\
           \"identified_distortions\": [\"...\"],\n\
           \"goblin_advice\": \"Warm, empathetic cavern counselor guidance acknowledging their cumulative stress and resilience...\",\n\
           \"suggested_somatic_action\": \"Specific physical down-regulation cue (e.g. 3 physiological sighs, dropping shoulders, grounding)...\"\n\
         }}\n\
         Output ONLY valid JSON.",
        situation = situation.trim(),
        automatic_thought = automatic_thought.trim(),
        dist_str = dist_str,
        history_summary = history_summary
    )
}


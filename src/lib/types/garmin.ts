export interface GarminBiometrics {
  id: number;
  timestamp: string;
  date: string;
  body_battery: number;
  body_battery_drain: number;
  body_battery_charge: number;
  stress_level: number;
  resting_hr: number;
  current_hr: number;
  hrv_status: string;
  hrv_rmssd: number;
  sleep_score: number;
  sleep_duration_sec: number;
  sleep_deep_sec: number;
  sleep_rem_sec: number;
  sleep_light_sec: number;
  sleep_awake_sec: number;
  steps: number;
  step_goal: number;
  active_calories: number;
  total_calories: number;
  intensity_minutes: number;
  vo2_max: number;
  spo2: number;
  respiration_rate: number;
}

export interface GarminActivity {
  id: number;
  activity_type: string;
  title: string;
  start_time: string;
  duration_sec: number;
  distance_meters: number;
  calories: number;
  avg_hr: number;
  max_hr: number;
  aerobic_training_effect: number;
  anaerobic_training_effect: number;
  category?: 'Workout' | 'Sleep' | 'Breathwork' | 'Rest' | 'DailyMovement';
  gps_coordinates?: [number, number][];
  temp_c?: number;
  aqi?: number;
  elevation_gain_m?: number;
  sleep_score?: number;
  deep_sleep_sec?: number;
  rem_sleep_sec?: number;
  light_sleep_sec?: number;
  awake_sec?: number;
  stress_avg?: number;
}

export interface AirQualityData {
  aqi: number;
  category: 'Good' | 'Moderate' | 'Unhealthy for Sensitive' | 'Unhealthy' | 'Hazardous';
  pm2_5: number;
  pm10: number;
  o3: number;
  no2: number;
  temp_c: number;
  humidity_percent: number;
  heat_index_c: number;
  location: string;
  environmental_strain_score: number; // 0 to 100
  goblin_weather_comment: string;
}

export interface GpsRouteTrack {
  id: string;
  name: string;
  activity_type: string;
  date: string;
  distance_km: number;
  duration_min: number;
  elevation_gain_m: number;
  avg_pace_min_km: string;
  avg_hr: number;
  coordinates: [number, number][];
}

export interface GoblinProfile {
  id: number;
  name: string;
  level: number;
  xp: number;
  xp_to_next_level: number;
  gold: number;
  evolution_rank: string;
  mood: 'Energetic' | 'Tired' | 'Exhausted' | 'Feral' | 'Zen' | 'Proud' | 'Sleeping' | 'Neutral';
  energy: number;
  hunger: number;
  streak_days: number;
  last_active: string;
}

export interface GoblinQuest {
  id: number;
  title: string;
  description: string;
  quest_category: string;
  target_metric: string;
  target_value: number;
  current_value: number;
  reward_xp: number;
  reward_gold: number;
  is_completed: boolean;
  is_claimed: boolean;
  expires_at?: string;
}

export interface HealthInsight {
  id: number;
  created_at: string;
  category: string;
  readiness_score: number;
  readiness_state: string;
  clinical_summary: string;
  goblin_reaction: string;
  actionable_tips: string;
}

export interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
  goblinMood?: string;
  audioBase64?: string;
}

export interface ChatResponse {
  response: string;
  goblin_mood: string;
  context_summary?: string;
}

export interface VisionAnalysisResponse {
  detected_type: string;
  extracted_text?: string;
  food_title?: string;
  serving_size?: string;
  estimated_calories?: number;
  protein_grams?: number;
  carbs_grams?: number;
  fat_grams?: number;
  fiber_grams?: number;
  sugar_grams?: number;
  sodium_mg?: number;
  water_ml?: number;
  health_score?: number;
  clinical_assessment: string;
  goblin_comment: string;
  goblin_xp_awarded: number;
  goblin_gold_awarded: number;
}

export type FoodIngestionResponse = VisionAnalysisResponse;

export interface GarminDeviceInfo {
  device_id: string;
  device_name: string;
  mac_address: string;
  rssi: number;
  is_paired: boolean;
  is_connected: boolean;
  battery_level?: number;
  last_sync_time?: string;
  pending_fit_files: number;
}

export interface GarminOffloadSummary {
  device_name: string;
  synced_activities: GarminActivity[];
  xp_earned: number;
  gold_earned: number;
  biometrics_updated: boolean;
  weather_streamed: boolean;
  weather_condition: string;
  last_sync_timestamp: string;
  status_message: string;
}

export interface GarminUsbDeviceInfo {
  is_attached: boolean;
  device_name: string;
  has_permission: boolean;
  mount_point?: string;
  status_message: string;
}

export interface GarminMtpSyncSummary {
  success: boolean;
  device_name: string;
  synced_count: number;
  total_bytes: number;
  xp_earned: number;
  gold_earned: number;
  activities: GarminActivity[];
  status_message: string;
  last_sync_timestamp: string;
}

export interface AppSettings {
  active_model: string;
  medgemma_model: string;
  huggingface_token?: string;
  litert_accelerator: string;
  litert_max_tokens: number;
  ollama_server_url: string;
  tts_voice_style: string;
  use_simulated_biometrics: boolean;
  goblin_name: string;
  goblin_personality: string;
}

export interface CalendarEventItem {
  id: string;
  date: string;
  title: string;
  description: string;
  location: string;
  startTime: string;
  endTime: string;
  isAllDay: boolean;
  calendarName: string;
  eventColor: string;
}

export interface NotificationSettings {
  enabled: boolean;
  step_milestones: boolean;
  body_battery_alerts: boolean;
  hydration_reminders: boolean;
  sleep_bedtime_alerts: boolean;
  quest_alerts: boolean;
  stress_alerts: boolean;
  sound_enabled: boolean;
  frequency_minutes: number;
}


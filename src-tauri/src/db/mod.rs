use rusqlite::{Connection, Result};
use tauri::Manager;

pub const SCHEMA_V1: &str = "
    CREATE TABLE settings (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        active_model TEXT NOT NULL DEFAULT 'gemma-4-E2B-it.litertlm',
        medgemma_model TEXT NOT NULL DEFAULT 'medgemma-1.5-4b-it.litertlm',
        huggingface_token TEXT DEFAULT '',
        litert_accelerator TEXT NOT NULL DEFAULT 'Auto',
        litert_max_tokens INTEGER NOT NULL DEFAULT 3000,
        ollama_server_url TEXT NOT NULL DEFAULT 'http://localhost:11434',
        tts_voice_style TEXT NOT NULL DEFAULT 'voice_styles/M1.json',
        use_simulated_biometrics BOOLEAN NOT NULL DEFAULT 1,
        goblin_name TEXT NOT NULL DEFAULT 'Gribble',
        goblin_personality TEXT NOT NULL DEFAULT 'Tough Love Trainer'
    );
    INSERT INTO settings (id) VALUES (1);

    CREATE TABLE goblin_profile (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        name TEXT NOT NULL DEFAULT 'Gribble',
        level INTEGER NOT NULL DEFAULT 1,
        xp INTEGER NOT NULL DEFAULT 0,
        xp_to_next_level INTEGER NOT NULL DEFAULT 200,
        gold INTEGER NOT NULL DEFAULT 0,
        evolution_rank TEXT NOT NULL DEFAULT 'Cave Scamp',
        mood TEXT NOT NULL DEFAULT 'Ready',
        energy INTEGER NOT NULL DEFAULT 100,
        hunger INTEGER NOT NULL DEFAULT 0,
        streak_days INTEGER NOT NULL DEFAULT 0,
        last_active DATETIME DEFAULT CURRENT_TIMESTAMP
    );
    INSERT INTO goblin_profile (id) VALUES (1);

    CREATE TABLE garmin_biometrics (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
        date TEXT NOT NULL,
        body_battery INTEGER NOT NULL DEFAULT 0,
        body_battery_drain INTEGER NOT NULL DEFAULT 0,
        body_battery_charge INTEGER NOT NULL DEFAULT 0,
        stress_level INTEGER NOT NULL DEFAULT 0,
        resting_hr INTEGER NOT NULL DEFAULT 0,
        current_hr INTEGER NOT NULL DEFAULT 0,
        hrv_status TEXT NOT NULL DEFAULT 'No Data',
        hrv_rmssd INTEGER NOT NULL DEFAULT 0,
        sleep_score INTEGER NOT NULL DEFAULT 0,
        sleep_duration_sec INTEGER NOT NULL DEFAULT 0,
        sleep_deep_sec INTEGER NOT NULL DEFAULT 0,
        sleep_rem_sec INTEGER NOT NULL DEFAULT 0,
        sleep_light_sec INTEGER NOT NULL DEFAULT 0,
        sleep_awake_sec INTEGER NOT NULL DEFAULT 0,
        steps INTEGER NOT NULL DEFAULT 0,
        step_goal INTEGER NOT NULL DEFAULT 10000,
        active_calories INTEGER NOT NULL DEFAULT 0,
        total_calories INTEGER NOT NULL DEFAULT 0,
        intensity_minutes INTEGER NOT NULL DEFAULT 0,
        vo2_max REAL NOT NULL DEFAULT 0.0,
        spo2 INTEGER NOT NULL DEFAULT 0,
        respiration_rate INTEGER NOT NULL DEFAULT 14
    );

    CREATE TABLE garmin_activities (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        activity_type TEXT NOT NULL,
        title TEXT NOT NULL,
        start_time DATETIME NOT NULL,
        duration_sec INTEGER NOT NULL,
        distance_meters REAL DEFAULT 0,
        calories INTEGER NOT NULL,
        avg_hr INTEGER NOT NULL,
        max_hr INTEGER NOT NULL,
        aerobic_training_effect REAL DEFAULT 3.2,
        anaerobic_training_effect REAL DEFAULT 1.1
    );

    CREATE TABLE goblin_quests (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        title TEXT NOT NULL,
        description TEXT NOT NULL,
        quest_category TEXT NOT NULL,
        target_metric TEXT NOT NULL,
        target_value REAL NOT NULL,
        current_value REAL NOT NULL DEFAULT 0,
        reward_xp INTEGER NOT NULL,
        reward_gold INTEGER NOT NULL,
        is_completed BOOLEAN NOT NULL DEFAULT 0,
        is_claimed BOOLEAN NOT NULL DEFAULT 0,
        expires_at DATETIME
    );

    CREATE TABLE health_insights (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        category TEXT NOT NULL,
        readiness_score INTEGER NOT NULL,
        readiness_state TEXT NOT NULL,
        clinical_summary TEXT NOT NULL,
        goblin_reaction TEXT NOT NULL,
        actionable_tips TEXT NOT NULL
    );

    CREATE TABLE goblin_chat_history (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        role TEXT NOT NULL,
        content TEXT NOT NULL,
        goblin_mood TEXT,
        audio_base64 TEXT,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    PRAGMA user_version = 1;
";

const DB_VERSION_NUM: usize = 1;

pub fn init_db(app_handle: &tauri::AppHandle) -> Result<Connection, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app_data_dir: {}", e))?;

    eprintln!("📂 [Garmin Goblin] app_data_dir: {:?}", app_dir);

    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir)
            .map_err(|e| format!("Failed to create base app_data_dir {:?}: {}", app_dir, e))?;
    }

    let db_path = app_dir.join("garmin_goblin.db");
    eprintln!("📂 [Garmin Goblin] Database path: {:?}", db_path);

    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database at {:?}: {}", db_path, e))?;

    let mut user_version: i32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    if user_version == 0 {
        let tables_count: i32 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='settings'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if tables_count > 0 {
            conn.pragma_update(None, "user_version", 1)
                .map_err(|e| e.to_string())?;
            user_version = 1;
        }
    }

    let schemas = [SCHEMA_V1];

    for (i, schema) in schemas.iter().enumerate().take(DB_VERSION_NUM) {
        if user_version == i as i32 {
            conn.execute_batch(schema).map_err(|e| e.to_string())?;
            conn.pragma_update(None, "user_version", (i + 1) as i32)
                .map_err(|e| e.to_string())?;
            user_version = (i + 1) as i32;
        }
    }

    // Ensure all required columns exist in settings if migrating from partial dev schema
    let _ = conn.execute("ALTER TABLE settings ADD COLUMN medgemma_model TEXT NOT NULL DEFAULT 'medgemma-1.5-4b-it.litertlm'", []);
    let _ = conn.execute("ALTER TABLE settings ADD COLUMN litert_max_tokens INTEGER NOT NULL DEFAULT 3000", []);
    let _ = conn.execute("ALTER TABLE settings ADD COLUMN huggingface_token TEXT DEFAULT ''", []);

    // Ensure all required columns exist in garmin_biometrics
    let _ = conn.execute("ALTER TABLE garmin_biometrics ADD COLUMN respiration_rate INTEGER NOT NULL DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE garmin_biometrics ADD COLUMN vo2_max REAL NOT NULL DEFAULT 0.0", []);
    let _ = conn.execute("ALTER TABLE garmin_biometrics ADD COLUMN spo2 INTEGER NOT NULL DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE garmin_biometrics ADD COLUMN body_battery_drain INTEGER NOT NULL DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE garmin_biometrics ADD COLUMN body_battery_charge INTEGER NOT NULL DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE garmin_biometrics ADD COLUMN hrv_status TEXT NOT NULL DEFAULT 'No Data'", []);
    let _ = conn.execute("ALTER TABLE garmin_biometrics ADD COLUMN hrv_rmssd INTEGER NOT NULL DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE garmin_biometrics ADD COLUMN sleep_deep_sec INTEGER NOT NULL DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE garmin_biometrics ADD COLUMN sleep_rem_sec INTEGER NOT NULL DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE garmin_biometrics ADD COLUMN sleep_light_sec INTEGER NOT NULL DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE garmin_biometrics ADD COLUMN sleep_awake_sec INTEGER NOT NULL DEFAULT 0", []);

    // Ensure all required columns exist in garmin_activities
    let _ = conn.execute("ALTER TABLE garmin_activities ADD COLUMN aerobic_training_effect REAL DEFAULT 0.0", []);
    let _ = conn.execute("ALTER TABLE garmin_activities ADD COLUMN anaerobic_training_effect REAL DEFAULT 0.0", []);
    let _ = conn.execute("ALTER TABLE garmin_activities ADD COLUMN fit_sha256 TEXT", []);
    let _ = conn.execute("CREATE UNIQUE INDEX IF NOT EXISTS idx_garmin_activities_fit_sha256 ON garmin_activities(fit_sha256)", []);
    let _ = conn.execute("CREATE UNIQUE INDEX IF NOT EXISTS idx_garmin_biometrics_date ON garmin_biometrics(date)", []);

    // Ensure garmin_devices table exists
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS garmin_devices (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        device_id TEXT NOT NULL,
        device_name TEXT NOT NULL,
        mac_address TEXT NOT NULL,
        is_paired BOOLEAN NOT NULL DEFAULT 0,
        is_connected BOOLEAN NOT NULL DEFAULT 0,
        battery_level INTEGER,
        last_sync_time DATETIME,
        pending_fit_files INTEGER NOT NULL DEFAULT 0
    )", []);

    // Ensure food_logs table exists
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS food_logs (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        date TEXT NOT NULL,
        timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
        meal_type TEXT NOT NULL DEFAULT 'snack',
        description TEXT NOT NULL,
        calories INTEGER DEFAULT 0,
        carbs_g REAL DEFAULT 0,
        protein_g REAL DEFAULT 0,
        fat_g REAL DEFAULT 0,
        fiber_g REAL DEFAULT 0,
        sugar_g REAL DEFAULT 0,
        sodium_mg REAL DEFAULT 0,
        water_ml INTEGER DEFAULT 0,
        notes TEXT,
        image_path TEXT,
        source_type TEXT NOT NULL DEFAULT 'manual',
        confidence_score REAL
    )", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_food_logs_date ON food_logs(date)", []);

    // Ensure emotional_logs table exists
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS emotional_logs (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        date TEXT NOT NULL,
        timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
        time_of_day TEXT DEFAULT 'afternoon',
        mood TEXT NOT NULL DEFAULT 'balanced',
        energy_level INTEGER NOT NULL DEFAULT 5,
        motivation_level INTEGER NOT NULL DEFAULT 5,
        stress_level INTEGER NOT NULL DEFAULT 3,
        perceived_recovery INTEGER NOT NULL DEFAULT 5,
        stressors_json TEXT DEFAULT '[]',
        notes TEXT,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    )", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_emotional_logs_date ON emotional_logs(date)", []);

    // Ensure calendar_events table exists
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS calendar_events (
        id TEXT PRIMARY KEY,
        date TEXT NOT NULL,
        title TEXT NOT NULL,
        description TEXT,
        location TEXT,
        start_time DATETIME NOT NULL,
        end_time DATETIME NOT NULL,
        is_all_day BOOLEAN NOT NULL DEFAULT 0,
        calendar_name TEXT,
        event_color TEXT,
        updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
    )", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_calendar_events_date ON calendar_events(date)", []);

    // Ensure carit_sessions table exists for daily cognitive readiness tests
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS carit_sessions (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        date TEXT NOT NULL,
        timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
        time_of_day TEXT NOT NULL DEFAULT 'afternoon',
        mode TEXT NOT NULL DEFAULT 'daily',
        total_trials INTEGER NOT NULL,
        hit_count INTEGER NOT NULL,
        miss_count INTEGER NOT NULL,
        false_alarm_count INTEGER NOT NULL,
        corr_reject_count INTEGER NOT NULL,
        total_accuracy REAL NOT NULL,
        go_accuracy REAL NOT NULL,
        nogo_accuracy REAL NOT NULL,
        mean_rt_ms REAL NOT NULL,
        median_rt_ms REAL NOT NULL,
        rt_std_dev_ms REAL NOT NULL,
        d_prime REAL NOT NULL,
        cognitive_score INTEGER NOT NULL,
        xp_awarded INTEGER NOT NULL,
        gold_awarded INTEGER NOT NULL,
        trial_data_json TEXT
    )", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_carit_sessions_date ON carit_sessions(date)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_carit_sessions_timestamp ON carit_sessions(timestamp)", []);

    // Ensure facename_sessions table exists for daily associative memory tests
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS facename_sessions (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        date TEXT NOT NULL,
        timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
        time_of_day TEXT NOT NULL DEFAULT 'afternoon',
        mode TEXT NOT NULL DEFAULT 'daily',
        total_memorized INTEGER NOT NULL,
        total_recalled INTEGER NOT NULL,
        correct_recall_count INTEGER NOT NULL,
        recall_accuracy REAL NOT NULL,
        mean_recall_rt_ms REAL NOT NULL,
        median_recall_rt_ms REAL NOT NULL,
        distractor_accuracy REAL NOT NULL,
        memory_score INTEGER NOT NULL,
        xp_awarded INTEGER NOT NULL,
        gold_awarded INTEGER NOT NULL,
        trial_data_json TEXT
    )", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_facename_sessions_date ON facename_sessions(date)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_facename_sessions_timestamp ON facename_sessions(timestamp)", []);

    // Ensure vismotor_sessions table exists for daily choice reaction & speed tests
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS vismotor_sessions (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        date TEXT NOT NULL,
        timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
        time_of_day TEXT NOT NULL DEFAULT 'afternoon',
        mode TEXT NOT NULL DEFAULT 'daily',
        total_trials INTEGER NOT NULL,
        correct_count INTEGER NOT NULL,
        incorrect_count INTEGER NOT NULL,
        accuracy REAL NOT NULL,
        mean_rt_ms REAL NOT NULL,
        mean_left_rt_ms REAL NOT NULL,
        mean_right_rt_ms REAL NOT NULL,
        hemispheric_difference_ms REAL NOT NULL,
        rt_std_dev_ms REAL NOT NULL,
        vismotor_score INTEGER NOT NULL,
        xp_awarded INTEGER NOT NULL,
        gold_awarded INTEGER NOT NULL,
        trial_data_json TEXT
    )", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_vismotor_sessions_date ON vismotor_sessions(date)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_vismotor_sessions_timestamp ON vismotor_sessions(timestamp)", []);

    // Ensure cbt_thought_records table exists for Therapy Mode thought restructuring
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS cbt_thought_records (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        date TEXT NOT NULL,
        timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
        situation TEXT NOT NULL,
        automatic_thought TEXT NOT NULL,
        distortions_json TEXT NOT NULL DEFAULT '[]',
        distress_before INTEGER NOT NULL DEFAULT 5,
        reframed_thought TEXT NOT NULL,
        distress_after INTEGER NOT NULL DEFAULT 3,
        garmin_stress INTEGER DEFAULT 0,
        garmin_hr INTEGER DEFAULT 0,
        goblin_advice TEXT
    )", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_cbt_thought_records_date ON cbt_thought_records(date)", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_cbt_thought_records_timestamp ON cbt_thought_records(timestamp)", []);

    // Seed default initial biometrics & quests if tables are empty
    seed_initial_data(&conn)?;

    Ok(conn)
}

fn seed_initial_data(conn: &Connection) -> Result<(), String> {
    // Purge any dummy test activities
    let _ = conn.execute(
        "DELETE FROM garmin_activities WHERE title IN ('Cavern Trail Run', 'Goblin Cavern 5K', 'Goblin Dungeon Sprint')",
        [],
    );

    let _ = conn.execute(
        "DELETE FROM garmin_biometrics WHERE (steps <= 100 AND resting_hr = 0 AND sleep_score = 0) OR date > date('now', 'localtime')",
        [],
    );

    let _ = conn.execute(
        "UPDATE garmin_biometrics SET sleep_duration_sec = (sleep_deep_sec + sleep_rem_sec + sleep_light_sec) WHERE sleep_duration_sec > 43200 AND (sleep_deep_sec + sleep_rem_sec + sleep_light_sec) > 0",
        [],
    );

    // Seed initial quests if empty with 0 starting progress
    let quests_count: i32 = conn
        .query_row("SELECT count(*) FROM goblin_quests", [], |row| row.get(0))
        .unwrap_or(0);

    if quests_count == 0 {
        conn.execute_batch(
            "INSERT INTO goblin_quests (title, description, quest_category, target_metric, target_value, current_value, reward_xp, reward_gold, is_completed, is_claimed)
             VALUES 
             ('The 10,000 Step Dungeon Crawl', 'March 10,000 steps to explore the treacherous cavern depths.', 'Steps', 'steps', 10000, 0, 150, 40, 0, 0),
             ('Deep Slumber Potion', 'Achieve at least 80 Sleep Score to recharge your Goblin life force.', 'Sleep', 'sleep_score', 80, 0, 120, 35, 0, 0),
             ('Iron Heart Endurance', 'Log 30 Intensity Minutes of vigorous movement.', 'Workout', 'intensity_minutes', 30, 0, 180, 50, 0, 0),
             ('Zen Breath of the Cavern', 'Keep average daily stress under 35 points.', 'Recovery', 'stress_level', 35, 0, 100, 30, 0, 0);"
        ).map_err(|e| e.to_string())?;
    }

    Ok(())
}

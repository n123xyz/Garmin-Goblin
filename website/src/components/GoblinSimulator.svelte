<script lang="ts">
    // Interactive Goblin & Biometric Simulator State
    let level = $state(4);
    let xp = $state(740);
    let maxXp = $state(1000);
    let gold = $state(185);
    let mood = $state<'Ecstatic' | 'Energetic' | 'Resting' | 'Tired'>('Energetic');
    let satiety = $state(85);

    // Biometrics
    let hr = $state(68);
    let hrv = $state(52); // RMSSD
    let stress = $state(28); // 0-100
    let sleepScore = $state(88); // 0-100
    let aiInsight = $state("Local Assistant: 'Resting HR is stable at 68 bpm and HRV is 52ms. Good recovery recorded.'");

    let activeEffect = $state<string | null>(null);

    function triggerEffect(name: string) {
        activeEffect = name;
        setTimeout(() => {
            activeEffect = null;
        }, 1200);
    }

    function simulateWorkout() {
        triggerEffect("+450 XP | Workout Logged!");
        xp += 450;
        gold += 35;
        hr = 156;
        stress = 64;
        if (xp >= maxXp) {
            level += 1;
            xp -= maxXp;
            maxXp = Math.round(maxXp * 1.3);
            mood = 'Ecstatic';
        } else {
            mood = 'Energetic';
        }
        aiInsight = "Local Assistant: '5.2km run imported via USB MTP (.FIT). Heart rate peaked at 156 bpm. Solid workout!'";
    }

    function simulateSleep() {
        triggerEffect("+350 XP | 8.5h Sleep!");
        xp += 350;
        gold += 25;
        hr = 54;
        hrv = 64;
        stress = 14;
        sleepScore = 94;
        if (xp >= maxXp) {
            level += 1;
            xp -= maxXp;
            maxXp = Math.round(maxXp * 1.3);
            mood = 'Ecstatic';
        } else {
            mood = 'Energetic';
        }
        aiInsight = "Local Assistant: '8.5h sleep logged with 2h 15m deep sleep. Well rested for today.'";
    }

    function simulateFeed() {
        triggerEffect("+100 XP | Fed Berry!");
        satiety = 100;
        xp += 100;
        gold = Math.max(0, gold - 15);
        if (xp >= maxXp) {
            level += 1;
            xp -= maxXp;
            maxXp = Math.round(maxXp * 1.3);
        }
        mood = 'Ecstatic';
        aiInsight = "Gribble munches happily! 'Satiety full! Gribble gets an XP boost on your next workout.'";
    }

    function simulateBreath() {
        triggerEffect("-30 Stress | Box Breathing!");
        stress = Math.max(10, stress - 30);
        hr = 58;
        hrv = 58;
        xp += 150;
        mood = 'Resting';
        aiInsight = "Local Assistant: 'Box breathing completed. Stress score decreased from 58 to 28.'";
    }
</script>

<div class="simulator-card glassmorphic">
    <div class="sim-header">
        <div class="goblin-badge">
            <span class="goblin-avatar">👹</span>
            <div class="goblin-info">
                <div class="name-row">
                    <span class="goblin-name">Gribble</span>
                    <span class="level-tag">Lvl {level}</span>
                </div>
                <span class="mood-tag">Mood: <b>{mood}</b></span>
            </div>
        </div>
        <div class="gold-badge">
            <span class="gold-icon">🪙</span>
            <span class="gold-amount">{gold} Gold</span>
        </div>
    </div>

    <!-- XP Bar -->
    <div class="xp-container">
        <div class="xp-labels">
            <span>Goblin Experience</span>
            <span>{xp} / {maxXp} XP</span>
        </div>
        <div class="xp-bar">
            <div class="xp-fill" style="width: {Math.min(100, Math.round((xp / maxXp) * 100))}%"></div>
        </div>
    </div>

    <!-- Live Biometric Gauges -->
    <div class="gauges-grid">
        <div class="gauge-item">
            <span class="gauge-icon">❤️</span>
            <div class="gauge-val">{hr} <small>bpm</small></div>
            <span class="gauge-lbl">Heart Rate</span>
        </div>
        <div class="gauge-item">
            <span class="gauge-icon">⚡</span>
            <div class="gauge-val">{hrv} <small>ms</small></div>
            <span class="gauge-lbl">HRV (RMSSD)</span>
        </div>
        <div class="gauge-item">
            <span class="gauge-icon">🧘</span>
            <div class="gauge-val">{stress} <small>/100</small></div>
            <span class="gauge-lbl">Stress Level</span>
        </div>
        <div class="gauge-item">
            <span class="gauge-icon">🌙</span>
            <div class="gauge-val">{sleepScore}%</div>
            <span class="gauge-lbl">Sleep Quality</span>
        </div>
    </div>

    <!-- Interactive Simulator Buttons -->
    <div class="sim-actions">
        <span class="actions-title">Interactive Health Simulation:</span>
        <div class="button-row">
            <button class="action-btn workout" onclick={simulateWorkout}>
                <span>🏃</span>
                <span>Log 5K Run</span>
            </button>
            <button class="action-btn sleep" onclick={simulateSleep}>
                <span>🛌</span>
                <span>Log 8.5h Sleep</span>
            </button>
            <button class="action-btn breath" onclick={simulateBreath}>
                <span>🌬️</span>
                <span>Box Breathing</span>
            </button>
            <button class="action-btn feed" onclick={simulateFeed}>
                <span>🍄</span>
                <span>Feed Gribble</span>
            </button>
        </div>
    </div>

    <!-- Toast Effect Banner -->
    {#if activeEffect}
        <div class="effect-banner">
            ✨ {activeEffect}
        </div>
    {/if}

    <!-- On-Device MedGemma Insight Box -->
    <div class="ai-box">
        <div class="ai-box-header">
            <span class="chip-icon">🤖</span>
            <span>On-Device LiteRT Health Summary</span>
            <span class="latency-badge">NPU Accelerated</span>
        </div>
        <p class="ai-text">{aiInsight}</p>
    </div>
</div>

<style>
    .simulator-card {
        border-radius: 24px;
        padding: 2rem;
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
        position: relative;
        overflow: hidden;
        border-color: rgba(74, 222, 128, 0.28);
    }

    .sim-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .goblin-badge {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .goblin-avatar {
        font-size: 2.8rem;
        filter: drop-shadow(0 0 12px rgba(74, 222, 128, 0.4));
        animation: float 3s ease-in-out infinite;
    }

    @keyframes float {
        0%, 100% { transform: translateY(0px); }
        50% { transform: translateY(-4px); }
    }

    .goblin-info {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }

    .name-row {
        display: flex;
        align-items: center;
        gap: 0.6rem;
    }

    .goblin-name {
        font-size: 1.4rem;
        font-weight: 800;
        color: var(--text-heading);
    }

    .level-tag {
        background: rgba(74, 222, 128, 0.2);
        color: var(--accent-light);
        border: 1px solid rgba(74, 222, 128, 0.4);
        padding: 0.15rem 0.6rem;
        border-radius: 99px;
        font-size: 0.75rem;
        font-weight: 750;
    }

    .mood-tag {
        font-size: 0.85rem;
        color: var(--text-muted);
    }

    .mood-tag b {
        color: var(--accent-light);
    }

    .gold-badge {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        background: rgba(251, 191, 36, 0.12);
        border: 1px solid rgba(251, 191, 36, 0.3);
        padding: 0.4rem 0.85rem;
        border-radius: 99px;
        color: var(--gold);
        font-weight: 750;
        font-size: 0.95rem;
    }

    .xp-container {
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }

    .xp-labels {
        display: flex;
        justify-content: space-between;
        font-size: 0.85rem;
        color: var(--text-muted);
        font-weight: 600;
    }

    .xp-bar {
        height: 10px;
        background: rgba(30, 41, 35, 0.8);
        border-radius: 99px;
        overflow: hidden;
        border: 1px solid rgba(74, 222, 128, 0.15);
    }

    .xp-fill {
        height: 100%;
        background: linear-gradient(90deg, #15803d, #4ade80);
        border-radius: 99px;
        transition: width 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
        box-shadow: 0 0 10px rgba(74, 222, 128, 0.5);
    }

    .gauges-grid {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 0.75rem;
    }

    .gauge-item {
        background: rgba(30, 41, 35, 0.45);
        border: 1px solid var(--border);
        border-radius: 14px;
        padding: 0.85rem 0.5rem;
        text-align: center;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.2rem;
    }

    .gauge-icon {
        font-size: 1.2rem;
    }

    .gauge-val {
        font-size: 1.25rem;
        font-weight: 850;
        color: var(--text-heading);
    }

    .gauge-val small {
        font-size: 0.75rem;
        font-weight: 500;
        color: var(--text-muted);
    }

    .gauge-lbl {
        font-size: 0.75rem;
        color: var(--text-muted);
    }

    .sim-actions {
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
    }

    .actions-title {
        font-size: 0.85rem;
        text-transform: uppercase;
        letter-spacing: 0.06em;
        color: var(--accent-light);
        font-weight: 750;
    }

    .button-row {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 0.65rem;
    }

    .action-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        padding: 0.75rem 1rem;
        border-radius: 12px;
        background: rgba(30, 41, 35, 0.6);
        border: 1px solid var(--border);
        color: var(--text-heading);
        font-weight: 650;
        font-size: 0.9rem;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .action-btn:hover {
        transform: translateY(-2px);
        background: rgba(74, 222, 128, 0.15);
        border-color: var(--accent-light);
    }

    .effect-banner {
        position: absolute;
        top: 1rem;
        right: 1rem;
        background: var(--accent-light);
        color: #090b0a;
        font-weight: 800;
        font-size: 0.85rem;
        padding: 0.4rem 0.85rem;
        border-radius: 99px;
        box-shadow: 0 4px 14px rgba(74, 222, 128, 0.6);
        animation: pop 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
    }

    @keyframes pop {
        0% { transform: scale(0.6); opacity: 0; }
        100% { transform: scale(1); opacity: 1; }
    }

    .ai-box {
        background: rgba(13, 18, 15, 0.7);
        border: 1px solid rgba(74, 222, 128, 0.25);
        border-radius: 16px;
        padding: 1.15rem 1.25rem;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .ai-box-header {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.85rem;
        font-weight: 750;
        color: var(--accent-light);
    }

    .latency-badge {
        margin-left: auto;
        background: rgba(74, 222, 128, 0.15);
        border: 1px solid rgba(74, 222, 128, 0.35);
        padding: 0.15rem 0.5rem;
        border-radius: 6px;
        font-size: 0.75rem;
        font-family: var(--font-mono);
    }

    .ai-text {
        margin: 0;
        font-size: 0.925rem;
        color: #e2e8f0;
        line-height: 1.55;
        font-style: italic;
    }

    @media (max-width: 600px) {
        .gauges-grid {
            grid-template-columns: repeat(2, 1fr);
        }
        .button-row {
            grid-template-columns: 1fr;
        }
    }
</style>

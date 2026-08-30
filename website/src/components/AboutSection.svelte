<script lang="ts">
</script>

<div class="about-container">
    <div class="about-header">
        <span class="eyebrow">About the Project</span>
        <h2>Why Garmin Goblin?</h2>
        <p class="subtitle">A local-first, open-source companion app for your Garmin watch.</p>
    </div>

    <div class="about-grid">
        <div class="about-card glassmorphic">
            <div class="card-icon">📁</div>
            <h3>Local Storage, No Accounts</h3>
            <p>
                Garmin Goblin doesn't require a Garmin Connect account or an active internet connection. Your workout files, daily steps, and heart rate logs are saved in a standard SQLite database directly on your device.
            </p>
        </div>

        <div class="about-card glassmorphic">
            <div class="card-icon">⚡</div>
            <h3>Fast USB MTP File Transfer</h3>
            <p>
                Syncing dense multi-hour workout files over Bluetooth can take minutes. With USB MTP support, you can plug your watch into your phone or PC with a USB-C cable and offload months of <code>.FIT</code> files in seconds.
            </p>
        </div>

        <div class="about-card glassmorphic highlighted-card">
            <div class="card-icon">🤖</div>
            <h3>Local On-Device AI</h3>
            <p>
                Get workout summaries and sleep analysis without uploading your health data to third-party cloud APIs.
            </p>
            <p class="highlight-text">
                Using <b>Google LiteRT LM</b>, Garmin Goblin runs quantized models like <b>MedGemma 1.5 4B</b> or <b>Gemma 4</b> directly on supported mobile NPUs (Snapdragon, Dimensity, Tensor), or connects to your local <b>Ollama</b> instance on desktop.
            </p>
        </div>

        <div class="about-card glassmorphic">
            <div class="card-icon">👹</div>
            <h3>Gribble the Goblin Companion</h3>
            <p>
                A gamified virtual pet that makes daily health tracking a little more fun. Gribble gains XP, levels up, and awards gold when you hit your step goal, get good sleep, and complete workouts.
            </p>
        </div>

        <div class="about-card glassmorphic">
            <div class="card-icon">⏱️</div>
            <h3>Cognitive & Breathing Exercises</h3>
            <p>
                Includes cognitive reaction and memory tests (CARIT Go/No-Go, face-name recall, visual reaction speed) and box breathing timers to help you track mental sharpness and wind down at night.
            </p>
        </div>

        <div class="about-card glassmorphic">
            <div class="card-icon">🔓</div>
            <h3>Free & Open Source</h3>
            <p>
                No subscriptions, no ads, and no telemetry tracking. Built under the MIT license using Tauri v2, Svelte 5, Rust, and Google LiteRT.
            </p>
        </div>
    </div>
</div>

<style>
    .about-container {
        max-width: 1150px;
        margin: 0 auto;
        padding: 4rem 1.5rem;
    }

    .about-header {
        text-align: center;
        margin-bottom: 4rem;
    }

    .eyebrow {
        color: var(--accent-light);
        text-transform: uppercase;
        font-size: 0.85rem;
        font-weight: 750;
        letter-spacing: 0.12em;
        display: block;
        margin-bottom: 0.5rem;
    }

    .about-header h2 {
        font-size: 2.75rem;
        font-weight: 850;
        color: var(--text-heading);
        margin: 0 0 1rem 0;
        letter-spacing: -0.03em;
    }

    .subtitle {
        color: var(--text-muted);
        font-size: 1.15rem;
        max-width: 650px;
        margin: 0 auto;
        line-height: 1.6;
    }

    .about-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
        gap: 2rem;
        margin-bottom: 4rem;
    }

    .about-card {
        border-radius: 20px;
        padding: 2.25rem;
        transition: transform 0.2s ease, border-color 0.2s ease;
        display: flex;
        flex-direction: column;
    }

    .about-card:hover {
        transform: translateY(-4px);
        border-color: rgba(74, 222, 128, 0.4);
    }

    .highlighted-card {
        grid-column: span 2;
        background: radial-gradient(circle at top right, rgba(34, 197, 94, 0.15) 0%, rgba(19, 23, 21, 0.7) 70%);
        border-color: rgba(74, 222, 128, 0.35);
    }

    @media (max-width: 850px) {
        .highlighted-card {
            grid-column: span 1;
        }
    }

    .card-icon {
        font-size: 2.2rem;
        margin-bottom: 1.25rem;
    }

    .about-card h3 {
        font-size: 1.35rem;
        font-weight: 750;
        color: var(--text-heading);
        margin: 0 0 0.85rem 0;
        letter-spacing: -0.02em;
    }

    .about-card p {
        color: var(--text);
        line-height: 1.65;
        font-size: 0.975rem;
        margin: 0 0 1rem 0;
    }

    .about-card p:last-child {
        margin-bottom: 0;
    }

    .highlight-text {
        color: #e2e8f0;
        background: rgba(34, 197, 94, 0.08);
        padding: 0.85rem 1rem;
        border-radius: 10px;
        border-left: 3px solid var(--accent);
    }
</style>

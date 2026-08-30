<script lang="ts">
    import GoblinSimulator from './components/GoblinSimulator.svelte';
    import DeviceCarousel from './components/DeviceCarousel.svelte';
    import AboutSection from './components/AboutSection.svelte';
    import DocsSection from './components/DocsSection.svelte';
    import SupportSection from './components/SupportSection.svelte';

    type Tab = 'welcome' | 'about' | 'docs' | 'support';
    let activeTab = $state<Tab>('welcome');

    function setTab(tab: Tab) {
        activeTab = tab;
        window.scrollTo({ top: 0, behavior: 'smooth' });
    }
</script>

<div class="app-layout">
    <!-- Header Navigation -->
    <header class="app-header">
        <div class="logo-container" onclick={() => setTab('welcome')} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && setTab('welcome')}>
            <img src="./goblin.svg" alt="Garmin Goblin Logo" class="logo-img" />
            <span class="logo-text">Garmin <span class="green-text">Goblin</span></span>
        </div>
        <nav class="nav-links">
            <button class="nav-btn" class:active={activeTab === 'welcome'} onclick={() => setTab('welcome')}>Welcome</button>
            <button class="nav-btn" class:active={activeTab === 'about'} onclick={() => setTab('about')}>About</button>
            <button class="nav-btn" class:active={activeTab === 'docs'} onclick={() => setTab('docs')}>Docs</button>
            <button class="nav-btn" class:active={activeTab === 'support'} onclick={() => setTab('support')}>Support</button>
        </nav>
        <div class="header-actions">
            <a href="https://github.com/n123xyz/Garmin-Goblin" target="_blank" rel="noopener noreferrer" class="github-icon-link" aria-label="GitHub Repository">
                <svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path></svg>
            </a>
        </div>
    </header>

    <!-- Main Content Area -->
    <main class="main-content">
        {#if activeTab === 'welcome'}
            <div class="welcome-container">
                <!-- Hero Section -->
                <div class="hero-banner">
                    <span class="hero-badge">Open Source Garmin Companion & Fitness RPG</span>
                    <h1 class="hero-title">A local, gamified app<br><span class="gradient-text">for your Garmin watch.</span></h1>
                    <p class="hero-desc">
                        Sync workouts and health data over <b>Bluetooth LE</b> or <b>USB cable</b>, keep all your data on your own device in SQLite, level up your goblin pet <b>Gribble</b>, and chat with local AI models.
                    </p>
                    <div class="hero-cta-row">
                        <a href="https://github.com/n123xyz/Garmin-Goblin/releases" target="_blank" rel="noopener noreferrer" class="btn btn-primary">
                            <span>📲</span>
                            <span>Download Android APK</span>
                        </a>
                        <button class="btn btn-outline" onclick={() => setTab('docs')}>
                            <span>📖</span>
                            <span>Documentation</span>
                        </button>
                    </div>
                </div>

                <!-- Two-Panel Section: Mockup & Simulator -->
                <div class="two-panel-grid">
                    <!-- Left Panel: Phone Mockup -->
                    <div class="panel panel-left">
                        <div class="phone-mockup">
                            <div class="phone-speaker"></div>
                            <div class="phone-screen">
                                <div class="app-mockup-content">
                                    <div class="mockup-topbar">
                                        <span class="mockup-time">9:41</span>
                                        <span class="mockup-chip">Local NPU</span>
                                    </div>
                                    <div class="mockup-goblin-card">
                                        <div class="mockup-goblin-row">
                                            <span class="mockup-avatar">👹</span>
                                            <div>
                                                <div class="mockup-gname">Gribble <small>Lvl 4</small></div>
                                                <div class="mockup-status">🟢 USB MTP Connected • 8,420 steps</div>
                                            </div>
                                        </div>
                                        <div class="mockup-xp">
                                            <div class="mockup-xp-bar" style="width: 74%"></div>
                                        </div>
                                    </div>
                                    <div class="mockup-vitals">
                                        <div class="vital-tile">
                                            <span class="v-val">68</span>
                                            <span class="v-lbl">BPM</span>
                                        </div>
                                        <div class="vital-tile">
                                            <span class="v-val">52ms</span>
                                            <span class="v-lbl">HRV</span>
                                        </div>
                                        <div class="vital-tile">
                                            <span class="v-val">81</span>
                                            <span class="v-lbl">BATTERY</span>
                                        </div>
                                    </div>
                                    <div class="mockup-medgemma-card">
                                        <div class="medgemma-title">🤖 Local AI Assistant</div>
                                        <div class="medgemma-text">"7h 45m sleep logged with good deep sleep. You're recovered and ready for your workout today."</div>
                                    </div>
                                    <div class="mockup-btn">⚡ Sync USB (MTP)</div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <!-- Right Panel: Interactive Simulator -->
                    <div class="panel panel-right">
                        <GoblinSimulator />
                    </div>
                </div>

                <!-- Features Grid Section -->
                <div class="features-section">
                    <div class="sec-header">
                        <span class="sec-badge">Features</span>
                        <h3>Everything You Need, Kept On Your Device</h3>
                    </div>
                    <div class="features-grid">
                        <div class="feat-card glassmorphic">
                            <div class="feat-icon">⚡</div>
                            <h4>USB MTP Sync</h4>
                            <p>Direct USB-C cable transfer. Copy months of workout <code>.FIT</code> files and continuous biometric logs in seconds.</p>
                        </div>
                        <div class="feat-card glassmorphic">
                            <div class="feat-icon">📶</div>
                            <h4>Bluetooth LE Sync</h4>
                            <p>Background sync for daily steps, heart rate, and phone notification forwarding using open protocols.</p>
                        </div>
                        <div class="feat-card glassmorphic">
                            <div class="feat-icon">🧠</div>
                            <h4>On-Device AI</h4>
                            <p>Run quantized Gemma and MedGemma models locally on mobile NPUs via LiteRT or Ollama on desktop.</p>
                        </div>
                        <div class="feat-card glassmorphic">
                            <div class="feat-icon">🎮</div>
                            <h4>Goblin Companion</h4>
                            <p>Logging workouts, steps, and sleep levels up Gribble, awards gold, and unlocks daily quests.</p>
                        </div>
                        <div class="feat-card glassmorphic">
                            <div class="feat-icon">🔬</div>
                            <h4>Cognitive Tests</h4>
                            <p>Built-in CARIT (Go / No-Go), face-name association, and visual reaction speed exercises.</p>
                        </div>
                        <div class="feat-card glassmorphic">
                            <div class="feat-icon">💾</div>
                            <h4>Local SQLite Storage</h4>
                            <p>All data stays in a local SQLite database on your device. No cloud accounts, no subscriptions, no tracking.</p>
                        </div>
                    </div>
                </div>

                <!-- Supported Devices Section -->
                <div class="devices-section">
                    <div class="sec-header">
                        <span class="sec-badge">Broad Compatibility</span>
                        <h3>Supported Garmin Devices</h3>
                    </div>
                    <DeviceCarousel />
                </div>
            </div>
        {:else if activeTab === 'about'}
            <AboutSection />
        {:else if activeTab === 'docs'}
            <DocsSection />
        {:else if activeTab === 'support'}
            <SupportSection />
        {/if}
    </main>

    <!-- Footer -->
    <footer class="app-footer">
        <div class="footer-content">
            <p>&copy; 2026 Garmin Goblin Contributors. Open source under the MIT License.</p>
            <div class="footer-links">
                <a href="https://github.com/n123xyz/Garmin-Goblin" target="_blank" rel="noopener noreferrer">GitHub</a>
                <a href="#docs-introduction" onclick={() => setTab('docs')}>Docs</a>
                <a href="https://github.com/n123xyz/Garmin-Goblin/blob/main/LICENSE" target="_blank" rel="noopener noreferrer">License</a>
            </div>
        </div>
    </footer>
</div>

<style>
    .app-layout {
        display: flex;
        flex-direction: column;
        min-height: 100vh;
        background: radial-gradient(circle at top, rgba(34, 197, 94, 0.08) 0%, rgba(9, 11, 10, 0) 60%), #090b0a;
    }

    /* Header Styling */
    .app-header {
        height: 72px;
        border-bottom: 1px solid var(--border);
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 2rem;
        background: rgba(9, 11, 10, 0.8);
        backdrop-filter: blur(16px);
        -webkit-backdrop-filter: blur(16px);
        position: sticky;
        top: 0;
        z-index: 50;
    }

    .logo-container {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        cursor: pointer;
        user-select: none;
    }

    .logo-img {
        width: 32px;
        height: 32px;
        border-radius: 8px;
        object-fit: cover;
        display: block;
        box-shadow: 0 0 12px rgba(16, 185, 129, 0.25);
    }

    .logo-text {
        font-size: 1.25rem;
        font-weight: 850;
        color: var(--text-heading);
        letter-spacing: -0.02em;
    }

    .green-text {
        color: var(--accent-light);
    }

    .nav-links {
        display: flex;
        gap: 0.5rem;
    }

    .nav-btn {
        background: transparent;
        border: none;
        color: var(--text-muted);
        font-size: 0.95rem;
        font-weight: 600;
        padding: 0.5rem 1rem;
        border-radius: 8px;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .nav-btn:hover {
        color: var(--text-heading);
        background: rgba(74, 222, 128, 0.08);
    }

    .nav-btn.active {
        color: var(--accent-light);
        background: rgba(74, 222, 128, 0.15);
    }

    .header-actions {
        display: flex;
        align-items: center;
    }

    .github-icon-link {
        color: var(--text-muted);
        padding: 0.5rem;
        border-radius: 8px;
        display: flex;
        align-items: center;
        transition: all 0.15s ease;
    }

    .github-icon-link:hover {
        color: var(--text-heading);
        background: rgba(74, 222, 128, 0.1);
    }

    /* Main Content */
    .main-content {
        flex: 1;
        display: flex;
        flex-direction: column;
    }

    .welcome-container {
        max-width: 1200px;
        margin: 0 auto;
        padding: 3.5rem 1.5rem 5rem 1.5rem;
        display: flex;
        flex-direction: column;
        gap: 4.5rem;
    }

    /* Hero Banner */
    .hero-banner {
        text-align: center;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1.25rem;
        max-width: 850px;
        margin: 0 auto;
    }

    .hero-badge {
        background: rgba(74, 222, 128, 0.15);
        color: var(--accent-light);
        border: 1px solid rgba(74, 222, 128, 0.35);
        padding: 0.35rem 1rem;
        border-radius: 99px;
        font-size: 0.85rem;
        font-weight: 750;
    }

    .hero-title {
        font-size: 3.5rem;
        font-weight: 900;
        line-height: 1.1;
        color: var(--text-heading);
        margin: 0;
        letter-spacing: -0.04em;
    }

    .gradient-text {
        background: linear-gradient(135deg, #4ade80 0%, #22c55e 50%, #fbbf24 100%);
        -webkit-background-clip: text;
        background-clip: text;
        -webkit-text-fill-color: transparent;
    }

    .hero-desc {
        font-size: 1.2rem;
        color: var(--text);
        line-height: 1.6;
        margin: 0;
    }

    .hero-cta-row {
        display: flex;
        gap: 1rem;
        margin-top: 0.75rem;
        flex-wrap: wrap;
        justify-content: center;
    }

    .btn {
        display: inline-flex;
        align-items: center;
        gap: 0.6rem;
        padding: 0.85rem 1.75rem;
        border-radius: 14px;
        font-weight: 700;
        font-size: 1rem;
        text-decoration: none;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .btn-primary {
        background: #22c55e;
        color: #090b0a;
        border: 1px solid #4ade80;
        box-shadow: 0 4px 18px rgba(34, 197, 94, 0.4);
    }

    .btn-primary:hover {
        background: #4ade80;
        transform: translateY(-2px);
        box-shadow: 0 6px 24px rgba(34, 197, 94, 0.6);
    }

    .btn-outline {
        background: rgba(30, 41, 35, 0.6);
        color: var(--text-heading);
        border: 1px solid var(--border);
    }

    .btn-outline:hover {
        background: rgba(74, 222, 128, 0.15);
        border-color: var(--accent-light);
        transform: translateY(-2px);
    }

    /* Two-Panel Layout */
    .two-panel-grid {
        display: grid;
        grid-template-columns: 1fr 1.25fr;
        gap: 2.5rem;
        align-items: center;
    }

    @media (max-width: 950px) {
        .two-panel-grid {
            grid-template-columns: 1fr;
        }
        .hero-title {
            font-size: 2.75rem;
        }
    }

    /* Phone Mockup */
    .phone-mockup {
        width: 320px;
        margin: 0 auto;
        border: 12px solid #1a221d;
        border-radius: 46px;
        background: #0d120f;
        box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.7), 0 0 30px rgba(34, 197, 94, 0.15);
        position: relative;
        overflow: hidden;
    }

    .phone-speaker {
        width: 60px;
        height: 5px;
        background: #2e3831;
        border-radius: 99px;
        margin: 12px auto 8px auto;
    }

    .phone-screen {
        padding: 1.25rem 1rem;
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .app-mockup-content {
        display: flex;
        flex-direction: column;
        gap: 0.85rem;
    }

    .mockup-topbar {
        display: flex;
        justify-content: space-between;
        align-items: center;
        font-size: 0.8rem;
        color: var(--text-muted);
    }

    .mockup-chip {
        background: rgba(74, 222, 128, 0.2);
        color: var(--accent-light);
        padding: 0.1rem 0.4rem;
        border-radius: 4px;
        font-size: 0.7rem;
        font-family: var(--font-mono);
    }

    .mockup-goblin-card {
        background: rgba(30, 41, 35, 0.6);
        border: 1px solid var(--border);
        border-radius: 16px;
        padding: 0.85rem;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .mockup-goblin-row {
        display: flex;
        align-items: center;
        gap: 0.6rem;
    }

    .mockup-avatar {
        font-size: 1.8rem;
    }

    .mockup-gname {
        font-weight: 800;
        font-size: 0.95rem;
        color: var(--text-heading);
    }

    .mockup-status {
        font-size: 0.75rem;
        color: #86efac;
    }

    .mockup-xp {
        height: 6px;
        background: rgba(19, 23, 21, 0.8);
        border-radius: 99px;
        overflow: hidden;
    }

    .mockup-xp-bar {
        height: 100%;
        background: var(--accent-light);
    }

    .mockup-vitals {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 0.4rem;
    }

    .vital-tile {
        background: rgba(30, 41, 35, 0.4);
        border: 1px solid var(--border);
        border-radius: 10px;
        padding: 0.6rem 0.2rem;
        text-align: center;
        display: flex;
        flex-direction: column;
    }

    .v-val {
        font-size: 1.1rem;
        font-weight: 850;
        color: var(--text-heading);
    }

    .v-lbl {
        font-size: 0.65rem;
        color: var(--text-muted);
    }

    .mockup-medgemma-card {
        background: rgba(13, 18, 15, 0.8);
        border: 1px solid rgba(74, 222, 128, 0.25);
        border-radius: 12px;
        padding: 0.75rem;
        display: flex;
        flex-direction: column;
        gap: 0.3rem;
    }

    .medgemma-title {
        font-size: 0.75rem;
        font-weight: 750;
        color: var(--accent-light);
    }

    .medgemma-text {
        font-size: 0.75rem;
        color: #cbd5e1;
        font-style: italic;
        line-height: 1.4;
    }

    .mockup-btn {
        background: #22c55e;
        color: #090b0a;
        font-weight: 800;
        font-size: 0.85rem;
        padding: 0.65rem;
        border-radius: 10px;
        text-align: center;
        box-shadow: 0 2px 8px rgba(34, 197, 94, 0.3);
    }

    /* Features Grid */
    .features-section {
        display: flex;
        flex-direction: column;
        gap: 2.5rem;
    }

    .sec-header {
        text-align: center;
    }

    .sec-badge {
        color: var(--accent-light);
        text-transform: uppercase;
        font-size: 0.85rem;
        font-weight: 750;
        letter-spacing: 0.1em;
        display: block;
        margin-bottom: 0.4rem;
    }

    .sec-header h3 {
        font-size: 2.25rem;
        font-weight: 850;
        color: var(--text-heading);
        margin: 0;
    }

    .features-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
        gap: 1.5rem;
    }

    .feat-card {
        border-radius: 18px;
        padding: 2rem;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        transition: transform 0.2s ease, border-color 0.2s ease;
    }

    .feat-card:hover {
        transform: translateY(-3px);
        border-color: rgba(74, 222, 128, 0.4);
    }

    .feat-icon {
        font-size: 2rem;
        margin-bottom: 0.5rem;
    }

    .feat-card h4 {
        font-size: 1.25rem;
        font-weight: 750;
        color: var(--text-heading);
        margin: 0;
    }

    .feat-card p {
        font-size: 0.95rem;
        color: var(--text);
        line-height: 1.55;
        margin: 0;
    }

    /* Devices Section */
    .devices-section {
        display: flex;
        flex-direction: column;
        gap: 2rem;
    }

    /* Footer */
    .app-footer {
        border-top: 1px solid var(--border);
        padding: 2rem;
        background: rgba(9, 11, 10, 0.8);
    }

    .footer-content {
        max-width: 1200px;
        margin: 0 auto;
        display: flex;
        justify-content: space-between;
        align-items: center;
        flex-wrap: wrap;
        gap: 1rem;
        color: var(--text-muted);
        font-size: 0.9rem;
    }

    .footer-links {
        display: flex;
        gap: 1.5rem;
    }

    .footer-links a {
        color: var(--text-muted);
        text-decoration: none;
        transition: color 0.15s ease;
    }

    .footer-links a:hover {
        color: var(--accent-light);
    }
</style>

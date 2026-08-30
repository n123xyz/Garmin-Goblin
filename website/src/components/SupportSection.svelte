<script lang="ts">
    const faqs = [
        {
            q: "Do I need the official Garmin Connect app or an account?",
            a: "No! Garmin Goblin connects directly to your watch over Bluetooth LE and USB MTP using open-source reverse-engineered protocol implementations. You do not need an account, an internet connection, or Garmin's proprietary software."
        },
        {
            q: "How does USB MTP offloading work on Android?",
            a: "Connect your watch to your Android phone via a standard USB-C to Garmin charging cable or OTG adapter. When you tap 'Sync USB (MTP)', Android prompts for USB Host permission. Once authorized, Garmin Goblin reads directly from the watch's internal MTP filesystem (/GARMIN/ACTIVITY and /GARMIN/MONITOR) to ingest workout .FIT files in seconds."
        },
        {
            q: "Which Garmin watches are compatible?",
            a: "The Garmin Forerunner 165 is currently our only hardware-tested and verified watch. Other modern Garmin watches supporting standard BLE GATT and USB MTP storage (such as other Forerunner models, Fenix, Epix, Venu, and Instinct) are expected to work with standard protocols, but remain community-supported. Feedback and test reports are welcome!"
        },
        {
            q: "What are the hardware requirements for on-device MedGemma AI?",
            a: "On Android, MedGemma 1.5 4B IT utilizes Google LiteRT LM accelerated by device NPUs (Qualcomm Hexagon Snapdragon 8 Gen 2/3/4, Google Tensor G3/G4 on Pixel 8/9, or MediaTek Dimensity). On Linux desktop, Garmin Goblin seamlessly falls back to your local Ollama instance."
        },
        {
            q: "Where is my health data stored?",
            a: "All biometrics, sleep intervals, parsed workouts, and AI conversations are stored locally in a standard SQLite database within the app's local sandbox on your device. Garmin Goblin contains zero telemetry, analytics trackers, or external cloud sync APIs."
        }
    ];

    let openFaqIndex = $state<number | null>(0);

    function toggleFaq(idx: number) {
        openFaqIndex = openFaqIndex === idx ? null : idx;
    }
</script>

<div class="support-container">
    <div class="support-header">
        <span class="eyebrow">Get Involved</span>
        <h2>Community & Support</h2>
        <p class="subtitle">Garmin Goblin is free, open-source, and powered by contributors like you.</p>
    </div>

    <div class="support-grid">
        <!-- Developer Contribution Card -->
        <div class="support-card glassmorphic">
            <div class="card-icon">💻</div>
            <h3>For Developers</h3>
            <p>
                We welcome contributions! Whether you want to optimize on-device NPU inference, add reverse-engineered Garmin telemetry fields, or improve Svelte interfaces.
            </p>
            
            <div class="guide-list">
                <div class="guide-item">
                    <span class="num">1</span>
                    <div class="text">
                        <strong>Fork & Clone</strong>
                        <span>Fork the repository on GitHub and pull your feature branch.</span>
                    </div>
                </div>
                <div class="guide-item">
                    <span class="num">2</span>
                    <div class="text">
                        <strong>Setup Toolchains</strong>
                        <span>Install Rust, pnpm, and Android Studio NDK by following our <a href="#docs-installation">Installation Guide</a>.</span>
                    </div>
                </div>
                <div class="guide-item">
                    <span class="num">3</span>
                    <div class="text">
                        <strong>Verify & Submit PR</strong>
                        <span>Ensure <code>pnpm run check</code> and <code>cargo clippy</code> pass cleanly with zero warnings, then open a Pull Request.</span>
                    </div>
                </div>
            </div>

            <a href="https://github.com/n123xyz/Garmin-Goblin" target="_blank" rel="noopener noreferrer" class="btn btn-github">
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path></svg>
                View Repository on GitHub
            </a>
        </div>

        <!-- Issues & Bug Tracker Card -->
        <div class="support-card glassmorphic highlighted-card">
            <div class="card-icon">🐛</div>
            <h3>Issues & Discussions</h3>
            <p>
                Encountered a bug, have an untested watch model, or want to suggest a new feature?
            </p>
            <p>
                Our issue tracker is open to everyone. When filing watch pairing issues, please include your watch firmware version and relevant <code>adb logcat</code> logs.
            </p>

            <div class="action-links">
                <a href="https://github.com/n123xyz/Garmin-Goblin/issues/new?template=bug_report.md" target="_blank" rel="noopener noreferrer" class="btn btn-secondary">
                    Report a Bug
                </a>
                <a href="https://github.com/n123xyz/Garmin-Goblin/issues/new?template=feature_request.md" target="_blank" rel="noopener noreferrer" class="btn btn-secondary">
                    Request a Feature
                </a>
            </div>
        </div>
    </div>

    <!-- FAQ Accordion Section -->
    <div class="faq-section">
        <h3>Frequently Asked Questions</h3>
        <div class="faq-list">
            {#each faqs as faq, i}
                <div class="faq-item glassmorphic" class:open={openFaqIndex === i}>
                    <button class="faq-question" onclick={() => toggleFaq(i)} aria-expanded={openFaqIndex === i}>
                        <span>{faq.q}</span>
                        <span class="faq-toggle">{openFaqIndex === i ? '−' : '+'}</span>
                    </button>
                    {#if openFaqIndex === i}
                        <div class="faq-answer">
                            <p>{faq.a}</p>
                        </div>
                    {/if}
                </div>
            {/each}
        </div>
    </div>
</div>

<style>
    .support-container {
        max-width: 1100px;
        margin: 0 auto;
        padding: 4rem 1.5rem;
    }

    .support-header {
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

    .support-header h2 {
        font-size: 2.75rem;
        font-weight: 850;
        color: var(--text-heading);
        margin: 0 0 1rem 0;
        letter-spacing: -0.03em;
    }

    .subtitle {
        color: var(--text-muted);
        font-size: 1.15rem;
        max-width: 600px;
        margin: 0 auto;
    }

    .support-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
        gap: 2rem;
        margin-bottom: 5rem;
    }

    .support-card {
        border-radius: 20px;
        padding: 2.5rem;
        display: flex;
        flex-direction: column;
    }

    .card-icon {
        font-size: 2.2rem;
        margin-bottom: 1.25rem;
    }

    .support-card h3 {
        font-size: 1.4rem;
        font-weight: 750;
        color: var(--text-heading);
        margin: 0 0 0.75rem 0;
    }

    .support-card p {
        color: var(--text);
        line-height: 1.6;
        font-size: 0.975rem;
        margin: 0 0 1.5rem 0;
    }

    .guide-list {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
        margin-bottom: 2rem;
    }

    .guide-item {
        display: flex;
        gap: 1rem;
        align-items: flex-start;
    }

    .guide-item .num {
        width: 28px;
        height: 28px;
        border-radius: 50%;
        background: rgba(74, 222, 128, 0.15);
        border: 1px solid rgba(74, 222, 128, 0.4);
        color: var(--accent-light);
        display: flex;
        align-items: center;
        justify-content: center;
        font-weight: 700;
        font-size: 0.85rem;
        flex-shrink: 0;
    }

    .guide-item .text {
        display: flex;
        flex-direction: column;
        gap: 0.2rem;
    }

    .guide-item strong {
        color: var(--text-heading);
        font-size: 0.95rem;
    }

    .guide-item span {
        color: var(--text-muted);
        font-size: 0.875rem;
        line-height: 1.5;
    }

    .btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 0.6rem;
        padding: 0.85rem 1.5rem;
        border-radius: 12px;
        font-weight: 650;
        font-size: 0.95rem;
        text-decoration: none;
        transition: all 0.15s ease;
        margin-top: auto;
    }

    .btn-github {
        background: #1e2923;
        color: #f3f4f6;
        border: 1px solid var(--border);
    }

    .btn-github:hover {
        background: #27382f;
        border-color: var(--accent-light);
        transform: translateY(-2px);
    }

    .btn-secondary {
        background: rgba(74, 222, 128, 0.12);
        color: var(--accent-light);
        border: 1px solid rgba(74, 222, 128, 0.3);
    }

    .btn-secondary:hover {
        background: rgba(74, 222, 128, 0.2);
        border-color: var(--accent-light);
    }

    .action-links {
        display: flex;
        gap: 1rem;
        flex-wrap: wrap;
        margin-top: auto;
    }

    /* FAQ Styling */
    .faq-section {
        max-width: 850px;
        margin: 0 auto;
    }

    .faq-section h3 {
        font-size: 1.85rem;
        font-weight: 800;
        color: var(--text-heading);
        text-align: center;
        margin-bottom: 2rem;
    }

    .faq-list {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .faq-item {
        border-radius: 14px;
        overflow: hidden;
        transition: border-color 0.2s ease;
    }

    .faq-item.open {
        border-color: rgba(74, 222, 128, 0.4);
    }

    .faq-question {
        width: 100%;
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1.25rem 1.5rem;
        background: transparent;
        border: none;
        color: var(--text-heading);
        font-weight: 650;
        font-size: 1.05rem;
        cursor: pointer;
        text-align: left;
    }

    .faq-toggle {
        font-size: 1.4rem;
        color: var(--accent-light);
        margin-left: 1rem;
        line-height: 1;
    }

    .faq-answer {
        padding: 0 1.5rem 1.5rem 1.5rem;
        color: var(--text);
        line-height: 1.65;
        font-size: 0.95rem;
        border-top: 1px solid rgba(74, 222, 128, 0.08);
        padding-top: 1rem;
    }

    .faq-answer p {
        margin: 0;
    }
</style>

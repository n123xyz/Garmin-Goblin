<script lang="ts">
    const devices = [
        {
            family: "Garmin Forerunner 165",
            models: "Forerunner 165 / 165 Music",
            desc: "Primary test device. Verified for Bluetooth LE pairing, background telemetry sync, and USB MTP .FIT file offloading on Android 15 and Linux.",
            icon: "🏃",
            badge: "Tested Hardware"
        },
        {
            family: "Other Forerunner Series",
            models: "245 • 255 • 265 • 955 • 965",
            desc: "Uses standard Garmin MTP file structure and BLE protocol. Expected to work, but not yet hardware-tested.",
            icon: "⌚",
            badge: "Untested / Expected"
        },
        {
            family: "Fenix & Epix Series",
            models: "Fenix 7 • Fenix 8 • Epix Gen 2",
            desc: "Same standard MTP filesystem and Gadgetbridge BLE profile. Community testing welcome.",
            icon: "🏔️",
            badge: "Untested / Expected"
        },
        {
            family: "Venu & Vivoactive",
            models: "Venu 2 • Venu 3 • Vivoactive 5",
            desc: "Standard Garmin BLE vitals and USB MTP storage layout. Community testing welcome.",
            icon: "✨",
            badge: "Untested / Expected"
        },
        {
            family: "Instinct Series",
            models: "Instinct 2 • Instinct 2X Solar",
            desc: "Standard Garmin BLE and USB MTP interface. Community testing welcome.",
            icon: "🧭",
            badge: "Untested / Expected"
        }
    ];

    let activeIndex = $state(0);

    function next() {
        activeIndex = (activeIndex + 1) % devices.length;
    }

    function prev() {
        activeIndex = (activeIndex - 1 + devices.length) % devices.length;
    }
</script>

<div class="carousel-container">
    <div class="carousel-controls">
        <button class="nav-arrow" onclick={prev} aria-label="Previous device">←</button>
        <span class="carousel-counter">{activeIndex + 1} / {devices.length}</span>
        <button class="nav-arrow" onclick={next} aria-label="Next device">→</button>
    </div>

    <div class="carousel-track">
        {#each devices as dev, i}
            <div class="device-card glassmorphic" class:active={activeIndex === i}>
                <div class="card-top">
                    <span class="device-icon">{dev.icon}</span>
                    <span class="device-badge">{dev.badge}</span>
                </div>
                <h4 class="device-family">{dev.family}</h4>
                <div class="device-models">{dev.models}</div>
                <p class="device-desc">{dev.desc}</p>
                <div class="support-tags">
                    {#if dev.badge === 'Tested Hardware'}
                        <span class="tag">✓ Verified BLE</span>
                        <span class="tag">✓ Verified MTP</span>
                        <span class="tag">✓ Verified FIT</span>
                    {:else}
                        <span class="tag tag-untested">Standard BLE</span>
                        <span class="tag tag-untested">Standard MTP</span>
                        <span class="tag tag-untested">Untested</span>
                    {/if}
                </div>
            </div>
        {/each}
    </div>
</div>

<style>
    .carousel-container {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
        width: 100%;
    }

    .carousel-controls {
        display: flex;
        justify-content: flex-end;
        align-items: center;
        gap: 1rem;
    }

    .nav-arrow {
        width: 38px;
        height: 38px;
        border-radius: 50%;
        background: rgba(30, 41, 35, 0.6);
        border: 1px solid var(--border);
        color: var(--text-heading);
        font-size: 1.2rem;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.15s ease;
    }

    .nav-arrow:hover {
        background: rgba(74, 222, 128, 0.2);
        border-color: var(--accent-light);
        color: var(--accent-light);
    }

    .carousel-counter {
        font-size: 0.85rem;
        color: var(--text-muted);
        font-family: var(--font-mono);
    }

    .carousel-track {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
        gap: 1.25rem;
    }

    .device-card {
        border-radius: 18px;
        padding: 1.75rem;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        transition: all 0.2s ease;
    }

    .device-card.active {
        border-color: rgba(74, 222, 128, 0.45);
        background: rgba(19, 23, 21, 0.8);
        box-shadow: 0 0 20px rgba(74, 222, 128, 0.15);
    }

    .card-top {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .device-icon {
        font-size: 2rem;
    }

    .device-badge {
        background: rgba(74, 222, 128, 0.15);
        color: var(--accent-light);
        border: 1px solid rgba(74, 222, 128, 0.35);
        padding: 0.2rem 0.65rem;
        border-radius: 99px;
        font-size: 0.75rem;
        font-weight: 700;
    }

    .device-family {
        font-size: 1.25rem;
        font-weight: 800;
        color: var(--text-heading);
        margin: 0;
    }

    .device-models {
        font-size: 0.85rem;
        color: var(--accent-light);
        font-weight: 600;
    }

    .device-desc {
        font-size: 0.9rem;
        color: var(--text);
        line-height: 1.5;
        margin: 0;
    }

    .support-tags {
        display: flex;
        gap: 0.5rem;
        margin-top: auto;
        padding-top: 0.75rem;
        border-top: 1px solid rgba(74, 222, 128, 0.1);
        flex-wrap: wrap;
    }

    .tag {
        font-size: 0.75rem;
        color: #86efac;
        font-weight: 600;
    }

    .tag-untested {
        color: var(--text-muted);
        font-style: italic;
    }
</style>

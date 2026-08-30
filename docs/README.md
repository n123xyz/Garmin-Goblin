# Garmin Goblin Documentation 👹

**Garmin Goblin** is a local-first companion app and gamified health tracker for Garmin watches. Built with **Tauri v2**, **Svelte 5**, and **Rust**, it connects directly to Garmin watches via **USB MTP** and **Bluetooth LE**, stores all data locally in SQLite, and features a virtual goblin companion named **Gribble** who levels up as you complete workouts and hit daily health targets.

---

## Core Principles

1. **Local Storage**: All workout `.FIT` files, heart rate logs, sleep intervals, and chat logs are stored in a local SQLite database on your device. No cloud account or external internet connection is required.
2. **Direct Watch Connection**: Connects to watches via Bluetooth LE (using protocols reverse-engineered by the Gadgetbridge project) or directly over a USB cable via MTP for fast file offloading.
3. **Local AI**: Runs quantized models like MedGemma 1.5 4B and Gemma 4 on supported mobile NPUs via Google LiteRT (or local Ollama on desktop) to summarize your workouts and sleep data locally.
4. **Gamified Motivation**: An interactive goblin companion, Gribble, gains experience points and levels up as you log workouts, steps, and sleep.

---

## Documentation Sections

Explore the technical guides below:

### 1. [System Architecture](file:///home/user/Documents/garmin/garmin-goblin/docs/architecture.md)
Overview of the frontend (Svelte 5) and backend (Rust Tauri v2), IPC commands, SQLite schema migrations, Android Kotlin LiteRT plugin, BLE protocol handling, and USB MTP file transfer.

### 2. [Features](file:///home/user/Documents/garmin/garmin-goblin/docs/features.md)
Detailed walkthrough of Garmin BLE & USB MTP sync, Gribble progression and quests, local AI integration, cognitive tests (CARIT, FACENAME, VISMOTOR), and breathing exercises.

### 3. [Installation & Setup](file:///home/user/Documents/garmin/garmin-goblin/docs/installation.md)
Instructions for running on Linux desktop, compiling the Android APK, extracting Qualcomm Hexagon NPU libraries, and configuring USB OTG permissions on Android.

### 4. [Developer Guide](file:///home/user/Documents/garmin/garmin-goblin/docs/development.md)
Codebase layout, database schema migrations, registering new Tauri IPC commands, and testing standards.

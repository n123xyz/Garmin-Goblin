# Garmin Goblin 👹

**Garmin Goblin** is an open-source, local-first companion app and gamified health tracker for Garmin watches. Built with **Tauri v2**, **Svelte 5**, and **Rust**, it connects directly to Garmin watches via **USB MTP** and **Bluetooth Low Energy**, stores your data locally in SQLite, and features a virtual goblin companion named **Gribble** who levels up as you complete workouts, hit step targets, and get good sleep.

On Android devices with supported NPUs, it can also run **Google MedGemma 1.5 4B** and **Gemma 4** locally via **Google LiteRT** for on-device health chat and workout summaries, with Ollama fallback on desktop.

---

## 🌐 Project Links

* **Website & Docs:** [n123xyz.github.io/Garmin-Goblin](https://n123xyz.github.io/Garmin-Goblin/)
* **Repository:** [github.com/n123xyz/Garmin-Goblin](https://github.com/n123xyz/Garmin-Goblin)
* **Technical Docs:** [docs/README.md](docs/README.md)

---

## 🛠️ Built With

![Tauri v2](https://img.shields.io/badge/Tauri_v2-%2324C8DB?style=for-the-badge&logo=tauri&logoColor=white)
![Svelte 5](https://img.shields.io/badge/Svelte_5-%23FF3E00?style=for-the-badge&logo=svelte&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-black?style=for-the-badge&logo=rust&logoColor=white)
![Kotlin](https://img.shields.io/badge/Kotlin-%237F52FF?style=for-the-badge&logo=kotlin&logoColor=white)
![Android](https://img.shields.io/badge/Android-%233DDC84?style=for-the-badge&logo=android&logoColor=white)
![Google LiteRT](https://img.shields.io/badge/Google_LiteRT-%234285F4?style=for-the-badge&logo=google&logoColor=white)
![SQLite](https://img.shields.io/badge/SQLite-%2307405E?style=for-the-badge&logo=sqlite&logoColor=white)
![License: MIT](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)

---

## ✨ Features

- **Direct Garmin Sync (BLE & USB MTP)**:
  - **USB MTP Offload**: Plug your watch into your phone or PC with a USB cable to copy `.FIT` activity files, continuous heart rate, and sleep data in seconds.
  - **Bluetooth Low Energy**: Background sync for steps, heart rate, and notifications using protocol implementations based on Gadgetbridge.
  - **No Cloud Required**: Works directly with your watch. No Garmin Connect account or internet connection needed.
- **Local SQLite Storage**:
  - All workout data, biometrics, and chat logs are stored in a local SQLite database on your device.
- **Gribble the Goblin Companion**:
  - A gamified companion that gains XP and levels up when you log workouts, reach your step goal, and get good sleep.
  - Earn gold to buy gear and snacks for Gribble in the companion shop.
- **Local AI (LiteRT & Ollama)**:
  - Runs quantized MedGemma 1.5 4B and Gemma 4 on supported mobile NPUs (Qualcomm Snapdragon, MediaTek Dimensity, Google Tensor) via LiteRT.
  - Summarizes workouts and sleep, calculates recovery trends, and answers fitness questions without sending data to external APIs.
  - Falls back to local Ollama on desktop.
- **Cognitive Reaction & Memory Tasks**:
  - Includes three cognitive tasks:
    - **CARIT**: Go / No-Go reaction time and impulse control task.
    - **FACENAME**: Face-to-name association and recall task.
    - **VISMOTOR**: Visual-spatial coordination and reaction speed.
- **Breathing & Mindfulness**:
  - Box breathing and body scan visualizers to help wind down before sleep.
  - Voice, text, and photo meal logging.

---

## 🚀 Quick Start

For detailed setup instructions, see the [Installation Guide](docs/installation.md).

### Desktop Development (Linux)
```bash
git clone https://github.com/n123xyz/Garmin-Goblin.git
cd Garmin-Goblin

# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev
```

### Android APK Build
Requires Android Studio, SDK (API 34+), NDK, and Kotlin:

```bash
# 1. Fetch Qualcomm NPU libraries
chmod +x ./libraries/fetch_qualcomm_library.sh
./libraries/fetch_qualcomm_library.sh

# 2. Build release APK
pnpm tauri android build --apk

# 3. Install on connected phone
adb install -r src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk
```

---

## ⌚ Hardware Compatibility

* **Tested Device:** **Garmin Forerunner 165** (tested for BLE pairing, telemetry sync, and USB MTP `.FIT` offloading on Android 15 and Linux).
* **Other Models:** Other modern Garmin devices using standard MTP filesystem layout (`/GARMIN/ACTIVITY`, `/GARMIN/MONITOR`) and Garmin BLE protocols should theoretically work, but have not been tested yet. If you test another model, please let us know!

---

## 📄 License & Attributions

Licensed under the [MIT License](LICENSE).

This project incorporates and builds upon:
- **[Gadgetbridge](https://gadgetbridge.org/)**: Reverse-engineered Garmin BLE protocols.
- **[Google LiteRT](https://ai.google.dev/edge/litert)**: On-device mobile ML runtime.
- **[Tauri](https://tauri.app/)**: Rust desktop and mobile application framework.
- **[MedGemma](https://huggingface.co/litert-community/MedGemma-1.5-4B-IT)**: Google MedGemma model weights subject to the Gemma Open Model License.

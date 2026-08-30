# Installation & Setup Guide 🛠️

This guide covers building and running **Garmin Goblin** on Linux desktops and compiling the Android APK for mobile devices.

---

## 1. Prerequisites

Before building, install the core development toolchains:

### Toolchains:
1. **Rust & Cargo**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source "$HOME/.cargo/env"
   ```
2. **Node.js (v20+) & pnpm**:
   ```bash
   npm install -g pnpm
   ```
3. **Linux Native Dependencies (Desktop)**:
   ```bash
   sudo apt update && sudo apt install -y \
       libglib2.0-dev libgtk-3-dev libjavascriptcoregtk-4.1-dev \
       libsoup-3.0-dev libwebkitgtk-6.0-dev libssl-dev libwebkit2gtk-4.1-dev \
       pkg-config build-essential
   ```

---

## 2. Desktop Setup & Running

1. **Clone the repository**:
   ```bash
   git clone https://github.com/n123xyz/Garmin-Goblin.git
   cd Garmin-Goblin
   ```

2. **Install Frontend Dependencies**:
   ```bash
   pnpm install
   ```

3. **Run in Desktop Development Mode**:
   ```bash
   pnpm tauri dev
   ```

4. **Build Production Desktop Package**:
   ```bash
   pnpm tauri build
   ```
   The generated desktop binaries (AppImage, `.deb`, or raw binary) will be located in `src-tauri/target/release/bundle/`.

---

## 3. Android Mobile Compilation (LiteRT + NPU)

### A. Environment Configuration
Ensure Android Studio, Android SDK (API 34+), NDK (`r27` or higher), and Kotlin are configured in your shell profile:

```bash
export ANDROID_HOME=$HOME/Android/Sdk
export NDK=$ANDROID_HOME/ndk/30.0.14904198
export JAVA_HOME=/path/to/android-studio/jbr
export KOTLIN_HOME=$HOME/.sdkman/candidates/kotlin/2.3.21
export PATH=$JAVA_HOME/bin:$KOTLIN_HOME/bin:$PATH
```

### B. Extract Qualcomm Hexagon NPU Libraries
Garmin Goblin bundles NPU acceleration libraries for Qualcomm Snapdragon devices. Extract the runtime libraries before compiling:

```bash
chmod +x ./libraries/fetch_qualcomm_library.sh
./libraries/fetch_qualcomm_library.sh
```

### C. Run on Connected Android Device
Connect your Android phone via USB with USB Debugging enabled:

```bash
# Verify device connection
adb devices

# Start live development build
pnpm tauri android dev
```

### D. Build Release APK
To build the standalone release APK:

```bash
pnpm tauri android build --apk
```

The compiled universal APK will be created at:
`src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk`

Install directly via ADB:
```bash
adb install -r src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk
```

---

## 4. Connecting Your Garmin Watch

> [!NOTE]
> **Tested Hardware**: The **Garmin Forerunner 165** is currently the only watch actively tested and verified by the maintainers. Other Garmin watches supporting USB MTP and standard BLE protocols are expected to work, but are community-supported.

### Via USB MTP (Recommended for initial bulk offload)
1. Plug your Garmin watch into your phone using a USB-C to Garmin charging cable or OTG adapter.
2. Ensure your watch is unlocked and placed in **MTP mode** (default on modern Garmin watches).
3. Tap **"Sync USB (MTP)"** in the app.
4. When prompted by Android, grant USB permission to access the Garmin device.

### Via Bluetooth Low Energy
1. Open your phone's Bluetooth settings and ensure Bluetooth is turned on.
2. Place your Garmin watch in pairing mode (*Settings > Connectivity > Phone > Pair Phone*).
3. In Garmin Goblin, open the device card and tap **"Scan & Pair BLE"**.

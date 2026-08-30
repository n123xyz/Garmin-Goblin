# Features 🌟

Garmin Goblin combines direct Garmin watch synchronization, a gamified companion pet, on-device AI summaries, and cognitive assessment exercises into a single offline application.

---

## 1. Dual Garmin Sync (BLE & USB MTP)

Garmin Goblin supports two methods for syncing with Garmin devices:

### Bluetooth Low Energy (BLE)
- **Background Sync**: Continuously syncs steps, heart rate, and current stress throughout the day.
- **Phone Notifications**: Forwards notifications and weather to the watch display.
- **Protocol Support**: Based on the Gadgetbridge project's reverse-engineered Garmin BLE protocols.
- **No Garmin Connect Needed**: Pairs and communicates directly with the watch without proprietary services.

### USB MTP Offload
- **Fast File Transfer**: Connect your watch to your phone or PC with a USB cable to copy `.FIT` files directly from the watch's internal storage (`/GARMIN/ACTIVITY`, `/GARMIN/MONITOR`, `/GARMIN/SLEEP`).
- **Full History**: Pulls months of dense workout files and all-day heart rate data in seconds.
- **Deduplication**: Uses SHA-256 hashes to avoid re-processing files that have already been imported.

---

## 2. Gribble: The Goblin Companion

To make health tracking more fun, your daily activity fuels **Gribble**, an animated virtual pet:

- **XP & Leveling**: Earn XP by logging workouts, hitting step goals, getting good sleep, and completing cognitive tasks.
- **Gold & Shop**: Earn gold as you hit milestones and spend it on snacks or equipment for Gribble.
- **Mood & Status**:
  - Gribble's mood reflects your recent activity and sleep.
  - Feeding Gribble increases his satiety and provides temporary XP bonuses.
- **Daily Quests**: Generates daily and weekly goals (e.g. *"Walk 10,000 steps"*, *"Log an 80+ sleep score"*, *"Complete a box breathing session"*).

---

## 3. On-Device AI (MedGemma & Gemma)

Garmin Goblin can run local language models to summarize your health data:

- **Mobile NPU Acceleration**: Uses Google LiteRT LM to run quantized models (such as MedGemma 1.5 4B or Gemma 4) on supported mobile processors (Qualcomm Snapdragon, MediaTek Dimensity, Google Tensor).
- **Desktop Ollama**: Automatically connects to a local Ollama instance on desktop.
- **Workout & Sleep Summaries**: Generates plain-text summaries of your workouts, resting heart rate trends, and sleep stages.
- **Chat**: Ask fitness, training, and recovery questions with all data processed locally.

---

## 4. Cognitive Reaction & Memory Tasks

Garmin Goblin includes three neuropsychological tasks adapted from open cognitive science research:

### CARIT (Inhibitory Control)
- **Go / No-Go Task**: Rapidly press for target shapes (Go) and withhold responses for distractor shapes (No-Go).
- **Metrics Tracked**: Reaction times (mean, median, standard deviation), omission errors (misses), commission errors (false alarms), and d-prime sensitivity score.

### FACENAME (Associative Memory)
- **Face-Name Memory Task**: Memorize face-name pairs, followed by a recognition test.
- **Metrics Tracked**: Recall accuracy, reaction time, and distractor rejection rate.

### VISMOTOR (Visual-Motor Coordination)
- **Visual-Motor Reaction Task**: Alternate responses to left and right visual cues to test spatial motor speed.
- **Metrics Tracked**: Left vs. right reaction times and hemispheric differences.

---

## 5. Breathing & Mindfulness

- **Box Breathing**: Interactive 4-4-4-4 visualizer to help relax before bed.
- **Body Scan**: Guided muscle relaxation visualizer.
- **Meal Logging**: Log meals via voice audio transcription, text notes, or photos to keep track of daily nutrition.

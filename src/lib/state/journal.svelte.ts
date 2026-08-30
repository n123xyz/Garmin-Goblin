import { invoke } from '@tauri-apps/api/core';
import toast from 'svelte-french-toast';

export interface FoodLog {
  id: number;
  date: string;
  timestamp: string;
  mealType: string;
  description: string;
  calories: number;
  carbsG: number;
  proteinG: number;
  fatG: number;
  fiberG: number;
  sugarG: number;
  sodiumMg: number;
  waterMl: number;
  notes: string | null;
  imagePath: string | null;
  sourceType: string;
  confidenceScore: number | null;
}

export interface FoodLogInput {
  id?: number;
  date: string;
  mealType: string;
  description: string;
  calories?: number;
  carbsG?: number;
  proteinG?: number;
  fatG?: number;
  fiberG?: number;
  sugarG?: number;
  sodiumMg?: number;
  waterMl?: number;
  notes?: string;
  imagePath?: string;
  sourceType?: string;
}

export interface DailyNutritionSummary {
  date: string;
  totalCalories: number;
  totalCarbsG: number;
  totalProteinG: number;
  totalFatG: number;
  totalWaterMl: number;
  mealCount: number;
  logs: FoodLog[];
}

export interface EmotionalLog {
  id: number;
  date: string;
  timestamp: string;
  timeOfDay: string;
  mood: string;
  energyLevel: number;
  motivationLevel: number;
  stressLevel: number;
  perceivedRecovery: number;
  stressors: string[];
  notes: string | null;
  createdAt: string;
}

export interface EmotionalLogInput {
  id?: number;
  date: string;
  timeOfDay?: string;
  mood: string;
  energyLevel: number;
  motivationLevel: number;
  stressLevel: number;
  perceivedRecovery: number;
  stressors: string[];
  notes?: string;
}

import type { CalendarEventItem } from '$lib/types/garmin';

export interface CalendarActivityPill {
  id: number;
  sport: string;
  title: string;
  durationMin: number;
  distanceKm: number;
  calories: number;
  startTime: string;
}

export interface CalendarDaySummary {
  date: string;
  dayOfMonth: number;
  isCurrentMonth: boolean;
  isToday: boolean;
  sleepScore: number;
  steps: number;
  restingHr: number;
  hrvStatus: string;
  hrvRmssd: number;
  stressLevel: number;
  totalFoodCalories: number;
  totalActiveCalories: number;
  foodCount: number;
  activityCount: number;
  latestMood?: string;
  avgEmotionalStress?: number;
  avgEnergyLevel?: number;
  activities: CalendarActivityPill[];
  foods: FoodLog[];
  emotions: EmotionalLog[];
  events: CalendarEventItem[];
}

export interface CalendarMonthResponse {
  year: number;
  month: number;
  monthName: string;
  days: CalendarDaySummary[];
}

const now = new Date();
const initialDateStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;

export const journalState = $state<{
  currentYear: number;
  currentMonth: number;
  selectedDate: string;
  calendarData: CalendarMonthResponse | null;
  selectedDaySummary: CalendarDaySummary | null;
  todaySummary: DailyNutritionSummary | null;
  todayEmotions: EmotionalLog[];
  isLoading: boolean;
}>({
  currentYear: now.getFullYear(),
  currentMonth: now.getMonth() + 1,
  selectedDate: initialDateStr,
  calendarData: null,
  selectedDaySummary: null,
  todaySummary: null,
  todayEmotions: [],
  isLoading: false,
});

export async function loadCalendarMonth(year?: number, month?: number): Promise<void> {
  const targetYear = year ?? journalState.currentYear;
  const targetMonth = month ?? journalState.currentMonth;

  journalState.isLoading = true;
  try {
    const res: CalendarMonthResponse = await invoke('get_calendar_month', {
      year: targetYear,
      month: targetMonth,
    });
    journalState.calendarData = res;
    journalState.currentYear = targetYear;
    journalState.currentMonth = targetMonth;

    // Update selected day summary if present
    const day = res.days.find((d) => d.date === journalState.selectedDate);
    if (day) {
      journalState.selectedDaySummary = day;
    } else if (res.days.length > 0) {
      journalState.selectedDaySummary = res.days[0];
      journalState.selectedDate = res.days[0].date;
    }
  } catch (err) {
    console.error('Failed to load calendar month:', err);
  } finally {
    journalState.isLoading = false;
  }
}

export async function loadTodayNutrition(dateStr?: string): Promise<void> {
  const target = dateStr || journalState.selectedDate;
  try {
    const summary: DailyNutritionSummary = await invoke('get_daily_nutrition_summary', {
      date: target,
    });
    journalState.todaySummary = summary;
  } catch (err) {
    console.error('Failed to load daily nutrition summary:', err);
  }
}

export async function loadEmotionalLogs(dateStr?: string): Promise<void> {
  const target = dateStr || journalState.selectedDate;
  try {
    const logs: EmotionalLog[] = await invoke('get_emotional_logs', {
      date: target,
    });
    journalState.todayEmotions = logs;
  } catch (err) {
    console.error('Failed to load emotional logs:', err);
  }
}

export async function saveFoodLog(input: FoodLogInput): Promise<FoodLog | null> {
  try {
    const saved: FoodLog = await invoke('save_food_log', { log: input });
    toast.success('Food log saved!');
    await loadTodayNutrition(input.date);
    await loadCalendarMonth(journalState.currentYear, journalState.currentMonth);
    return saved;
  } catch (err) {
    toast.error(`Failed to save food log: ${err}`);
    return null;
  }
}

export async function deleteFoodLog(id: number, dateStr: string): Promise<boolean> {
  try {
    await invoke('delete_food_log', { id });
    toast.success('Food entry deleted');
    await loadTodayNutrition(dateStr);
    await loadCalendarMonth(journalState.currentYear, journalState.currentMonth);
    return true;
  } catch (err) {
    toast.error(`Failed to delete food log: ${err}`);
    return false;
  }
}

export async function saveEmotionalLog(input: EmotionalLogInput): Promise<EmotionalLog | null> {
  try {
    const saved: EmotionalLog = await invoke('save_emotional_log', { log: input });
    toast.success('Emotional log recorded!');
    await loadEmotionalLogs(input.date);
    await loadCalendarMonth(journalState.currentYear, journalState.currentMonth);
    return saved;
  } catch (err) {
    toast.error(`Failed to save emotional log: ${err}`);
    return null;
  }
}

export async function deleteEmotionalLog(id: number, dateStr: string): Promise<boolean> {
  try {
    await invoke('delete_emotional_log', { id });
    toast.success('Emotional log removed');
    await loadEmotionalLogs(dateStr);
    await loadCalendarMonth(journalState.currentYear, journalState.currentMonth);
    return true;
  } catch (err) {
    toast.error(`Failed to delete emotional log: ${err}`);
    return false;
  }
}

export function setSelectedDate(dateStr: string): void {
  journalState.selectedDate = dateStr;
  if (journalState.calendarData) {
    const match = journalState.calendarData.days.find((d) => d.date === dateStr);
    if (match) {
      journalState.selectedDaySummary = match;
    }
  }
  loadTodayNutrition(dateStr);
  loadEmotionalLogs(dateStr);
}

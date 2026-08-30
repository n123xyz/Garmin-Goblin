import { invoke } from '@tauri-apps/api/core';
import type { CalendarEventItem } from '$lib/types/garmin';

export const calendarState = $state<{
  calendarPermissionGranted: boolean;
  isSyncingCalendar: boolean;
  calendarEventsForSelectedDate: CalendarEventItem[];
  lastCalendarSyncTime: string | null;
}>({
  calendarPermissionGranted: false,
  isSyncingCalendar: false,
  calendarEventsForSelectedDate: [],
  lastCalendarSyncTime: null,
});

export async function checkCalendarPermission(): Promise<boolean> {
  try {
    const isAndroid = /Android/i.test(navigator.userAgent);
    if (!isAndroid) {
      calendarState.calendarPermissionGranted = true;
      return true;
    }
    const res: any = await invoke('plugin:litert|check_calendar_permission');
    calendarState.calendarPermissionGranted = !!(res && res.granted);
    return calendarState.calendarPermissionGranted;
  } catch (e) {
    console.warn('Check calendar permission failed:', e);
    return false;
  }
}

export async function requestCalendarPermission(): Promise<boolean> {
  try {
    const isAndroid = /Android/i.test(navigator.userAgent);
    if (!isAndroid) {
      calendarState.calendarPermissionGranted = true;
      return true;
    }
    const res: any = await invoke('plugin:litert|request_calendar_permission');
    calendarState.calendarPermissionGranted = !!(res && res.granted);
    return calendarState.calendarPermissionGranted;
  } catch (e) {
    console.warn('Request calendar permission failed:', e);
    return false;
  }
}

export async function syncDeviceCalendar(daysBack = 7, daysForward = 7): Promise<number> {
  if (calendarState.isSyncingCalendar) return 0;
  calendarState.isSyncingCalendar = true;
  try {
    const hasPerm = await checkCalendarPermission();
    if (!hasPerm) {
      return 0;
    }

    const isAndroid = /Android/i.test(navigator.userAgent);
    if (isAndroid) {
      const nowMs = Date.now();
      const startMs = nowMs - daysBack * 86400000;
      const endMs = nowMs + daysForward * 86400000;

      const nativeRes: any = await invoke('plugin:litert|get_calendar_events', {
        payload: {
          startTimeEpochMs: startMs,
          endTimeEpochMs: endMs,
        },
      });

      if (nativeRes && Array.isArray(nativeRes.events)) {
        const mappedEvents: CalendarEventItem[] = nativeRes.events.map((ev: any) => {
          const startDate = new Date(ev.startTime);
          const dateStr = `${startDate.getFullYear()}-${String(startDate.getMonth() + 1).padStart(2, '0')}-${String(startDate.getDate()).padStart(2, '0')}`;
          return {
            id: String(ev.id || `${dateStr}_${ev.startTime}`),
            date: dateStr,
            title: ev.title || 'Untitled Event',
            description: ev.description || '',
            location: ev.location || '',
            startTime: new Date(ev.startTime).toISOString(),
            endTime: new Date(ev.endTime).toISOString(),
            isAllDay: !!ev.isAllDay,
            calendarName: ev.calendarName || 'Samsung Calendar',
            eventColor: ev.eventColor || '#10B981',
          };
        });

        if (mappedEvents.length > 0) {
          await invoke('save_calendar_events', { events: mappedEvents });
        }
        calendarState.lastCalendarSyncTime = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
        return mappedEvents.length;
      }
    }
    return 0;
  } catch (e) {
    console.warn('Failed to sync device calendar:', e);
    return 0;
  } finally {
    calendarState.isSyncingCalendar = false;
  }
}

export async function loadCalendarEventsForDate(dateStr: string): Promise<CalendarEventItem[]> {
  try {
    const events = await invoke<CalendarEventItem[]>('get_calendar_events_for_date', { date: dateStr });
    calendarState.calendarEventsForSelectedDate = events || [];
    return calendarState.calendarEventsForSelectedDate;
  } catch (e) {
    console.warn('Failed to load calendar events for date:', e);
    calendarState.calendarEventsForSelectedDate = [];
    return [];
  }
}

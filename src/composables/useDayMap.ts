import { ref, computed } from "vue";
import type { DayState, TimesheetRecord } from "../types/pmp";

const WEEK = ["日", "一", "二", "三", "四", "五", "六"];

/** 获取当前真实日期 yyyy-MM-dd（每次调用实时计算，避免缓存过期） */
export function getTodayStr(): string {
  const now = new Date();
  const y = now.getFullYear();
  const m = String(now.getMonth() + 1).padStart(2, "0");
  const d = String(now.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

export function useDayMap() {
  const dayMap = ref<Map<string, DayState>>(new Map());
  const selectedDay = ref("");
  const today = ref("");

  function parseYmd(s: string): Date {
    const [y, m, d] = s.split("-").map(Number);
    return new Date(y, m - 1, d);
  }

  function formatDayLabel(dateStr: string): string {
    const dt = parseYmd(dateStr);
    return `${dateStr} 周${WEEK[dt.getDay()]}`;
  }

  function isWeekend(dateStr: string): boolean {
    const wd = parseYmd(dateStr).getDay();
    return wd === 0 || wd === 6;
  }

  function isFilledRecord(r: TimesheetRecord | null | undefined): boolean {
    if (!r) return false;
    if (r.task_overtime != null && r.task_overtime !== "") return true;
    if (Array.isArray(r.id_list) && r.id_list.length > 0) return true;
    if (r.approval_status != null && r.approval_status !== "") return true;
    return false;
  }

  function isRestRecord(r: TimesheetRecord | null | undefined): boolean {
    if (!r) return false;
    const rest = r.is_rest === "1" || (r.is_rest as unknown) === 1 || r.is_rest === "true";
    const leave = r.is_leave === "1" || (r.is_leave as unknown) === 1 || r.is_leave === "true";
    return rest || leave;
  }

  function buildDayStates(list: TimesheetRecord[], todayStr: string): Map<string, DayState> {
    const map = new Map<string, DayState>();

    for (const r of list) {
      const date = r.submit_date;
      if (!date) continue;

      let disabled = false;
      let reason: DayState["reason"] = "未填";

      if (date > todayStr) {
        disabled = true;
        reason = "未来";
      } else if (isRestRecord(r)) {
        disabled = true;
        reason =
          r.is_leave === "1" || (r.is_leave as unknown) === 1 || r.is_leave === "true"
            ? "请假"
            : "休息";
      } else if (isFilledRecord(r)) {
        disabled = true;
        reason = "已填";
      }

      map.set(date, { date, disabled, reason, raw: r });
    }

    // 确保今天存在于 map 中
    if (!map.has(todayStr)) {
      const weekend = isWeekend(todayStr);
      map.set(todayStr, {
        date: todayStr,
        disabled: weekend,
        reason: weekend ? "休息" : "未填",
        raw: null,
      });
    }

    return map;
  }

  function pickDefaultDay(map: Map<string, DayState>, todayStr: string): string {
    const todayInfo = map.get(todayStr);
    if (todayInfo && !todayInfo.disabled) return todayStr;

    const dates = [...map.keys()].sort((a, b) => (a < b ? 1 : -1));
    for (const d of dates) {
      if (!map.get(d)?.disabled) {
        // 超过 3 天的未填不再自动选中，退回今天
        const diff = daysDiff(todayStr, d);
        if (diff <= 3) return d;
        break;
      }
    }
    return todayStr;
  }

  /** 计算 from 到 to 相差几天（from - to） */
  function daysDiff(from: string, to: string): number {
    const a = parseYmd(from);
    const b = parseYmd(to);
    return Math.round((a.getTime() - b.getTime()) / 86400000);
  }

  function updateFromTimesheets(list: TimesheetRecord[], todayStr: string) {
    today.value = todayStr;
    const map = buildDayStates(list, todayStr);
    dayMap.value = map;

    if (!selectedDay.value || !map.has(selectedDay.value) || map.get(selectedDay.value)?.disabled) {
      selectedDay.value = pickDefaultDay(map, todayStr);
    }
  }

  const sortedDates = computed(() => {
    return [...dayMap.value.keys()].sort((a, b) => (a < b ? 1 : -1));
  });

  const selectedInfo = computed(() => {
    return dayMap.value.get(selectedDay.value) ?? null;
  });

  return {
    dayMap,
    selectedDay,
    today,
    sortedDates,
    selectedInfo,
    formatDayLabel,
    updateFromTimesheets,
  };
}

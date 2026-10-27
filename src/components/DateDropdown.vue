<template>
  <div class="day-dd" :class="{ open: isOpen }">
    <button
      type="button"
      class="day-dd-trigger"
      @click.stop="isOpen = !isOpen"
      :aria-expanded="isOpen"
    >
      <span class="left">
        <StatusTag :reason="selectedInfo?.reason ?? '未填'" />
        <span class="date-txt">{{ displayLabel }}</span>
      </span>
      <span class="chev">▾</span>
    </button>
    <div class="day-dd-panel" role="listbox">
      <button
        v-for="date in sortedDates"
        :key="date"
        type="button"
        class="day-dd-item"
        :class="{
          disabled: dayMap.get(date)?.disabled,
          active: date === selectedDay,
        }"
        :aria-selected="date === selectedDay"
        @click="onSelect(date)"
      >
        <StatusTag :reason="dayMap.get(date)?.reason ?? '未填'" />
        <span class="date-txt">{{ formatDayLabel(date) }}</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from "vue";
import StatusTag from "./StatusTag.vue";
import type { DayState } from "../types/pmp";

const props = defineProps<{
  dayMap: Map<string, DayState>;
  sortedDates: string[];
  selectedDay: string;
  selectedInfo: DayState | null;
  formatDayLabel: (date: string) => string;
}>();

const emit = defineEmits<{
  (e: "select", date: string): void;
}>();

const isOpen = ref(false);

const displayLabel = computed(() => {
  return props.selectedDay ? props.formatDayLabel(props.selectedDay) : "请选择日期";
});

function onSelect(date: string) {
  const info = props.dayMap.get(date);
  if (info?.disabled) {
    return;
  }
  emit("select", date);
  isOpen.value = false;
}

watch(isOpen, (open) => {
  if (open) {
    nextTick(() => {
      const active = document.querySelector(".day-dd-panel .day-dd-item.active");
      if (active) {
        active.scrollIntoView({ block: "nearest" });
      }
    });
  }
});

// 点击外部关闭
if (typeof document !== "undefined") {
  document.addEventListener("click", (e: MouseEvent) => {
    const el = document.querySelector(".day-dd");
    if (el && !el.contains(e.target as Node)) {
      isOpen.value = false;
    }
  });
}
</script>

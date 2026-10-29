<template>
  <div class="app-select" :class="{ 'drop-top': placement === 'top' }" ref="wrapRef">
    <button
      :id="id"
      type="button"
      class="as-trigger"
      :class="{ open: isOpen }"
      :aria-expanded="isOpen"
      :aria-haspopup="'listbox'"
      @click.stop="isOpen = !isOpen"
    >
      <span class="as-value">{{ currentLabel }}</span>
      <svg class="as-chev" viewBox="0 0 12 8" aria-hidden="true">
        <path d="M1 1.5l5 5 5-5" />
      </svg>
    </button>

    <transition name="as-pop">
      <div v-if="isOpen" class="as-panel" role="listbox">
        <button
          v-for="opt in options"
          :key="opt.value"
          type="button"
          class="as-option"
          :class="{ active: opt.value === modelValue }"
          role="option"
          :aria-selected="opt.value === modelValue"
          @click.stop="onPick(opt.value)"
        >
          <span class="as-option-label">{{ opt.label }}</span>
          <svg v-if="opt.value === modelValue" class="as-check" viewBox="0 0 14 14" aria-hidden="true">
            <path d="M2.5 7.5l3 3 6-7" />
          </svg>
        </button>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";

const props = defineProps<{
  modelValue: string;
  options: { value: string; label: string }[];
  id?: string;
  /** 面板展开方向；靠近容器底部的场景用 top 避免被 overflow:hidden 裁切 */
  placement?: "bottom" | "top";
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
}>();

const isOpen = ref(false);
const wrapRef = ref<HTMLElement | null>(null);

const currentLabel = computed(
  () => props.options.find((o) => o.value === props.modelValue)?.label ?? props.modelValue,
);

function onPick(value: string) {
  emit("update:modelValue", value);
  isOpen.value = false;
}

function onClickOutside(e: MouseEvent) {
  if (wrapRef.value && !wrapRef.value.contains(e.target as Node)) {
    isOpen.value = false;
  }
}

onMounted(() => document.addEventListener("click", onClickOutside));
onUnmounted(() => document.removeEventListener("click", onClickOutside));
</script>

<style scoped>
.app-select {
  position: relative;
}

/* 触发器：与输入框同一套规格；高度可通过 --as-h 覆盖 */
.as-trigger {
  width: 100%;
  height: var(--as-h, 38px);
  min-height: var(--as-h, 38px);
  margin: 0;
  padding: 0 10px 0 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  background: var(--card);
  color: var(--text);
  border: 1px solid var(--line);
  border-radius: 9px;
  font-size: 13px;
  font-weight: 500;
  text-align: left;
  cursor: pointer;
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
}

.as-trigger:hover {
  border-color: #d3d8e0;
}

.as-trigger.open,
.as-trigger:focus-visible {
  outline: none;
  border-color: var(--brand);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.14);
}

.as-value {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.as-chev {
  width: 11px;
  height: 7px;
  flex-shrink: 0;
  fill: none;
  stroke: var(--muted);
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
  transition: transform 0.16s cubic-bezier(0.16, 1, 0.3, 1), stroke 0.15s ease;
}

.as-trigger.open .as-chev {
  transform: rotate(180deg);
  stroke: var(--brand);
}

/* 选项面板 */
.as-panel {
  position: absolute;
  left: 0;
  right: 0;
  top: calc(100% + 6px);
  z-index: 30;
  background: var(--card);
  border: 1px solid var(--line);
  border-radius: 10px;
  box-shadow: 0 10px 28px rgba(16, 24, 40, 0.14);
  padding: 4px;
  transform-origin: top center;
  /* 选项较多时（如更新镜像节点清单）限高滚动，避免撑出容器 */
  max-height: 264px;
  overflow-y: auto;
  overscroll-behavior: contain;
}

/* 向上展开：贴底部的选择器用 */
.drop-top .as-panel {
  top: auto;
  bottom: calc(100% + 6px);
  transform-origin: bottom center;
}

.as-option {
  width: 100%;
  min-height: 0;
  height: 34px;
  margin: 0;
  padding: 0 10px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  background: transparent;
  border: none;
  border-radius: 7px;
  color: var(--text);
  font-size: 13px;
  font-weight: 500;
  text-align: left;
  cursor: pointer;
  transition: background 0.12s ease, color 0.12s ease;
}

.as-option:hover {
  background: var(--bg);
}

.as-option.active {
  background: var(--brand-light);
  color: var(--brand-dark);
  font-weight: 600;
}

.as-check {
  width: 13px;
  height: 13px;
  flex-shrink: 0;
  fill: none;
  stroke: currentColor;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

/* 展开/收起动效：150ms，轻微上浮 */
.as-pop-enter-active,
.as-pop-leave-active {
  transition: opacity 0.15s cubic-bezier(0.16, 1, 0.3, 1),
    transform 0.15s cubic-bezier(0.16, 1, 0.3, 1);
}

.as-pop-enter-from,
.as-pop-leave-to {
  opacity: 0;
  transform: translateY(-4px) scale(0.98);
}
</style>

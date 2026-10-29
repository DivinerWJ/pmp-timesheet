<template>
  <div class="titlebar">
    <button class="tb-btn" aria-label="最小化" title="最小化" @click="appWindow?.minimize()">
      <svg viewBox="0 0 12 12" aria-hidden="true"><path d="M2.5 6h7" /></svg>
    </button>
    <button class="tb-btn close" aria-label="关闭" title="关闭" @click="appWindow?.close()">
      <svg viewBox="0 0 12 12" aria-hidden="true"><path d="M3.2 3.2l5.6 5.6M8.8 3.2l-5.6 5.6" /></svg>
    </button>
  </div>
</template>

<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";

// 纯浏览器预览（无 Tauri 运行时）时拿不到窗口句柄，按钮退化为空操作
let appWindow: ReturnType<typeof getCurrentWindow> | null = null;
try {
  appWindow = getCurrentWindow();
} catch {
  appWindow = null;
}
</script>

<style scoped>
/* 自绘窗口控制条：叠在页面右上角，整条可拖拽、双击最大化 */
.titlebar {
  position: fixed;
  top: 0;
  right: 0;
  z-index: 100;
  display: flex;
  height: 34px;
  border-top-right-radius: 10px;
}

.tb-btn {
  width: 44px;
  height: 34px;
  min-height: 0;
  display: grid;
  place-items: center;
  padding: 0;
  margin: 0;
  border: none;
  border-radius: 0;
  background: transparent;
  color: var(--muted);
  cursor: pointer;
  transition: background 0.12s ease, color 0.12s ease;
}

.tb-btn:hover {
  background: rgba(15, 23, 42, 0.08);
  color: var(--text);
}

.tb-btn.close:hover {
  background: #e81123;
  color: #fff;
}

.tb-btn svg {
  width: 12px;
  height: 12px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

/* 品牌色顶栏上的白色按钮 */
.titlebar.on-brand .tb-btn {
  color: rgba(255, 255, 255, 0.85);
}

.titlebar.on-brand .tb-btn:hover {
  background: rgba(255, 255, 255, 0.16);
  color: #fff;
}

.titlebar.on-brand .tb-btn.close:hover {
  background: #e81123;
}
</style>

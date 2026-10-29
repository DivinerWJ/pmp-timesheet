<template>
  <transition name="upd-fade">
    <div v-if="isOpen" class="upd-mask" data-tauri-drag-region="false" @click.self="onMaskClick">
      <div class="upd-card" role="dialog" aria-modal="true" aria-label="软件更新">
        <header class="upd-head">
          <span class="head-mark">
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <path d="M12 4v9" />
              <path d="M8.5 9.5L12 13l3.5-3.5" />
              <path d="M5 16.5v1.5A1.5 1.5 0 0 0 6.5 19.5h11A1.5 1.5 0 0 0 19 18v-1.5" />
            </svg>
          </span>
          <div class="head-copy">
            <h3>软件更新</h3>
            <p v-if="currentVersion">当前版本 v{{ currentVersion }}</p>
            <p v-else>PMP 工时</p>
          </div>
          <button
            v-if="canClose"
            type="button"
            class="head-close"
            aria-label="关闭"
            title="关闭"
            @click="closeDialog"
          >
            <svg viewBox="0 0 16 16" aria-hidden="true">
              <path d="M3.5 3.5l9 9M12.5 3.5l-9 9" />
            </svg>
          </button>
        </header>

        <div class="upd-body">
          <!-- 检查中 -->
          <div v-if="phase === 'checking'" class="state">
            <span class="spinner" aria-hidden="true"></span>
            <p class="state-title">正在检查更新…</p>
            <p class="state-sub">连接到发布服务器查询最新版本</p>
          </div>

          <!-- 已是最新 -->
          <div v-else-if="phase === 'latest'" class="state">
            <span class="badge ok" aria-hidden="true">
              <svg viewBox="0 0 24 24"><path d="M7 12.5l3.4 3.4L17 9" /></svg>
            </span>
            <p class="state-title">已是最新版本</p>
            <p class="state-sub">你正在使用 v{{ currentVersion || "—" }}</p>
          </div>

          <!-- 发现新版本 -->
          <div v-else-if="phase === 'available'" class="state">
            <p class="state-title">发现新版本 v{{ latestVersion }}</p>
            <div class="ver-row">
              <span class="ver old">v{{ currentVersion }}</span>
              <svg class="ver-arrow" viewBox="0 0 16 16" aria-hidden="true">
                <path d="M2.5 8h10M9.5 4.5L13 8l-3.5 3.5" />
              </svg>
              <span class="ver new">v{{ latestVersion }}</span>
            </div>
            <div v-if="releaseNotes" class="notes">
              <p class="notes-label">更新内容</p>
              <p class="notes-text">{{ releaseNotes }}</p>
            </div>
            <p v-else class="state-sub">下载安装包后会自动重启应用完成升级</p>
          </div>

          <!-- 下载中 -->
          <div v-else-if="phase === 'downloading'" class="state">
            <p class="state-title">正在下载更新</p>
            <div class="bar" role="progressbar" :aria-valuenow="percent" aria-valuemin="0" aria-valuemax="100">
              <span class="bar-fill" :style="{ width: percent + '%' }"></span>
            </div>
            <div class="bar-meta">
              <span>{{ percent }}%</span>
              <span class="dim">{{ sizeText }}</span>
            </div>
            <p v-if="activeSourceLabel" class="src-active">线路 · {{ activeSourceLabel }}</p>
            <p class="state-sub">下载完成后会自动重启，期间请勿退出应用</p>
          </div>

          <!-- 安装完成 -->
          <div v-else-if="phase === 'ready'" class="state">
            <span class="badge ok" aria-hidden="true">
              <svg viewBox="0 0 24 24"><path d="M7 12.5l3.4 3.4L17 9" /></svg>
            </span>
            <p class="state-title">安装完成</p>
            <p class="state-sub">正在重启应用…</p>
          </div>

          <!-- 出错 -->
          <div v-else-if="phase === 'error'" class="state">
            <span class="badge err" aria-hidden="true">
              <svg viewBox="0 0 24 24">
                <path d="M12 7.5v5.2" />
                <circle cx="12" cy="16.4" r="0.9" fill="currentColor" stroke="none" />
              </svg>
            </span>
            <p class="state-title">检查更新失败</p>
            <p class="state-sub">{{ errorMessage }}</p>
          </div>

          <!-- 下载线路：GitHub 在国内经常连不上，允许用户自己挑镜像 -->
          <div v-if="phase === 'available' || phase === 'error'" class="src-row">
            <span class="src-label">下载线路</span>
            <AppSelect
              v-model="selectedSource"
              :options="sourceOptions"
              placement="top"
              class="src-select"
            />
          </div>
        </div>

        <footer class="upd-foot">
          <template v-if="phase === 'available'">
            <button type="button" class="btn ghost" @click="closeDialog">稍后</button>
            <button type="button" class="btn primary" @click="installUpdate">立即更新</button>
          </template>
          <template v-else-if="phase === 'downloading'">
            <button type="button" class="btn primary" disabled>下载中 {{ percent }}%</button>
          </template>
          <template v-else-if="phase === 'latest'">
            <button type="button" class="btn primary" @click="closeDialog">知道了</button>
          </template>
          <template v-else-if="phase === 'error'">
            <button type="button" class="btn ghost" @click="closeDialog">关闭</button>
            <button type="button" class="btn primary" @click="retry">重试</button>
          </template>
        </footer>
      </div>
    </div>
  </transition>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useUpdater } from "../composables/useUpdater";
import AppSelect from "./AppSelect.vue";

const {
  isOpen,
  phase,
  currentVersion,
  latestVersion,
  releaseNotes,
  errorMessage,
  percent,
  sizeText,
  sourceOptions,
  selectedSource,
  activeSourceLabel,
  installUpdate,
  retry,
  closeDialog,
} = useUpdater();

/** 下载中和安装重启阶段不给关闭入口，避免用户误以为能中断 */
const canClose = computed(() => phase.value !== "downloading" && phase.value !== "ready");

function onMaskClick() {
  if (canClose.value) closeDialog();
}
</script>

<style scoped>
.upd-mask {
  position: fixed;
  inset: 0;
  z-index: 300;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: rgba(16, 24, 40, 0.44);
  -webkit-backdrop-filter: blur(2px);
  backdrop-filter: blur(2px);
}

.upd-card {
  width: 100%;
  max-width: 360px;
  display: flex;
  flex-direction: column;
  background: var(--card);
  border-radius: 16px;
  box-shadow: 0 24px 64px rgba(16, 24, 40, 0.28);
  overflow: hidden;
  animation: upd-in 0.18s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes upd-in {
  from {
    opacity: 0;
    transform: translateY(8px) scale(0.985);
  }
}

/* 头部沿用登录页/设置弹层的品牌渐变语言 */
.upd-head {
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 16px 16px 15px;
  background:
    radial-gradient(140% 120% at 100% 0%, rgba(255, 255, 255, 0.26) 0%, rgba(255, 255, 255, 0) 58%),
    linear-gradient(135deg, #1c4fd6 0%, #2f6bf0 52%, #4f8bff 100%);
  color: #fff;
}

.head-mark {
  width: 34px;
  height: 34px;
  flex-shrink: 0;
  display: grid;
  place-items: center;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.2);
}

.head-mark svg {
  width: 19px;
  height: 19px;
  fill: none;
  stroke: #fff;
  stroke-width: 1.7;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.head-copy {
  min-width: 0;
  flex: 1;
}

.head-copy h3 {
  margin: 0;
  font-size: 14.5px;
  font-weight: 650;
  letter-spacing: 0.01em;
}

.head-copy p {
  margin: 3px 0 0;
  font-size: 11.5px;
  color: rgba(255, 255, 255, 0.76);
}

.head-close {
  width: 24px;
  height: 24px;
  min-height: 0;
  padding: 0;
  margin: 0;
  flex-shrink: 0;
  display: grid;
  place-items: center;
  background: transparent;
  border: none;
  border-radius: 7px;
  color: rgba(255, 255, 255, 0.82);
  cursor: pointer;
}

.head-close:hover {
  background: rgba(255, 255, 255, 0.18);
  color: #fff;
}

.head-close svg {
  width: 12px;
  height: 12px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.7;
  stroke-linecap: round;
}

.upd-body {
  padding: 20px 18px 4px;
}

.state {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding-bottom: 16px;
}

.state-title {
  margin: 0;
  font-size: 14.5px;
  font-weight: 650;
  color: var(--text);
  letter-spacing: -0.01em;
}

.state-sub {
  margin: 6px 0 0;
  font-size: 12px;
  line-height: 1.55;
  color: var(--muted);
}

/* 状态徽章 */
.badge {
  width: 40px;
  height: 40px;
  margin-bottom: 12px;
  display: grid;
  place-items: center;
  border-radius: 50%;
}

.badge svg {
  width: 21px;
  height: 21px;
  fill: none;
  stroke: currentColor;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.badge.ok {
  background: rgba(16, 185, 129, 0.12);
  color: #059669;
}

.badge.err {
  background: rgba(220, 38, 38, 0.1);
  color: var(--danger);
}

.spinner {
  width: 26px;
  height: 26px;
  margin-bottom: 14px;
  border-radius: 50%;
  border: 2.5px solid rgba(47, 107, 240, 0.18);
  border-top-color: var(--brand);
  animation: spin 0.7s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* 版本对比 */
.ver-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 10px;
}

.ver {
  font-size: 12.5px;
  font-variant-numeric: tabular-nums;
  padding: 3px 9px;
  border-radius: 999px;
}

.ver.old {
  color: var(--muted);
  background: var(--bg);
}

.ver.new {
  color: var(--brand);
  background: var(--brand-light, rgba(47, 107, 240, 0.1));
  font-weight: 650;
}

.ver-arrow {
  width: 14px;
  height: 14px;
  fill: none;
  stroke: var(--muted);
  stroke-width: 1.6;
  stroke-linecap: round;
  stroke-linejoin: round;
}

/* 发行说明 */
.notes {
  width: 100%;
  margin-top: 14px;
  text-align: left;
  background: var(--bg);
  border: 1px solid var(--line);
  border-radius: 10px;
  padding: 10px 12px;
  max-height: 132px;
  overflow-y: auto;
}

.notes-label {
  margin: 0 0 5px;
  font-size: 11px;
  font-weight: 600;
  color: var(--muted);
  letter-spacing: 0.02em;
}

.notes-text {
  margin: 0;
  font-size: 12px;
  line-height: 1.65;
  color: var(--text);
  white-space: pre-wrap;
  word-break: break-word;
}

/* 下载线路选择：与弹窗其余控件同一套规格 */
.src-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 0 16px;
}

.src-label {
  flex-shrink: 0;
  font-size: 12px;
  color: var(--muted);
}

.src-select {
  flex: 1;
  min-width: 0;
  --as-h: 32px;
}

.src-select :deep(.as-trigger) {
  font-size: 12.5px;
  padding: 0 8px 0 10px;
  border-radius: 8px;
}

.src-active {
  width: 100%;
  margin: 6px 0 0;
  text-align: right;
  font-size: 11px;
  color: var(--muted);
  font-variant-numeric: tabular-nums;
}

/* 下载进度 */
.bar {
  width: 100%;
  height: 6px;
  margin-top: 14px;
  border-radius: 999px;
  background: var(--bg);
  overflow: hidden;
}

.bar-fill {
  display: block;
  height: 100%;
  border-radius: 999px;
  background: linear-gradient(90deg, var(--brand) 0%, #4f8bff 100%);
  transition: width 0.2s ease;
}

.bar-meta {
  width: 100%;
  margin-top: 8px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 11.5px;
  font-weight: 600;
  color: var(--brand);
  font-variant-numeric: tabular-nums;
}

.bar-meta .dim {
  color: var(--muted);
  font-weight: 500;
}

/* 底部操作栏 */
.upd-foot {
  display: flex;
  gap: 9px;
  justify-content: flex-end;
  padding: 12px 16px;
  border-top: 1px solid var(--line);
  background: #fafbfd;
}

.upd-foot:empty {
  display: none;
}

.btn {
  width: auto;
  min-height: 34px;
  height: 34px;
  margin: 0;
  padding: 0 15px;
  border-radius: 9px;
  font-size: 12.5px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.15s ease, border-color 0.15s ease;
}

.btn.primary {
  background: var(--brand);
  color: #fff;
  border: 1px solid var(--brand);
}

.btn.primary:hover:not(:disabled) {
  background: var(--brand-dark);
  border-color: var(--brand-dark);
}

.btn.primary:disabled {
  opacity: 0.75;
  cursor: default;
  width: 100%;
}

.btn.ghost {
  background: var(--card);
  color: var(--muted);
  border: 1px solid var(--line);
}

.btn.ghost:hover {
  color: var(--text);
  border-color: #cfd8e6;
}

.upd-fade-enter-active,
.upd-fade-leave-active {
  transition: opacity 0.15s ease;
}

.upd-fade-enter-from,
.upd-fade-leave-to {
  opacity: 0;
}
</style>

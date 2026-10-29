<template>
  <div class="user-avatar-wrap" ref="wrapRef">
    <button
      class="avatar-btn"
      :class="{ open: isOpen, 'has-title': !!title }"
      @click.stop="isOpen = !isOpen"
      :title="username"
      :aria-expanded="isOpen"
    >
      <span class="avatar-circle">{{ avatarLetter }}</span>
      <span v-if="title" class="brand-title">{{ title }}</span>
      <svg class="avatar-chev" viewBox="0 0 12 8" aria-hidden="true">
        <path d="M1 1.5l5 5 5-5" />
      </svg>
    </button>

    <transition name="menu-pop">
      <div v-if="isOpen" class="avatar-dropdown" :class="{ 'align-left': !!title }">
        <div class="dropdown-header">
          <span class="avatar-circle large">{{ avatarLetter }}</span>
          <div class="user-info">
            <div class="user-name">{{ username }}</div>
            <div class="user-meta">默认 {{ defaultHours }}h · {{ saveOrSubmit === '2' ? '提交' : '保存' }}</div>
          </div>
          <span class="version-chip">v{{ appVersion }}</span>
        </div>
        <div class="dropdown-divider"></div>
        <div class="dropdown-menu">
          <button class="dropdown-item" @click="onSettings">
            <svg class="item-icon" viewBox="0 0 20 20" aria-hidden="true">
              <circle cx="10" cy="10" r="2.6" />
              <path d="M10 2.8v2M10 15.2v2M2.8 10h2M15.2 10h2M4.9 4.9l1.4 1.4M13.7 13.7l1.4 1.4M15.1 4.9l-1.4 1.4M6.3 13.7l-1.4 1.4" />
            </svg>
            配置管理
          </button>
          <button class="dropdown-item" @click="onCheckUpdate">
            <svg class="item-icon" viewBox="0 0 20 20" aria-hidden="true">
              <path d="M10 3.6v7" />
              <path d="M7.2 7.9L10 10.7l2.8-2.8" />
              <path d="M4.3 13.4v1.4a1.6 1.6 0 0 0 1.6 1.6h8.2a1.6 1.6 0 0 0 1.6-1.6v-1.4" />
            </svg>
            检查更新
          </button>
          <button class="dropdown-item danger" @click="onLogout">
            <svg class="item-icon" viewBox="0 0 20 20" aria-hidden="true">
              <path d="M12.5 6.5v-1a1.5 1.5 0 0 0-1.5-1.5H5.5A1.5 1.5 0 0 0 4 5.5v9A1.5 1.5 0 0 0 5.5 16H11a1.5 1.5 0 0 0 1.5-1.5v-1" />
              <path d="M8.5 10H17M14.5 7.5L17 10l-2.5 2.5" />
            </svg>
            退出登录
          </button>
        </div>
      </div>
    </transition>

    <!-- 设置面板 -->
    <div v-if="showSettings" class="sheet-overlay" @click.self="showSettings = false">
      <div class="sheet" role="dialog" aria-modal="true" aria-label="配置管理">
        <header class="sheet-head">
          <div class="head-icon">
            <svg viewBox="0 0 20 20" aria-hidden="true">
              <circle cx="10" cy="10" r="2.6" />
              <path d="M10 2.8v2M10 15.2v2M2.8 10h2M15.2 10h2M4.9 4.9l1.4 1.4M13.7 13.7l1.4 1.4M15.1 4.9l-1.4 1.4M6.3 13.7l-1.4 1.4" />
            </svg>
          </div>
          <div class="head-text">
            <h3>配置管理</h3>
            <p>账号与应用凭据，保存后会重新登录</p>
          </div>
          <button type="button" class="icon-btn" aria-label="关闭" title="关闭" @click="showSettings = false">
            <svg viewBox="0 0 16 16" aria-hidden="true">
              <path d="M3.5 3.5l9 9M12.5 3.5l-9 9" />
            </svg>
          </button>
        </header>

        <div class="sheet-body">
          <div class="grp">
            <div class="field">
              <label class="f-label" for="set-baseUrl">服务器地址</label>
              <input id="set-baseUrl" v-model="settingsForm.baseUrl" type="text" spellcheck="false" autocomplete="url" />
            </div>
          </div>

          <div class="grp">
            <div class="pair">
              <div class="field">
                <label class="f-label" for="set-username">用户名</label>
                <input id="set-username" v-model="settingsForm.username" type="text" spellcheck="false" autocomplete="username" />
              </div>
              <div class="field">
                <label class="f-label" for="set-password">密码</label>
                <input
                  id="set-password"
                  v-model="settingsForm.password"
                  type="password"
                  autocomplete="current-password"
                  placeholder="输入密码以保存修改"
                />
              </div>
            </div>
          </div>

          <div class="grp divided">
            <div class="pair">
              <div class="field">
                <label class="f-label" for="set-clientId">Client ID</label>
                <input id="set-clientId" v-model="settingsForm.clientId" type="text" spellcheck="false" autocomplete="off" />
              </div>
              <div class="field">
                <label class="f-label" for="set-clientSecret">Client Secret</label>
                <div class="input-with-action">
                  <input
                    id="set-clientSecret"
                    v-model="settingsForm.clientSecret"
                    :type="showClientSecret ? 'text' : 'password'"
                    autocomplete="off"
                  />
                  <button
                    type="button"
                    class="ghost-icon"
                    :aria-label="showClientSecret ? '隐藏 Client Secret' : '显示 Client Secret'"
                    :title="showClientSecret ? '隐藏 Client Secret' : '显示 Client Secret'"
                    @click="showClientSecret = !showClientSecret"
                  >
                    <svg v-if="showClientSecret" viewBox="0 0 24 24" aria-hidden="true">
                      <path d="M3 3l18 18" />
                      <path d="M10.6 5.5A9.9 9.9 0 0 1 12 5.4c5 0 9 5 9 6.6 0 .8-1 2.2-2.6 3.5" />
                      <path d="M6.3 7.4C4.2 8.9 3 10.6 3 12c0 1.6 4 6.6 9 6.6 1.5 0 2.9-.4 4.1-1.1" />
                      <path d="M9.9 10.2a3 3 0 0 0 4 4.2" />
                    </svg>
                    <svg v-else viewBox="0 0 24 24" aria-hidden="true">
                      <path d="M3 12c0-1.6 4-6.6 9-6.6s9 5 9 6.6-4 6.6-9 6.6S3 13.6 3 12z" />
                      <circle cx="12" cy="12" r="2.9" />
                    </svg>
                  </button>
                </div>
              </div>
            </div>
          </div>

          <div class="grp">
            <div class="pair">
              <div class="field">
                <label class="f-label" for="set-hours">默认工时</label>
                <input id="set-hours" v-model.number="settingsForm.defaultHours" type="number" step="0.5" min="0" />
              </div>
              <div class="field">
                <label class="f-label" for="set-saveOrSubmit">保存 / 提交</label>
                <AppSelect
                  id="set-saveOrSubmit"
                  v-model="settingsForm.saveOrSubmit"
                  :options="[
                    { value: '1', label: '保存' },
                    { value: '2', label: '提交' },
                  ]"
                />
              </div>
            </div>
            <div class="field">
              <label class="f-label" for="set-desc">默认工作描述</label>
              <input id="set-desc" v-model="settingsForm.defaultTaskDesc" type="text" placeholder="可留空" />
            </div>
          </div>

          <div class="grp">
            <div class="switch-row">
              <div class="switch-text">
                <div class="switch-title">记住工作描述</div>
                <div class="switch-desc">提交后保留描述，下次打开自动填入</div>
              </div>
              <button
                type="button"
                class="switch"
                :class="{ on: rememberTaskDesc }"
                role="switch"
                :aria-checked="rememberTaskDesc ? 'true' : 'false'"
                :title="rememberTaskDesc ? '点击关闭' : '点击开启'"
                @click="toggleRememberDesc"
              >
                <span class="switch-knob"></span>
              </button>
            </div>

            <div class="switch-row">
              <div class="switch-text">
                <div class="switch-title">记住选中的项目</div>
                <div class="switch-desc">下次打开自动选中上次提交的项目</div>
              </div>
              <button
                type="button"
                class="switch"
                :class="{ on: rememberTaskId }"
                role="switch"
                :aria-checked="rememberTaskId ? 'true' : 'false'"
                :title="rememberTaskId ? '点击关闭' : '点击开启'"
                @click="toggleRememberTask"
              >
                <span class="switch-knob"></span>
              </button>
            </div>
          </div>
        </div>

        <footer class="sheet-foot">
          <button type="button" class="btn-ghost" @click="showSettings = false">取消</button>
          <button type="button" class="btn-primary" @click="onSaveSettings">保存并重新登录</button>
        </footer>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { useAppVersion } from "../composables/useAppVersion";
import AppSelect from "./AppSelect.vue";

const appVersion = useAppVersion();

const props = defineProps<{
  username: string;
  /** 与头像合成一个入口的应用标题；传入后菜单改为左对齐 */
  title?: string;
  defaultHours: number;
  saveOrSubmit: string;
  baseUrl?: string;
  /** 已保存的密码，用于设置面板回填（掩码显示） */
  password?: string;
  clientId?: string;
  clientSecret?: string;
  defaultTaskDesc?: string;
  /** 是否记住工作描述（提交后保留、下次自动填入） */
  rememberTaskDesc?: boolean;
  /** 是否记住选中的任务（下次自动选中） */
  rememberTaskId?: boolean;
}>();

const emit = defineEmits<{
  (e: "logout"): void;
  (e: "check-update"): void;
  (e: "preferences", prefs: { rememberTaskDesc: boolean; rememberTaskId: boolean }): void;
  (
    e: "relogin",
    req: {
      baseUrl?: string;
      username: string;
      password: string;
      clientId: string;
      clientSecret: string;
      defaultHours?: number;
      saveOrSubmit?: string;
      defaultTaskDesc?: string;
    }
  ): void;
}>();

const isOpen = ref(false);
const showSettings = ref(false);
const showClientSecret = ref(false);
const wrapRef = ref<HTMLElement | null>(null);

const avatarLetter = computed(() => {
  return (props.username || "?").charAt(0).toUpperCase();
});

const settingsForm = ref({
  baseUrl: "",
  username: "",
  password: "",
  clientId: "",
  clientSecret: "",
  defaultHours: 7.5,
  saveOrSubmit: "2",
  defaultTaskDesc: "",
});

// 每次打开设置面板时，从 props 重新填充表单
watch(showSettings, (open) => {
  if (open) {
    settingsForm.value = {
      baseUrl: props.baseUrl || "",
      username: props.username,
      password: props.password || "",
      clientId: props.clientId || "",
      clientSecret: props.clientSecret || "",
      defaultHours: props.defaultHours,
      saveOrSubmit: props.saveOrSubmit,
      defaultTaskDesc: props.defaultTaskDesc || "",
    };
  }
});

function onSettings() {
  isOpen.value = false;
  showSettings.value = true;
}

function onCheckUpdate() {
  isOpen.value = false;
  emit("check-update");
}

function onLogout() {
  isOpen.value = false;
  emit("logout");
}

function toggleRememberDesc() {
  emit("preferences", {
    rememberTaskDesc: !props.rememberTaskDesc,
    rememberTaskId: !!props.rememberTaskId,
  });
}

function toggleRememberTask() {
  emit("preferences", {
    rememberTaskDesc: !!props.rememberTaskDesc,
    rememberTaskId: !props.rememberTaskId,
  });
}

function onSaveSettings() {
  showSettings.value = false;
  emit("relogin", {
    baseUrl: settingsForm.value.baseUrl || undefined,
    username: settingsForm.value.username,
    password: settingsForm.value.password,
    clientId: settingsForm.value.clientId,
    clientSecret: settingsForm.value.clientSecret,
    defaultHours: settingsForm.value.defaultHours,
    saveOrSubmit: settingsForm.value.saveOrSubmit,
    defaultTaskDesc: settingsForm.value.defaultTaskDesc || undefined,
  });
}

function onClickOutside(e: MouseEvent) {
  if (wrapRef.value && !wrapRef.value.contains(e.target as Node)) {
    isOpen.value = false;
  }
}

onMounted(() => {
  document.addEventListener("click", onClickOutside);
});

onUnmounted(() => {
  document.removeEventListener("click", onClickOutside);
});
</script>

<style scoped>
.user-avatar-wrap {
  position: relative;
  display: inline-block;
}

/* 左上角入口：头像 + 应用标题合成一个整体 */
.avatar-btn {
  display: flex;
  align-items: center;
  gap: 9px;
  height: 40px;
  min-height: 40px;
  padding: 0 8px 0 0;
  margin: 0;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 999px;
  cursor: pointer;
  -webkit-user-select: none;
  user-select: none;
  transition: background 0.15s ease;
}

.avatar-btn.has-title {
  padding: 5px;
}

.avatar-btn:hover,
.avatar-btn.open {
  background: rgba(255, 255, 255, 0.14);
}

.avatar-btn:active:not(:disabled) {
  transform: none;
}

.brand-title {
  font-size: 17px;
  font-weight: 700;
  letter-spacing: 0.02em;
  color: #fff;
  white-space: nowrap;
  cursor: pointer;
  -webkit-user-select: none;
  user-select: none;
}

.avatar-circle {
  width: 30px;
  height: 30px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.2);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  font-weight: 700;
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.22);
}

.avatar-chev {
  width: 9px;
  height: 6px;
  margin-left: 1px;
  fill: none;
  stroke: rgba(255, 255, 255, 0.75);
  stroke-width: 1.9;
  stroke-linecap: round;
  stroke-linejoin: round;
  transition: transform 0.16s cubic-bezier(0.16, 1, 0.3, 1), stroke 0.15s ease;
}

.avatar-btn:hover .avatar-chev {
  stroke: #fff;
}

.avatar-btn.open .avatar-chev {
  transform: rotate(180deg);
}

.avatar-circle.large {
  width: 40px;
  height: 40px;
  font-size: 17px;
  background: linear-gradient(135deg, var(--brand) 0%, var(--brand-dark) 100%);
  box-shadow: 0 2px 8px rgba(29, 78, 216, 0.28);
}

.avatar-dropdown {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  width: 236px;
  background: var(--card);
  border: 1px solid var(--line);
  border-radius: 14px;
  box-shadow: 0 4px 12px rgba(16, 24, 40, 0.08), 0 16px 40px rgba(16, 24, 40, 0.16);
  z-index: 100;
  overflow: hidden;
  transform-origin: top right;
}

.avatar-dropdown.align-left {
  left: 0;
  right: auto;
  transform-origin: top left;
}

/* 弹出动效：150ms，从右上角轻微展开 */
.menu-pop-enter-active,
.menu-pop-leave-active {
  transition: opacity 0.15s cubic-bezier(0.16, 1, 0.3, 1),
    transform 0.15s cubic-bezier(0.16, 1, 0.3, 1);
}

.menu-pop-enter-from,
.menu-pop-leave-to {
  opacity: 0;
  transform: translateY(-5px) scale(0.96);
}

.dropdown-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 14px 12px;
}

.user-info {
  min-width: 0;
  flex: 1;
}

.user-name {
  font-size: 14px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.user-meta {
  font-size: 12px;
  color: var(--muted);
  margin-top: 2px;
}

.version-chip {
  flex-shrink: 0;
  font-size: 10.5px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: var(--brand);
  background: var(--brand-light);
  padding: 3px 8px;
  border-radius: 999px;
}

.dropdown-divider {
  height: 1px;
  background: var(--line);
}

.dropdown-menu {
  padding: 6px;
}

.dropdown-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 10px;
  border: none;
  background: none;
  cursor: pointer;
  font-size: 13.5px;
  font-weight: 500;
  color: var(--text);
  text-align: left;
  border-radius: 9px;
  margin: 0;
  min-height: 0;
  transition: background 0.12s ease;
}

.dropdown-item:hover {
  background: var(--bg);
}

.dropdown-item.danger {
  color: var(--danger);
}

.dropdown-item .item-icon {
  width: 17px;
  height: 17px;
  flex-shrink: 0;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.5;
  stroke-linecap: round;
  stroke-linejoin: round;
}

/* ===== 设置面板 ===== */
.sheet-overlay {
  position: fixed;
  inset: 0;
  background: rgba(16, 24, 40, 0.36);
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
}

.sheet {
  display: flex;
  flex-direction: column;
  width: 100%;
  max-width: 420px;
  max-height: 88vh;
  background: var(--card);
  border-radius: 14px;
  box-shadow: 0 24px 64px rgba(16, 24, 40, 0.24);
  overflow: hidden;
  animation: sheet-in 0.18s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes sheet-in {
  from {
    opacity: 0;
    transform: translateY(10px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

/* 弹层头部：与登录页同款品牌渐变，压住整个弹层的气质 */
.sheet-head {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 18px;
  background: linear-gradient(135deg, var(--brand) 0%, var(--brand-dark) 100%);
  flex-shrink: 0;
}

.head-icon {
  flex-shrink: 0;
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.18);
  display: grid;
  place-items: center;
}

.head-icon svg {
  width: 19px;
  height: 19px;
  fill: none;
  stroke: #fff;
  stroke-width: 1.5;
  stroke-linecap: round;
}

.head-text {
  flex: 1;
  min-width: 0;
}

.head-text h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 650;
  letter-spacing: -0.01em;
  color: #fff;
}

.head-text p {
  margin: 3px 0 0;
  font-size: 11.5px;
  color: rgba(255, 255, 255, 0.75);
  line-height: 1.5;
}

.icon-btn {
  margin: 0 0 0 auto;
  flex-shrink: 0;
  width: 28px;
  height: 28px;
  min-height: 0;
  padding: 0;
  display: grid;
  place-items: center;
  background: transparent;
  border: none;
  border-radius: 8px;
  color: rgba(255, 255, 255, 0.8);
  cursor: pointer;
  transition: background 0.12s ease, color 0.12s ease;
}

.icon-btn:hover {
  background: rgba(255, 255, 255, 0.18);
  color: #fff;
}

.icon-btn svg {
  width: 14px;
  height: 14px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.7;
  stroke-linecap: round;
}

.sheet-body {
  padding: 14px 18px 20px;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
}

/* 分组：只靠统一行距铺开，不用标题和分割线 */
.grp {
  margin-top: 10px;
}

.grp:first-child {
  margin-top: 0;
}

/* 与「账号」之间用一条通栏细线隔开 */
.grp.divided {
  margin-top: 14px;
  padding-top: 14px;
  border-top: 1px solid var(--line);
}

.field + .field {
  margin-top: 10px;
}

.f-label {
  display: block;
  font-size: 11.5px;
  font-weight: 500;
  color: var(--muted);
  margin-bottom: 6px;
}

.sheet-body input,
.sheet-body select {
  width: 100%;
  height: 38px;
  padding: 0 12px;
  margin: 0;
  font-size: 13px;
  border: 1px solid var(--line);
  border-radius: 9px;
  background: var(--card);
  color: var(--text);
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
}

.sheet-body input::placeholder {
  color: var(--disabled-text);
}

.sheet-body input:hover {
  border-color: #d3d8e0;
}

.sheet-body input:focus,
.sheet-body select:focus {
  outline: none;
  border-color: var(--brand);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.14);
}

.pair {
  display: flex;
  gap: 10px;
}

.pair .field {
  flex: 1;
  min-width: 0;
}

.pair .field + .field {
  margin-top: 0;
}

.pair + .field {
  margin-top: 10px;
}

/* 带按钮的输入框 */
.input-with-action {
  position: relative;
}

.input-with-action input {
  padding-right: 42px;
}

.ghost-icon {
  position: absolute;
  right: 5px;
  top: 50%;
  transform: translateY(-50%);
  width: 28px;
  height: 28px;
  min-height: 0;
  padding: 0;
  margin: 0;
  display: grid;
  place-items: center;
  background: transparent;
  border: none;
  border-radius: 7px;
  color: var(--muted);
  cursor: pointer;
}

.ghost-icon:hover {
  background: var(--bg);
  color: var(--text);
}

.ghost-icon svg {
  width: 16px;
  height: 16px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.6;
  stroke-linecap: round;
  stroke-linejoin: round;
}

/* 开关行 */
.switch-row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 10px 0;
}

.switch-row + .switch-row {
  margin-top: 4px;
}

.switch-text {
  flex: 1;
  min-width: 0;
}

.switch-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--text);
}

.switch-desc {
  font-size: 11.5px;
  line-height: 1.5;
  color: var(--muted);
  margin-top: 2px;
}

.switch {
  flex-shrink: 0;
  width: 40px;
  min-width: 40px;
  height: 22px;
  min-height: 22px;
  padding: 0;
  margin: 0;
  border: none;
  border-radius: 11px;
  background: #d5d9e0;
  position: relative;
  cursor: pointer;
  transition: background 0.18s cubic-bezier(0.16, 1, 0.3, 1);
}

.switch:active:not(:disabled) {
  transform: none;
}

.switch-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 2px rgba(16, 24, 40, 0.24);
  transition: transform 0.18s cubic-bezier(0.16, 1, 0.3, 1);
}

.switch.on {
  background: var(--brand);
}

.switch.on .switch-knob {
  transform: translateX(18px);
}

/* 底部固定操作栏 */
.sheet-foot {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
  padding: 12px 18px;
  border-top: 1px solid var(--line);
  background: #fafbfd;
  flex-shrink: 0;
}

.sheet-foot button {
  width: auto;
  min-height: 36px;
  height: 36px;
  margin: 0;
  padding: 0 16px;
  border-radius: 9px;
  font-size: 13px;
  font-weight: 600;
}

.btn-ghost {
  background: var(--card);
  color: var(--text);
  border: 1px solid var(--line);
}

.btn-ghost:hover {
  background: var(--bg);
  color: var(--text);
}

.btn-primary {
  background: var(--brand);
  color: #fff;
  border: none;
}

.btn-primary:hover {
  background: var(--brand-dark);
}

@media (max-width: 400px) {
  .sheet {
    max-height: 92vh;
  }

  .pair {
    flex-direction: column;
    gap: 10px;
  }
}
</style>

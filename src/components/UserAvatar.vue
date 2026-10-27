<template>
  <div class="user-avatar-wrap" ref="wrapRef">
    <button class="avatar-btn" @click.stop="isOpen = !isOpen" :title="username">
      <span class="avatar-circle">{{ avatarLetter }}</span>
    </button>

    <div v-if="isOpen" class="avatar-dropdown">
      <div class="dropdown-header">
        <span class="avatar-circle large">{{ avatarLetter }}</span>
        <div class="user-info">
          <div class="user-name">{{ username }}</div>
          <div class="user-meta">默认 {{ defaultHours }}h · {{ saveOrSubmit === '2' ? '提交' : '保存' }}</div>
          <div class="user-version">v{{ appVersion }}</div>
        </div>
      </div>
      <div class="dropdown-divider"></div>
      <button class="dropdown-item" @click="onSettings">
        <span class="icon">⚙</span> 配置管理
      </button>
      <button class="dropdown-item danger" @click="onLogout">
        <span class="icon">↩</span> 退出登录
      </button>
    </div>

    <!-- 设置面板 -->
    <div v-if="showSettings" class="settings-overlay" @click.self="showSettings = false">
      <div class="settings-panel">
        <div class="settings-header">
          <h3>配置管理</h3>
          <span class="settings-version">v{{ appVersion }}</span>
          <button class="close-btn" @click="showSettings = false">✕</button>
        </div>
        <div class="settings-body">
          <div class="form-group">
            <label class="field-label">服务器地址</label>
            <input v-model="settingsForm.baseUrl" type="text" />
          </div>
          <div class="form-group">
            <label class="field-label">用户名</label>
            <input v-model="settingsForm.username" type="text" />
          </div>
          <div class="form-group">
            <label class="field-label">密码</label>
            <input v-model="settingsForm.password" type="password" placeholder="修改密码请填写" />
          </div>
          <div class="form-row">
            <div class="form-group half">
              <label class="field-label">Client ID</label>
              <input v-model="settingsForm.clientId" type="text" />
            </div>
            <div class="form-group half">
              <label class="field-label">Client Secret</label>
              <div class="password-wrap">
                <input v-model="settingsForm.clientSecret" :type="showClientSecret ? 'text' : 'password'" />
                <button type="button" class="password-toggle" @click="showClientSecret = !showClientSecret" :title="showClientSecret ? '隐藏 Client Secret' : '显示 Client Secret'">
                  <span v-if="showClientSecret" class="eye-icon">🙈</span>
                  <span v-else class="eye-icon">👁</span>
                </button>
              </div>
            </div>
          </div>
          <div class="form-row">
            <div class="form-group half">
              <label class="field-label">默认工时</label>
              <input v-model.number="settingsForm.defaultHours" type="number" step="0.5" min="0" />
            </div>
            <div class="form-group half">
              <label class="field-label">保存/提交</label>
              <select v-model="settingsForm.saveOrSubmit">
                <option value="1">保存</option>
                <option value="2">提交</option>
              </select>
            </div>
          </div>
          <div class="form-group">
            <label class="field-label">默认工作描述</label>
            <input v-model="settingsForm.defaultTaskDesc" type="text" />
          </div>
          <div class="settings-footer">
            <button class="secondary" @click="showSettings = false">取消</button>
            <button @click="onSaveSettings">保存并重新登录</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { useAppVersion } from "../composables/useAppVersion";

const appVersion = useAppVersion();

const props = defineProps<{
  username: string;
  defaultHours: number;
  saveOrSubmit: string;
  baseUrl?: string;
  clientId?: string;
  clientSecret?: string;
  defaultTaskDesc?: string;
}>();

const emit = defineEmits<{
  (e: "logout"): void;
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
      password: "",
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

function onLogout() {
  isOpen.value = false;
  emit("logout");
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

.avatar-btn {
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
  margin: 0;
  min-height: auto;
  width: auto;
}

.avatar-circle {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--brand);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  font-weight: 700;
}

.avatar-circle.large {
  width: 40px;
  height: 40px;
  font-size: 18px;
}

.avatar-dropdown {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  width: 240px;
  background: var(--card);
  border: 1px solid var(--line);
  border-radius: var(--radius-md);
  box-shadow: 0 8px 24px rgba(16, 24, 40, 0.12);
  z-index: 100;
  overflow: hidden;
}

.dropdown-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
}

.user-info {
  min-width: 0;
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

.user-version {
  font-size: 11px;
  color: var(--brand);
  margin-top: 4px;
  font-weight: 600;
}

.dropdown-divider {
  height: 1px;
  background: var(--line);
}

.dropdown-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px;
  border: none;
  background: none;
  cursor: pointer;
  font-size: 14px;
  color: var(--text);
  text-align: left;
  border-radius: 0;
  margin: 0;
  min-height: auto;
}

.dropdown-item:hover {
  background: var(--bg);
}

.dropdown-item.danger {
  color: var(--danger);
}

.dropdown-item .icon {
  font-size: 16px;
  width: 20px;
  text-align: center;
}

/* 设置面板 */
.settings-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.settings-panel {
  background: var(--card);
  border-radius: var(--radius-lg);
  width: 100%;
  max-width: 440px;
  max-height: 80vh;
  overflow: auto;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.2);
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--line);
}

.settings-header h3 {
  margin: 0;
  font-size: 16px;
}

.settings-version {
  font-size: 12px;
  color: var(--muted);
  font-weight: 500;
}

.close-btn {
  background: none;
  border: none;
  font-size: 18px;
  cursor: pointer;
  color: var(--muted);
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  margin: 0;
  width: auto;
  min-height: auto;
}

.close-btn:hover {
  background: var(--bg);
}

.settings-body {
  padding: 20px;
}

.settings-body .form-group {
  margin-bottom: 14px;
}

.settings-body input,
.settings-body select {
  margin-bottom: 0;
}

.form-row {
  display: flex;
  gap: 12px;
}

.form-group.half {
  flex: 1;
}

.settings-footer {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
  margin-top: 20px;
}

.settings-footer button {
  padding: 10px 20px;
  min-height: auto;
  margin: 0;
}

.settings-footer .secondary {
  background: var(--card);
  color: var(--text);
  border: 1px solid var(--line);
}
</style>

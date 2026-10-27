<template>
  <LoadingMask :visible="loadingCount > 0" :text="loadingText" />

  <!-- 登录页面 -->
  <LoginPage
    v-if="!isLoggedIn"
    :loginFn="loginWithConfig"
    :savedUsername="savedUsername"
    :savedPassword="savedPassword"
    :savedBaseUrl="savedBaseUrl"
    :savedClientId="configClientId"
    :savedClientSecret="configClientSecret"
    :initialError="autoLoginError"
    @loginSuccess="onLoginSuccess"
  />

  <!-- 主界面 -->
  <template v-else>
    <!-- 顶部栏 -->
    <div class="top-bar" data-tauri-drag-region>
      <h2 class="top-title">PMP 工时</h2>
      <UserAvatar
        :username="username"
        :defaultHours="defaultHours"
        :saveOrSubmit="saveOrSubmit"
        :baseUrl="configBaseUrl"
        :clientId="configClientId"
        :clientSecret="configClientSecret"
        :defaultTaskDesc="configDefaultTaskDesc"
        @logout="onLogout"
        @relogin="onReloginFromSettings"
      />
    </div>

    <div class="card">
      <div class="date-head">
        <label class="field-label">日期（默认今天）</label>
        <button
          v-if="selectedDay && selectedDay !== today"
          type="button"
          class="secondary btn-today"
          @click="selectToday"
        >
          今天
        </button>
      </div>
      <div class="date-row">
        <DateDropdown
          :dayMap="dayMap"
          :sortedDates="sortedDates"
          :selectedDay="selectedDay"
          :selectedInfo="selectedInfo"
          :formatDayLabel="formatDayLabel"
          @select="onDaySelect"
        />
        <button
          type="button"
          class="secondary btn-side"
          :disabled="loadingCount > 0"
          @click="reloadTimesheets"
        >
          查工时
        </button>
      </div>

      <label class="field-label">工时（默认 {{ defaultHours }}，可改）</label>
      <input v-model.number="hours" type="number" step="0.5" min="0" />

      <label class="field-label">工作描述</label>
      <textarea v-model="taskDesc" placeholder="今天做了什么…"></textarea>
    </div>

    <div class="card">
      <div class="date-head">
        <label class="field-label">选择任务（单选）</label>
        <button
          type="button"
          class="secondary btn-side"
          :disabled="loadingCount > 0"
          style="width: auto; margin: 0; padding: 6px 12px; font-size: 13px"
          @click="reloadTasks"
        >
          刷新任务
        </button>
      </div>
      <TaskList :tasks="tasks" @select="onTaskSelect" />
    </div>

    <button type="button" :disabled="!canSubmit" @click="onSubmit">确认提交</button>
    <div :class="['msg', resultClass]">{{ resultMsg }}</div>

    <!-- 提交成功弹窗 -->
    <div v-if="showSuccess" class="preview-overlay" @click.self="closeSuccess">
      <div class="success-panel">
        <div class="success-icon">✓</div>
        <h3>提交成功</h3>
        <p class="success-detail">{{ successMsg }}</p>
        <button @click="closeSuccess">确认</button>
      </div>
    </div>

    <!-- 提交预览弹窗 -->
    <div v-if="showPreview" class="preview-overlay" @click.self="showPreview = false">
      <div class="preview-panel">
        <h3>提交预览</h3>
        <div class="preview-body">
          <div class="preview-row"><span class="preview-label">日期</span><span>{{ previewData.day }}</span></div>
          <div class="preview-row"><span class="preview-label">工时</span><span>{{ previewData.hours }}h</span></div>
          <div class="preview-row"><span class="preview-label">任务</span><span>{{ previewData.projectCode }} {{ previewData.projectName }} - {{ previewData.taskName }}</span></div>
          <div class="preview-row"><span class="preview-label">taskId</span><span class="mono">{{ previewData.taskId }}</span></div>
          <div class="preview-row"><span class="preview-label">描述</span><span>{{ previewData.taskDesc }}</span></div>
        </div>
        <div class="preview-actions">
          <button class="secondary" @click="showPreview = false">取消</button>
          <button @click="confirmSubmit">确认提交</button>
        </div>
      </div>
    </div>
  </template>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import LoadingMask from "./components/LoadingMask.vue";
import DateDropdown from "./components/DateDropdown.vue";
import TaskList from "./components/TaskList.vue";
import LoginPage from "./components/LoginPage.vue";
import UserAvatar from "./components/UserAvatar.vue";
import { usePmpApi } from "./composables/usePmpApi";
import { useDayMap, getTodayStr } from "./composables/useDayMap";
import type { TaskItem } from "./types/pmp";

const {
  loadingCount,
  loadingText,
  getConfig,
  relogin,
  listTasks,
  listTimesheets,
  submitTimesheet,
  loginWithConfig,
} = usePmpApi();

const { dayMap, selectedDay, today, sortedDates, selectedInfo, formatDayLabel, updateFromTimesheets } =
  useDayMap();

// 登录状态
const isLoggedIn = ref(false);
const username = ref("");
const defaultHours = ref(7.5);
const saveOrSubmit = ref("2");
const configBaseUrl = ref("");
const configClientId = ref("");
const configClientSecret = ref("");
const configDefaultTaskDesc = ref("");

// 自动登录失败原因（展示在登录页）
const autoLoginError = ref("");

// 保存的凭据（用于登录页预填充）
const savedUsername = ref("");
const savedPassword = ref("");
const savedBaseUrl = ref("");

// 表单状态
const hours = ref(7.5);
const taskDesc = ref("");
const tasks = ref<TaskItem[]>([]);
const selectedTaskId = ref("");
const resultMsg = ref("");
const resultClass = ref("");
const showPreview = ref(false);
const showSuccess = ref(false);
const successMsg = ref("");

// 提交按钮可用状态
const canSubmit = computed(() => {
  if (loadingCount.value > 0) return false;
  if (!selectedDay.value) return false;
  const dayInfo = dayMap.value.get(selectedDay.value);
  if (dayInfo?.disabled) return false;
  if (!taskDesc.value.trim()) return false;
  if (!selectedTaskId.value) return false;
  return true;
});

// 提交预览数据
const previewData = computed(() => {
  const task = tasks.value.find(
    (t) => (t.taskId || t.id) === selectedTaskId.value,
  );
  return {
    day: selectedDay.value,
    hours: hours.value,
    taskDesc: taskDesc.value.trim(),
    taskId: selectedTaskId.value,
    taskName: task?.name || "",
    projectName: task?.projectName || "",
    projectCode: task?.projectCode || "",
  };
});

async function tryAutoLogin() {
  try {
    const cfg = await getConfig();
    username.value = cfg.username;
    defaultHours.value = cfg.default_hours;
    saveOrSubmit.value = cfg.save_or_submit;
    configBaseUrl.value = cfg.base_url || "";
    configClientId.value = cfg.client_id || "";
    configClientSecret.value = cfg.client_secret || "";
    configDefaultTaskDesc.value = cfg.default_task_desc || "";
    today.value = cfg.today;

    // 保存凭据供登录页预填充
    savedUsername.value = cfg.username || "";
    savedPassword.value = cfg.password || "";
    savedBaseUrl.value = cfg.base_url || "";

    if (cfg.config_error) {
      // 配置有问题，显示登录页
      autoLoginError.value = cfg.config_error;
      isLoggedIn.value = false;
      return;
    }

    // 尝试登录
    const loginRes = await relogin();
    if (loginRes.ok) {
      isLoggedIn.value = true;
      await loadMainData();
    } else {
      isLoggedIn.value = false;
      autoLoginError.value = loginRes.error || "自动登录失败";
    }
  } catch (e: unknown) {
    isLoggedIn.value = false;
    autoLoginError.value = e instanceof Error ? e.message : String(e);
  }
}

async function loadMainData() {
  try {
    const cfg = await getConfig();
    today.value = cfg.today;
    selectedDay.value = cfg.today;
    hours.value = cfg.default_hours;
    username.value = cfg.username;
    defaultHours.value = cfg.default_hours;
    saveOrSubmit.value = cfg.save_or_submit;
    configBaseUrl.value = cfg.base_url || "";
    configClientId.value = cfg.client_id || "";
    configClientSecret.value = cfg.client_secret || "";
    configDefaultTaskDesc.value = cfg.default_task_desc || "";

    const [taskList] = await Promise.all([listTasks(1, 50), reloadTimesheets()]);
    tasks.value = taskList;

    if (taskList.length > 0) {
      selectedTaskId.value = (taskList[0].taskId || taskList[0].id || "") as string;
    }
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    resultClass.value = "err";
    resultMsg.value = msg;
  }
}

async function onLoginSuccess() {
  isLoggedIn.value = true;
  autoLoginError.value = "";
  await loadMainData();
}

function onLogout() {
  isLoggedIn.value = false;
  autoLoginError.value = "";
  resultMsg.value = "";
  resultClass.value = "";
}

async function onReloginFromSettings(params: {
  baseUrl?: string;
  username: string;
  password: string;
  clientId: string;
  clientSecret: string;
  defaultHours?: number;
  saveOrSubmit?: string;
  defaultTaskDesc?: string;
}) {
  const res = await loginWithConfig(params);
  if (res.ok) {
    username.value = params.username;
    if (params.defaultHours) defaultHours.value = params.defaultHours;
    if (params.saveOrSubmit) saveOrSubmit.value = params.saveOrSubmit;
    await loadMainData();
  } else {
    resultClass.value = "err";
    resultMsg.value = res.error || "重新登录失败";
  }
}

async function reloadTimesheets() {
  try {
    // 每次刷新都重新获取真实日期，避免长时间挂起后 today 过期
    const freshToday = getTodayStr();
    today.value = freshToday;
    const records = await listTimesheets(1, 31);
    updateFromTimesheets(records, freshToday);
    resultMsg.value = "";
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    resultClass.value = "err";
    resultMsg.value = msg;
  }
}

async function reloadTasks() {
  try {
    await relogin();
    tasks.value = await listTasks(1, 50);
    if (tasks.value.length > 0) {
      selectedTaskId.value = (tasks.value[0].taskId || tasks.value[0].id || "") as string;
    }
    resultMsg.value = "";
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    resultClass.value = "err";
    resultMsg.value = msg;
  }
}

function onDaySelect(date: string) {
  selectedDay.value = date;
  resultMsg.value = "";
}

function onTaskSelect(taskId: string) {
  selectedTaskId.value = taskId;
}

function selectToday() {
  selectedDay.value = today.value;
  const info = dayMap.value.get(today.value);
  if (info?.disabled) {
    resultClass.value = "err";
    resultMsg.value = `今天（${today.value}）为「${info.reason}」，不可提交`;
  } else {
    resultMsg.value = "";
  }
}

async function onSubmit() {
  const dayInfo = dayMap.value.get(selectedDay.value);
  if (dayInfo?.disabled) {
    resultClass.value = "err";
    resultMsg.value = `日期 ${selectedDay.value} 不可填写（${dayInfo.reason}）`;
    return;
  }

  if (!taskDesc.value.trim()) {
    resultClass.value = "err";
    resultMsg.value = "请填写工作描述";
    return;
  }

  if (!selectedTaskId.value) {
    resultClass.value = "err";
    resultMsg.value = "请选择一个任务";
    return;
  }

  // 显示预览弹窗
  showPreview.value = true;
}

async function confirmSubmit() {
  showPreview.value = false;
  try {
    const res = await submitTimesheet({
      day: selectedDay.value,
      taskIds: [selectedTaskId.value],
      taskDesc: taskDesc.value.trim(),
      hours: hours.value,
    });

    if (res.ok) {
      const task = tasks.value.find((t) => (t.taskId || t.id) === selectedTaskId.value);
      successMsg.value = `${selectedDay.value}  ${hours.value}h  ${task?.projectCode || ''} ${task?.name || ''}`;
      await reloadTimesheets();
      showSuccess.value = true;
    } else {
      resultClass.value = "err";
      resultMsg.value = res.error || "提交失败";
    }
  } catch (e: unknown) {
    resultClass.value = "err";
    resultMsg.value = e instanceof Error ? e.message : String(e);
  }
}

function closeSuccess() {
  showSuccess.value = false;
  taskDesc.value = "";
}

function toggleDevTools() {
  invoke("toggle_devtools");
}

onMounted(() => {
  tryAutoLogin();

  // Cmd+Shift+I / Ctrl+Shift+I 打开 DevTools
  document.addEventListener("keydown", (e: KeyboardEvent) => {
    if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === "I") {
      e.preventDefault();
      toggleDevTools();
    }
  });
});
</script>

<style scoped>
.top-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin: 0 -14px 12px;
  padding: 14px 14px 14px 80px;
  background: linear-gradient(135deg, var(--brand) 0%, var(--brand-dark) 100%);
  border-radius: 0 0 var(--radius-md) var(--radius-md);
  position: sticky;
  top: 0;
  z-index: 10;
}

.top-title {
  font-size: 18px;
  font-weight: 700;
  margin: 0;
  color: #fff;
  letter-spacing: 0.02em;
}

/* 提交预览弹窗 */
.preview-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.preview-panel {
  background: var(--card);
  border-radius: var(--radius-lg);
  width: 100%;
  max-width: 400px;
  padding: 24px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.2);
}

.preview-panel h3 {
  margin: 0 0 16px;
  font-size: 16px;
}

.preview-body {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 20px;
}

.preview-row {
  display: flex;
  gap: 12px;
  font-size: 14px;
  line-height: 1.5;
}

.preview-label {
  flex-shrink: 0;
  width: 56px;
  color: var(--muted);
  text-align: right;
}

.mono {
  font-family: monospace;
  font-size: 12px;
  word-break: break-all;
}

.preview-actions {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
}

.preview-actions button {
  padding: 10px 20px;
  min-height: auto;
  margin: 0;
}

.preview-actions .secondary {
  background: #fff;
  color: var(--text);
  border: 1px solid var(--line);
}

/* 提交成功弹窗 */
.success-panel {
  background: var(--card);
  border-radius: var(--radius-lg);
  width: 100%;
  max-width: 320px;
  padding: 32px 24px 24px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.2);
  text-align: center;
}

.success-icon {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  background: var(--ok-light);
  color: #067647;
  font-size: 28px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0 auto 16px;
}

.success-panel h3 {
  margin: 0 0 8px;
  font-size: 18px;
  font-weight: 700;
}

.success-detail {
  font-size: 13px;
  color: var(--muted);
  margin: 0 0 20px;
  line-height: 1.5;
}

.success-panel button {
  width: 100%;
}
</style>

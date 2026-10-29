<template>
  <TitleBar :class="{ 'on-brand': isLoggedIn }" />
  <LoadingMask :visible="loadingCount > 0" :text="loadingText" />

  <!-- 登录页面 -->
  <LoginPage
    v-if="!isLoggedIn"
    :loginFn="loginWithConfig"
    :savedBaseUrl="configBaseUrl"
    :savedUsername="username"
    :savedPassword="configPassword"
    :savedClientId="configClientId"
    :savedClientSecret="configClientSecret"
    :initialError="autoLoginError"
    @loginSuccess="onLoginSuccess"
  />

  <!-- 主界面 -->
  <template v-else>
    <!-- 顶部栏：应用标题与账号入口合成一个整体，右侧留给窗口按钮 -->
    <div class="top-bar" data-tauri-drag-region>
      <UserAvatar
        title="PMP 工时"
        :username="username"
        :defaultHours="defaultHours"
        :saveOrSubmit="saveOrSubmit"
        :baseUrl="configBaseUrl"
        :password="configPassword"
        :clientId="configClientId"
        :clientSecret="configClientSecret"
        :defaultTaskDesc="configDefaultTaskDesc"
        :rememberTaskDesc="rememberTaskDesc"
        :rememberTaskId="rememberTaskId"
        @logout="onLogout"
        @relogin="onReloginFromSettings"
        @preferences="onPreferencesChange"
        @check-update="onCheckUpdate"
      />
    </div>

    <!-- 表单滚动区：滚动手柄从导航栏下方开始，不会顶到顶部 -->
    <div class="app-body">
      <!-- 日期 -->
      <section class="sec">
        <div class="sec-head">
          <span class="sec-title">日期</span>
          <div class="sec-tools">
            <button
              v-if="selectedDay && selectedDay !== today"
              type="button"
              class="tool-btn"
              @click="selectToday"
            >
              回到今天
            </button>
            <button
              type="button"
              class="tool-btn"
              :disabled="loadingCount > 0"
              @click="reloadTimesheets"
            >
              <svg class="tool-ico" viewBox="0 0 16 16" aria-hidden="true">
                <path d="M13.2 9.5a5.4 5.4 0 1 1-1.4-5.1" />
                <path d="M13.4 2.9v3.3h-3.3" />
              </svg>
              刷新
            </button>
          </div>
        </div>
        <DateDropdown
          :dayMap="dayMap"
          :sortedDates="sortedDates"
          :selectedDay="selectedDay"
          :selectedInfo="selectedInfo"
          :formatDayLabel="formatDayLabel"
          @select="onDaySelect"
        />
      </section>

      <!-- 工时 -->
      <section class="sec">
        <div class="sec-head">
          <span class="sec-title">工时</span>
          <span class="sec-hint">默认 {{ defaultHours }} 小时</span>
        </div>
        <div class="hours-field">
          <button
            type="button"
            class="step-btn"
            aria-label="减少 0.5 小时"
            @click="bumpHours(-0.5)"
          >
            <svg viewBox="0 0 12 12" aria-hidden="true"><path d="M2.5 6h7" /></svg>
          </button>
          <label class="hours-input">
            <input v-model.number="hours" type="number" step="0.5" min="0" inputmode="decimal" />
            <span class="hours-unit">h</span>
          </label>
          <button
            type="button"
            class="step-btn"
            aria-label="增加 0.5 小时"
            @click="bumpHours(0.5)"
          >
            <svg viewBox="0 0 12 12" aria-hidden="true"><path d="M2.5 6h7M6 2.5v7" /></svg>
          </button>
        </div>
      </section>

      <!-- 工作描述 -->
      <section class="sec">
        <div class="sec-head">
          <span class="sec-title">工作描述</span>
          <span class="sec-hint">{{ taskDesc.trim() ? taskDesc.length + " 字" : "必填" }}</span>
        </div>
        <textarea v-model="taskDesc" class="desc" rows="3" placeholder="今天做了什么…"></textarea>
      </section>

      <!-- 任务 -->
      <section class="sec">
        <div class="sec-head">
          <span class="sec-title">任务</span>
          <div class="sec-tools">
            <span v-if="tasks.length" class="sec-hint sec-hint-inline">{{ tasks.length }} 个进行中</span>
            <button
              type="button"
              class="tool-btn"
              :disabled="loadingCount > 0"
              @click="reloadTasks"
            >
              <svg class="tool-ico" viewBox="0 0 16 16" aria-hidden="true">
                <path d="M13.2 9.5a5.4 5.4 0 1 1-1.4-5.1" />
                <path d="M13.4 2.9v3.3h-3.3" />
              </svg>
              刷新
            </button>
          </div>
        </div>
        <TaskList :tasks="tasks" :selectedTaskId="selectedTaskId" @select="onTaskSelect" />
      </section>
    </div>

    <!-- 底部常驻提交栏 -->
    <div class="composer">
      <p v-if="resultMsg" class="composer-msg" :class="resultClass">{{ resultMsg }}</p>
      <button type="button" class="composer-btn" :disabled="!canSubmit" @click="onSubmit">
        <span>确认提交</span>
        <span class="composer-meta">{{ hours }} h</span>
      </button>
    </div>

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

  <!-- 自动更新弹窗：登录前后都可弹出 -->
  <UpdateDialog />
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import LoadingMask from "./components/LoadingMask.vue";
import TitleBar from "./components/TitleBar.vue";
import DateDropdown from "./components/DateDropdown.vue";
import TaskList from "./components/TaskList.vue";
import LoginPage from "./components/LoginPage.vue";
import UserAvatar from "./components/UserAvatar.vue";
import UpdateDialog from "./components/UpdateDialog.vue";
import { usePmpApi } from "./composables/usePmpApi";
import { useDayMap, getTodayStr } from "./composables/useDayMap";
import { useUpdater } from "./composables/useUpdater";
import type { ConfigResponse, TaskItem } from "./types/pmp";

const { checkForUpdate } = useUpdater();

const {
  loadingCount,
  loadingText,
  getConfig,
  savePreferences,
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
const configPassword = ref("");
const configClientId = ref("");
const configClientSecret = ref("");
const configDefaultTaskDesc = ref("");

// 记忆开关：是否记住工作描述 / 是否记住选中的任务
const rememberTaskDesc = ref(true);
const rememberTaskId = ref(true);
// 上次提交使用的任务，仅用于「记住选中任务」时回填
const lastTaskId = ref("");

// 自动登录失败原因（展示在登录页）
const autoLoginError = ref("");

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

/** 工时加减 0.5，最小 0 */
function bumpHours(delta: number) {
  const base = Number(hours.value);
  const next = (Number.isFinite(base) ? base : 0) + delta;
  hours.value = Math.max(0, Math.round(next * 2) / 2);
}

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

/** 把后端配置回填到界面状态（后端可能返回空配置，全部按可选处理） */
function applyConfig(cfg: ConfigResponse) {
  username.value = cfg.username || "";
  defaultHours.value = cfg.default_hours ?? 7.5;
  saveOrSubmit.value = cfg.save_or_submit || "2";
  configBaseUrl.value = cfg.base_url || "";
  configPassword.value = cfg.password || "";
  configClientId.value = cfg.client_id || "";
  configClientSecret.value = cfg.client_secret || "";
  configDefaultTaskDesc.value = cfg.default_task_desc || "";
  rememberTaskDesc.value = cfg.remember_task_desc ?? true;
  rememberTaskId.value = cfg.remember_task_id ?? true;
  lastTaskId.value = cfg.last_task_id || "";

  if (cfg.today) today.value = cfg.today;
}

async function tryAutoLogin() {
  let cfg: ConfigResponse;
  try {
    cfg = await getConfig();
  } catch (e: unknown) {
    // 只有真正的异常（命令调用失败等）才走到这里；配置缺失不会再抛错
    isLoggedIn.value = false;
    autoLoginError.value = e instanceof Error ? e.message : String(e);
    return;
  }

  applyConfig(cfg);

  if (cfg.needs_setup) {
    // 首次使用 / 配置不完整：安静地展示登录页，不报错
    isLoggedIn.value = false;
    return;
  }

  if (cfg.config_error) {
    // 配置文件损坏等需要用户处理的问题
    autoLoginError.value = cfg.config_error;
    isLoggedIn.value = false;
    return;
  }

  // 尝试登录
  try {
    const loginRes = await relogin();
    if (loginRes.ok) {
      isLoggedIn.value = true;
      await loadMainData();
    } else {
      isLoggedIn.value = false;
      autoLoginError.value = loginRes.error || "自动登录失败，请重新输入密码";
    }
  } catch (e: unknown) {
    isLoggedIn.value = false;
    autoLoginError.value = e instanceof Error ? e.message : String(e);
  }
}

async function loadMainData() {
  try {
    const cfg = await getConfig();
    applyConfig(cfg);
    selectedDay.value = cfg.today || getTodayStr();
    hours.value = cfg.default_hours ?? 7.5;

    // 工作描述：开启「记住」时回填上次提交的内容，否则退回配置里的默认描述
    if (!taskDesc.value.trim()) {
      const remembered = rememberTaskDesc.value ? cfg.last_task_desc || "" : "";
      taskDesc.value = remembered || cfg.default_task_desc || "";
    }

    const [taskList] = await Promise.all([listTasks(1, 50), reloadTimesheets()]);
    tasks.value = taskList;
    applyTaskSelection(taskList);
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    resultClass.value = "err";
    resultMsg.value = msg;
  }
}

function taskIdOf(task: TaskItem): string {
  return (task.taskId || task.id || "") as string;
}

/**
 * 决定默认选中哪个任务：
 * - 开关打开：优先上次提交用的任务，找不到就选第一个（下次不用选）
 * - 开关关闭：保持空，由用户自己点
 */
function pickTaskId(list: TaskItem[]): string {
  if (!rememberTaskId.value) return "";
  if (lastTaskId.value) {
    const hit = list.find((t) => taskIdOf(t) === lastTaskId.value);
    if (hit) return lastTaskId.value;
  }
  return list.length > 0 ? taskIdOf(list[0]) : "";
}

/** 保留用户当前仍有效的选择，否则按开关重新挑选 */
function applyTaskSelection(list: TaskItem[], force = false) {
  const stillExists = list.some((t) => taskIdOf(t) === selectedTaskId.value);
  if (!force && selectedTaskId.value && stillExists) return;
  selectedTaskId.value = pickTaskId(list);
}

async function onLoginSuccess() {
  isLoggedIn.value = true;
  autoLoginError.value = "";
  await loadMainData();
}

async function onLogout() {
  // 退出账号只是回到登录页，配置仍保存在本地；刷新一次，保证登录页回填的是最新保存的值
  try {
    applyConfig(await getConfig());
  } catch {
    // 读不到配置就沿用内存里的值，反正表单也允许重填
  }
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
    applyTaskSelection(tasks.value, true);
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
  // 开关打开时记住这次选择，提交成功后由后端落盘
  if (rememberTaskId.value) lastTaskId.value = taskId;
}

/** 配置管理里切换两个「记住」开关：即时保存，不需要重新登录 */
async function onPreferencesChange(prefs: {
  rememberTaskDesc: boolean;
  rememberTaskId: boolean;
}) {
  const prevDesc = rememberTaskDesc.value;
  const prevTask = rememberTaskId.value;
  rememberTaskDesc.value = prefs.rememberTaskDesc;
  rememberTaskId.value = prefs.rememberTaskId;
  if (rememberTaskId.value && !selectedTaskId.value) {
    selectedTaskId.value = pickTaskId(tasks.value);
  }

  try {
    await savePreferences(prefs);
  } catch (e: unknown) {
    // 保存失败就回滚开关，避免界面和配置不一致
    rememberTaskDesc.value = prevDesc;
    rememberTaskId.value = prevTask;
    resultClass.value = "err";
    resultMsg.value = `保存设置失败：${e instanceof Error ? e.message : String(e)}`;
  }
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
      if (rememberTaskId.value) lastTaskId.value = selectedTaskId.value;
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
  // 开启「记住工作描述」时保留内容，方便下次接着改；否则按老规矩清空
  if (!rememberTaskDesc.value) taskDesc.value = "";
}

function toggleDevTools() {
  invoke("toggle_devtools");
}

/** 菜单里手动检查：无论结果如何都给反馈 */
function onCheckUpdate() {
  void checkForUpdate();
}

onMounted(() => {
  tryAutoLogin();

  // 启动后静默检查更新：延迟几秒，避开登录与首屏渲染
  window.setTimeout(() => {
    void checkForUpdate({ silent: true });
  }, 3500);

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
  flex-shrink: 0;
  padding: 10px 96px 10px 8px;
  background: linear-gradient(135deg, var(--brand) 0%, var(--brand-dark) 100%);
  position: relative;
  z-index: 10;
  /* 标题栏只作拖拽/入口，不参与文本选择，光标也保持箭头 */
  cursor: default;
  -webkit-user-select: none;
  user-select: none;
}

/* 表单滚动区：独立滚动，滚动手柄不会穿过导航栏 */
.app-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  background: var(--card);
}

/* ---------- 分段：细线条划分，不用卡片 ---------- */
.sec {
  padding: 16px;
}

.sec + .sec {
  border-top: 1px solid var(--line);
}

/* 最后一段贴着底部提交栏，多留些呼吸空间 */
.sec:last-child {
  padding-bottom: 22px;
}

.sec-head {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
  min-height: 20px;
}

.sec-title {
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.03em;
  color: var(--muted);
}

.sec-hint {
  margin-left: auto;
  font-size: 11.5px;
  color: var(--disabled-text);
  font-variant-numeric: tabular-nums;
}

/* 段头右侧动作组 */
.sec-tools {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 6px;
}

.sec-tools .sec-hint-inline {
  margin-left: 0;
}

.tool-btn {
  width: auto;
  min-height: 0;
  margin: 0;
  padding: 3px 7px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: transparent;
  border: none;
  border-radius: 6px;
  color: var(--muted);
  font-size: 11.5px;
  font-weight: 600;
  letter-spacing: 0.02em;
  cursor: pointer;
  transition: color 0.15s ease, background 0.15s ease;
}

.tool-btn:hover:not(:disabled) {
  color: var(--brand);
  background: var(--brand-light);
}

.tool-btn:active:not(:disabled) {
  transform: none;
}

.tool-ico {
  width: 12px;
  height: 12px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.6;
  stroke-linecap: round;
  stroke-linejoin: round;
}

/* ---------- 工时：步进器 ---------- */
.hours-field {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px;
  border: 1px solid var(--line);
  border-radius: 10px;
  background: var(--card);
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
}

.hours-field:focus-within {
  border-color: var(--brand);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.12);
}

.step-btn {
  width: 34px;
  height: 34px;
  min-height: 0;
  margin: 0;
  padding: 0;
  display: grid;
  place-items: center;
  background: transparent;
  border: none;
  border-radius: 7px;
  color: var(--muted);
  cursor: pointer;
  transition: background 0.14s ease, color 0.14s ease;
}

.step-btn:hover {
  background: var(--bg);
  color: var(--text);
}

.step-btn:active {
  transform: none;
}

.step-btn svg {
  width: 12px;
  height: 12px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.7;
  stroke-linecap: round;
}

.hours-input {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: baseline;
  justify-content: center;
  gap: 3px;
}

.hours-input input {
  width: 58px;
  margin: 0;
  padding: 0;
  border: none;
  background: transparent;
  text-align: center;
  font-size: 19px;
  font-weight: 650;
  letter-spacing: -0.01em;
  font-variant-numeric: tabular-nums;
  color: var(--text);
}

.hours-input input:focus {
  border: none;
  box-shadow: none;
  outline: none;
}

.hours-input input::-webkit-outer-spin-button,
.hours-input input::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.hours-unit {
  font-size: 12px;
  font-weight: 600;
  color: var(--muted);
}

/* ---------- 工作描述 ---------- */
.desc {
  width: 100%;
  min-height: 84px;
  margin: 0;
  padding: 10px 12px;
  border: 1px solid var(--line);
  border-radius: 10px;
  background: var(--card);
  font-size: 13.5px;
  line-height: 1.65;
  resize: none;
}

.desc::placeholder {
  color: var(--disabled-text);
}

/* ---------- 底部提交栏 ---------- */
.composer {
  flex-shrink: 0;
  padding: 10px 16px 14px;
  background: var(--card);
  border-top: 1px solid var(--line);
}

.composer-msg {
  margin: 0 0 8px;
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}

.composer-msg.err {
  color: var(--danger);
}

.composer-msg.ok {
  color: var(--ok);
}

.composer-btn {
  width: 100%;
  height: 44px;
  min-height: 44px;
  margin: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.01em;
}

.composer-meta {
  padding: 2px 7px;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.18);
  font-size: 11.5px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.composer-btn:disabled .composer-meta {
  background: rgba(255, 255, 255, 0.14);
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

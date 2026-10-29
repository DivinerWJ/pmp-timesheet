<template>
  <div class="login-page">
    <div class="login-shell">
      <!-- 品牌区：左侧登录文案；logo 作为右侧背景水印；名称与版本正常陈列 -->
      <header class="hero" data-tauri-drag-region="deep">
        <img class="hero-mark" src="/logo-mono.svg" alt="" aria-hidden="true" />
        <div class="hero-copy">
          <h1 class="hero-title">登录</h1>
          <p class="hero-sub">用企业账号连接 PMP 服务器</p>
        </div>
        <div class="hero-brand">
          <span class="hero-name">PMP 工时</span>
          <span class="hero-ver">v{{ appVersion }}</span>
        </div>
      </header>

      <div class="panel">
        <form class="login-form" @submit.prevent="onLogin">
          <div class="field">
            <label class="f-label" for="username">用户名</label>
            <div class="input-shell has-icon">
              <svg class="f-icon" viewBox="0 0 24 24" aria-hidden="true">
                <circle cx="12" cy="8.5" r="3.6" />
                <path d="M5.2 20c0-3.3 3.1-5.6 6.8-5.6s6.8 2.3 6.8 5.6" />
              </svg>
              <input
                id="username"
                v-model="form.username"
                type="text"
                autocomplete="username"
                spellcheck="false"
                placeholder="域账号"
                required
              />
            </div>
          </div>

          <div class="field">
            <label class="f-label" for="password">密码</label>
            <div class="input-shell has-icon">
              <svg class="f-icon" viewBox="0 0 24 24" aria-hidden="true">
                <rect x="4.8" y="10.6" width="14.4" height="9.4" rx="2.6" />
                <path d="M8.6 10.6V8.1a3.4 3.4 0 0 1 6.8 0v2.5" />
              </svg>
              <input
                id="password"
                class="with-actions"
                v-model="form.password"
                :type="showPassword ? 'text' : 'password'"
                autocomplete="current-password"
                placeholder="请输入密码"
                required
              />
              <button
                type="button"
                class="ghost-icon"
                :aria-label="showPassword ? '隐藏密码' : '显示密码'"
                :title="showPassword ? '隐藏密码' : '显示密码'"
                @click="showPassword = !showPassword"
              >
                <svg v-if="showPassword" viewBox="0 0 24 24" aria-hidden="true">
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

        <details class="more">
          <summary>
            <span class="chev" aria-hidden="true">
              <svg viewBox="0 0 12 12"><path d="M4 2l4 4-4 4" /></svg>
            </span>
            高级设置
            <span class="more-hint">默认工时 · 提交方式 · 默认描述</span>
          </summary>
          <div class="more-body">
            <div class="pair">
              <div class="field">
                <label class="f-label" for="defaultHours">默认工时</label>
                <input id="defaultHours" v-model.number="form.defaultHours" type="number" step="0.5" min="0" />
              </div>
              <div class="field">
                <label class="f-label" for="saveOrSubmit">保存 / 提交</label>
                <AppSelect
                  id="saveOrSubmit"
                  class="login-select"
                  v-model="form.saveOrSubmit"
                  :options="[
                    { value: '1', label: '保存' },
                    { value: '2', label: '提交' },
                  ]"
                />
              </div>
            </div>
            <div class="field">
              <label class="f-label" for="defaultTaskDesc">默认工作描述</label>
              <input id="defaultTaskDesc" v-model="form.defaultTaskDesc" type="text" placeholder="可留空" />
            </div>
          </div>
        </details>

        <div v-if="errorMsg" class="error" role="alert" @dblclick="copyError" title="双击复制">
          <span class="error-text">{{ errorMsg }}</span>
          <span class="error-hint">双击复制</span>
        </div>

        <button type="submit" class="submit" :disabled="loading">
          <span v-if="loading" class="spinner" aria-hidden="true"></span>
          {{ loading ? "正在连接…" : "登录" }}
        </button>

        <!-- 服务设置入口：地址或凭据缺失时变红并带提醒图标 -->
        <div class="service-entry">
          <button
            type="button"
            class="service-link"
            :class="{ invalid: serviceIncomplete }"
            :title="serviceIncomplete ? '服务器地址或应用凭据还没填完整' : '修改服务器地址与应用凭据'"
            @click="showService = true"
          >
            <svg v-if="serviceIncomplete" class="warn-icon" viewBox="0 0 16 16" aria-hidden="true">
              <circle cx="8" cy="8" r="6.5" />
              <path d="M8 4.8v3.7" />
              <circle cx="8" cy="11.2" r="0.75" fill="currentColor" stroke="none" />
            </svg>
            服务设置
          </button>
        </div>
        </form>
      </div>
    </div>

    <!-- 服务设置弹层：服务器地址 + 应用凭据，全部竖排 -->
    <div v-if="showService" class="sheet-mask" @click.self="showService = false">
      <div class="sheet" role="dialog" aria-modal="true" aria-labelledby="service-title">
        <header class="sheet-head">
          <span id="service-title" class="sheet-title">服务设置</span>
          <button type="button" class="sheet-close" aria-label="关闭" @click="showService = false">
            <svg viewBox="0 0 16 16" aria-hidden="true">
              <path d="M4.5 4.5l7 7M11.5 4.5l-7 7" />
            </svg>
          </button>
        </header>

        <div class="sheet-body">
          <div class="field">
            <label class="f-label" for="svc-baseUrl">服务器地址</label>
            <div class="input-shell has-icon">
              <svg class="f-icon" viewBox="0 0 24 24" aria-hidden="true">
                <circle cx="12" cy="12" r="8" />
                <path d="M4 12h16" />
                <path d="M12 4c2.2 2.3 3.3 5 3.3 8s-1.1 5.7-3.3 8c-2.2-2.3-3.3-5-3.3-8S9.8 6.3 12 4z" />
              </svg>
              <input
                id="svc-baseUrl"
                v-model="form.baseUrl"
                type="text"
                inputmode="url"
                autocomplete="url"
                spellcheck="false"
                placeholder="https://pmp.example.com"
              />
            </div>
          </div>

          <div class="field">
            <label class="f-label" for="svc-clientId">Client ID</label>
            <div class="input-shell has-icon">
              <svg class="f-icon" viewBox="0 0 24 24" aria-hidden="true">
                <circle cx="9" cy="15" r="4.2" />
                <path d="M12.2 11.8L19 5" />
                <path d="M16.6 7.4l2.2 2.2" />
              </svg>
              <input
                id="svc-clientId"
                v-model="form.clientId"
                type="text"
                autocomplete="off"
                spellcheck="false"
                placeholder="client_id"
              />
            </div>
          </div>

          <div class="field">
            <label class="f-label" for="svc-clientSecret">Client Secret</label>
            <div class="input-shell has-icon">
              <svg class="f-icon" viewBox="0 0 24 24" aria-hidden="true">
                <rect x="4.8" y="10.6" width="14.4" height="9.4" rx="2.6" />
                <path d="M8.6 10.6V8.1a3.4 3.4 0 0 1 6.8 0v2.5" />
              </svg>
              <input
                id="svc-clientSecret"
                class="with-actions"
                v-model="form.clientSecret"
                :type="showClientSecret ? 'text' : 'password'"
                autocomplete="off"
                placeholder="client_secret"
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

          <p class="sheet-tip">这三项由 IT 提供，登录成功后会自动保存，之后无需再填。</p>
        </div>

        <footer class="sheet-foot">
          <button type="button" class="btn-primary" @click="showService = false">完成</button>
        </footer>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useAppVersion } from "../composables/useAppVersion";
import AppSelect from "./AppSelect.vue";

const appVersion = useAppVersion();

const emit = defineEmits<{
  (e: "loginSuccess"): void;
}>();

const props = defineProps<{
  loginFn: (req: {
    baseUrl?: string;
    username: string;
    password: string;
    clientId: string;
    clientSecret: string;
    defaultHours?: number;
    saveOrSubmit?: string;
    defaultTaskDesc?: string;
  }) => Promise<{ ok: boolean; error?: string }>;
  /** 已保存的连接与账号信息（首次使用为空，登录成功过就有值） */
  savedBaseUrl?: string;
  savedUsername?: string;
  savedPassword?: string;
  /** 已保存的 Client 凭据（属于配置项，不要求每次重输） */
  savedClientId?: string;
  savedClientSecret?: string;
  initialError?: string;
}>();

// 首次使用时配置为空，表单自然是空的；已保存过配置（如退出账号后）则回填，省得重输
const form = ref({
  baseUrl: props.savedBaseUrl || "",
  username: props.savedUsername || "",
  password: props.savedPassword || "",
  clientId: props.savedClientId || "",
  clientSecret: props.savedClientSecret || "",
  defaultHours: 7.5,
  saveOrSubmit: "2",
  defaultTaskDesc: "",
});

const loading = ref(false);
const errorMsg = ref(props.initialError || "");
const showPassword = ref(false);
const showClientSecret = ref(false);
// 服务设置弹层
const showService = ref(false);

/** 服务设置是否缺项：地址、Client ID、Client Secret 任一为空即视为未配置完整 */
const serviceIncomplete = computed(
  () =>
    !form.value.baseUrl.trim() ||
    !form.value.clientId.trim() ||
    !form.value.clientSecret.trim(),
);

watch(
  () => props.initialError,
  (val) => {
    if (val) errorMsg.value = val;
  },
);

// 配置异步加载晚于登录页挂载时回填（不覆盖用户已输入内容）
watch(
  () => [
    props.savedBaseUrl,
    props.savedUsername,
    props.savedPassword,
    props.savedClientId,
    props.savedClientSecret,
  ],
  ([b, u, p, cid, csec]) => {
    if (b && !form.value.baseUrl) form.value.baseUrl = b;
    if (u && !form.value.username) form.value.username = u;
    if (p && !form.value.password) form.value.password = p;
    if (cid && !form.value.clientId) form.value.clientId = cid;
    if (csec && !form.value.clientSecret) form.value.clientSecret = csec;
  },
);

async function onLogin() {
  if (!form.value.username.trim() || !form.value.password) {
    errorMsg.value = "请填写用户名和密码";
    return;
  }

  // 服务设置缺项：直接打开弹层让用户补齐，而不是丢一句笼统的报错
  if (serviceIncomplete.value) {
    const missing: string[] = [];
    if (!form.value.baseUrl.trim()) missing.push("服务器地址");
    if (!form.value.clientId.trim()) missing.push("Client ID");
    if (!form.value.clientSecret.trim()) missing.push("Client Secret");
    errorMsg.value = `请先补全服务设置：${missing.join("、")}`;
    showService.value = true;
    return;
  }

  loading.value = true;
  errorMsg.value = "";

  try {
    const res = await props.loginFn({
      baseUrl: form.value.baseUrl || undefined,
      username: form.value.username,
      password: form.value.password,
      clientId: form.value.clientId,
      clientSecret: form.value.clientSecret,
      defaultHours: form.value.defaultHours,
      saveOrSubmit: form.value.saveOrSubmit,
      defaultTaskDesc: form.value.defaultTaskDesc || undefined,
    });

    if (res.ok) {
      emit("loginSuccess");
    } else {
      errorMsg.value = res.error || "登录失败";
    }
  } catch (e: unknown) {
    errorMsg.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}

async function copyError() {
  try {
    await navigator.clipboard.writeText(errorMsg.value);
  } catch {
    // 忽略复制失败
  }
}
</script>

<style scoped>
/*
 * 排版意图：不用悬浮卡片 + 大圆角，而是白纸上的分栏细线。
 * 信息按「服务器 / 账号 / 凭据」三段组织，段落间距明显大于字段间距，
 * 让眼睛先看到结构，再逐段读字段。
 */
/* 页面：淡蓝渐变底，一叠「品牌蓝头部 + 白色表单卡」 */
.login-page {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: flex;
  flex-direction: column;
  align-items: center;
  overflow-y: auto;
  background:
    radial-gradient(120% 78% at 50% 0%, #d9e6fd 0%, rgba(217, 230, 253, 0) 60%),
    linear-gradient(180deg, #e9f0fd 0%, #f5f8fc 54%, #edf1f8 100%);
}

.login-shell {
  width: 100%;
  max-width: 400px;
  flex-shrink: 0;
  /* 顶部 44px 给自绘窗口按钮让位；margin auto 在内容超高时归零，不会被裁顶 */
  margin: auto 0;
  padding: 44px 20px 34px;
}

/* 品牌头部：主题蓝渐变 + 淡光斑，logo 作为背景水印融进卡片右侧 */
.hero {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 24px 22px 30px;
  border-radius: 20px;
  overflow: hidden;
  background:
    radial-gradient(140% 120% at 100% 0%, rgba(255, 255, 255, 0.3) 0%, rgba(255, 255, 255, 0) 56%),
    linear-gradient(135deg, #1c4fd6 0%, #2f6bf0 48%, #4f8bff 100%);
  box-shadow: 0 16px 32px -14px rgba(28, 79, 214, 0.55);
  color: #fff;
  /* 品牌卡是拖拽把手：禁止选中文字，光标保持箭头 */
  cursor: default;
  -webkit-user-select: none;
  user-select: none;
}

/* 水印：压在卡片右缘被裁切，只留淡淡轮廓 */
.hero-mark {
  position: absolute;
  right: -36px;
  top: 52%;
  width: 148px;
  height: 148px;
  transform: translateY(-50%) rotate(-5deg);
  opacity: 0.1;
  pointer-events: none;
  user-select: none;
  z-index: 0;
}

.hero-copy {
  position: relative;
  z-index: 1;
  min-width: 0;
}

.hero-brand {
  position: relative;
  z-index: 1;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 3px;
}

.hero-name {
  font-size: 16px;
  font-weight: 700;
  letter-spacing: 0.02em;
  line-height: 1.2;
}

.hero-ver {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.72);
  font-variant-numeric: tabular-nums;
  letter-spacing: 0.02em;
}

.hero-title {
  margin: 0 0 5px;
  font-size: 25px;
  line-height: 1.2;
  font-weight: 650;
  letter-spacing: -0.01em;
}

.hero-sub {
  margin: 0;
  font-size: 12.5px;
  line-height: 1.6;
  color: rgba(255, 255, 255, 0.82);
}

/* 表单卡片：压在头部下缘，形成层次 */
.panel {
  margin: -16px 0 0;
  padding: 18px 16px 16px;
  border-radius: 16px;
  background: var(--card);
  box-shadow:
    0 14px 30px -16px rgba(15, 23, 42, 0.3),
    0 1px 0 rgba(15, 23, 42, 0.04);
}

.login-form {
  margin: 0;
}

/* 字段 */
.field + .field {
  margin-top: 12px;
}

.f-label {
  display: block;
  font-size: 11.5px;
  font-weight: 500;
  color: var(--muted);
  margin-bottom: 6px;
  letter-spacing: 0.01em;
}

.login-form input,
.login-form select {
  width: 100%;
  height: 42px;
  padding: 0 12px;
  margin: 0;
  font-size: 13.5px;
  border: 1px solid #dee4ee;
  border-radius: 10px;
  background: #f6f9fd;
  color: var(--text);
  transition: border-color 0.15s ease, box-shadow 0.15s ease, background 0.15s ease;
}

.login-form input::placeholder {
  color: var(--disabled-text);
}

.login-form input:hover {
  border-color: #c8d4e6;
  background: #fff;
}

.login-form input:focus,
.login-form select:focus {
  outline: none;
  border-color: var(--brand);
  background: #fff;
  box-shadow: 0 0 0 3px rgba(47, 107, 240, 0.15);
}

.login-form select {
  cursor: pointer;
}

/* 带图标的输入框 */
.input-shell {
  position: relative;
  display: flex;
  align-items: center;
}

.input-shell .f-icon {
  position: absolute;
  left: 12px;
  width: 15px;
  height: 15px;
  flex-shrink: 0;
  fill: none;
  stroke: var(--disabled-text);
  stroke-width: 1.6;
  stroke-linecap: round;
  stroke-linejoin: round;
  pointer-events: none;
  transition: stroke 0.15s ease;
}

.input-shell.has-icon input {
  padding-left: 36px;
}

.input-shell input.with-actions {
  padding-right: 40px;
}

.input-shell:focus-within .f-icon {
  stroke: var(--brand);
}

.ghost-icon {
  position: absolute;
  right: 6px;
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

/* 高级设置 */
.more {
  margin-top: 20px;
  border: 1px solid var(--line);
  border-radius: 10px;
  background: var(--bg);
}

.more summary {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 11px 12px;
  font-size: 12.5px;
  font-weight: 500;
  color: var(--text);
  cursor: pointer;
  list-style: none;
}

.more summary::-webkit-details-marker {
  display: none;
}

.chev svg {
  width: 10px;
  height: 10px;
  fill: none;
  stroke: var(--muted);
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
  transition: transform 0.18s cubic-bezier(0.16, 1, 0.3, 1);
  display: block;
}

.more[open] .chev svg {
  transform: rotate(90deg);
}

.more-hint {
  margin-left: auto;
  font-size: 11px;
  color: var(--disabled-text);
}

.more-body {
  /* 自定义下拉与 42px 输入框同高 */
  --as-h: 42px;
  padding: 4px 12px 14px;
}

/* 成对字段：同一行并排，窄窗口自动堆叠 */
.pair {
  display: flex;
  gap: 10px;
  margin-top: 12px;
}

/* 与「账号」之间用一条通栏细线隔开 */
.pair.divided {
  margin-top: 14px;
  padding-top: 14px;
  border-top: 1px solid var(--line);
}

.pair .field {
  flex: 1;
  min-width: 0;
}

.pair .field + .field {
  margin-top: 0;
}

.more-body .field + .field {
  margin-top: 10px;
}

.more-body .pair {
  margin-top: 0;
}

.more-body .pair .field + .field {
  margin-top: 0;
}

/* 服务设置入口：居中文字按钮，缺项时转红 + 提醒图标 */
.service-entry {
  margin-top: 14px;
  text-align: center;
}

.service-link {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  width: auto;
  padding: 3px 6px;
  border: none;
  border-radius: 5px;
  background: none;
  color: var(--brand);
  font-size: 12.5px;
  font-weight: 500;
  letter-spacing: 0.01em;
  cursor: pointer;
  transition: color 0.15s ease, background 0.15s ease;
}

.service-link:hover {
  color: var(--brand-dark);
  background: var(--brand-light);
}

.service-link.invalid {
  color: var(--danger);
}

.service-link.invalid:hover {
  color: var(--danger);
  background: var(--danger-light);
}

.warn-icon {
  width: 13px;
  height: 13px;
  flex-shrink: 0;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.4;
  stroke-linecap: round;
}

/* 错误提示 */
.error {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  margin-top: 16px;
  padding: 10px 12px;
  background: var(--danger-light);
  border-left: 2px solid var(--danger);
  border-radius: 6px;
  color: var(--danger);
  font-size: 12.5px;
  line-height: 1.6;
  cursor: copy;
}

.error-text {
  flex: 1;
  min-width: 0;
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 180px;
  overflow-y: auto;
}

.error-hint {
  flex-shrink: 0;
  font-size: 10.5px;
  color: rgba(198, 40, 40, 0.55);
  padding-top: 1px;
}

/* 主按钮 */
.submit {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  height: 42px;
  min-height: 42px;
  margin-top: 18px;
  padding: 0 16px;
  border: none;
  border-radius: 10px;
  background: var(--brand);
  color: #fff;
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.02em;
  cursor: pointer;
  transition: background 0.15s ease, transform 0.1s ease;
}

.submit:hover:not(:disabled) {
  background: var(--brand-dark);
}

.submit:active:not(:disabled) {
  transform: translateY(1px);
}

.submit:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.spinner {
  width: 13px;
  height: 13px;
  border: 2px solid rgba(255, 255, 255, 0.4);
  border-top-color: #fff;
  border-radius: 50%;
  animation: login-spin 0.7s linear infinite;
}

@keyframes login-spin {
  to {
    transform: rotate(360deg);
  }
}

/* 服务设置弹层 */
.sheet-mask {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: rgba(15, 23, 42, 0.32);
  animation: sheet-mask-in 0.16s ease;
}

.sheet {
  display: flex;
  flex-direction: column;
  width: 100%;
  max-width: 340px;
  max-height: calc(100vh - 32px);
  background: var(--card);
  border-radius: 14px;
  box-shadow: 0 18px 48px rgba(15, 23, 42, 0.18);
  overflow: hidden;
  animation: sheet-in 0.18s cubic-bezier(0.16, 1, 0.3, 1);
}

.sheet-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 14px 16px 8px;
}

.sheet-title {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text);
  letter-spacing: 0.01em;
}

.sheet-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 6px;
  background: none;
  color: var(--muted);
  cursor: pointer;
  transition: background 0.15s ease, color 0.15s ease;
}

.sheet-close:hover {
  background: var(--bg);
  color: var(--text);
}

.sheet-close svg {
  width: 14px;
  height: 14px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.5;
  stroke-linecap: round;
}

.sheet-body {
  padding: 4px 16px 2px;
  overflow-y: auto;
}

.sheet-body .field + .field {
  margin-top: 12px;
}

.sheet-tip {
  margin: 12px 0 0;
  font-size: 11.5px;
  line-height: 1.6;
  color: var(--disabled-text);
}

.sheet-foot {
  display: flex;
  justify-content: flex-end;
  padding: 12px 16px 16px;
}

.btn-primary {
  width: auto;
  height: 34px;
  padding: 0 18px;
  border: none;
  border-radius: 8px;
  background: var(--brand);
  color: #fff;
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.01em;
  cursor: pointer;
  transition: background 0.15s ease;
}

.btn-primary:hover {
  background: var(--brand-dark);
}

@keyframes sheet-mask-in {
  from {
    opacity: 0;
  }
}

@keyframes sheet-in {
  from {
    opacity: 0;
    transform: translateY(6px) scale(0.995);
  }
}

@media (max-width: 400px) {
  .login-shell {
    padding: 14px 16px 28px;
  }

  .pair {
    flex-direction: column;
    gap: 10px;
  }
}
</style>

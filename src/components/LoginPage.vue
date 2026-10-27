<template>
  <div class="login-page">
    <div class="login-card">
      <div class="login-header">
        <img class="login-logo" src="/logo.svg" alt="logo" />
        <h1 class="login-title">PMP 工时填写</h1>
        <p class="login-subtitle">请登录您的账号</p>
      </div>

      <form @submit.prevent="onLogin" class="login-form">
        <div class="form-group">
          <label class="field-label">服务器地址</label>
          <input v-model="form.baseUrl" type="text" placeholder="http://your-server.example.com" />
        </div>

        <div class="form-group">
          <label class="field-label">用户名 <span class="required">*</span></label>
          <input v-model="form.username" type="text" placeholder="请输入用户名" required />
        </div>

        <div class="form-group">
          <label class="field-label">密码 <span class="required">*</span></label>
          <div class="password-wrap">
            <input v-model="form.password" :type="showPassword ? 'text' : 'password'" placeholder="请输入密码" required />
            <button type="button" class="password-toggle" @click="showPassword = !showPassword" :title="showPassword ? '隐藏密码' : '显示密码'">
              <span v-if="showPassword" class="eye-icon">🙈</span>
              <span v-else class="eye-icon">👁</span>
            </button>
          </div>
        </div>

        <details class="advanced-section">
          <summary>高级设置</summary>
          <div class="form-row">
            <div class="form-group half">
              <label class="field-label">Client ID</label>
              <input v-model="form.clientId" type="text" placeholder="可选" />
            </div>
            <div class="form-group half">
              <label class="field-label">Client Secret</label>
              <div class="password-wrap">
                <input v-model="form.clientSecret" :type="showClientSecret ? 'text' : 'password'" placeholder="可选" />
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
              <input v-model.number="form.defaultHours" type="number" step="0.5" min="0" />
            </div>
            <div class="form-group half">
              <label class="field-label">保存/提交</label>
              <select v-model="form.saveOrSubmit">
                <option value="1">保存</option>
                <option value="2">提交</option>
              </select>
            </div>
          </div>
          <div class="form-group">
            <label class="field-label">默认工作描述</label>
            <input v-model="form.defaultTaskDesc" type="text" placeholder="可选" />
          </div>
        </details>

        <div v-if="errorMsg" class="login-error" @dblclick="copyError" title="双击复制">
          {{ errorMsg }}
        </div>

        <button type="submit" class="login-btn" :disabled="loading">
          {{ loading ? '登录中…' : '登录' }}
        </button>
      </form>

      <div class="login-version">v{{ appVersion }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { useAppVersion } from "../composables/useAppVersion";

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
  savedUsername?: string;
  savedPassword?: string;
  savedBaseUrl?: string;
  savedClientId?: string;
  savedClientSecret?: string;
  initialError?: string;
}>();

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

watch(
  () => props.initialError,
  (val) => {
    if (val) errorMsg.value = val;
  },
);

// 配置异步加载晚于登录页挂载，加载完成后回填（不覆盖用户已输入内容）
watch(
  () => [props.savedUsername, props.savedPassword, props.savedBaseUrl, props.savedClientId, props.savedClientSecret],
  ([u, p, b, cid, csec]) => {
    if (u && !form.value.username) form.value.username = u;
    if (p && !form.value.password) form.value.password = p;
    if (b && !form.value.baseUrl) form.value.baseUrl = b;
    if (cid && !form.value.clientId) form.value.clientId = cid;
    if (csec && !form.value.clientSecret) form.value.clientSecret = csec;
  },
);

async function onLogin() {
  if (!form.value.username || !form.value.password) {
    errorMsg.value = "请填写用户名和密码";
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
.login-page {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: flex;
  background: linear-gradient(135deg, var(--brand) 0%, var(--brand-dark) 100%);
  overflow-y: auto;
  padding: 20px;
}

.login-card {
  /* auto margin 居中：内容超高时不会裁掉顶部，仍可滚动 */
  margin: auto;
  background: var(--card);
  border-radius: var(--radius-lg);
  padding: 32px 28px;
  width: 100%;
  max-width: 420px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.15);
}

.login-header {
  text-align: center;
  margin-bottom: 28px;
}

.login-logo {
  width: 64px;
  height: 64px;
  margin: 0 auto 16px;
  object-fit: contain;
}

.login-title {
  font-size: 22px;
  font-weight: 700;
  margin: 0 0 6px;
  color: var(--text);
}

.login-subtitle {
  font-size: 14px;
  color: var(--muted);
  margin: 0;
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.form-group {
  margin-bottom: 12px;
}

.form-row {
  display: flex;
  gap: 12px;
}

.form-group.half {
  flex: 1;
}

.required {
  color: var(--danger);
}

.login-form input,
.login-form select {
  margin-bottom: 0;
}

.advanced-section {
  margin-bottom: 12px;
}

.advanced-section summary {
  font-size: 13px;
  color: var(--muted);
  cursor: pointer;
  padding: 8px 0;
}

.advanced-section[open] summary {
  margin-bottom: 12px;
}

.login-error {
  background: var(--danger-light);
  color: var(--danger);
  padding: 10px 14px;
  border-radius: var(--radius-sm);
  font-size: 13px;
  margin-bottom: 12px;
  border: 1px solid #fecaca;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 220px;
  overflow-y: auto;
  cursor: copy;
}

.login-btn {
  width: 100%;
  padding: 12px;
  font-size: 16px;
  font-weight: 600;
  border: none;
  border-radius: var(--radius-sm);
  background: var(--brand);
  color: #fff;
  cursor: pointer;
  margin-top: 4px;
  transition: background 0.15s ease, transform 0.1s ease;
}

.login-btn:hover:not(:disabled) {
  background: var(--brand-dark);
}

.login-btn:active:not(:disabled) {
  transform: scale(0.98);
}

.login-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.login-version {
  text-align: center;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  margin-top: 16px;
}

@media (max-width: 440px) {
  .login-card {
    padding: 24px 20px;
  }
  .form-row {
    flex-direction: column;
    gap: 0;
  }
}
</style>

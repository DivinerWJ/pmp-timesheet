import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { relaunch } from "@tauri-apps/plugin-process";
import {
  buildSourceOptions,
  buildSources,
  isKnownSource,
  sourceLabel,
} from "../config/updateSources";

/**
 * 自动更新状态机
 * idle → checking → (available → downloading → ready) | latest | error
 *
 * 状态放在模块作用域，任何组件调用 useUpdater() 拿到的都是同一份，
 * 弹窗、菜单入口、启动静默检查共用。
 *
 * 检查和下载都走自己注册的 Rust 命令（src-tauri/src/update.rs），
 * 而不是插件的 JS API —— 只有这样才能在 GitHub 被卡住时切换到镜像线路。
 */
export type UpdatePhase =
  | "idle"
  | "checking"
  | "latest"
  | "available"
  | "downloading"
  | "ready"
  | "error";

interface UpdateInfo {
  version: string;
  currentVersion: string;
  body?: string | null;
}

interface CheckOutcome {
  currentVersion: string;
  update: UpdateInfo | null;
}

interface ProgressPayload {
  downloaded: number;
  total: number | null;
  source: string;
}

const SOURCE_KEY = "pmp.update.source";

const isOpen = ref(false);
const phase = ref<UpdatePhase>("idle");
const currentVersion = ref("");
const latestVersion = ref("");
const releaseNotes = ref("");
const errorMessage = ref("");
const downloadedBytes = ref(0);
const totalBytes = ref(0);
/** 本次下载实际走的线路前缀（空串 = 直连），由后端进度事件回报 */
const activeSource = ref("");

let hasUpdate = false;
let checking = false;

/** 浏览器预览（无 Tauri 运行时）时调用必然失败，提前拦掉 */
function inTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** 目前只发布了 Windows 与 macOS 的更新包，Linux 走手动下载 */
function unsupportedPlatformHint(): string | null {
  if (typeof navigator === "undefined") return null;
  const ua = navigator.userAgent;
  if (/Linux/i.test(ua) && !/Android/i.test(ua)) {
    return "当前系统暂不支持应用内更新，请到 GitHub Releases 下载新版本";
  }
  return null;
}

function describeError(e: unknown): string {
  const msg = e instanceof Error ? e.message : String(e ?? "");
  if (/timeout|timed out|operation was canceled/i.test(msg))
    return "连接更新服务器超时，可切换下载线路后重试";
  if (/network|connect|dns|failed to fetch|error sending request|tcp/i.test(msg))
    return "无法连接更新服务器，可切换下载线路后重试";
  if (/signature/i.test(msg)) return "安装包签名校验失败，已中止更新";
  if (/所有下载线路/.test(msg)) return "所有下载线路都不可用，可换一条线路再试";
  return msg || "检查更新失败";
}

function formatSize(bytes: number): string {
  if (!bytes) return "0 MB";
  const mb = bytes / 1024 / 1024;
  return mb >= 100 ? `${Math.round(mb)} MB` : `${mb.toFixed(1)} MB`;
}

/* ---------------- 下载线路选择（记住上次选择） ---------------- */

/** 后端从 nodes.json 拉到的节点清单；为空时上游会回落到内置兜底节点 */
const mirrorNodes = ref<string[]>([]);
let nodesLoading = false;

/** 首次需要镜像时才拉清单，直连模式下不产生额外请求 */
async function ensureMirrorNodes(): Promise<void> {
  if (mirrorNodes.value.length || nodesLoading) return;
  nodesLoading = true;
  try {
    const hosts = await invoke<string[]>("mirror_nodes");
    if (Array.isArray(hosts) && hosts.length) mirrorNodes.value = hosts;
  } catch {
    /* 拉不到就交给 updateSources 里的兜底节点，不影响更新 */
  } finally {
    nodesLoading = false;
  }
}

function readSourceId(): string {
  try {
    const saved = localStorage.getItem(SOURCE_KEY);
    if (saved && isKnownSource(saved)) return saved;
  } catch {
    /* 隐私模式下 localStorage 可能不可用，忽略 */
  }
  return "auto";
}

const sourceId = ref(readSourceId());

const sourceOptions = computed(() => buildSourceOptions(mirrorNodes.value));

const selectedSource = computed({
  get: () => sourceId.value,
  set: (value: string) => {
    if (!isKnownSource(value)) return;
    sourceId.value = value;
    try {
      localStorage.setItem(SOURCE_KEY, value);
    } catch {
      /* 存不上就算了，不影响本次使用 */
    }
  },
});

const activeSourceLabel = computed(() =>
  activeSource.value ? sourceLabel(activeSource.value) : "",
);

const percent = computed(() => {
  if (!totalBytes.value) return 0;
  return Math.min(99, Math.round((downloadedBytes.value / totalBytes.value) * 100));
});

const sizeText = computed(() => {
  if (!totalBytes.value) return `已下载 ${formatSize(downloadedBytes.value)}`;
  return `${formatSize(downloadedBytes.value)} / ${formatSize(totalBytes.value)}`;
});

/** 检查更新。silent = true 时只有「发现新版本」才弹窗，用于启动后台检查 */
async function checkForUpdate(options: { silent?: boolean } = {}): Promise<void> {
  const silent = options.silent ?? false;

  if (!inTauri()) {
    phase.value = "error";
    errorMessage.value = "当前是浏览器预览，更新检查仅在安装后的应用中可用";
    if (!silent) isOpen.value = true;
    return;
  }

  const unsupported = unsupportedPlatformHint();
  if (unsupported) {
    phase.value = "error";
    errorMessage.value = unsupported;
    if (!silent) isOpen.value = true;
    return;
  }

  if (checking) {
    if (!silent) isOpen.value = true;
    return;
  }

  checking = true;
  errorMessage.value = "";
  phase.value = "checking";
  if (!silent) isOpen.value = true;

  try {
    // 需要镜像的线路先取节点清单（直连模式后台拉，不阻塞检查）
    if (sourceId.value !== "github") await ensureMirrorNodes();
    else void ensureMirrorNodes();

    const outcome = await invoke<CheckOutcome>("update_check", {
      sources: buildSources(sourceId.value, mirrorNodes.value),
    });

    currentVersion.value = outcome.currentVersion;

    if (outcome.update) {
      hasUpdate = true;
      latestVersion.value = outcome.update.version;
      releaseNotes.value = (outcome.update.body ?? "").trim();
      phase.value = "available";
      isOpen.value = true;
    } else {
      hasUpdate = false;
      phase.value = "latest";
      if (!silent) isOpen.value = true;
    }
  } catch (e) {
    hasUpdate = false;
    phase.value = "error";
    errorMessage.value = describeError(e);
    if (!silent) isOpen.value = true;
  } finally {
    checking = false;
  }
}

/** 下载并安装；结束后重启应用生效 */
async function installUpdate(): Promise<void> {
  if (!hasUpdate) {
    void checkForUpdate();
    return;
  }

  phase.value = "downloading";
  downloadedBytes.value = 0;
  totalBytes.value = 0;
  activeSource.value = "";
  errorMessage.value = "";

  let unlisten: UnlistenFn | null = null;
  try {
    unlisten = await listen<ProgressPayload>("update-progress", ({ payload }) => {
      activeSource.value = payload.source;
      if (payload.total) totalBytes.value = payload.total;
      downloadedBytes.value = payload.downloaded;
    });

    await invoke<string>("update_install", {
      sources: buildSources(sourceId.value, mirrorNodes.value),
    });

    // 安装程序接管后 Windows 进程已被关闭，能走到这里说明需要手动重启
    phase.value = "ready";
    await relaunch();
  } catch (e) {
    phase.value = "error";
    errorMessage.value = describeError(e);
  } finally {
    unlisten?.();
  }
}

/** 出错后的重试：已知有新版本时直接重新下载，否则重新检查 */
function retry(): void {
  if (hasUpdate) void installUpdate();
  else void checkForUpdate();
}

/** 关闭弹窗：下载/安装过程中不允许关闭，避免用户误以为可以中断 */
function closeDialog(): void {
  if (phase.value === "downloading" || phase.value === "ready") return;
  isOpen.value = false;
  // 下次打开重新从干净状态开始（保留版本号信息）
  if (phase.value !== "available") phase.value = "idle";
}

export function useUpdater() {
  return {
    isOpen,
    phase,
    currentVersion,
    latestVersion,
    releaseNotes,
    errorMessage,
    downloadedBytes,
    totalBytes,
    percent,
    sizeText,
    sourceOptions,
    selectedSource,
    activeSourceLabel,
    checkForUpdate,
    installUpdate,
    retry,
    closeDialog,
  };
}

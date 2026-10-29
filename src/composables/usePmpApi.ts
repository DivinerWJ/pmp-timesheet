import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  ConfigResponse,
  LoginResponse,
  TaskItem,
  TimesheetRecord,
} from "../types/pmp";

const loadingCount = ref(0);
const loadingText = ref("加载中…");

export function usePmpApi() {
  async function call<T>(
    fn: () => Promise<T>,
    text?: string,
  ): Promise<T> {
    loadingCount.value += 1;
    if (text) loadingText.value = text;
    else if (loadingCount.value === 1) loadingText.value = "加载中…";
    try {
      return await fn();
    } finally {
      loadingCount.value = Math.max(0, loadingCount.value - 1);
      if (loadingCount.value === 0) loadingText.value = "加载中…";
    }
  }

  async function getConfig(): Promise<ConfigResponse> {
    return call(
      () => invoke<ConfigResponse>("get_config"),
      "登录检查…",
    );
  }

  async function login(): Promise<LoginResponse> {
    return call(
      () => invoke<LoginResponse>("login"),
      "登录中…",
    );
  }

  async function relogin(): Promise<LoginResponse> {
    return call(
      () => invoke<LoginResponse>("relogin"),
      "重新登录…",
    );
  }

  async function listTasks(pageNo = 1, pageSize = 50): Promise<TaskItem[]> {
    const result = await call<{ list: TaskItem[] }>(
      () =>
        invoke<{ list: TaskItem[] }>("list_tasks", {
          pageNo,
          pageSize,
        }),
      "加载任务…",
    );
    return result.list;
  }

  async function listTimesheets(
    pageNo = 1,
    pageSize = 31,
  ): Promise<TimesheetRecord[]> {
    const result = await call<{ list: TimesheetRecord[] }>(
      () =>
        invoke<{ list: TimesheetRecord[] }>("list_timesheets", {
          pageNo,
          pageSize,
        }),
      "查询工时…",
    );
    return result.list;
  }

  async function submitTimesheet(params: {
    day: string;
    taskIds: string[];
    taskDesc: string;
    hours: number;
    saveOrSubmit?: string;
  }): Promise<{ ok: boolean; result?: unknown; error?: string }> {
    return call(
      () =>
        invoke<{ ok: boolean; result?: unknown; error?: string }>(
          "submit_timesheet",
          {
            req: {
              day: params.day,
              task_ids: params.taskIds,
              task_desc: params.taskDesc,
              hours: params.hours,
              save_or_submit: params.saveOrSubmit ?? null,
            },
          },
        ),
      "提交中…",
    );
  }

  async function loginWithConfig(params: {
    baseUrl?: string;
    username: string;
    password: string;
    clientId: string;
    clientSecret: string;
    defaultHours?: number;
    saveOrSubmit?: string;
    defaultTaskDesc?: string;
  }): Promise<{ ok: boolean; error?: string }> {
    return call(
      () =>
        invoke<{ ok: boolean; error?: string }>("login_with_config", {
          req: {
            base_url: params.baseUrl ?? null,
            username: params.username,
            password: params.password,
            client_id: params.clientId,
            client_secret: params.clientSecret,
            default_hours: params.defaultHours ?? null,
            save_or_submit: params.saveOrSubmit ?? null,
            default_task_desc: params.defaultTaskDesc ?? null,
          },
        }),
      "登录中…",
    );
  }

  async function savePreferences(params: {
    rememberTaskDesc?: boolean;
    rememberTaskId?: boolean;
  }): Promise<boolean> {
    return call(() =>
      invoke<boolean>("save_preferences", {
        req: {
          remember_task_desc: params.rememberTaskDesc ?? null,
          remember_task_id: params.rememberTaskId ?? null,
        },
      }),
    );
  }

  return {
    loadingCount,
    loadingText,
    getConfig,
    savePreferences,
    login,
    relogin,
    listTasks,
    listTimesheets,
    submitTimesheet,
    loginWithConfig,
  };
}

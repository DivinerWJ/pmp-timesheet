/** 任务项 */
export interface TaskItem {
  taskId?: string;
  id?: string;
  name?: string;
  projectName?: string;
  projectCode?: string;
  projectId?: string;
  status?: string;
  planWorkingHours?: unknown;
  [key: string]: unknown;
}

/** 工时记录 */
export interface TimesheetRecord {
  submit_date?: string;
  is_fill?: string;
  task_overtime?: unknown;
  approval_status?: unknown;
  approval_content?: unknown;
  task_type?: unknown;
  id_list?: unknown[];
  office_name?: string;
  create_date?: string;
  is_rest?: string;
  is_leave?: string;
  [key: string]: unknown;
}

/** 配置摘要 */
export interface ConfigResponse {
  username: string;
  password: string;
  default_hours: number;
  save_or_submit: string;
  config_path: string;
  today: string;
  config_error: string | null;
  /** 还没有可用配置（首次使用 / 配置不完整），应展示引导而不是报错 */
  needs_setup: boolean;
  logged_in: boolean;
  base_url: string;
  client_id: string;
  client_secret: string;
  default_task_desc: string;
  /** 是否记住工作描述：提交后保留，下次打开自动填入 */
  remember_task_desc: boolean;
  /** 是否记住选中的任务：下次打开自动选中 */
  remember_task_id: boolean;
  /** 上次提交使用的工作描述 */
  last_task_desc: string;
  /** 上次提交选中的任务 ID */
  last_task_id: string;
}

/** 登录响应 */
export interface LoginResponse {
  ok: boolean;
  user_id: string | null;
  expires_at: string | null;
  error: string | null;
}

/** 日期状态 */
export interface DayState {
  date: string;
  disabled: boolean;
  reason: "未填" | "已填" | "休息" | "请假" | "未来";
  raw: TimesheetRecord | null;
}

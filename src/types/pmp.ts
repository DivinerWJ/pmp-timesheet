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
  logged_in: boolean;
  base_url: string;
  client_id: string;
  client_secret: string;
  default_task_desc: string;
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

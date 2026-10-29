<template>
  <div>
    <div v-if="!tasks.length" class="meta">暂无进行中任务</div>
    <label
      v-for="(task, i) in tasks"
      :key="getTaskId(task) || i"
      class="task"
      :class="{ selected: getTaskId(task) === selectedTaskId }"
    >
      <input
        class="radio-native"
        type="radio"
        name="task"
        :value="getTaskId(task)"
        :checked="getTaskId(task) === selectedTaskId"
        @change="$emit('select', getTaskId(task))"
      />
      <span class="radio-mark" aria-hidden="true"></span>
      <div class="task-text">
        <div class="task-main">
          <span class="tag">{{ task.projectCode || "" }}</span>
          <span class="task-project">{{ task.projectName || "" }}</span>
        </div>
        <div class="task-sub">{{ task.name || "" }}</div>
      </div>
    </label>
  </div>
</template>

<script setup lang="ts">
import type { TaskItem } from "../types/pmp";

defineProps<{
  tasks: TaskItem[];
  /** 当前选中的任务 ID：由外部控制，保证「记住选中项目」打开时勾选状态正确 */
  selectedTaskId?: string;
}>();

defineEmits<{
  (e: "select", taskId: string): void;
}>();

function getTaskId(task: TaskItem): string {
  return (task.taskId || task.id || "") as string;
}
</script>

<style scoped>
/* 行：整行可点，选中时给一层极浅的品牌底色 */
.task {
  padding: 10px 8px;
  margin: 0 -8px;
  border-radius: 10px;
  transition: background 0.14s ease;
}

.task:hover {
  background: var(--bg);
}

.task.selected {
  background: var(--brand-light);
}

/* 原生 radio 只保留可访问性，视觉上隐藏 */
.radio-native {
  position: absolute;
  width: 1px;
  height: 1px;
  opacity: 0;
  margin: 0;
  padding: 0;
  pointer-events: none;
}

/* 自绘圆点：未选为空心细环，选中为实心蓝点 */
.radio-mark {
  position: relative;
  flex-shrink: 0;
  width: 18px;
  height: 18px;
  margin-top: 3px;
  border-radius: 50%;
  border: 1.5px solid #ced3dc;
  background: var(--card);
  transition: border-color 0.15s ease, background 0.15s ease;
}

.radio-mark::after {
  content: "";
  position: absolute;
  top: 50%;
  left: 50%;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--brand);
  transform: translate(-50%, -50%) scale(0);
  transition: transform 0.16s cubic-bezier(0.16, 1, 0.3, 1);
}

.task:hover .radio-mark {
  border-color: var(--brand);
}

.task.selected .radio-mark {
  border-color: var(--brand);
  background: var(--card);
}

.task.selected .radio-mark::after {
  transform: translate(-50%, -50%) scale(1);
}

/* 键盘聚焦也给出焦点环 */
.radio-native:focus-visible + .radio-mark {
  border-color: var(--brand);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.18);
}

/* 文本区：编号 + 项目名一行，子名称次行；id 不再展示 */
.task-text {
  flex: 1;
  min-width: 0;
}

.task-main {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.task-project {
  font-size: 13px;
  font-weight: 600;
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-sub {
  font-size: 12px;
  line-height: 1.5;
  color: var(--muted);
  margin-top: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 选中行底色与编号同色系，编号改为白底描边，避免「糊」进背景 */
.task.selected .tag {
  background: var(--card);
  color: var(--brand);
  box-shadow: inset 0 0 0 1px rgba(59, 130, 246, 0.35);
}
</style>

<template>
  <div>
    <div v-if="!tasks.length" class="meta">暂无进行中任务</div>
    <label
      v-for="(task, i) in tasks"
      :key="getTaskId(task) || i"
      class="task"
    >
      <input
        type="radio"
        name="task"
        :value="getTaskId(task)"
        :checked="i === 0"
        @change="$emit('select', getTaskId(task))"
      />
      <div>
        <div>
          <span class="tag">{{ task.projectCode || "" }}</span>
          {{ task.projectName || "" }}
        </div>
        <div>{{ task.name || "" }}</div>
        <div class="meta">{{ getTaskId(task) }}</div>
      </div>
    </label>
  </div>
</template>

<script setup lang="ts">
import type { TaskItem } from "../types/pmp";

defineProps<{
  tasks: TaskItem[];
}>();

defineEmits<{
  (e: "select", taskId: string): void;
}>();

function getTaskId(task: TaskItem): string {
  return (task.taskId || task.id || "") as string;
}
</script>

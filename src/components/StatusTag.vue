<template>
  <span class="status-tag" :class="cssClass" :title="reason">{{ reason }}</span>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  reason: string;
}>();

const cssClass = computed(() => {
  switch (props.reason) {
    case "未填":
      return "wei";
    case "已填":
      return "yi";
    case "休息":
      return "xiu";
    case "请假":
      return "jia";
    case "未来":
      return "wei-lai";
    default:
      return "yi";
  }
});
</script>

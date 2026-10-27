<template>
  <div class="loading-mask" :class="{ show: visible }" aria-live="polite" aria-busy="true">
    <div class="loading-spinner"></div>
    <div class="loading-text">{{ text }}</div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  visible: boolean;
  text: string;
}>();
</script>

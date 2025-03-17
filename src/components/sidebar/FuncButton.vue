<script setup lang="ts">
import { NIcon, NTooltip } from "naive-ui";

const props = defineProps<{
  icon: Object;
  tip: string;
}>();
</script>

<template>
  <div class="btn-content">
    <NTooltip trigger="hover">
      <template #trigger>
        <NIcon><component :is="props.icon" /></NIcon>
      </template>
      {{ props.tip }}
    </NTooltip>
  </div>
</template>

<style lang="scss" scoped>
.btn-content {
  height: 40px;
  display: flex;
  justify-content: center;
  align-items: center;

  cursor: pointer;
  transition: transform 0.3s ease;

  &:hover {
    color: var(--primary-hover-color);
    transform: scale(1.1);
  }
  &:active {
    color: var(--primary-pressed-color);
    transform: scale(0.9);
  }
}
</style>

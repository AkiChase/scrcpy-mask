<script setup lang="ts">
import { NIcon, NTooltip } from "naive-ui";
import { useRoute, useRouter } from "vue-router";

const props = defineProps<{
  name: string;
  icon: Object;
  tip: string;
}>();

const router = useRouter();
const route = useRoute();
function nav(name: string) {
  router.replace({ name });
}
</script>

<template>
  <div
    :class="{
      'nav-active': route.name === props.name,
      'nav-content': true,
    }"
    @click="nav(props.name)"
  >
    <NTooltip trigger="hover">
      <template #trigger>
        <NIcon><component :is="props.icon" /></NIcon>
      </template>
      {{ props.tip }}
    </NTooltip>
  </div>
</template>

<style lang="scss" scoped>
.nav-content {
  flex-shrink: 0;
  height: 50px;
  color: var(--gray-color);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.3s ease;
  box-sizing: border-box;
  font-size: 28px;
  cursor: pointer;

  &:hover {
    color: var(--primary-hover-color);
    transform: scale(1.05);
  }
  &:active {
    color: var(--primary-pressed-color);
    transform: scale(0.9);
  }
}

.nav-active {
  color: var(--primary-color);
  border-left: 3px solid var(--primary-color);
  border-radius: 3px;
}
</style>

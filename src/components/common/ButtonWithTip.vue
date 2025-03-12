<script setup lang="ts">
import { NTooltip, NIcon, NButton } from "naive-ui";

const props = defineProps<{
  tip: string;
  icon?: Object;
  circle?: boolean;
  disabled?: boolean;
  round?: boolean;
  size?: "tiny" | "small" | "medium" | "large";
  secondary?: boolean;
  tertiary?: boolean;
  text?: boolean;
}>();

defineOptions({ inheritAttrs: false });
</script>

<template>
  <NTooltip trigger="hover">
    <template #trigger>
      <NButton
        v-bind="$attrs"
        :circle="props.circle"
        :disabled="props.disabled"
        :round="props.round"
        :size="props.size"
        :secondary="props.secondary"
        :tertiary="props.tertiary"
        :text="props.text"
      >
        <template v-if="props.icon" #icon>
          <NIcon><component :is="props.icon" /></NIcon>
        </template>
        <slot></slot>
      </NButton>
    </template>
    {{ props.tip }}
  </NTooltip>
</template>

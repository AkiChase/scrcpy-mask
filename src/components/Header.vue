<script setup lang="ts">
import { NButtonGroup, NButton, NIcon } from "naive-ui";
import { Close } from "@vicons/ionicons5";
import { Maximize16Regular, Subtract16Regular } from "@vicons/fluent";
import { appWindow } from "@tauri-apps/api/window";

async function maximizeOrRestore() {
  appWindow.isMaximized().then((maximized) => {
    maximized ? appWindow.unmaximize() : appWindow.maximize();
  });
}
</script>

<template>
  <div data-tauri-drag-region class="header">
    <n-button-group>
      <n-button quaternary :focusable="false" @click="appWindow.minimize()">
        <template #icon>
          <n-icon><Subtract16Regular /></n-icon>
        </template>
      </n-button>
      <n-button quaternary :focusable="false" @click="maximizeOrRestore">
        <template #icon>
          <n-icon><Maximize16Regular /></n-icon>
        </template>
      </n-button>
      <n-button
        quaternary
        :focusable="false"
        class="close"
        @click="appWindow.close()"
      >
        <template #icon>
          <n-icon><Close /></n-icon>
        </template>
      </n-button>
    </n-button-group>
  </div>
</template>

<style scoped lang="scss">
.header {
  background-color: var(--bg-color);
  color: var(--light-color);
  grid-area: header;
  display: flex;
  justify-content: end;
  align-items: center;
  border-radius: 0 10px 0 0;

  .close {
    border-radius: 0 10px 0 0;
    &:hover {
      background-color: var(--red-color);
    }
    &:active {
      background-color: var(--red-pressed-color);
    }
  }
}
</style>

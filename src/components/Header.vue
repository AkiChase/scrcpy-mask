<script setup lang="ts">
import { NButtonGroup, NButton, NIcon } from "naive-ui";
import { Close } from "@vicons/ionicons5";
import { Maximize16Regular, Subtract16Regular } from "@vicons/fluent";
import { getCurrent } from "@tauri-apps/api/window";

async function maximizeOrRestore() {
  const appWindow = getCurrent();
  appWindow.isMaximized().then((maximized) => {
    maximized ? appWindow.unmaximize() : appWindow.maximize();
  });
}
</script>

<template>
  <div class="header">
    <div data-tauri-drag-region class="drag"></div>
    <NButtonGroup>
      <NButton quaternary :focusable="false" @click="getCurrent().minimize()">
        <template #icon>
          <NIcon><Subtract16Regular /></NIcon>
        </template>
      </NButton>
      <NButton quaternary :focusable="false" @click="maximizeOrRestore">
        <template #icon>
          <NIcon><Maximize16Regular /></NIcon>
        </template>
      </NButton>
      <NButton
        quaternary
        :focusable="false"
        class="close"
        @click="getCurrent().close()"
      >
        <template #icon>
          <NIcon><Close /></NIcon>
        </template>
      </NButton>
    </NButtonGroup>
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
  user-select: none;
  -webkit-user-select: none;

  .n-button-group{
    flex-shrink: 0;
  }

  .drag{
    flex-grow: 1;
    height: 100%;
  }

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

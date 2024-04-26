<script setup lang="ts">
import Sidebar from "./components/Sidebar.vue";
import Header from "./components/Header.vue";
import {
  darkTheme,
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
} from "naive-ui";
import { Store } from "@tauri-apps/plugin-store";
import { getCurrent } from "@tauri-apps/api/window";
import { KeyMappingConfig } from "./keyMappingConfig";
import { onMounted } from "vue";
import { useGlobalStore } from "./store/global";

const store = useGlobalStore();

onMounted(async () => {
  // loading keyMappingConfigList from local store
  const localStore = new Store("store.bin");
  let keyMappingConfigList: KeyMappingConfig[] | null = await localStore.get(
    "keyMappingConfigList"
  );
  if (keyMappingConfigList === null || keyMappingConfigList.length === 0) {
    keyMappingConfigList = [
      {
        relativeSize: await calMaskSize(),
        title: "空白方案",
        list: [],
      },
    ];
  }

  store.keyMappingConfigList = keyMappingConfigList;
});

async function calMaskSize() {
  const appWindow = getCurrent();
  const ml = 70;
  const mt = 30;
  let size = (await appWindow.outerSize()).toLogical(
    await appWindow.scaleFactor()
  );
  return {
    w: Math.round(size.width) - ml,
    h: Math.round(size.height) - mt,
  };
}
</script>

<template>
  <NConfigProvider :theme="darkTheme" class="container">
    <NMessageProvider>
      <Header />
      <NDialogProvider>
        <RouterView v-slot="{ Component }">
          <KeepAlive>
            <component :is="Component" />
          </KeepAlive>
        </RouterView>
      </NDialogProvider>
      <Sidebar />
    </NMessageProvider>
  </NConfigProvider>
</template>

<style>
.container {
  background-color: transparent;
  height: 100%;
  display: grid;
  grid-template-columns: 70px 1fr;
  grid-template-rows: 30px 1fr;
  grid-template-areas:
    "sidebar header"
    "sidebar content";
}

.n-scrollbar-container {
  background-color: var(--bg-color);
}

.n-scrollbar-content {
  height: 100%;
}
</style>
import { getCurrent } from "@tauri-apps/api/webview"; import { Store } from
"@tauri-apps/plugin-store"; import { KeyMappingConfig } from
"./keyMappingConfig";

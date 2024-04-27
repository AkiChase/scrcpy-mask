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
    // unable to get mask element when app is not ready
    // so we use the stored mask area to get relative size
    const maskArea: {
      posX: number;
      posY: number;
      sizeW: number;
      sizeH: number;
    } | null = await localStore.get("maskArea");
    let relativeSize = { w: 800, h: 600 };
    if (maskArea !== null) {
      relativeSize = {
        w: maskArea.sizeW,
        h: maskArea.sizeH,
      };
    }
    keyMappingConfigList = [
      {
        relativeSize,
        title: "空白方案",
        list: [],
      },
    ];
  }
  store.keyMappingConfigList = keyMappingConfigList;
  // loading curKeyMappingIndex from local store
  let curKeyMappingIndex: number | null = await localStore.get(
    "curKeyMappingIndex"
  );
  if (
    curKeyMappingIndex === null ||
    curKeyMappingIndex >= keyMappingConfigList.length
  ) {
    curKeyMappingIndex = 0;
    localStore.set("curKeyMappingIndex", curKeyMappingIndex);
  }
  store.curKeyMappingIndex = curKeyMappingIndex;
});
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

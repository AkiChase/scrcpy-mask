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
import { useRouter } from "vue-router";

const store = useGlobalStore();
const router = useRouter();

onMounted(async () => {
  router.replace({ name: "mask" });

  const localStore = new Store("store.bin");
  // loading screenSize from local store
  const screenSize = await localStore.get<{ sizeW: number; sizeH: number }>(
    "screenSize"
  );
  if (screenSize !== null) {
    store.screenSizeW = screenSize.sizeW;
    store.screenSizeH = screenSize.sizeH;
  }

  // loading keyMappingConfigList from local store
  let keyMappingConfigList = await localStore.get<KeyMappingConfig[]>(
    "keyMappingConfigList"
  );
  if (keyMappingConfigList === null || keyMappingConfigList.length === 0) {
    // add empty key mapping config
    // unable to get mask element when app is not ready
    // so we use the stored mask area to get relative size
    const maskArea = await localStore.get<{
      posX: number;
      posY: number;
      sizeW: number;
      sizeH: number;
    }>("maskArea");
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
    await localStore.set("keyMappingConfigList", keyMappingConfigList);
  }
  store.keyMappingConfigList = keyMappingConfigList;

  // loading curKeyMappingIndex from local store
  let curKeyMappingIndex = await localStore.get<number>("curKeyMappingIndex");
  if (
    curKeyMappingIndex === null ||
    curKeyMappingIndex >= keyMappingConfigList.length
  ) {
    curKeyMappingIndex = 0;
    localStore.set("curKeyMappingIndex", curKeyMappingIndex);
  }
  store.curKeyMappingIndex = curKeyMappingIndex;

  // loading maskButton from local store
  let maskButton = await localStore.get<{
    show: boolean;
    transparency: number;
  }>("maskButton");
  if (maskButton === null) {
    maskButton = {
      show: true,
      transparency: 0.5,
    };
    await localStore.set("maskButton", maskButton);
  }
  store.maskButton = maskButton;
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

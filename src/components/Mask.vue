<script setup lang="ts">
import { onActivated, ref } from "vue";
import { NDialog } from "naive-ui";
import { useGlobalStore } from "../store/global";
import { onBeforeRouteLeave, useRouter } from "vue-router";
import {
  initShortcuts,
  listenToKeyEvent,
  unlistenToKeyEvent,
  updateScreenSizeAndMaskArea,
} from "../hotkey";
import { getCurrent } from "@tauri-apps/api/window";

const maskRef = ref<HTMLElement | null>(null);

const store = useGlobalStore();
const router = useRouter();

let isShortcutInited = false;

onBeforeRouteLeave(() => {
  if (isShortcutInited) {
    if (maskRef.value) {
      unlistenToKeyEvent();
    }
  }
});

onActivated(async () => {
  if (isShortcutInited) {
    if (maskRef.value) {
      listenToKeyEvent();
    }
    return;
  }
  if (store.controledDevice) {
    if (maskRef.value) {
      const mt = 30;
      const ml = 70;
      const appWindow = getCurrent();
      const size = (await appWindow.outerSize()).toLogical(
        await appWindow.scaleFactor()
      );
      updateScreenSizeAndMaskArea(
        [store.screenSizeW, store.screenSizeH],
        [size.width - ml, size.height - mt]
      );
      initShortcuts(maskRef.value);
      listenToKeyEvent();
      isShortcutInited = true;
    }
  }
});

function toStartServer() {
  router.replace({ name: "device" });
}

// TODO 按键设置
// TODO 渲染按钮
</script>

<template>
  <div v-show="!store.controledDevice" class="notice">
    <div class="content">
      <NDialog
        :closable="false"
        title="未找到受控设备"
        content="请启动服务端并控制任意设备"
        positive-text="去启动"
        type="warning"
        @positive-click="toStartServer"
      />
    </div>
  </div>
  <div
    v-show="store.controledDevice"
    @contextmenu.prevent
    class="mask"
    ref="maskRef"
  ></div>
</template>

<style scoped lang="scss">
.mask {
  background-color: transparent;
  overflow: hidden;
  cursor: pointer;
}
.notice {
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;

  .content {
    width: 80%;
  }
}
</style>

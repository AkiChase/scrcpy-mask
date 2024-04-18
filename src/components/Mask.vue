<script setup lang="ts">
import { onActivated, ref } from "vue";
import { NDialog } from "naive-ui";
import { useGlobalStore } from "../store/global";
import { onBeforeRouteLeave, useRouter } from "vue-router";
import {
  applyShortcuts,
  clearShortcuts,
  listenToKeyEvent,
  unlistenToKeyEvent,
  updateScreenSizeAndMaskArea,
} from "../hotkey";
import { getCurrent } from "@tauri-apps/api/window";

const maskRef = ref<HTMLElement | null>(null);

const store = useGlobalStore();
const router = useRouter();

onBeforeRouteLeave(() => {
  if (maskRef.value && store.controledDevice) {
    unlistenToKeyEvent();
    clearShortcuts();
  }
});

onActivated(async () => {
  if (maskRef.value && store.controledDevice) {
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

    applyShortcuts(maskRef.value);
    listenToKeyEvent();
  }
});

function toStartServer() {
  router.replace({ name: "device" });
}

// TODO 3. 根据配置渲染按钮
  // 配置文件读取到store中，不要每次都io读取

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

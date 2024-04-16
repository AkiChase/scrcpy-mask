<script setup lang="ts">
import { onActivated, ref } from "vue";
import { NDialog } from "naive-ui";
import { useGlobalStore } from "../store/global";
import { onBeforeRouteLeave, useRouter } from "vue-router";
import { initShortcuts, listenToKeyEvent, unlistenToKeyEvent } from "../hotkey";

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
      initShortcuts([store.screenSizeW, store.screenSizeH], maskRef.value);
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
  background-color: rgba(255, 255, 255, 0.2);
  overflow: hidden;
  cursor: pointer;
}
.notice {
  background-color: rgba(255, 255, 255, 0.2);
  display: flex;
  justify-content: center;
  align-items: center;

  .content {
    width: 80%;
  }
}
</style>

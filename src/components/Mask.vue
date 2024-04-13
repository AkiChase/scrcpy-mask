<script setup lang="ts">
import { onActivated, ref } from "vue";
import { NDialog } from "naive-ui";
import { useGlobalStore } from "../store/global";
import { useRouter } from "vue-router";
import { getCurrent } from "@tauri-apps/api/window";
import { initShortcuts } from "../hotkey";
import { getScreenSize } from "../invoke";

const maskRef = ref<HTMLElement | null>(null);

const store = useGlobalStore();
const router = useRouter();

let isShortcutInited = false;

onActivated(async () => {
  if (isShortcutInited) {
    maskRef.value?.focus();
    return;
  }
  if (store.controledDevice) {
    let screenSize = await getScreenSize(store.controledDevice.device.id);
    if (maskRef.value) {
      const appWindow = getCurrent();
      let posFactor = await appWindow.scaleFactor();
      initShortcuts(maskRef.value, posFactor, screenSize);
      isShortcutInited = true;
      maskRef.value.focus();
      console.log("热键已载入");
    }
  }
});

function toStartServer() {
  router.replace({ name: "device" });
}

// TODO 监听快捷键
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
    tabindex="-1"
    class="mask"
    ref="maskRef"
  ></div>
</template>

<style scoped lang="scss">
.mask {
  background-color: rgba(255, 255, 255, 0.2);
  overflow: hidden;

  &:focus {
    outline: none;
    box-shadow: 0 0 5px var(--primary-color);
  }
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

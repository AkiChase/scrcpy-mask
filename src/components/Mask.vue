<script setup lang="ts">
import { onActivated, ref } from "vue";
import { NDialog } from "naive-ui";
import { useGlobalStore } from "../store/global";
import { useRouter } from "vue-router";
import { appWindow } from "@tauri-apps/api/window";
import { initKeyboardShortcuts, initMouseShortcuts } from "../hotkey";
import { getScreenSize } from "../invoke";

const maskRef = ref<HTMLElement | null>(null);

const store = useGlobalStore();
const router = useRouter();

let isShortcutInited = false;

onActivated(async () => {
  if (isShortcutInited) return;
  if (store.controledDevices.length) {
    let screenSize = await getScreenSize(store.controledDevices[0].device.id);
    if (maskRef.value) {
      let posFactor = await appWindow.scaleFactor();
      initMouseShortcuts(maskRef.value, posFactor, screenSize);
      initKeyboardShortcuts(maskRef.value);
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
  <div v-show="store.controledDevices.length === 0" class="notice">
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
    v-show="store.controledDevices.length"
    tabindex="-1"
    class="mask"
    ref="maskRef"
  ></div>
  <!-- <div class="mask" ref="maskRef"></div> -->
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

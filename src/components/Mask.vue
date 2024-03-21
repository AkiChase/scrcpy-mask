<script setup lang="ts">
import { onMounted, ref } from "vue";
import { NDialog } from "naive-ui";
import { useGlobalStore } from "../store/global";
import { useRouter } from "vue-router";
import { TouchAction, touch } from "../frontcommand/scrcpyMaskCmd";
import { appWindow } from "@tauri-apps/api/window";

const maskRef = ref<HTMLElement | null>(null);

const store = useGlobalStore();
const router = useRouter();

let posFactor = 1;
onMounted(async () => {
  posFactor = await appWindow.scaleFactor();
});

function clientxToPosx(clientx: number) {
  return Math.floor((clientx - 70) * posFactor);
}

function clientyToPosy(clienty: number) {
  return Math.floor((clienty - 30) * posFactor);
}

function toStartServer() {
  router.replace({ name: "device" });
}

async function handleMouseDown(event: MouseEvent) {
  event.preventDefault();
  // TODO 目前先不考虑按键的情况，仅仅注入点击事件
  if (event.button === 0) {
    // Check if left mouse button is pressed (button code 0)
    maskRef.value?.addEventListener("mousemove", handleMouseMove);
    await touch({
      action: TouchAction.Down,
      pointerId: 0,
      screen: {
        w: store.screenSize.w,
        h: store.screenSize.h,
      },
      pos: {
        x: clientxToPosx(event.clientX),
        y: clientyToPosy(event.clientY),
      },
    });
  }
}

async function handleMouseUp(event: MouseEvent) {
  event.preventDefault();

  if (event.button === 0) {
    // Check if left mouse button is pressed (button code 0)
    maskRef.value?.removeEventListener("mousemove", handleMouseMove);
    await touch({
      action: TouchAction.Up,
      pointerId: 0,
      screen: {
        w: store.screenSize.w,
        h: store.screenSize.h,
      },
      pos: {
        x: clientxToPosx(event.clientX),
        y: clientyToPosy(event.clientY),
      },
    });
  }
}

async function handleMouseMove(event: MouseEvent) {
  event.preventDefault();
  await touch({
      action: TouchAction.Move,
      pointerId: 0,
      screen: {
        w: store.screenSize.w,
        h: store.screenSize.h,
      },
      pos: {
        x: clientxToPosx(event.clientX),
        y: clientyToPosy(event.clientY),
      },
    });
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
    class="mask"
    ref="maskRef"
    @mousedown="handleMouseDown"
    @mouseup="handleMouseUp"
  ></div>
</template>

<style scoped lang="scss">
.mask {
  background-color: rgba(255, 255, 255, 0.2);
  overflow: hidden;
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

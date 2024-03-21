<script setup lang="ts">
import { ref } from "vue";
import { NDialog } from "naive-ui";
import { useGlobalStore } from "../store/global";
import { useRouter } from "vue-router";

const maskRef = ref<HTMLElement | null>(null);

const store = useGlobalStore();
const router = useRouter();

function toStartServer() {
  router.replace({ name: "device" });
}

function handleMouseDown(event: MouseEvent) {
  event.preventDefault();
  // 目前先不考虑按键的情况，仅仅注入点击事件
  console.log("down", event);
  maskRef.value?.addEventListener("mousemove", handleMouseMove);
}

function handleMouseUp(event: MouseEvent) {
  event.preventDefault();

  console.log("up", event);
  maskRef.value?.removeEventListener("mousemove", handleMouseMove);
}

function handleMouseMove(event: MouseEvent) {
  event.preventDefault();
  console.log("move", event);
}

// TODO 监听快捷键
// TODO 按键设置
// TODO 渲染按钮
</script>

<template>
  <div v-show="!store.isServerRunning" class="notice">
    <div class="content">
      <NDialog
        :closable="false"
        title="服务端未启动"
        content="请先启动服务端并控制任意设备"
        positive-text="去启动"
        type="warning"
        @positive-click="toStartServer"
      />
    </div>
  </div>
  <div
    v-show="store.isServerRunning"
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

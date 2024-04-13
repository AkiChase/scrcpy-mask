<script setup lang="ts">
import { getCurrent } from "@tauri-apps/api/window";
import { onActivated, onMounted, ref } from "vue";
import { onBeforeRouteLeave } from "vue-router";

// TODO 添加右侧按键列表用于拖放
// TODO 在进入此页面时扩宽窗口，离开时恢复窗口大小

const keyboardElement = ref<HTMLElement | null>(null);
const mouseX = ref(0);
const mouseY = ref(0);

let posFactor = 1;
function clientxToPosx(clientx: number) {
  return clientx < 70 ? 0 : Math.floor((clientx - 70) * posFactor);
}

function clientyToPosy(clienty: number) {
  return clienty < 30 ? 0 : Math.floor((clienty - 30) * posFactor);
}

let ignoreMousemove = true;
function mousemoveHandler(event: MouseEvent) {
  ignoreMousemove = !ignoreMousemove;
  if (ignoreMousemove) return;
  mouseX.value = clientxToPosx(event.clientX);
  mouseY.value = clientyToPosy(event.clientY);
}

onMounted(async () => {
  const appWindow = getCurrent();
  posFactor = await appWindow.scaleFactor();
});

onActivated(() => {
  keyboardElement.value?.addEventListener("mousemove", mousemoveHandler);
});

onBeforeRouteLeave(() => {
  keyboardElement.value?.removeEventListener("mousemove", mousemoveHandler);
});
</script>

<template>
  <div ref="keyboardElement" class="keyboard">
    此处最好用其他颜色的蒙版，和右侧的按键列表区同色
    <div>{{ mouseX }}, {{ mouseY }}</div>
  </div>
</template>

<style scoped>
.keyboard {
  background-color: rgba(255, 255, 255, 0.5);
  overflow: hidden;
}
</style>

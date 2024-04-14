<script setup lang="ts">
import { Ref, onActivated, ref } from "vue";
import { onBeforeRouteLeave } from "vue-router";

// TODO 添加右侧按键列表用于拖放
// TODO 在进入此页面时扩宽窗口，离开时恢复窗口大小

const keyboardElement = ref<HTMLElement | null>(null);
const mouseX = ref(0);
const mouseY = ref(0);

function clientxToPosx(clientx: number) {
  return clientx < 70 ? 0 : Math.floor(clientx - 70);
}

function clientyToPosy(clienty: number) {
  return clienty < 30 ? 0 : Math.floor(clienty - 30);
}

let ignoreMousemove = true;
function mousemoveHandler(event: MouseEvent) {
  ignoreMousemove = !ignoreMousemove;
  if (ignoreMousemove) return;
  mouseX.value = clientxToPosx(event.clientX);
  mouseY.value = clientyToPosy(event.clientY);
}

const keyboardCodeList: Ref<string[]> = ref([]);
function keyupHandler(event: KeyboardEvent) {
  event.preventDefault();
  if (keyboardCodeList.value.length > 10) {
    keyboardCodeList.value.shift();
    keyboardCodeList.value.push(event.code);
  } else keyboardCodeList.value.push(event.code);
}

onActivated(() => {
  keyboardElement.value?.addEventListener("mousemove", mousemoveHandler);
  document.addEventListener("keyup", keyupHandler);
});

onBeforeRouteLeave(() => {
  keyboardElement.value?.removeEventListener("mousemove", mousemoveHandler);
  document.removeEventListener("keyup", keyupHandler);
});
</script>

<template>
  <div ref="keyboardElement" class="keyboard">
    此处最好用其他颜色的蒙版，和右侧的按键列表区同色
    <div>{{ mouseX }}, {{ mouseY }}</div>
    <div v-for="code in keyboardCodeList">
      {{ code }}
    </div>
  </div>
</template>

<style scoped>
.keyboard {
  background-color: rgba(255, 255, 255, 0.5);
  overflow: hidden;
}
</style>

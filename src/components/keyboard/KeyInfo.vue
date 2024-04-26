<script setup lang="ts">
import { NIcon } from "naive-ui";
import { CloseCircle } from "@vicons/ionicons5";
import { Ref, onActivated, ref } from "vue";
import { onBeforeRouteLeave } from "vue-router";

const showFlag = defineModel({ required: true });

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

function mousedownHandler(event: MouseEvent) {
  event.preventDefault();
  const key = `M${event.button}`;
  if (keyboardCodeList.value.length > 10) {
    keyboardCodeList.value.shift();
    keyboardCodeList.value.push(key);
  } else keyboardCodeList.value.push(key);
}

onActivated(() => {
  const keyboardElement = document.getElementById("keyboardElement");
  if (keyboardElement) {
    keyboardElement.addEventListener("mousemove", mousemoveHandler);
    keyboardElement.addEventListener("mousedown", mousedownHandler);
    document.addEventListener("keyup", keyupHandler);
  }
});

onBeforeRouteLeave(() => {
  const keyboardElement = document.getElementById("keyboardElement");
  if (keyboardElement) {
    keyboardElement.removeEventListener("mousemove", mousemoveHandler);
    keyboardElement.removeEventListener("mousedown", mousedownHandler);
    document.removeEventListener("keyup", keyupHandler);
  }
});

let lastPosX = 0;
let lastPosY = 0;
function dragHandler(downEvent: MouseEvent) {
  if (
    downEvent.target instanceof HTMLElement &&
    downEvent.target.className === "key-info-header"
  ) {
    const target = downEvent.target;
    downEvent.preventDefault();
    target.style.setProperty("cursor", "grabbing");
    const element = downEvent.target.parentElement;
    const x = downEvent.clientX;
    const y = downEvent.clientY;
    const moveHandler = (moveEvent: MouseEvent) => {
      let left = lastPosX + moveEvent.clientX - x;
      let top = lastPosY + moveEvent.clientY - y;
      element?.style.setProperty("left", `${left < 0 ? 0 : left}px`);
      element?.style.setProperty("top", `${top < 0 ? 0 : top}px`);
    };
    window.addEventListener("mousemove", moveHandler);
    const upHandler = (upEvent: MouseEvent) => {
      lastPosX = lastPosX + upEvent.clientX - x;
      lastPosY = lastPosY + upEvent.clientY - y;
      window.removeEventListener("mousemove", moveHandler);
      window.removeEventListener("mouseup", upHandler);
      target.style.setProperty("cursor", "grab");
    };
    window.addEventListener("mouseup", upHandler);
  }
}
</script>

<template>
  <div v-show="showFlag" class="key-info" @contextmenu.prevent>
    <div class="key-info-header" @mousedown="dragHandler">
      Key Info
      <div class="key-info-close" @click="showFlag = false">
        <NIcon><CloseCircle></CloseCircle></NIcon>
      </div>
    </div>
    <div class="key-info-content">
      <div style="border-bottom: 1px solid var(--light-color)">
        {{ mouseX }}, {{ mouseY }}
      </div>
      <div v-if="keyboardCodeList.length === 0">Press any key</div>
      <div v-for="code in keyboardCodeList">
        {{ code }}
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.key-info {
  color: var(--light-color);
  background-color: var(--content-bg-color);
  width: 120px;
  border-radius: 10px;
  user-select: none;
  -webkit-user-select: none;
  position: absolute;
  z-index: 8;

  .key-info-header {
    background-color: var(--gray-color);
    color: var(--bg-color);
    font-weight: bold;
    height: 20px;
    border-radius: 10px 10px 0 0;
    text-align: center;
    cursor: grab;
    position: relative;

    .key-info-close {
      position: absolute;
      transition: color 0.3s;
      right: 5px;
      top: 0;
      height: 100%;
      display: flex;
      align-items: center;
      color: var(--content-bg-color);
      font-size: 14px;
      cursor: pointer;
      &:hover {
        color: var(--red-color);
      }
      &:active {
        color: var(--red-pressed-color);
      }
    }
  }

  .key-info-content {
    text-align: center;
    border: 1px solid var(--gray-color);
    padding: 10px;
  }
}
</style>

<script setup lang="ts">
import { Ref, onActivated, ref } from "vue";
import { NDialog, useMessage } from "naive-ui";
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

const store = useGlobalStore();
const router = useRouter();
const message = useMessage();

const renderedButtons: Ref<any[]> = ref([]);

onBeforeRouteLeave(() => {
  const maskElement = document.getElementById("maskElement") as HTMLElement;

  if (store.controledDevice) {
    unlistenToKeyEvent();
    clearShortcuts(maskElement);
  }
});

onActivated(async () => {
  const maskElement = document.getElementById("maskElement") as HTMLElement;

  if (store.controledDevice) {
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

    if (
      applyShortcuts(
        maskElement,
        store.keyMappingConfigList[store.curKeyMappingIndex]
      )
    ) {
      refreshKeyMappingButton();
      listenToKeyEvent();
    } else {
      message.error("按键方案异常，请删除此方案");
    }
  }
});

function toStartServer() {
  router.replace({ name: "device" });
}

function refreshKeyMappingButton() {
  const maskElement = document.getElementById("maskElement") as HTMLElement;

  const curKeyMappingConfig =
    store.keyMappingConfigList[store.curKeyMappingIndex];
  const relativeSize = curKeyMappingConfig.relativeSize;
  const maskSizeW = maskElement.clientWidth;
  const maskSizeH = maskElement.clientHeight;
  const relativePosToMaskPos = (x: number, y: number) => {
    return {
      x: Math.round((x / relativeSize.w) * maskSizeW),
      y: Math.round((y / relativeSize.h) * maskSizeH),
    };
  };
  const buttons = [];
  for (let keyObject of curKeyMappingConfig.list) {
    const { x, y } = relativePosToMaskPos(keyObject.posX, keyObject.posY);
    buttons.push({
      ...keyObject,
      x,
      y,
    });
  }
  renderedButtons.value = buttons;
}
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
    id="maskElement"
  >
    <template v-for="button in renderedButtons">
      <div
        v-if="button.type === 'SteeringWheel'"
        class="mask-steering-wheel"
        :style="{
          left: button.x - 75 + 'px',
          top: button.y - 75 + 'px',
        }"
      >
        <div class="wheel-container">
          <i />
          <span>{{ button.key.up }}</span>
          <i />
          <span>{{ button.key.left }}</span>
          <i />
          <span>{{ button.key.right }}</span>
          <i />
          <span>{{ button.key.down }}</span>
          <i />
        </div>
      </div>
      <div
        v-else
        class="mask-button"
        :style="{
          left: button.x + 'px',
          top: button.y - 14 + 'px',
        }"
      >
        {{ button.key }}
      </div>
    </template>
  </div>
</template>

<style scoped lang="scss">
.mask {
  background-color: transparent;
  overflow: hidden;
  cursor: pointer;
  position: relative;
  border-right: 1px solid var(--bg-color);
  border-bottom: 1px solid var(--bg-color);
  border-radius: 0 0 5px 0;

  & > .mask-button {
    position: absolute;
    background-color: rgba(0, 0, 0, 0.2);
    color: rgba(255, 255, 255, 0.6);
    border-radius: 5px;
    padding: 5px;
    font-size: 12px;
  }

  & > .mask-steering-wheel {
    position: absolute;
    background-color: rgba(0, 0, 0, 0.2);
    color: rgba(255, 255, 255, 0.6);
    border-radius: 50%;
    width: 150px;
    height: 150px;
    font-size: 12px;

    .wheel-container {
      display: grid;
      grid-template-columns: repeat(3, 50px);
      grid-template-rows: repeat(3, 50px);
      justify-items: center;
      align-items: center;
    }
  }
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

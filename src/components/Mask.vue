<script setup lang="ts">
import { onActivated } from "vue";
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
import { KeySteeringWheel } from "../keyMappingConfig";

const store = useGlobalStore();
const router = useRouter();
const message = useMessage();

onBeforeRouteLeave(() => {
  if (store.controledDevice) {
    unlistenToKeyEvent();
    clearShortcuts();
  }
});

onActivated(async () => {
  const maskElement = document.getElementById("maskElement") as HTMLElement;

  if (store.controledDevice) {
    updateScreenSizeAndMaskArea(
      [store.screenSizeW, store.screenSizeH],
      [maskElement.clientWidth, maskElement.clientHeight]
    );

    if (
      applyShortcuts(
        maskElement,
        store.keyMappingConfigList[store.curKeyMappingIndex]
      )
    ) {
      listenToKeyEvent();
    } else {
      message.error("按键方案异常，请删除此方案");
    }
  }
});

function toStartServer() {
  router.replace({ name: "device" });
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
  <template v-if="store.keyMappingConfigList.length">
    <div @contextmenu.prevent class="mask" id="maskElement"></div>
    <div class="button-layer">
      <!-- <div style="position: absolute;height: 100%;width: 1px;background-color: red;left: 50%;"></div>
      <div style="position: absolute;width: 100%;height: 1px;background-color: red;top: 56.6%;"></div> -->
      <template
        v-for="button in store.keyMappingConfigList[store.curKeyMappingIndex]
          .list"
      >
        <div
          v-if="button.type === 'SteeringWheel'"
          class="mask-steering-wheel"
          :style="{
            left: button.posX - 75 + 'px',
            top: button.posY - 75 + 'px',
          }"
        >
          <div class="wheel-container">
            <i />
            <span>{{ (button as KeySteeringWheel).key.up }}</span>
            <i />
            <span>{{ (button as KeySteeringWheel).key.left }}</span>
            <i />
            <span>{{ (button as KeySteeringWheel).key.right }}</span>
            <i />
            <span>{{ (button as KeySteeringWheel).key.down }}</span>
            <i />
          </div>
        </div>
        <div
          v-else
          class="mask-button"
          :style="{
            left: button.posX + 'px',
            top: button.posY - 14 + 'px',
          }"
        >
          {{ button.key }}
        </div>
      </template>
    </div>
  </template>
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
  user-select: none;
  -webkit-user-select: none;
  z-index: 2;
}

.button-layer {
  position: absolute;
  left: 70px;
  top: 30px;
  right: 0;
  bottom: 0;
  background-color: transparent;
  user-select: none;
  -webkit-user-select: none;
  z-index: 1;

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
  z-index: 1;
  position: absolute;
  left: 70px;
  top: 30px;
  right: 0;
  bottom: 0;
  z-index: 3;

  .content {
    width: 80%;
  }
}
</style>

<script setup lang="ts">
import { onActivated, onMounted, onUnmounted, ref } from "vue";
import { NDialog, useMessage } from "naive-ui";
import { useGlobalStore } from "../store/global";
import { onBeforeRouteLeave, useRouter } from "vue-router";
import {
  KeyInputHandler,
  applyShortcuts,
  clearShortcuts,
  listenToEvent,
  unlistenToEvent,
} from "../tools/hotkey";
import { KeySteeringWheel } from "../tools/keyMappingConfig";
import ScreenStream from "./ScreenStream.vue";
import { useI18n } from "vue-i18n";
import { primaryInit, secondaryClean, secondaryInit } from "../tools/init";
import { NonReactiveStore } from "../store/noneReactiveStore";
import { useRotation } from "../tools/hooks";

await primaryInit(); // suspend for primary initialization

const { t } = useI18n();
const store = useGlobalStore();
const router = useRouter();
const message = useMessage();
const rotation = useRotation();

let initFlag = false;

const curPageActive = ref(false);

onBeforeRouteLeave(() => {
  curPageActive.value = false;
  if (store.controledDevice) {
    unlistenToEvent();
    clearShortcuts();
    if (NonReactiveStore.mem.keyInputFlag)
      KeyInputHandler.removeEventListener();
  }
});

onActivated(async () => {
  curPageActive.value = true;

  if (store.controledDevice) {
    if (
      applyShortcuts(
        store.keyMappingConfigList[store.curKeyMappingIndex],
        store,
        message,
        t
      )
    ) {
      listenToEvent();
    } else {
      message.error(t("pages.Mask.keyconfigException"));
    }
  }
  await rotation();
});

onMounted(async () => {
  await secondaryInit();
  initFlag = true;
});

onUnmounted(() => {
  secondaryClean();
});

function toStartServer() {
  router.replace({ name: "device" });
}
</script>

<template>
  <div class="content-container">
    <div v-show="store.controledDevice === null" class="notice">
      <div class="content">
        <NDialog
          :closable="false"
          :title="$t('pages.Mask.noControledDevice.title')"
          :content="$t('pages.Mask.noControledDevice.content')"
          :positive-text="$t('pages.Mask.noControledDevice.positiveText')"
          type="warning"
          @positive-click="toStartServer"
        />
      </div>
    </div>
    <template v-if="store.keyMappingConfigList.length">
      <div @contextmenu.prevent class="mask" id="maskElement"></div>
      <ScreenStream
        v-if="
          curPageActive && store.controledDevice && store.screenStream.enable
        "
      />
      <div
        v-if="store.maskKeyTip.show"
        :style="'--transparency: ' + store.maskKeyTip.transparency"
        class="button-layer"
      >
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
            {{ button.type === "Fire" ? "Fire" : button.key }}
          </div>
        </template>
      </div>
    </template>
  </div>
</template>

<style scoped lang="scss">
@use "../css/common.scss";

.content-container {
  @include common.contentContainer;
}

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
  width: 100%;
  height: 100%;
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
    opacity: var(--transparency);
    position: absolute;
    background-color: black;
    color: white;
    border-radius: 5px;
    padding: 5px;
    font-size: 12px;
  }

  & > .mask-steering-wheel {
    opacity: var(--transparency);
    position: absolute;
    background-color: black;
    color: white;
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

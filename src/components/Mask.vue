<script setup lang="ts">
import { h, nextTick, onActivated, onMounted, ref } from "vue";
import { NDialog, NInput, useDialog, useMessage } from "naive-ui";
import { useGlobalStore } from "../store/global";
import { onBeforeRouteLeave, useRouter } from "vue-router";
import {
  applyShortcuts,
  clearShortcuts,
  listenToEvent,
  unlistenToEvent,
  updateScreenSizeAndMaskArea,
} from "../hotkey";
import { KeyMappingConfig, KeySteeringWheel } from "../keyMappingConfig";
import { getVersion } from "@tauri-apps/api/app";
import { fetch } from "@tauri-apps/plugin-http";
import { open } from "@tauri-apps/plugin-shell";
import { sendSetClipboard } from "../frontcommand/controlMsg";
import { getCurrent, PhysicalSize } from "@tauri-apps/api/window";
import { AndroidKeycode } from "../frontcommand/android";
import { Store } from "@tauri-apps/plugin-store";
import { useI18n } from "vue-i18n";
import { SendKeyAction, sendKey } from "../frontcommand/scrcpyMaskCmd";

const { t } = useI18n();
const store = useGlobalStore();
const router = useRouter();
const message = useMessage();
const dialog = useDialog();

const showInputBoxRef = ref(false);
const inputBoxVal = ref("");
const inputInstRef = ref<HTMLInputElement | null>(null);

onBeforeRouteLeave(() => {
  if (store.controledDevice) {
    unlistenToEvent();
    clearShortcuts();
  }
});

onActivated(async () => {
  cleanAfterimage();
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
      listenToEvent();
    } else {
      message.error(t("pages.Mask.keyconfigException"));
    }
  }
});

onMounted(async () => {
  await loadLocalStore();
  store.checkUpdate = checkUpdate;
  store.showInputBox = showInputBox;
  if (store.checkUpdateAtStart) checkUpdate();
});

async function loadLocalStore() {
  const localStore = new Store("store.bin");
  // loading screenSize from local store
  const screenSize = await localStore.get<{ sizeW: number; sizeH: number }>(
    "screenSize"
  );
  if (screenSize !== null) {
    store.screenSizeW = screenSize.sizeW;
    store.screenSizeH = screenSize.sizeH;
  }

  // loading keyMappingConfigList from local store
  let keyMappingConfigList = await localStore.get<KeyMappingConfig[]>(
    "keyMappingConfigList"
  );
  if (keyMappingConfigList === null || keyMappingConfigList.length === 0) {
    // add empty key mapping config
    // unable to get mask element when app is not ready
    // so we use the stored mask area to get relative size
    const maskArea = await localStore.get<{
      posX: number;
      posY: number;
      sizeW: number;
      sizeH: number;
    }>("maskArea");
    let relativeSize = { w: 800, h: 600 };
    if (maskArea !== null) {
      relativeSize = {
        w: maskArea.sizeW,
        h: maskArea.sizeH,
      };
    }
    keyMappingConfigList = [
      {
        relativeSize,
        title: t("pages.Mask.blankConfig"),
        list: [],
      },
    ];
    await localStore.set("keyMappingConfigList", keyMappingConfigList);
  }
  store.keyMappingConfigList = keyMappingConfigList;

  // loading curKeyMappingIndex from local store
  let curKeyMappingIndex = await localStore.get<number>("curKeyMappingIndex");
  if (
    curKeyMappingIndex === null ||
    curKeyMappingIndex >= keyMappingConfigList.length
  ) {
    curKeyMappingIndex = 0;
    localStore.set("curKeyMappingIndex", curKeyMappingIndex);
  }
  store.curKeyMappingIndex = curKeyMappingIndex;

  // loading maskButton from local store
  let maskButton = await localStore.get<{
    show: boolean;
    transparency: number;
  }>("maskButton");
  store.maskButton = maskButton ?? {
    show: true,
    transparency: 0.5,
  };

  // loading checkUpdateAtStart from local store
  let checkUpdateAtStart = await localStore.get<boolean>("checkUpdateAtStart");
  store.checkUpdateAtStart = checkUpdateAtStart ?? true;
}

async function cleanAfterimage() {
  const appWindow = getCurrent();
  const oldSize = await appWindow.outerSize();
  const newSize = new PhysicalSize(oldSize.width, oldSize.height + 1);
  await appWindow.setSize(newSize);
  await appWindow.setSize(oldSize);
}

function handleInputBoxClick(event: MouseEvent) {
  if (event.target === document.getElementById("input-box")) {
    showInputBox(false);
  }
}

function handleInputKeyUp(event: KeyboardEvent) {
  if (event.key === "Enter") {
    pasteText();
  } else if (event.key === "Escape") {
    showInputBox(false);
  }
}

function showInputBox(flag: boolean) {
  if (flag) {
    unlistenToEvent();
    inputBoxVal.value = "";
    showInputBoxRef.value = true;
    document.addEventListener("keyup", handleInputKeyUp);
    nextTick(() => {
      inputInstRef.value?.focus();
    });
  } else {
    document.removeEventListener("keyup", handleInputKeyUp);
    inputInstRef.value?.blur();
    showInputBoxRef.value = false;
    listenToEvent();
    nextTick(() => {
      cleanAfterimage();
    });
  }
}

function sleep(time: number) {
  return new Promise<void>((resolve) => {
    setTimeout(() => {
      resolve();
    }, time);
  });
}

async function pasteText() {
  showInputBox(false);
  if (!inputBoxVal.value) return;
  sendSetClipboard({
    sequence: new Date().getTime() % 100000,
    text: inputBoxVal.value,
    paste: true,
  });
  await sleep(300);
  // send enter
  await sendKey({
    action: SendKeyAction.Default,
    keycode: AndroidKeycode.AKEYCODE_ENTER,
  });
}

function toStartServer() {
  router.replace({ name: "device" });
}

function renderUpdateInfo(content: string) {
  const pList = content.split("\r\n").map((line: string) => h("p", line));
  return h("div", { style: "margin: 20px 0" }, pList);
}

function compareVersion(v1: string, v2: string) {
  const [x1, y1, z1] = v1.split(".");
  const [x2, y2, z2] = v2.split(".");

  if (x1 !== x2) {
    return parseInt(x1) > parseInt(x2) ? 1 : -1;
  }
  if (y1 !== y2) {
    return parseInt(y1) > parseInt(y2) ? 1 : -1;
  }
  if (z1 !== z2) {
    return parseInt(z1) > parseInt(z2) ? 1 : -1;
  }

  return 0;
}

async function checkUpdate() {
  try {
    const curVersion = await getVersion();
    const res = await fetch(
      "https://api.github.com/repos/AkiChase/scrcpy-mask/releases/latest",
      {
        connectTimeout: 5000,
      }
    );
    if (res.status !== 200) {
      message.error(t("pages.Mask.checkUpdate.failed"));
    } else {
      const data = await res.json();
      const latestVersion = (data.tag_name as string).slice(1);
      if (compareVersion(curVersion, latestVersion) >= 0) {
        message.success(
          t("pages.Mask.checkUpdate.isLatest", [latestVersion, curVersion])
        );
        return;
      }
      const body = data.body as string;
      dialog.info({
        title: t("pages.Mask.checkUpdate.notLatest.title", [latestVersion]),
        content: () => renderUpdateInfo(body),
        positiveText: t("pages.Mask.checkUpdate.notLatest.positiveText"),
        negativeText: t("pages.Mask.checkUpdate.notLatest.negativeText"),
        onPositiveClick: () => {
          open(data.html_url);
        },
      });
    }
  } catch (e) {
    console.error(e);
    message.error(t("pages.Mask.checkUpdate.failed"));
  }
}
</script>

<template>
  <div v-show="!store.controledDevice" class="notice">
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
    <div
      v-show="showInputBoxRef"
      class="input-box"
      id="input-box"
      @click="handleInputBoxClick"
    >
      <NInput
        ref="inputInstRef"
        v-model:value="inputBoxVal"
        type="text"
        :placeholder="$t('pages.Mask.inputBoxPlaceholder')"
      />
    </div>
    <div
      v-if="store.maskButton.show"
      :style="'--transparency: ' + store.maskButton.transparency"
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

.input-box {
  z-index: 4;
  position: absolute;
  left: 70px;
  top: 30px;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);

  .n-input {
    width: 50%;
    position: absolute;
    left: 0;
    right: 0;
    bottom: 15%;
    margin: auto;
    background-color: var(--content-bg-color);
  }
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
h, import { getVersion } from "@tauri-apps/api/app";

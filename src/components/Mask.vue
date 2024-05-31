<script setup lang="ts">
import { h, onActivated, onMounted, ref } from "vue";
import { MessageReactive, NDialog, useDialog, useMessage } from "naive-ui";
import { useGlobalStore } from "../store/global";
import { onBeforeRouteLeave, useRouter } from "vue-router";
import {
  KeyInputHandler,
  applyShortcuts,
  clearShortcuts,
  listenToEvent,
  unlistenToEvent,
} from "../hotkey";
import { KeySteeringWheel } from "../keyMappingConfig";
import ScreenStream from "./ScreenStream.vue";
import { getVersion } from "@tauri-apps/api/app";
import { fetch } from "@tauri-apps/plugin-http";
import { open } from "@tauri-apps/plugin-shell";
import { getCurrent, PhysicalSize } from "@tauri-apps/api/window";
import { Store } from "@tauri-apps/plugin-store";
import { useI18n } from "vue-i18n";
import { checkAdbAvailable } from "../invoke";
import { loadLocalStorage } from "../storeLoader";

const { t } = useI18n();
const store = useGlobalStore();
const router = useRouter();
const message = useMessage();
const dialog = useDialog();

const curPageActive = ref(false);

onBeforeRouteLeave(() => {
  curPageActive.value = false;
  if (store.controledDevice) {
    unlistenToEvent();
    clearShortcuts();
    if (store.keyInputFlag) KeyInputHandler.removeEventListener();
  }
});

onActivated(async () => {
  curPageActive.value = true;
  cleanAfterimage();

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
});

onMounted(async () => {
  store.screenStreamClientId = genClientId();
  await loadLocalStore();
  store.checkUpdate = checkUpdate;
  if (store.checkUpdateAtStart) checkUpdate();
  store.checkAdb = checkAdb;
  setTimeout(() => {
    checkAdb();
    // listen to window resize event
    const maskElement = document.getElementById("maskElement") as HTMLElement;
    const appWindow = getCurrent();
    appWindow.onResized(() => {
      store.maskSizeH = maskElement.clientHeight;
      store.maskSizeW = maskElement.clientWidth;
    });
  }, 500);
});

let checkAdbMessage: MessageReactive | null = null;
async function checkAdb() {
  try {
    if (checkAdbMessage) {
      checkAdbMessage.destroy();
      checkAdbMessage = null;
    }
    await checkAdbAvailable();
  } catch (e) {
    checkAdbMessage = message.error(t("pages.Mask.checkAdb", [e]), {
      duration: 0,
    });
  }
}

function genClientId() {
  let result = "";
  const characters =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  const charactersLength = characters.length;
  for (let i = 0; i < 16; i++) {
    result += characters.charAt(Math.floor(Math.random() * charactersLength));
  }
  return result;
}

async function loadLocalStore() {
  const localStore = new Store("store.bin");
  await loadLocalStorage(localStore, store, t);
}

async function cleanAfterimage() {
  const appWindow = getCurrent();
  const oldSize = await appWindow.outerSize();
  const newSize = new PhysicalSize(oldSize.width, oldSize.height + 1);
  await appWindow.setSize(newSize);
  await appWindow.setSize(oldSize);
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
    <ScreenStream
      :cid="store.screenStreamClientId"
      v-if="curPageActive && store.controledDevice && store.screenStream.enable"
    />
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
          {{ button.type === "Fire" ? "Fire" : button.key }}
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

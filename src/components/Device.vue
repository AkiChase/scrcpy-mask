<script setup lang="ts">
import {
  Ref,
  computed,
  h,
  nextTick,
  onActivated,
  onMounted,
  onUnmounted,
  ref,
} from "vue";
import {
  Device,
  adbDevices,
  pushServerFile,
  forwardServerPort,
  startScrcpyServer,
  getDeviceScreenSize,
  adbConnect,
  getCurClientInfo,
} from "../invoke";
import {
  NH4,
  NInput,
  NInputNumber,
  NButton,
  NDataTable,
  NDropdown,
  NEmpty,
  NTooltip,
  NFlex,
  NIcon,
  NSpin,
  NScrollbar,
  DataTableColumns,
  DropdownOption,
  useDialog,
  useMessage,
  NInputGroup,
} from "naive-ui";
import { CloseCircle, InformationCircle, Refresh } from "@vicons/ionicons5";
import { UnlistenFn, listen } from "@tauri-apps/api/event";
import { shutdown } from "../frontcommand/scrcpyMaskCmd";
import { useGlobalStore } from "../store/global";
import { useI18n } from "vue-i18n";
import { closeExternalControl, connectExternalControl } from "../websocket";
import { LogicalSize, getCurrent } from "@tauri-apps/api/window";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

const { t } = useI18n();
const dialog = useDialog();
const store = useGlobalStore();
const message = useMessage();

const port = ref(27183);
const wireless_address = ref("");
const ws_address = ref("");

//#region listener
let deviceWaitForMetadataTask: ((deviceName: string) => void) | null = null;
let deviceWaitForScreenSizeTask: ((w: number, h: number) => void) | null = null;

let unlisten: UnlistenFn | undefined;
let lastClipboard = "";

onMounted(async () => {
  unlisten = await listen("device-reply", (event) => {
    try {
      let payload = JSON.parse(event.payload as string);
      switch (payload.type) {
        case "MetaData":
          deviceWaitForMetadataTask?.(payload.deviceName);
          break;
        case "ClipboardChanged":
          if (payload.clipboard === lastClipboard) break;
          lastClipboard = payload.clipboard;
          writeText(payload.clipboard);
          console.log(payload);
          break;
        case "ClipboardSetAck":
          break;
        case "DeviceRotation":
          if (deviceWaitForScreenSizeTask) {
            deviceWaitForScreenSizeTask(payload.width, payload.height);
          } else {
            store.screenSizeW = payload.width;
            store.screenSizeH = payload.height;
            message.info(t("pages.Device.rotation", [payload.rotation * 90]));
          }
          if (store.rotation.enable) {
            let maskW: number;
            let maskH: number;
            if (payload.width >= payload.height) {
              maskW = Math.round(store.rotation.horizontalLength);
              maskH = Math.round(maskW * (payload.height / payload.width));
            } else {
              maskH = Math.round(store.rotation.verticalLength);
              maskW = Math.round(maskH * (payload.width / payload.height));
            }
            getCurrent().setSize(new LogicalSize(maskW + 70, maskH + 30));
          }
          break;
        default:
          console.log("Unknown reply", payload);
          break;
      }
    } catch (e) {
      console.error(e);
    }
  });
});

onActivated(async () => {
  let curClientInfo = await getCurClientInfo();
  if (store.controledDevice) {
    // update controledDevice if client not exists
    if (!curClientInfo) {
      await shutdown();
      store.controledDevice = null;
      message.warning(t("pages.Device.alreadyDisconnected"));
    }
  } else {
    // restore controledDevice if client exists
    if (curClientInfo) {
      message.warning(t("pages.Device.alreadyControled"));
      store.screenSizeW = curClientInfo.width;
      store.screenSizeH = curClientInfo.height;
      store.controledDevice = {
        scid: curClientInfo.scid,
        deviceName: curClientInfo.device_name,
        deviceID: curClientInfo.device_id,
      };
    }
  }

  await refreshDevices();
});

onUnmounted(() => {
  if (unlisten !== undefined) unlisten();
});
//#endregion

//#region table
const devices: Ref<Device[]> = ref([]);
const availableDevice = computed(() => {
  return devices.value.filter((d) => {
    return store.controledDevice?.deviceID !== d.id;
  });
});
const tableCols: DataTableColumns = [
  {
    title: "ID",
    key: "id",
  },
  {
    title: t("pages.Device.status"),
    key: "status",
  },
];

// record last operated row index
let rowIndex = -1;

// table row contextmenu and click event handler
const tableRowProps = (_: any, index: number) => {
  return {
    onContextmenu: (e: MouseEvent) => {
      e.preventDefault();
      showMenu.value = false;
      rowIndex = index;
      nextTick().then(() => {
        showMenu.value = true;
        menuX.value = e.clientX;
        menuY.value = e.clientY;
      });
    },
    onclick: (e: MouseEvent) => {
      e.preventDefault();
      showMenu.value = false;
      rowIndex = index;
      nextTick().then(() => {
        showMenu.value = true;
        menuX.value = e.clientX;
        menuY.value = e.clientY;
      });
    },
  };
};
//#endregion

//#region controled device

async function shutdownSC() {
  dialog.warning({
    title: t("pages.Device.shutdown.title"),
    content: t("pages.Device.shutdown.content"),
    positiveText: t("pages.Device.shutdown.positiveText"),
    negativeText: t("pages.Device.shutdown.negativeText"),
    onPositiveClick: async () => {
      await shutdown();
      store.controledDevice = null;
    },
  });
}
//#endregion

//#region menu
const menuX = ref(0);
const menuY = ref(0);
const showMenu = ref(false);
const menuOptions: DropdownOption[] = [
  {
    label: () => h("span", t("pages.Device.menu.control")),
    key: "control",
  },
  {
    label: () => h("span", t("pages.Device.menu.screen")),
    key: "screen",
  },
];

function onMenuClickoutside() {
  showMenu.value = false;
}

async function deviceControl() {
  let curClientInfo = await getCurClientInfo();
  if (curClientInfo) {
    message.error(t("pages.Device.alreadyControled"));
    store.controledDevice = {
      scid: curClientInfo.scid,
      deviceName: curClientInfo.device_name,
      deviceID: curClientInfo.device_id,
    };
    store.hideLoading();
    return;
  }

  if (!port.value) {
    port.value = 27183;
  }

  message.info(t("pages.Device.deviceControl.controlInfo"));

  const device = devices.value[rowIndex];

  let scid = (
    "00000000" + Math.floor(Math.random() * 100000).toString(16)
  ).slice(-8);

  await pushServerFile(device.id);
  await forwardServerPort(device.id, scid, port.value);
  await startScrcpyServer(device.id, scid, `127.0.0.1:${port.value}`);

  // connection timeout check
  let id = setTimeout(async () => {
    if (deviceWaitForMetadataTask) {
      await shutdown();
      store.controledDevice = null;
      store.hideLoading();
      message.error(t("pages.Device.deviceControl.connectTimeout"));
    }
  }, 6000);

  // add cb for metadata
  deviceWaitForMetadataTask = (deviceName: string) => {
    store.controledDevice = {
      scid,
      deviceName,
      deviceID: device.id,
    };
    deviceWaitForMetadataTask = null;
    if (!deviceWaitForScreenSizeTask) {
      nextTick(() => {
        clearTimeout(id);
        store.hideLoading();
      });
    }
  };

  // add cb for screen size
  deviceWaitForScreenSizeTask = (w: number, h: number) => {
    store.screenSizeW = w;
    store.screenSizeH = h;
    deviceWaitForScreenSizeTask = null;
    if (!deviceWaitForMetadataTask) {
      nextTick(() => {
        clearTimeout(id);
        store.hideLoading();
      });
    }
  };
}

async function deviceGetScreenSize() {
  let id = devices.value[rowIndex].id;
  const size = await getDeviceScreenSize(id);
  store.hideLoading();
  message.success(
    t("pages.Device.deviceGetScreenSize") + `${size[0]} x ${size[1]}`,
    { keepAliveOnHover: true }
  );
}

async function onMenuSelect(key: string) {
  showMenu.value = false;
  store.showLoading();
  switch (key) {
    case "control":
      await deviceControl();
      break;
    case "screen":
      await deviceGetScreenSize();
      break;
  }
}
//#endregion

async function refreshDevices() {
  store.showLoading();
  try {
    devices.value = await adbDevices();
  } catch (e) {
    message.error(t("pages.Device.adbDeviceError"));
    console.error(e);
  }
  store.hideLoading();
}

async function connectDevice() {
  if (!wireless_address.value) {
    message.error(t("pages.Device.inputWirelessAddress"));
    return;
  }

  store.showLoading();
  try {
    message.info(await adbConnect(wireless_address.value));
    await refreshDevices();
  } catch (e) {
    message.error("t('pages.Device.adbConnectError')");
    console.error(e);
  }
}

function connectWS() {
  if (!ws_address.value) {
    message.error(t("pages.Device.inputWsAddress"));
    return;
  }

  store.showLoading();
  connectExternalControl(ws_address.value, message, store, t);
}

function closeWS() {
  closeExternalControl();
}
</script>

<template>
  <NScrollbar>
    <div class="device">
      <NSpin :show="store.showLoadingRef">
        <NH4 prefix="bar">{{ $t("pages.Device.localPort") }}</NH4>
        <NInputNumber
          v-model:value="port"
          :show-button="false"
          :min="16384"
          :max="49151"
          :placeholder="$t('pages.Device.localPortPlaceholder')"
          style="max-width: 300px"
        />
        <NH4 prefix="bar">{{ $t("pages.Device.wireless") }}</NH4>
        <NInputGroup style="max-width: 300px">
          <NInput
            v-model:value="wireless_address"
            clearable
            :placeholder="$t('pages.Device.wirelessPlaceholder')"
          />
          <NButton type="primary" @click="connectDevice">{{
            $t("pages.Device.connect")
          }}</NButton>
        </NInputGroup>
        <NH4 prefix="bar">{{ $t("pages.Device.externalControl") }}</NH4>
        <NInputGroup style="max-width: 300px">
          <NInput
            v-model:value="ws_address"
            clearable
            :placeholder="$t('pages.Device.wsAddress')"
            :disabled="store.externalControlled"
          />
          <NButton
            v-if="store.externalControlled"
            type="error"
            @click="closeWS"
            >{{ $t("pages.Device.wsClose") }}</NButton
          >
          <NButton v-else type="primary" @click="connectWS">{{
            $t("pages.Device.wsConnect")
          }}</NButton>
        </NInputGroup>
        <NH4 prefix="bar">{{ $t("pages.Device.controledDevice") }}</NH4>
        <div class="controled-device-list">
          <NEmpty
            size="small"
            :description="$t('pages.Device.noControledDevice')"
            v-if="!store.controledDevice"
          />
          <div class="controled-device" v-if="store.controledDevice">
            <div>
              {{ store.controledDevice.deviceName }} ({{
                store.controledDevice.deviceID
              }})
            </div>
            <div class="device-op">
              <NTooltip trigger="hover">
                <template #trigger>
                  <NButton quaternary circle type="info">
                    <template #icon>
                      <NIcon><InformationCircle /></NIcon>
                    </template>
                  </NButton>
                </template>
                scid: {{ store.controledDevice.scid }}
              </NTooltip>
              <NButton quaternary circle type="error" @click="shutdownSC()">
                <template #icon>
                  <NIcon><CloseCircle /></NIcon>
                </template>
              </NButton>
            </div>
          </div>
        </div>
        <NFlex justify="space-between" align="center">
          <NH4 style="margin: 20px 0" prefix="bar">{{
            $t("pages.Device.availableDevice")
          }}</NH4>
          <NButton
            tertiary
            circle
            type="primary"
            @click="refreshDevices"
            style="margin-right: 20px"
          >
            <template #icon>
              <NIcon><Refresh /></NIcon>
            </template>
          </NButton>
        </NFlex>
        <NDataTable
          max-height="120"
          :columns="tableCols"
          :data="availableDevice"
          :row-props="tableRowProps"
          :pagination="false"
          :bordered="false"
        />
        <NDropdown
          placement="bottom-start"
          trigger="manual"
          :x="menuX"
          :y="menuY"
          :options="menuOptions"
          :show="showMenu"
          :on-clickoutside="onMenuClickoutside"
          @select="onMenuSelect"
        />
      </NSpin>
    </div>
  </NScrollbar>
</template>

<style scoped lang="scss">
.device {
  color: var(--light-color);
  background-color: var(--bg-color);
  padding: 0 20px;
  height: 100%;
}

.controled-device-list {
  .controled-device {
    padding: 10px 20px;
    background-color: var(--content-bg-color);
    border: 2px solid var(--content-hl-color);
    border-bottom: none;
    border-radius: 5px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    transition: background-color 0.3s;
    &:last-child {
      border-bottom: 2px solid var(--content-hl-color);
    }
    &:hover {
      background-color: var(--content-hl-color);
    }

    .device-op {
      display: flex;
      align-items: center;
      gap: 10px;
    }
  }
}
</style>

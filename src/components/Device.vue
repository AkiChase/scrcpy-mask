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
} from "../invoke";
import {
  NH4,
  NP,
  NInputNumber,
  NButton,
  NDataTable,
  NDropdown,
  NEmpty,
  NTooltip,
  NFlex,
  NFormItem,
  NIcon,
  NSpin,
  NScrollbar,
  DataTableColumns,
  DropdownOption,
  useDialog,
  useMessage,
} from "naive-ui";
import { CloseCircle, InformationCircle } from "@vicons/ionicons5";
import { Refresh } from "@vicons/ionicons5";
import { UnlistenFn, listen } from "@tauri-apps/api/event";
import { Store } from "@tauri-apps/plugin-store";
import { shutdown } from "../frontcommand/scrcpyMaskCmd";
import { useGlobalStore } from "../store/global";

const dialog = useDialog();
const store = useGlobalStore();
const message = useMessage();

const port = ref(27183);

const localStore = new Store("store.bin");

//#region listener
let deviceWaitForMetadataTask: ((deviceName: string) => void) | null = null;

let unlisten: UnlistenFn | undefined;
onMounted(async () => {
  unlisten = await listen("device-reply", (event) => {
    try {
      let payload = JSON.parse(event.payload as string);
      switch (payload.type) {
        case "MetaData":
          deviceWaitForMetadataTask?.(payload.deviceName);
          break;
        case "ClipboardChanged":
          console.log("剪切板变动", payload.clipboard);
          break;
        case "ClipboardSetAck":
          console.log("剪切板设置成功", payload.sequence);
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
    return store.controledDevice?.device.id !== d.id;
  });
});
const tableCols: DataTableColumns = [
  {
    title: "ID",
    key: "id",
  },
  {
    title: "Status",
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
    title: "Warning",
    content: "确定关闭Scrcpy控制服务?",
    positiveText: "确定",
    negativeText: "取消",
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
    label: () => h("span", "控制此设备"),
    key: "control",
  },
];

function onMenuClickoutside() {
  showMenu.value = false;
}

async function onMenuSelect(key: string) {
  showMenu.value = false;
  store.showLoading();
  switch (key) {
    case "control":
      if (!port.value) {
        port.value = 27183;
      }

      if (!(store.screenSizeW > 0) || !(store.screenSizeH > 0)) {
        message.error("请正确输入当前控制设备的屏幕尺寸");
        store.screenSizeW = 0;
        store.screenSizeH = 0;
        store.hideLoading();
        return;
      }

      if (store.controledDevice) {
        message.error("请先关闭当前控制设备");
        store.hideLoading();
        return;
      }

      localStore.set("screenSize", {
        sizeW: store.screenSizeW,
        sizeH: store.screenSizeH,
      });
      message.info("屏幕尺寸已保存，正在启动控制服务，请保持设备亮屏");

      let device = devices.value[rowIndex];

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
          message.error("设备连接超时");
        }
      }, 6000);

      // add cb for metadata
      deviceWaitForMetadataTask = (deviceName: string) => {
        store.controledDevice = {
          scid,
          deviceName,
          device,
        };
        nextTick(() => {
          deviceWaitForMetadataTask = null;
          clearTimeout(id);
          store.hideLoading();
        });
      };
      break;
  }
}
//#endregion

async function refreshDevices() {
  store.showLoading();
  devices.value = await adbDevices();
  store.hideLoading();
}
</script>

<template>
  <NScrollbar>
    <div class="device">
      <NSpin :show="store.showLoadingRef">
        <NH4 prefix="bar">本地端口</NH4>
        <NInputNumber
          v-model:value="port"
          :show-button="false"
          :min="16384"
          :max="49151"
          style="max-width: 300px"
        />
        <NH4 prefix="bar">设备尺寸</NH4>
        <NFlex justify="left" align="center">
          <NFormItem label="宽度">
            <NInputNumber
              v-model:value="store.screenSizeW"
              placeholder="屏幕宽度"
              :min="0"
              :disabled="store.controledDevice !== null"
            />
          </NFormItem>
          <NFormItem label="高度">
            <NInputNumber
              v-model:value="store.screenSizeH"
              placeholder="屏幕高度"
              :min="0"
              :disabled="store.controledDevice !== null"
            />
          </NFormItem>
        </NFlex>
        <NP
          >提示：请正确输入当前控制设备的屏幕尺寸，这是成功发送触摸事件的必要参数</NP
        >
        <NH4 prefix="bar">受控设备</NH4>
        <div class="controled-device-list">
          <NEmpty
            size="small"
            description="No Controled Device"
            v-if="!store.controledDevice"
          />
          <div class="controled-device" v-if="store.controledDevice">
            <div>
              {{ store.controledDevice.deviceName }} ({{
                store.controledDevice.device.id
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
                scid: {{ store.controledDevice.scid }} <br />status:
                {{ store.controledDevice.device.status }} <br />screen:
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
          <NH4 style="margin: 20px 0" prefix="bar">可用设备</NH4>
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

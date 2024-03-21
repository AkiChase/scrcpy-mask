<script setup lang="ts">
import { Ref, computed, h, nextTick, onMounted, onUnmounted, ref } from "vue";
import {
  Device,
  adbDevices,
  openSocketServer,
  pushServerFile,
  reverseServerPort,
  startScrcpyServer,
} from "../invoke";
import {
  NH4,
  NInputGroup,
  NInputNumber,
  NButton,
  NDataTable,
  NDropdown,
  NEmpty,
  NTooltip,
  NFlex,
  NIcon,
  NSpin,
  DataTableColumns,
  DropdownOption,
  useMessage,
  useDialog,
} from "naive-ui";
import { CloseCircle, InformationCircle } from "@vicons/ionicons5";
import { Refresh } from "@vicons/ionicons5";
import { UnlistenFn, listen } from "@tauri-apps/api/event";
import { shutdown } from "../frontcommand/scrcpyMaskCmd";
import { useGlobalStore } from "../store/global";

const message = useMessage();
const dialog = useDialog();
const store = useGlobalStore();

const port = ref(27183);

//#region listener
const deviceWaitForMetadataTask: ((
  smid: string,
  deviceName: string
) => void)[] = [];

let unlisten: UnlistenFn | undefined;
onMounted(async () => {
  await refreshDevices();

  unlisten = await listen("device-reply", (event) => {
    try {
      let payload = JSON.parse(event.payload as string);
      switch (payload.msg) {
        case "MetaData":
          let task = deviceWaitForMetadataTask.shift();
          task?.(payload.smid, payload.deviceName);
          break;
        case "ClipboardChanged":
          console.log("剪切板变动", payload.clipboard, payload.smid);
          break;
        case "ClipboardSetAck":
          console.log("剪切板设置成功", payload.sequence, payload.smid);
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

onUnmounted(() => {
  if (unlisten !== undefined) unlisten();
});
//#endregion

//#region table
const devices: Ref<Device[]> = ref([]);
const availableDevice = computed(() => {
  return devices.value.filter((d) => {
    return !controledDevices.value.some((cd) => cd.device.id === d.id);
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
interface ControledDevice {
  scid: string;
  smid: string;
  deviceName: string;
  device: Device;
}

const controledDevices: Ref<ControledDevice[]> = ref([]);

async function shutdownSC(smid: string) {
  dialog.warning({
    title: "Warning",
    content: "确定关闭此设备的Scrcpy控制服务?",
    positiveText: "确定",
    negativeText: "取消",
    onPositiveClick: async () => {
      await shutdown({
        smid,
      });
      controledDevices.value.splice(
        controledDevices.value.findIndex((cd) => cd.smid === smid),
        1
      );
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
      if (!store.isServerRunning) {
        message.error("请先启动服务端");
        store.hideLoading();
        return;
      }
      let device = devices.value[rowIndex];
      let scid = (
        "00000000" + Math.floor(Math.random() * 100000).toString(16)
      ).slice(-8);

      await pushServerFile(device.id);
      await reverseServerPort(device.id, scid, port.value);
      await startScrcpyServer(device.id, scid);

      // wait for metadata
      deviceWaitForMetadataTask.push((smid: string, deviceName: string) => {
        controledDevices.value.push({
          scid,
          smid,
          deviceName,
          device,
        });
        nextTick(() => {
          store.hideLoading();
        });
      });
      break;
  }
}
//#endregion

async function startServer() {
  store.showLoading();
  store.isServerRunning = true;
  openSocketServer(port.value)
    .then(() => {
      message.success("服务端已启动: 127.0.0.1:" + port.value);
      store.hideLoading();
    })
    .catch((e) => {
      message.warning("服务端启动失败: " + e);
      store.hideLoading();
    });
}

async function refreshDevices() {
  store.showLoading();
  devices.value = await adbDevices();
  store.hideLoading();
}
</script>

<template>
  <div class="device">
    <NSpin :show="store.showLoadingRef">
      <NH4 prefix="bar">本地端口</NH4>
      <NInputGroup>
        <NInputNumber
          v-model:value="port"
          :show-button="false"
          :disabled="store.isServerRunning"
        />
        <NButton
          @click="startServer"
          :type="store.isServerRunning ? 'error' : 'primary'"
          ghost
          :disabled="store.isServerRunning"
          >{{ store.isServerRunning ? "服务端运行中" : "启动服务端" }}</NButton
        >
      </NInputGroup>
      <NH4 prefix="bar">受控设备</NH4>
      <div class="controled-device-list">
        <NEmpty
          size="small"
          description="No Controled Device"
          v-if="controledDevices.length === 0"
        />
        <div class="controled-device" v-for="cDevice in controledDevices">
          <div>{{ cDevice.deviceName }} ({{ cDevice.device.id }})</div>
          <div class="device-op">
            <NTooltip trigger="hover">
              <template #trigger>
                <NButton quaternary circle type="info">
                  <template #icon>
                    <NIcon><InformationCircle /></NIcon>
                  </template>
                </NButton>
              </template>
              smid: {{ cDevice.smid }} <br />scid: {{ cDevice.scid }}
              <br />status: {{ cDevice.device.status }}
            </NTooltip>

            <NButton
              quaternary
              circle
              type="error"
              @click="shutdownSC(cDevice.smid)"
            >
              <template #icon>
                <NIcon><CloseCircle /></NIcon>
              </template>
            </NButton>
          </div>
        </div>
      </div>
      <NFlex justify="space-between" align="center">
        <NH4 prefix="bar">可用设备</NH4>
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
</template>

<style scoped lang="scss">
.device {
  color: var(--light-color);
  background-color: var(--bg-color);
  padding: 0 25px;
}

.n-h4 {
  margin-top: 20px;
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

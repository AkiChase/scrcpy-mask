<script setup lang="ts">
import { Ref, h, nextTick, onMounted, ref } from "vue";
import {
  PhysicalPosition,
  PhysicalSize,
  appWindow,
} from "@tauri-apps/api/window";
import {
  NH4,
  NDataTable,
  NDropdown,
  NButton,
  NIcon,
  NFlex,
  NTooltip,
  useMessage,
  DataTableColumns,
  DropdownOption,
} from "naive-ui";
import { Refresh } from "@vicons/ionicons5";

import { getWindows, getWindowControls, WindowInfo } from "../../invoke";

onMounted(() => {
  getWindowsOrControls();
});

const message = useMessage();

//#region table

interface TableData {
  title: string;
  pos: string;
  size: string;
}

const isControlWindow = ref(false);

const tableCols: DataTableColumns = [
  {
    title: "Title",
    key: "title",
    ellipsis: {
      tooltip: true,
    },
  },
  {
    title: "Position",
    key: "pos",
    width: "150px",
  },
  {
    title: "Size",
    key: "size",
    width: "150px",
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

let windowList: WindowInfo[] = [];
const tableData: Ref<TableData[]> = ref([]);

// get and render window/control info list
async function getWindowsOrControls(hwnd?: number): Promise<void> {
  if (hwnd !== undefined) {
    let tmp: WindowInfo[] = await getWindowControls(hwnd);

    if (tmp.length === 0) {
      message.warning("此窗口不存在控件");
      return;
    }

    windowList = tmp;
    isControlWindow.value = true;
  } else {
    windowList = await getWindows();
    isControlWindow.value = false;
  }
  const newTableData = [];
  for (const winInfo of windowList) {
    newTableData.push({
      title: winInfo.title,
      pos: `${winInfo.x} , ${winInfo.y}`,
      size: `${winInfo.width} x ${winInfo.height}`,
    });
  }
  tableData.value = newTableData;
}

//#endregion

//#region menu
const menuX = ref(0);
const menuY = ref(0);
const showMenu = ref(false);
const menuOptions: DropdownOption[] = [
  {
    label: () => h("span", isControlWindow.value ? "选择此控件" : "选择此窗口"),
    key: "choose",
  },
  {
    label: () =>
      h("span", isControlWindow.value ? "返回窗口列表" : "查看控件列表"),
    key: "control",
  },
];

function onMenuClickoutside() {
  showMenu.value = false;
}

function onMenuSelect(key: string) {
  showMenu.value = false;
  if (key === "choose") {
    selectWindow(windowList[rowIndex]).then(() => {
      message.success("调整完成");
    });
  } else {
    if (isControlWindow.value) {
      // return to window list
      getWindowsOrControls();
    } else {
      if (rowIndex !== -1) {
        getWindowsOrControls(windowList[rowIndex].hwnd);
      } else {
        message.warning("请点击一个窗口或控件");
      }
    }
  }
}

// move and resize window to the selected window (control) area
async function selectWindow(ctrlInfo: WindowInfo) {
  // header size and sidebar size
  const mt = 30;
  const ml = 70;

  const factor = await appWindow.scaleFactor();

  const pos = new PhysicalPosition(ctrlInfo.x, ctrlInfo.y).toLogical(factor);
  pos.y -= mt;
  pos.x -= ml;

  if (pos.x <= 0 || pos.y <= 0) {
    message.warning("蒙版区域坐标过小，可能导致其他部分不可见");
  }

  const size = new PhysicalSize(ctrlInfo.width, ctrlInfo.height).toLogical(
    factor
  );
  size.width += ml;
  size.height += mt;

  await appWindow.setPosition(pos);
  await appWindow.setSize(size);
}
//#endregion

function refreshWindowList() {
  getWindowsOrControls();
  isControlWindow.value = false;
}
</script>

<template>
  <div class="window-list">
    <NFlex justify="space-between" align="center">
      <NH4 prefix="bar">{{ isControlWindow ? "控件列表" : "窗口列表" }}</NH4>
      <NButton
        tertiary
        circle
        type="primary"
        @click="refreshWindowList"
        style="margin-right: 20px"
      >
        <template #icon>
          <NIcon><Refresh /></NIcon>
        </template>
      </NButton>
    </NFlex>
    <NDataTable
      max-height="250"
      :columns="tableCols"
      :data="tableData"
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
    <p>提示：选择列表中的窗口/控件，即可将蒙版覆盖到对应窗口/控件所在区域</p>
    <p style="padding-left: 3em">
      若蒙版位置不恰当导致其他部分不可见，请使用
      <NTooltip trigger="hover">
        <template #trigger>
          <NButton text type="info">系统快捷键</NButton>
        </template>
        <p>Window: Win + ↑/↓</p>
        <p>MacOS: ⌃ + ⌘ + F</p>
      </NTooltip>
      将窗口最大化/恢复
    </p>
  </div>
</template>

<style scoped lang="scss"></style>

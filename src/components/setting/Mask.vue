<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import WindowList from "./WindowList.vue";
import {
  NH4,
  NForm,
  NGrid,
  NFormItemGi,
  NInputNumber,
  FormRules,
  NButton,
  NFlex,
  NIcon,
  FormInst,
  useMessage,
} from "naive-ui";
import {
  PhysicalPosition,
  PhysicalSize,
  appWindow,
} from "@tauri-apps/api/window";
import { SettingsOutline } from "@vicons/ionicons5";
import { UnlistenFn } from "@tauri-apps/api/event";

let unlistenResize: UnlistenFn = () => {};
let unlistenMove: UnlistenFn = () => {};

async function refreshAreaModel(size?: PhysicalSize, pos?: PhysicalPosition) {
  const factor = await appWindow.scaleFactor();
  // header size and sidebar size
  const mt = 30 * factor;
  const ml = 70 * factor;

  if (pos !== undefined) {
    areaModel.value.posX = Math.floor(pos.x + ml);
    areaModel.value.posY = Math.floor(pos.y + mt);
  }
  if (size !== undefined) {
    areaModel.value.sizeW = Math.floor(size.width - ml);
    areaModel.value.sizeH = Math.floor(size.height - mt);
  }
}

const message = useMessage();

const formRef = ref<FormInst | null>(null);

const areaModel = ref({
  posX: 0,
  posY: 0,
  sizeW: 0,
  sizeH: 0,
});

const areaFormRules: FormRules = {
  posX: {
    type: "number",
    required: true,
    trigger: ["blur", "input"],
    message: "请输入左上角X坐标",
  },
  posY: {
    type: "number",
    required: true,
    trigger: ["blur", "input"],
    message: "请输入左上角Y坐标",
  },
  sizeW: {
    type: "number",
    required: true,
    trigger: ["blur", "input"],
    message: "请输入蒙版宽度",
  },
  sizeH: {
    type: "number",
    required: true,
    trigger: ["blur", "input"],
    message: "请输入蒙版高度",
  },
};

function handleAdjustClick(e: MouseEvent) {
  e.preventDefault();
  formRef.value?.validate((errors) => {
    if (!errors) {
      adjustMaskArea().then(() => {
        message.success("调整完成");
      });
    } else {
      message.error("请正确输入蒙版的坐标和尺寸");
    }
  });
}

// move and resize window to the selected window (control) area
async function adjustMaskArea() {
  // header size and sidebar size
  const mt = 30;
  const ml = 70;

  const factor = await appWindow.scaleFactor();

  const pos = new PhysicalPosition(
    areaModel.value.posX,
    areaModel.value.posY
  ).toLogical(factor);
  pos.y -= mt;
  pos.x -= ml;

  if (pos.x <= 0 || pos.y <= 0) {
    message.warning("蒙版区域坐标过小，可能导致其他部分不可见");
  }

  const size = new PhysicalSize(
    areaModel.value.sizeW,
    areaModel.value.sizeH
  ).toLogical(factor);
  size.width += ml;
  size.height += mt;

  await appWindow.setPosition(pos);
  await appWindow.setSize(size);
}

onMounted(async () => {
  unlistenResize = await appWindow.onResized(({ payload: size }) => {
    refreshAreaModel(size, undefined);
  });
  unlistenMove = await appWindow.onMoved(({ payload: position }) => {
    refreshAreaModel(undefined, position);
  });
  refreshAreaModel(
    await appWindow.outerSize(),
    await appWindow.outerPosition()
  );
  console.log("监听窗口坐标、尺寸");
});

onUnmounted(() => {
  unlistenResize();
  unlistenMove();
  console.log("移除监听");
});
</script>

<template>
  <div class="setting-page">
    <WindowList />
    <n-flex justify="space-between">
      <n-h4 prefix="bar">手动调整</n-h4>
      <n-button
        tertiary
        circle
        type="primary"
        @click="handleAdjustClick"
        style="margin-right: 20px"
      >
        <template #icon>
          <n-icon><SettingsOutline /></n-icon>
        </template>
      </n-button>
    </n-flex>

    <n-form
      ref="formRef"
      :model="areaModel"
      :rules="areaFormRules"
      label-placement="left"
      label-width="auto"
      require-mark-placement="right-hanging"
    >
      <n-grid :cols="2" :x-gap="24">
        <n-form-item-gi label="X" path="posX">
          <n-input-number
            v-model:value="areaModel.posX"
            placeholder="左上角X坐标"
          />
        </n-form-item-gi>
        <n-form-item-gi label="Y" path="posY">
          <n-input-number
            v-model:value="areaModel.posY"
            placeholder="左上角Y坐标"
          />
        </n-form-item-gi>
        <n-form-item-gi label="W" path="sizeW">
          <n-input-number
            v-model:value="areaModel.sizeW"
            placeholder="蒙版宽度"
          />
        </n-form-item-gi>
        <n-form-item-gi label="H" path="sizeH">
          <n-input-number
            v-model:value="areaModel.sizeH"
            placeholder="蒙版高度"
          />
        </n-form-item-gi>
      </n-grid>
    </n-form>
  </div>
</template>

<style scoped></style>

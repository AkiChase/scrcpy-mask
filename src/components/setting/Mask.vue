<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import {
  NH4,
  NP,
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
  LogicalPosition,
  LogicalSize,
  PhysicalPosition,
  PhysicalSize,
  getCurrent,
} from "@tauri-apps/api/window";
import { Store } from "@tauri-apps/plugin-store";
import { SettingsOutline } from "@vicons/ionicons5";
import { UnlistenFn } from "@tauri-apps/api/event";

let unlistenResize: UnlistenFn = () => {};
let unlistenMove: UnlistenFn = () => {};
let factor = 1;

const localStore = new Store("store.bin");
const message = useMessage();
const formRef = ref<FormInst | null>(null);

// logical pos and size of the mask area
interface MaskArea {
  posX: number;
  posY: number;
  sizeW: number;
  sizeH: number;
}
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

async function refreshAreaModel(size?: PhysicalSize, pos?: PhysicalPosition) {
  const lSize = size?.toLogical(factor);
  const lPos = pos?.toLogical(factor);

  // header size and sidebar size
  const mt = 30;
  const ml = 70;

  // use logical position and size
  if (lSize !== undefined) {
    areaModel.value.sizeW = Math.round(lSize.width) - ml;
    areaModel.value.sizeH = Math.round(lSize.height) - mt;
  }
  if (lPos !== undefined) {
    areaModel.value.posX = Math.round(lPos.x) + ml;
    areaModel.value.posY = Math.round(lPos.y) + mt;
  }
}

function handleAdjustClick(e: MouseEvent) {
  e.preventDefault();
  formRef.value?.validate((errors) => {
    if (!errors) {
      adjustMaskArea().then(() => {
        localStore.set("maskArea", areaModel.value);
        message.success("蒙版区域已保存");
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

  const appWindow = getCurrent();

  const pos = new LogicalPosition(
    areaModel.value.posX - ml,
    areaModel.value.posY - mt
  );

  const size = new LogicalSize(
    areaModel.value.sizeW + ml,
    areaModel.value.sizeH + mt
  );

  await appWindow.setPosition(pos);
  await appWindow.setSize(size);
}

onMounted(async () => {
  const appWindow = getCurrent();
  factor = await appWindow.scaleFactor();

  let maskArea = await localStore.get<MaskArea>("maskArea");
  if (maskArea !== null) {
    areaModel.value = maskArea;
  }

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
});

onUnmounted(() => {
  unlistenResize();
  unlistenMove();
});
</script>

<template>
  <div class="setting-page">
    <NFlex justify="space-between" align="center">
      <NH4 prefix="bar">蒙版调整</NH4>
      <NButton
        tertiary
        circle
        type="primary"
        @click="handleAdjustClick"
        style="margin-right: 20px"
      >
        <template #icon>
          <NIcon><SettingsOutline /></NIcon>
        </template>
      </NButton>
    </NFlex>

    <NForm
      ref="formRef"
      :model="areaModel"
      :rules="areaFormRules"
      label-placement="left"
      label-width="auto"
      require-mark-placement="right-hanging"
    >
      <NGrid :cols="2" :x-gap="24">
        <NFormItemGi label="X" path="posX">
          <NInputNumber
            v-model:value="areaModel.posX"
            placeholder="左上角X坐标"
          />
        </NFormItemGi>
        <NFormItemGi label="Y" path="posY">
          <NInputNumber
            v-model:value="areaModel.posY"
            placeholder="左上角Y坐标"
          />
        </NFormItemGi>
        <NFormItemGi label="W" path="sizeW">
          <NInputNumber
            v-model:value="areaModel.sizeW"
            placeholder="蒙版宽度"
          />
        </NFormItemGi>
        <NFormItemGi label="H" path="sizeH">
          <NInputNumber
            v-model:value="areaModel.sizeH"
            placeholder="蒙版高度"
          />
        </NFormItemGi>
      </NGrid>
      <NP>提示：蒙版尺寸与设备尺寸将用于坐标转换，请保证尺寸的准确性</NP>
    </NForm>
  </div>
</template>

<style scoped></style>

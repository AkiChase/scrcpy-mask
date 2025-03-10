<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
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
  NSlider,
  NFormItem,
  NCheckbox,
  NInput,
} from "naive-ui";
import {
  LogicalPosition,
  LogicalSize,
  PhysicalPosition,
  PhysicalSize,
  getCurrentWindow,
} from "@tauri-apps/api/window";
import { SettingsOutline } from "@vicons/ionicons5";
import { UnlistenFn } from "@tauri-apps/api/event";
import { useGlobalStore } from "../../store/global";
import { useI18n } from "vue-i18n";
import { LocalStore } from "../../store/localStore";
import { NonReactiveStore } from "../../store/noneReactiveStore";

let unlistenResize: UnlistenFn = () => {};
let unlistenMove: UnlistenFn = () => {};
let factor = 1;

const { t } = useI18n();
const store = useGlobalStore();
const message = useMessage();
const formRef = ref<FormInst | null>(null);

// logical pos and size of the mask area
const curMaskArea = ref({
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
    message: () => t("pages.Setting.Mask.areaFormMissing.x"),
  },
  posY: {
    type: "number",
    required: true,
    trigger: ["blur", "input"],
    message: () => t("pages.Setting.Mask.areaFormMissing.y"),
  },
  sizeW: {
    type: "number",
    required: true,
    trigger: ["blur", "input"],
    message: () => t("pages.Setting.Mask.areaFormMissing.w"),
  },
  sizeH: {
    type: "number",
    required: true,
    trigger: ["blur", "input"],
    message: () => t("pages.Setting.Mask.areaFormMissing.h"),
  },
};

async function refreshCurMaskArea(size?: PhysicalSize, pos?: PhysicalPosition) {
  const lSize = size?.toLogical(factor);
  const lPos = pos?.toLogical(factor);

  // header size and sidebar size
  const mt = 30;
  const ml = 70;

  // use logical position and size
  if (lSize !== undefined) {
    curMaskArea.value.sizeW = Math.round(lSize.width) - ml;
    curMaskArea.value.sizeH = Math.round(lSize.height) - mt;
  }
  if (lPos !== undefined) {
    curMaskArea.value.posX = Math.round(lPos.x) + ml;
    curMaskArea.value.posY = Math.round(lPos.y) + mt;
  }
}

function handleAdjustClick(e: MouseEvent) {
  e.preventDefault();
  formRef.value?.validate((errors) => {
    if (!errors) {
      // save the mask area
      adjustMaskArea().then(() => {
        NonReactiveStore.setLocal("maskArea", curMaskArea.value);
        message.success(t("pages.Setting.Mask.areaSaved"));
      });
    } else {
      message.error(t("pages.Setting.Mask.incorrectArea"));
    }
  });
}

async function adjustMaskArea() {
  // header size and sidebar size
  const mt = 30;
  const ml = 70;

  const appWindow = getCurrentWindow();

  const pos = new LogicalPosition(
    curMaskArea.value.posX - ml,
    curMaskArea.value.posY - mt
  );

  const size = new LogicalSize(
    curMaskArea.value.sizeW + ml,
    curMaskArea.value.sizeH + mt
  );

  await appWindow.setPosition(pos);
  await appWindow.setSize(size);
}

onMounted(async () => {
  const appWindow = getCurrentWindow();
  factor = await appWindow.scaleFactor();

  unlistenResize = await appWindow.onResized(({ payload: size }) => {
    refreshCurMaskArea(size, undefined);
  });
  unlistenMove = await appWindow.onMoved(({ payload: position }) => {
    refreshCurMaskArea(undefined, position);
  });
  refreshCurMaskArea(
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
    <NH4 prefix="bar">{{ $t("pages.Setting.Mask.buttonPrompts") }}</NH4>
    <NFormItem
      :label="$t('pages.Setting.Mask.ifButtonPrompts')"
      label-placement="left"
    >
      <NCheckbox
        v-model:checked="store.maskKeyTip.show"
        @update:checked="LocalStore.set('maskKeyTip', store.maskKeyTip)"
      />
    </NFormItem>
    <NFormItem :label="$t('pages.Setting.Mask.opacity')" label-placement="left">
      <NSlider
        v-model:value="store.maskKeyTip.transparency"
        @update:value="LocalStore.set('maskKeyTip', store.maskKeyTip)"
        :min="0"
        :max="1"
        :step="0.01"
        style="max-width: 300px"
      ></NSlider>
    </NFormItem>

    <NForm
      ref="formRef"
      :model="curMaskArea"
      :rules="areaFormRules"
      label-placement="left"
      label-width="auto"
      require-mark-placement="right-hanging"
    >
      <NFlex justify="space-between" align="center">
        <NH4 prefix="bar">{{ $t("pages.Setting.Mask.areaAdjust") }}</NH4>
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
      <NGrid :cols="2" :x-gap="24">
        <NFormItemGi label="X" path="posX">
          <NInputNumber
            v-model:value="curMaskArea.posX"
            :placeholder="$t('pages.Setting.Mask.areaPlaceholder.x')"
          />
        </NFormItemGi>
        <NFormItemGi label="Y" path="posY">
          <NInputNumber
            v-model:value="curMaskArea.posY"
            :placeholder="$t('pages.Setting.Mask.areaFormPlaceholder.y')"
          />
        </NFormItemGi>
        <NFormItemGi label="W" path="sizeW">
          <NInputNumber
            v-model:value="curMaskArea.sizeW"
            :placeholder="$t('pages.Setting.Mask.areaFormPlaceholder.w')"
          />
        </NFormItemGi>
        <NFormItemGi label="H" path="sizeH">
          <NInputNumber
            v-model:value="curMaskArea.sizeH"
            :placeholder="$t('pages.Setting.Mask.areaFormPlaceholder.h')"
          />
        </NFormItemGi>
      </NGrid>
    </NForm>

    <NH4 prefix="bar">{{ $t("pages.Setting.Mask.rotation.title") }}</NH4>
    <NFormItem
      :label="$t('pages.Setting.Mask.rotation.rotateWithDevice')"
      label-placement="left"
    >
      <NCheckbox
        v-model:checked="store.rotation.enable"
        @update:checked="LocalStore.set('rotation', store.rotation)"
      />
    </NFormItem>
    <NGrid :cols="2">
      <NFormItemGi
        :label="$t('pages.Setting.Mask.rotation.verticalLength')"
        label-placement="left"
      >
        <NInputNumber
          v-model:value="store.rotation.verticalLength"
          @update:value="LocalStore.set('rotation', store.rotation)"
          :placeholder="$t('pages.Setting.Mask.rotation.verticalLength')"
        />
      </NFormItemGi>
      <NFormItemGi
        :label="$t('pages.Setting.Mask.rotation.horizontalLength')"
        label-placement="left"
      >
        <NInputNumber
          v-model:value="store.rotation.horizontalLength"
          @update:value="LocalStore.set('rotation', store.rotation)"
          :placeholder="$t('pages.Setting.Mask.rotation.horizontalLength')"
        />
      </NFormItemGi>
    </NGrid>

    <NH4 prefix="bar">ScreenStream</NH4>
    <NFormItem
      :label="$t('pages.Setting.Mask.screenStream.enable')"
      label-placement="left"
    >
      <NCheckbox
        v-model:checked="store.screenStream.enable"
        @update:checked="LocalStore.set('screenStream', store.screenStream)"
      />
    </NFormItem>
    <NFormItem
      :label="$t('pages.Setting.Mask.screenStream.address')"
      label-placement="left"
    >
      <NInput
        v-model:value="store.screenStream.address"
        @update:value="LocalStore.set('screenStream', store.screenStream)"
        clearable
        :placeholder="$t('pages.Setting.Mask.screenStream.addressPlaceholder')"
      />
    </NFormItem>
  </div>
</template>

<style scoped></style>

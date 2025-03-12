<script setup lang="ts">
import { ref, watch } from "vue";
import {
  NH4,
  NForm,
  NGrid,
  NFormItemGi,
  NInputNumber,
  FormRules,
  NFlex,
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
  getCurrentWindow,
} from "@tauri-apps/api/window";
import { Help, SettingsOutline } from "@vicons/ionicons5";
import { useGlobalStore } from "../../store/global";
import { useI18n } from "vue-i18n";
import { LocalStore } from "../../store/localStore";
import { NonReactiveStore } from "../../store/noneReactiveStore";
import ButtonWithTip from "../common/ButtonWithTip.vue";
import { openWebsite } from "../../tools/tools";

const { t } = useI18n();
const store = useGlobalStore();
const message = useMessage();
const formRef = ref<FormInst | null>(null);

const maskAreaFormModel = ref({
  posX: store.curMaskPos.x,
  posY: store.curMaskPos.y,
  sizeW: store.curMaskSize.w,
  sizeH: store.curMaskSize.h,
});

watch(
  () => store.curMaskSize,
  (curMaskSize) => {
    maskAreaFormModel.value.sizeW = curMaskSize.w;
    maskAreaFormModel.value.sizeH = curMaskSize.h;
  },
  { deep: true }
);
watch(
  () => store.curMaskPos,
  (curMaskPos) => {
    maskAreaFormModel.value.posX = curMaskPos.x;
    maskAreaFormModel.value.posY = curMaskPos.y;
  },
  { deep: true }
);

const maskAreaFormRules: FormRules = {
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

function handleAdjustClick(e: MouseEvent) {
  e.preventDefault();
  formRef.value?.validate((errors) => {
    if (!errors) {
      // save the mask area
      adjustWindowMaskArea().then(() => {
        NonReactiveStore.setLocal("maskArea", maskAreaFormModel.value);
        message.success(t("pages.Setting.Mask.areaSaved"));
      });
    } else {
      message.error(t("pages.Setting.Mask.incorrectArea"));
    }
  });
}

async function adjustWindowMaskArea() {
  // header size and sidebar size
  const mt = 30;
  const ml = 70;

  const appWindow = getCurrentWindow();

  const pos = new LogicalPosition(
    maskAreaFormModel.value.posX - ml,
    maskAreaFormModel.value.posY - mt
  );

  const size = new LogicalSize(
    maskAreaFormModel.value.sizeW + ml,
    maskAreaFormModel.value.sizeH + mt
  );

  await appWindow.setPosition(pos);
  await appWindow.setSize(size);
}
</script>

<template>
  <div>
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
      :model="maskAreaFormModel"
      :rules="maskAreaFormRules"
      label-placement="left"
      label-width="auto"
      require-mark-placement="right-hanging"
    >
      <NFlex justify="space-between" align="center">
        <NH4 prefix="bar">{{ $t("pages.Setting.Mask.areaAdjust") }}</NH4>
        <ButtonWithTip
          tertiary
          circle
          type="primary"
          @click="handleAdjustClick"
          :tip="$t('pages.Setting.Mask.btnAreaAdjustTip')"
          :icon="SettingsOutline"
        />
      </NFlex>
      <NGrid :cols="2" :x-gap="24">
        <NFormItemGi label="X" path="posX">
          <NInputNumber
            v-model:value="maskAreaFormModel.posX"
            :placeholder="$t('pages.Setting.Mask.areaPlaceholder.x')"
          />
        </NFormItemGi>
        <NFormItemGi label="Y" path="posY">
          <NInputNumber
            v-model:value="maskAreaFormModel.posY"
            :placeholder="$t('pages.Setting.Mask.areaFormPlaceholder.y')"
          />
        </NFormItemGi>
        <NFormItemGi label="W" path="sizeW">
          <NInputNumber
            v-model:value="maskAreaFormModel.sizeW"
            :placeholder="$t('pages.Setting.Mask.areaFormPlaceholder.w')"
          />
        </NFormItemGi>
        <NFormItemGi label="H" path="sizeH">
          <NInputNumber
            v-model:value="maskAreaFormModel.sizeH"
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
    <NFlex justify="space-between" align="center">
      <NH4 prefix="bar">ScreenStream</NH4>
      <ButtonWithTip
        tertiary
        circle
        type="primary"
        @click="openWebsite('https://github.com/dkrivoruchko/ScreenStream')"
        :tip="$t('pages.Setting.Mask.screenStream.btnHelp')"
        :icon="Help"
      />
    </NFlex>
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

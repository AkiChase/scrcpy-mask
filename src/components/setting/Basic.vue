<script setup lang="ts">
import {
  NH4,
  NButton,
  NInput,
  NSelect,
  NInputGroup,
  useMessage,
  NFlex,
  NCheckbox,
  NTooltip,
} from "naive-ui";
import { onMounted, ref } from "vue";
import { allLanguage } from "../../i18n";
import { useI18n } from "vue-i18n";
import { useGlobalStore } from "../../store/global";
import { LocalStore } from "../../store/localStore";
import { useCheckAdb } from "../../tools/hooks";

const { t } = useI18n();
const store = useGlobalStore();
const message = useMessage();

const languageOptions = Object.entries(allLanguage).map(([key, value]) => {
  return {
    label: value.label,
    value: key,
  };
});

const curAdbPath = ref("");
const checkAdb = useCheckAdb();

onMounted(async () => {
  curAdbPath.value = store.adbPath;
});

async function adjustAdbPath() {
  store.showLoading();
  store.changeAbdPath(curAdbPath.value);
  message.success(t("pages.Setting.Basic.adbPath.setSuccess"));
  await checkAdb();
  store.hideLoading();
}

function changeClipboardSync() {
  LocalStore.set("clipboardSync", store.clipboardSync);
}
</script>

<template>
  <div>
    <NH4 prefix="bar">{{ $t("pages.Setting.Basic.language") }}</NH4>
    <NSelect
      :value="store.language"
      @update:value="store.setLanguage"
      :options="languageOptions"
      style="max-width: 300px; margin: 20px 0"
    />
    <NH4 prefix="bar">{{ $t("pages.Setting.Basic.adbPath.title") }}</NH4>
    <NInputGroup style="max-width: 300px; margin-bottom: 20px">
      <NInput
        v-model:value="curAdbPath"
        clearable
        :placeholder="$t('pages.Setting.Basic.adbPath.placeholder')"
      />
      <NButton type="primary" @click="adjustAdbPath">{{
        $t("pages.Setting.Basic.adbPath.set")
      }}</NButton>
    </NInputGroup>
    <NH4 prefix="bar">{{ $t("pages.Setting.Basic.clipboardSync.title") }}</NH4>
    <NFlex vertical>
      <NCheckbox
        v-model:checked="store.clipboardSync.syncFromDevice"
        @update:checked="changeClipboardSync"
      >
        <NTooltip trigger="hover">
          <template #trigger>{{
            $t("pages.Setting.Basic.clipboardSync.syncFromDevice")
          }}</template>
          {{ $t("pages.Setting.Basic.clipboardSync.syncFromDeviceTip") }}
        </NTooltip>
      </NCheckbox>
      <NCheckbox
        v-model:checked="store.clipboardSync.pasteFromPC"
        @update:checked="changeClipboardSync"
      >
        <NTooltip trigger="hover">
          <template #trigger>{{
            $t("pages.Setting.Basic.clipboardSync.pasteFromPC")
          }}</template>
          {{ $t("pages.Setting.Basic.clipboardSync.pasteFromPCTip") }}
        </NTooltip>
      </NCheckbox>
    </NFlex>
  </div>
</template>

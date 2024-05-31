<script setup lang="ts">
import { Store } from "@tauri-apps/plugin-store";
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
import i18n from "../../i18n";
import { useI18n } from "vue-i18n";
import { setAdbPath } from "../../invoke";
import { useGlobalStore } from "../../store/global";

const { t } = useI18n();
const localStore = new Store("store.bin");
const store = useGlobalStore();
const message = useMessage();

const languageOptions = [
  { label: "简体中文", value: "zh-CN" },
  { label: "English", value: "en-US" },
];

const curLanguage = ref("en-US");

const adbPath = ref("");

onMounted(async () => {
  curLanguage.value = (await localStore.get<string>("language")) ?? "en-US";
  adbPath.value = (await localStore.get<string>("adbPath")) ?? "";
});

function changeLanguage(language: "zh-CN" | "en-US") {
  if (language === curLanguage.value) return;
  curLanguage.value = language;
  localStore.set("language", language);
  i18n.global.locale.value = language;
}

async function adjustAdbPath() {
  store.showLoading();
  await setAdbPath(adbPath.value);
  message.success(t("pages.Setting.Basic.adbPath.setSuccess"));
  await store.checkAdb();
  adbPath.value = (await localStore.get<string>("adbPath")) ?? "";
  store.hideLoading();
}

function changeClipboardSync() {
  localStore.set("clipboardSync", store.clipboardSync);
}
</script>

<template>
  <div class="setting-page">
    <NH4 prefix="bar">{{ $t("pages.Setting.Basic.language") }}</NH4>
    <NSelect
      :value="curLanguage"
      @update:value="changeLanguage"
      :options="languageOptions"
      style="max-width: 300px; margin: 20px 0"
    />
    <NH4 prefix="bar">{{ $t("pages.Setting.Basic.adbPath.title") }}</NH4>
    <NInputGroup style="max-width: 300px; margin-bottom: 20px">
      <NInput
        v-model:value="adbPath"
        clearable
        :placeholder="$t('pages.Setting.Basic.adbPath.placeholder')"
      />
      <NButton type="primary" @click="adjustAdbPath">{{
        $t("pages.Setting.Basic.adbPath.set")
      }}</NButton>
    </NInputGroup>
    <NH4 prefix="bar">剪切板同步</NH4>
    <NFlex vertical>
      <NCheckbox
        v-model:checked="store.clipboardSync.syncFromDevice"
        @update:checked="changeClipboardSync"
      >
        <NTooltip trigger="hover">
          <template #trigger>从设备同步</template>
          设备剪切板发生变化时自动同步更新电脑剪切板
        </NTooltip>
      </NCheckbox>
      <NCheckbox
        v-model:checked="store.clipboardSync.pasteFromPC"
        @update:checked="changeClipboardSync"
      >
        <NTooltip trigger="hover">
          <template #trigger>粘贴时同步</template>
          在按键输入模式下，按下 Ctrl + V 可将电脑剪切板内容同步粘贴到设备
        </NTooltip>
      </NCheckbox>
    </NFlex>
  </div>
</template>

<style scoped>
.setting-page {
  padding: 10px 25px;
}
</style>

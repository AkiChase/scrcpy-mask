<script setup lang="ts">
import { Store } from "@tauri-apps/plugin-store";
import {
  NH4,
  NButton,
  NInput,
  NSelect,
  NInputGroup,
  useMessage,
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
  </div>
</template>

<style scoped>
.setting-page {
  padding: 10px 25px;
}
</style>

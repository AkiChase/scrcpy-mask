<script setup lang="ts">
import { Store } from "@tauri-apps/plugin-store";
import { Refresh, TrashBinOutline } from "@vicons/ionicons5";
import {
  NH4,
  NP,
  NButton,
  NFlex,
  NList,
  NListItem,
  NModal,
  NInput,
  useDialog,
  NCard,
  NIcon,
  NSelect,
  NInputGroup,
  useMessage,
} from "naive-ui";
import { relaunch } from "@tauri-apps/plugin-process";
import { onMounted, ref } from "vue";
import i18n from "../../i18n";
import { useI18n } from "vue-i18n";
import { setAdbPath } from "../../invoke";
import { useGlobalStore } from "../../store/global";

const { t } = useI18n();
const localStore = new Store("store.bin");
const store = useGlobalStore();
const message = useMessage();
const dialog = useDialog();

const localStoreEntries = ref<[string, unknown][]>([]);
const showDataModal = ref(false);
const dataModalInputVal = ref("");
let curDataIndex = -1;

const languageOptions = [
  { label: "简体中文", value: "zh-CN" },
  { label: "English", value: "en-US" },
];

const curLanguage = ref("en-US");

const adbPath = ref("");

onMounted(async () => {
  refreshLocalData();
  curLanguage.value = (await localStore.get<string>("language")) ?? "en-US";
  adbPath.value = (await localStore.get<string>("adbPath")) ?? "";
});

async function refreshLocalData() {
  localStoreEntries.value = await localStore.entries();
}

function showLocalStore(index: number) {
  curDataIndex = index;
  dataModalInputVal.value = JSON.stringify(
    localStoreEntries.value[index][1],
    null,
    2
  );
  showDataModal.value = true;
}

function delLocalStore(key?: string) {
  if (key) {
    dialog.warning({
      title: t("pages.Setting.Basic.delLocalStore.dialog.title"),
      content: t("pages.Setting.Basic.delLocalStore.dialog.delKey", [key]),
      positiveText: t("pages.Setting.Basic.delLocalStore.dialog.positiveText"),
      negativeText: t("pages.Setting.Basic.delLocalStore.dialog.negativeText"),
      onPositiveClick: () => {
        localStore.delete(key);
        localStoreEntries.value.splice(curDataIndex, 1);
        showDataModal.value = false;
      },
    });
  } else {
    dialog.warning({
      title: t("pages.Setting.Basic.delLocalStore.dialog.title"),
      content: t("pages.Setting.Basic.delLocalStore.dialog.delAll"),
      positiveText: t("pages.Setting.Basic.delLocalStore.dialog.positiveText"),
      negativeText: t("pages.Setting.Basic.delLocalStore.dialog.negativeText"),
      onPositiveClick: () => {
        localStore.clear();
        relaunch();
      },
    });
  }
}

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
    <NFlex justify="space-between">
      <NH4 prefix="bar">{{ $t("pages.Setting.Basic.localStore") }}</NH4>
      <NFlex>
        <NButton
          tertiary
          circle
          type="primary"
          @click="delLocalStore()"
          style="margin-right: 20px"
        >
          <template #icon>
            <NIcon><TrashBinOutline /></NIcon>
          </template>
        </NButton>
        <NButton
          tertiary
          circle
          type="primary"
          @click="refreshLocalData()"
          style="margin-right: 20px"
        >
          <template #icon>
            <NIcon><Refresh /></NIcon>
          </template>
        </NButton>
      </NFlex>
    </NFlex>
    <NP>{{ $t("pages.Setting.Basic.delLocalStore.warning") }}</NP>
    <NList class="data-list" hoverable clickable>
      <NListItem v-for="(entrie, index) in localStoreEntries">
        <div @click="showLocalStore(index)">
          {{ entrie[0] }}
        </div>
      </NListItem>
    </NList>
  </div>
  <NModal v-model:show="showDataModal">
    <NCard
      style="width: 50%; height: 80%"
      :title="localStoreEntries[curDataIndex][0]"
    >
      <NFlex vertical style="height: 100%">
        <NInput
          type="textarea"
          style="flex-grow: 1"
          :value="dataModalInputVal"
          round
          readonly
        />
        <NButton
          type="success"
          round
          @click="delLocalStore(localStoreEntries[curDataIndex][0])"
          >{{ $t("pages.Setting.Basic.delCurData") }}</NButton
        >
      </NFlex>
    </NCard>
  </NModal>
</template>

<style scoped>
.setting-page {
  padding: 10px 25px;

  .data-list {
    margin: 20px 0;
  }
}
</style>

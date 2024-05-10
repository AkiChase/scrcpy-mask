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
} from "naive-ui";
import { relaunch } from "@tauri-apps/plugin-process";
import { onMounted, ref } from "vue";
import i18n from "../../i18n";

const localStore = new Store("store.bin");
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

onMounted(async () => {
  refreshLocalData();
  curLanguage.value = (await localStore.get<string>("language")) ?? "en-US";
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
      title: "Warning",
      content: `即将删除数据"${key}"，删除操作不可撤回，是否继续？`,
      positiveText: "删除",
      negativeText: "取消",
      onPositiveClick: () => {
        localStore.delete(key);
        localStoreEntries.value.splice(curDataIndex, 1);
        showDataModal.value = false;
      },
    });
  } else {
    dialog.warning({
      title: "Warning",
      content: "即将清空数据，操作不可撤回，且清空后将重启软件，是否继续？",
      positiveText: "删除",
      negativeText: "取消",
      onPositiveClick: () => {
        // localStore.clear();
        relaunch();
      },
    });
  }
}

function changeLanguage(language: "zh-CN" | "en-US") {
  if (language === curLanguage.value) return;
  curLanguage.value = language;
  localStore.set("language", language);
  i18n.global.locale = language;
}
</script>

<template>
  <div class="setting-page">
    <NH4 prefix="bar">语言</NH4>
    <NSelect
      :value="curLanguage"
      @update:value="changeLanguage"
      :options="languageOptions"
      style="max-width: 300px; margin: 20px 0"
    />
    <NFlex justify="space-between">
      <NH4 prefix="bar">本地数据</NH4>
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
    <NP
      >删除数据可能导致无法预料的后果，请慎重操作。若出现异常请尝试清空数据并重启软件。</NP
    >
    <NList class="data-list" hoverable clickable>
      <NListItem v-for="(entrie, index) in localStoreEntries">
        <div @click="showLocalStore(index)">
          {{ entrie[0] }}
        </div>
      </NListItem>
    </NList>
  </div>
  <NModal v-model:show="showDataModal">
    <NCard style="width: 50%; height: 80%" title="卡片">
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
          >删除当前数据</NButton
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

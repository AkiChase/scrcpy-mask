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
} from "naive-ui";
import { relaunch } from "@tauri-apps/plugin-process";
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";

const { t } = useI18n();
const localStore = new Store("store.bin");
const dialog = useDialog();

const localStoreEntries = ref<[string, unknown][]>([]);
const showDataModal = ref(false);
const dataModalInputVal = ref("");
let curDataIndex = -1;

onMounted(async () => {
  refreshLocalData();
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
      title: t("pages.Setting.Data.delLocalStore.dialog.title"),
      content: t("pages.Setting.Data.delLocalStore.dialog.delKey", [key]),
      positiveText: t("pages.Setting.Data.delLocalStore.dialog.positiveText"),
      negativeText: t("pages.Setting.Data.delLocalStore.dialog.negativeText"),
      onPositiveClick: () => {
        localStore.delete(key);
        localStoreEntries.value.splice(curDataIndex, 1);
        showDataModal.value = false;
      },
    });
  } else {
    dialog.warning({
      title: t("pages.Setting.Data.delLocalStore.dialog.title"),
      content: t("pages.Setting.Data.delLocalStore.dialog.delAll"),
      positiveText: t("pages.Setting.Data.delLocalStore.dialog.positiveText"),
      negativeText: t("pages.Setting.Data.delLocalStore.dialog.negativeText"),
      onPositiveClick: () => {
        localStore.clear();
        relaunch();
      },
    });
  }
}
</script>

<template>
  <div class="setting-page">
    <NFlex justify="space-between">
      <NH4 prefix="bar">{{ $t("pages.Setting.Data.localStore") }}</NH4>
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
    <NP>{{ $t("pages.Setting.Data.delLocalStore.warning") }}</NP>
    <NList class="data-list" hoverable clickable>
      <NListItem
        v-for="(entrie, index) in localStoreEntries"
        @click="showLocalStore(index)"
      >
        <div>
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
          >{{ $t("pages.Setting.Data.delCurData") }}</NButton
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

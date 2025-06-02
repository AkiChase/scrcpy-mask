<script setup lang="ts">
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
} from "naive-ui";
import { relaunch } from "@tauri-apps/plugin-process";
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { LocalStore } from "../../store/localStore";
import ButtonWithTip from "../common/ButtonWithTip.vue";
import { openPath } from "@tauri-apps/plugin-opener";

const { t } = useI18n();

const dialog = useDialog();

const localStoreEntries = ref<[string, unknown][]>([]);
const showDataModal = ref(false);
const dataModalInputVal = ref("");

let curDataIndex = -1;

onMounted(async () => {
  refreshLocalData();
});

async function refreshLocalData() {
  localStoreEntries.value = await LocalStore.entries();
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
        LocalStore.delete(key);
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
        LocalStore.clear();
        relaunch();
      },
    });
  }
}
</script>

<template>
  <div>
    <NH4 prefix="bar">{{ $t("pages.Setting.Data.logs") }}</NH4>
    <NButton
      text
      @click="openPath(LocalStore.logDir)"
      style="margin-bottom: 32px"
      >{{ LocalStore.logDir }}</NButton
    >

    <NFlex justify="space-between">
      <NH4 prefix="bar">{{ $t("pages.Setting.Data.localStore") }}</NH4>
      <NFlex>
        <ButtonWithTip
          tertiary
          circle
          type="primary"
          @click="delLocalStore()"
          :tip="$t('pages.Setting.Data.btnDelAll')"
          :icon="TrashBinOutline"
        />
        <ButtonWithTip
          tertiary
          circle
          type="primary"
          @click="refreshLocalData()"
          :tip="$t('pages.Setting.Data.btnRefresh')"
          :icon="Refresh"
        />
      </NFlex>
    </NFlex>
    <NButton text @click="openPath(LocalStore.dir)">{{ LocalStore.path }}</NButton>
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
  </div>
</template>

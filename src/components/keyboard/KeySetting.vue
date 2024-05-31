<script setup lang="ts">
import { Settings, CloseCircle, ReturnUpBack } from "@vicons/ionicons5";
import {
  NButton,
  NIcon,
  NH4,
  NSelect,
  NFlex,
  NP,
  NModal,
  NCard,
  NInput,
  useMessage,
} from "naive-ui";
import { computed, onActivated, onMounted, ref, watch } from "vue";
import { useGlobalStore } from "../../store/global";
import { Store } from "@tauri-apps/plugin-store";
import { loadDefaultKeyconfig } from "../../invoke";
import { KeyMappingConfig } from "../../keyMappingConfig";
import { useKeyboardStore } from "../../store/keyboard";
import { useI18n } from "vue-i18n";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

const { t } = useI18n();
const store = useGlobalStore();
const keyboardStore = useKeyboardStore();
const localStore = new Store("store.bin");
const message = useMessage();

const showImportModal = ref(false);
const showRenameModal = ref(false);
const importModalInputValue = ref("");
const renameModalInputValue = ref("");

const keyMappingNameOptions = computed(() => {
  return store.keyMappingConfigList.map((item, index) => {
    return {
      label: item.title,
      value: index,
    };
  });
});

const curRelativeSize = computed(() => {
  if (store.keyMappingConfigList.length === 0) {
    return { w: 800, h: 600 };
  }
  return store.keyMappingConfigList[store.curKeyMappingIndex].relativeSize;
});

const keySettingPos = ref({ x: 100, y: 100 });

onMounted(async () => {
  // loading keySettingPos from local store
  let storedPos = await localStore.get<{ x: number; y: number }>(
    "keySettingPos"
  );

  if (storedPos === null) {
    await localStore.set("keySettingPos", keySettingPos.value);
    storedPos = { x: 100, y: 100 };
  }
  // apply keySettingPos
  const keyboardElement = document.getElementById(
    "keyboardElement"
  ) as HTMLElement;
  const maxWidth = keyboardElement.clientWidth - 40;
  const maxHeight = keyboardElement.clientHeight - 40;
  keySettingPos.value.x = Math.max(0, Math.min(storedPos.x, maxWidth));
  keySettingPos.value.y = Math.max(0, Math.min(storedPos.y, maxHeight));
});

onActivated(() => {
  // reset editKeyMappingList as the same as keyMappingList
  resetKeyMappingConfig();
  // check config relative size
  checkConfigSize();
});

watch(
  () => store.curKeyMappingIndex,
  () => {
    // check config relative size
    checkConfigSize();
  }
);

function dragHandler(downEvent: MouseEvent) {
  const target = document.getElementById("keySettingBtn") as HTMLElement;
  const keyboardElement = document.getElementById(
    "keyboardElement"
  ) as HTMLElement;
  const maxWidth = keyboardElement.clientWidth - 40;
  const maxHeight = keyboardElement.clientHeight - 40;

  const oldX = keySettingPos.value.x;
  const oldY = keySettingPos.value.y;
  const x = downEvent.clientX;
  const y = downEvent.clientY;

  let moveFlag = false;
  const moveHandler = (moveEvent: MouseEvent) => {
    const newX = oldX + moveEvent.clientX - x;
    const newY = oldY + moveEvent.clientY - y;
    keySettingPos.value.x = Math.max(0, Math.min(newX, maxWidth));
    keySettingPos.value.y = Math.max(0, Math.min(newY, maxHeight));
  };

  const timer = setTimeout(() => {
    moveFlag = true;
    target.style.setProperty("cursor", "grabbing");
    window.addEventListener("mousemove", moveHandler);
  }, 1000);

  const upHandler = () => {
    clearTimeout(timer);
    window.removeEventListener("mousemove", moveHandler);
    window.removeEventListener("mouseup", upHandler);
    if (moveFlag) {
      // move up
      target.style.setProperty("cursor", "pointer");
      localStore.set("keySettingPos", keySettingPos.value);
    } else {
      // click up
      if (keyboardStore.editSwipePointsFlag) {
        keyboardStore.editSwipePointsFlag = false;
      } else {
        keyboardStore.activeButtonIndex = -1;
        keyboardStore.activeSteeringWheelButtonKeyIndex = -1;
        keyboardStore.showSettingFlag = !keyboardStore.showSettingFlag;
        if (
          keyboardStore.showSettingFlag &&
          store.keyMappingConfigList.length === 1
        ) {
          message.info(t("pages.KeyBoard.KeySetting.onlyOneConfig"));
        }
      }
    }
  };
  window.addEventListener("mouseup", upHandler);
}

function importKeyMappingConfig() {
  let keyMappingConfig;
  try {
    keyMappingConfig = JSON.parse(importModalInputValue.value);
  } catch (e) {
    console.error(e);
    message.error(t("pages.KeyBoard.KeySetting.importFailed"));
    return;
  }
  store.keyMappingConfigList.push(keyMappingConfig);
  store.setKeyMappingIndex(store.keyMappingConfigList.length - 1);
  showImportModal.value = false;
  localStore.set("keyMappingConfigList", store.keyMappingConfigList);
  message.success(t("pages.KeyBoard.KeySetting.importSuccess"));
}

async function importDefaultKeyMappingConfig() {
  const data = await loadDefaultKeyconfig();
  let defaultConfigs: KeyMappingConfig[];
  let count = 0;
  try {
    defaultConfigs = JSON.parse(data);
    for (const config of defaultConfigs) {
      store.keyMappingConfigList.push(config);
      count++;
    }
  } catch (e) {
    console.error(e);
    message.error(t("pages.KeyBoard.KeySetting.importDefaultFailed"));
    return;
  }

  localStore.set("keyMappingConfigList", store.keyMappingConfigList);
  message.success(t("pages.KeyBoard.KeySetting.importDefaultSuccess", [count]));
}

function createKeyMappingConfig() {
  if (keyboardStore.edited) {
    message.error(t("pages.KeyBoard.KeySetting.configEdited"));
    return;
  }

  const keyboardElement = document.getElementById(
    "keyboardElement"
  ) as HTMLElement;
  const newConfig: KeyMappingConfig = {
    title: t("pages.KeyBoard.KeySetting.newConfig"),
    relativeSize: {
      w: keyboardElement.clientWidth,
      h: keyboardElement.clientHeight,
    },
    list: [],
  };
  store.keyMappingConfigList.push(newConfig);
  store.setKeyMappingIndex(store.keyMappingConfigList.length - 1);
  localStore.set("keyMappingConfigList", store.keyMappingConfigList);
  message.success(t("pages.KeyBoard.KeySetting.newConfigSuccess"));
}

function copyCurKeyMappingConfig() {
  if (keyboardStore.edited) {
    message.error(t("pages.KeyBoard.KeySetting.configEdited"));
    return;
  }

  const curConfig = store.keyMappingConfigList[store.curKeyMappingIndex];
  const newTitle = t("pages.KeyBoard.KeySetting.copyConfigTitle", [
    curConfig.title,
  ]);
  const newConfig: KeyMappingConfig = {
    title: newTitle,
    relativeSize: curConfig.relativeSize,
    list: curConfig.list,
  };
  store.keyMappingConfigList.push(newConfig);
  keyboardStore.activeButtonIndex = -1;
  keyboardStore.activeSteeringWheelButtonKeyIndex = -1;
  store.setKeyMappingIndex(store.keyMappingConfigList.length - 1);
  localStore.set("keyMappingConfigList", store.keyMappingConfigList);
  message.success(t("pages.KeyBoard.KeySetting.copyConfigSuccess", [newTitle]));
}

function delCurKeyMappingConfig() {
  if (store.keyMappingConfigList.length <= 1) {
    message.error(t("pages.KeyBoard.KeySetting.delConfigLeast"));
    return;
  }
  const title = store.keyMappingConfigList[store.curKeyMappingIndex].title;
  store.keyMappingConfigList.splice(store.curKeyMappingIndex, 1);

  // reset active and edit status
  keyboardStore.activeButtonIndex = -1;
  keyboardStore.activeSteeringWheelButtonKeyIndex = -1;
  keyboardStore.edited = false;
  store.setKeyMappingIndex(
    store.curKeyMappingIndex > 0 ? store.curKeyMappingIndex - 1 : 0
  );
  localStore.set("keyMappingConfigList", store.keyMappingConfigList);
  message.success(t("pages.KeyBoard.KeySetting.delSuccess", [title]));
}

function renameKeyMappingConfig() {
  const newTitle = renameModalInputValue.value;
  showRenameModal.value = false;
  if (newTitle !== "") {
    store.keyMappingConfigList[store.curKeyMappingIndex].title = newTitle;
    localStore.set("keyMappingConfigList", store.keyMappingConfigList);
    message.success(t("pages.KeyBoard.KeySetting.renameSuccess", [newTitle]));
  } else {
    message.error(t("pages.KeyBoard.KeySetting.renameEmpty"));
  }
}

function exportKeyMappingConfig() {
  const config = store.keyMappingConfigList[store.curKeyMappingIndex];
  const data = JSON.stringify(config, null, 2);
  writeText(data)
    .then(() => {
      message.success(t("pages.KeyBoard.KeySetting.exportSuccess"));
    })
    .catch((e) => {
      console.error(e);
      message.error(t("pages.KeyBoard.KeySetting.exportFailed"));
    });
}

function saveKeyMappingConfig() {
  if (store.applyEditKeyMappingList()) {
    keyboardStore.edited = false;
  } else {
    message.error(t("pages.KeyBoard.KeySetting.saveKeyRepeat"));
  }
}

function checkConfigSize() {
  const keyboardElement = document.getElementById(
    "keyboardElement"
  ) as HTMLElement;
  const curKeyMappingConfig =
    store.keyMappingConfigList[store.curKeyMappingIndex];
  const relativeSize = curKeyMappingConfig.relativeSize;

  if (
    keyboardElement.clientWidth !== relativeSize.w ||
    keyboardElement.clientHeight !== relativeSize.h
  ) {
    message.warning(
      t("pages.KeyBoard.KeySetting.checkConfigSizeWarning", [
        curKeyMappingConfig.title,
      ])
    );
  }
}

function migrateKeyMappingConfig() {
  if (keyboardStore.edited) {
    message.error(t("pages.KeyBoard.KeySetting.configEdited"));
    return;
  }

  const keyboardElement = document.getElementById(
    "keyboardElement"
  ) as HTMLElement;
  const curKeyMappingConfig =
    store.keyMappingConfigList[store.curKeyMappingIndex];

  const relativeSize = curKeyMappingConfig.relativeSize;
  const sizeW = keyboardElement.clientWidth;
  const sizeH = keyboardElement.clientHeight;

  if (sizeW !== relativeSize.w || sizeH !== relativeSize.h) {
    // deep clone
    const newConfig = JSON.parse(JSON.stringify(curKeyMappingConfig));
    // migrate relativeSize
    newConfig.relativeSize = {
      w: sizeW,
      h: sizeH,
    };
    // migrate key pos
    for (const keyMapping of newConfig.list) {
      keyMapping.posX = Math.round((keyMapping.posX / relativeSize.w) * sizeW);
      keyMapping.posY = Math.round((keyMapping.posY / relativeSize.h) * sizeH);
    }
    // migrate title
    newConfig.title = t("pages.KeyBoard.KeySetting.migrateConfigTitle", [
      newConfig.title,
    ]);

    store.keyMappingConfigList.splice(
      store.curKeyMappingIndex + 1,
      0,
      newConfig
    );
    message.success(
      t("pages.KeyBoard.KeySetting.migrateConfigSuccess", [newConfig.title])
    );
    keyboardStore.activeButtonIndex = -1;
    keyboardStore.activeSteeringWheelButtonKeyIndex = -1;
    store.setKeyMappingIndex(store.curKeyMappingIndex + 1);
  } else {
    message.info(t("pages.KeyBoard.KeySetting.migrateConfigNeedless"));
  }
}

function selectKeyMappingConfig(index: number) {
  if (keyboardStore.edited) {
    message.error(t("pages.KeyBoard.KeySetting.configEdited"));
    return;
  }

  keyboardStore.activeButtonIndex = -1;
  keyboardStore.activeSteeringWheelButtonKeyIndex = -1;
  store.setKeyMappingIndex(index);
}

function resetKeyMappingConfig() {
  keyboardStore.activeButtonIndex = -1;
  keyboardStore.activeSteeringWheelButtonKeyIndex = -1;
  store.resetEditKeyMappingList();
  keyboardStore.edited = false;
}
</script>

<template>
  <NButton
    circle
    type="info"
    size="large"
    class="key-setting-btn"
    id="keySettingBtn"
    :title="$t('pages.KeyBoard.KeySetting.buttonDrag')"
    @mousedown="dragHandler"
    :style="{
      left: keySettingPos.x + 'px',
      top: keySettingPos.y + 'px',
    }"
  >
    <template #icon>
      <NIcon>
        <ReturnUpBack v-if="keyboardStore.editSwipePointsFlag" />
        <Settings v-else />
      </NIcon>
    </template>
  </NButton>
  <div
    class="key-setting"
    v-show="keyboardStore.showSettingFlag"
    @mousedown="
      keyboardStore.activeButtonIndex = -1;
      keyboardStore.activeSteeringWheelButtonKeyIndex = -1;
    "
  >
    <NButton
      text
      class="key-setting-close"
      @click="keyboardStore.showSettingFlag = false"
    >
      <NIcon><CloseCircle></CloseCircle></NIcon>
    </NButton>
    <NH4 prefix="bar">{{ $t("pages.KeyBoard.KeySetting.config") }}</NH4>
    <NSelect
      :value="store.curKeyMappingIndex"
      @update:value="selectKeyMappingConfig"
      :options="keyMappingNameOptions"
    />
    <NP style="margin-top: 20px">
      {{
        $t("pages.KeyBoard.KeySetting.configRelativeSize", [
          curRelativeSize.w,
          curRelativeSize.h,
        ])
      }}
    </NP>
    <NFlex style="margin-top: 20px">
      <template v-if="keyboardStore.edited">
        <NButton type="success" @click="saveKeyMappingConfig">{{
          $t("pages.KeyBoard.KeySetting.saveConfig")
        }}</NButton>
        <NButton type="error" @click="resetKeyMappingConfig">{{
          $t("pages.KeyBoard.KeySetting.resetConfig")
        }}</NButton>
      </template>
      <NButton @click="createKeyMappingConfig">{{
        $t("pages.KeyBoard.KeySetting.createConfig")
      }}</NButton>
      <NButton @click="copyCurKeyMappingConfig">{{
        $t("pages.KeyBoard.KeySetting.copyConfig")
      }}</NButton>
      <NButton @click="migrateKeyMappingConfig">{{
        $t("pages.KeyBoard.KeySetting.migrateConfig")
      }}</NButton>
      <NButton @click="delCurKeyMappingConfig">{{
        $t("pages.KeyBoard.KeySetting.delConfig")
      }}</NButton>
      <NButton
        @click="
          showRenameModal = true;
          renameModalInputValue =
            store.keyMappingConfigList[store.curKeyMappingIndex].title;
        "
        >{{ $t("pages.KeyBoard.KeySetting.renameConfig") }}</NButton
      >
    </NFlex>
    <NH4 prefix="bar">{{ $t("pages.KeyBoard.KeySetting.others") }}</NH4>
    <NFlex>
      <NButton
        @click="
          showImportModal = true;
          importModalInputValue = '';
        "
        >{{ $t("pages.KeyBoard.KeySetting.importConfig") }}</NButton
      >
      <NButton @click="exportKeyMappingConfig">{{
        $t("pages.KeyBoard.KeySetting.exportConfig")
      }}</NButton>
      <NButton @click="importDefaultKeyMappingConfig">{{
        $t("pages.KeyBoard.KeySetting.importDefaultConfig")
      }}</NButton>
      <NButton
        @click="keyboardStore.showKeyInfoFlag = !keyboardStore.showKeyInfoFlag"
        >{{ $t("pages.KeyBoard.KeySetting.keyInfo") }}</NButton
      >
    </NFlex>
    <NP style="margin-top: 40px">{{
      $t("pages.KeyBoard.KeySetting.addButtonTip")
    }}</NP>
  </div>
  <NModal v-model:show="showImportModal">
    <NCard style="width: 40%; height: 50%">
      <NFlex vertical style="height: 100%">
        <NInput
          type="textarea"
          style="flex-grow: 1"
          :placeholder="$t('pages.KeyBoard.KeySetting.importPlaceholder')"
          v-model:value="importModalInputValue"
          round
          clearable
        />
        <NButton type="success" round @click="importKeyMappingConfig">{{
          $t("pages.KeyBoard.KeySetting.import")
        }}</NButton>
      </NFlex>
    </NCard>
  </NModal>
  <NModal v-model:show="showRenameModal">
    <NCard
      style="width: 40%"
      :title="$t('pages.KeyBoard.KeySetting.renameTitle')"
    >
      <NFlex vertical>
        <NInput v-model:value="renameModalInputValue" clearable />
        <NButton type="success" round @click="renameKeyMappingConfig">{{
          $t("pages.KeyBoard.KeySetting.renameConfig")
        }}</NButton>
      </NFlex>
    </NCard>
  </NModal>
</template>

<style scoped lang="scss">
.key-setting-btn {
  position: absolute;
  z-index: 9;
}

.key-setting {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  width: 70%;
  height: 70%;
  margin: auto;
  background-color: var(--content-bg-color);
  padding: 0 50px;
  border: 1px solid var(--gray-color);
  border-radius: 10px;
  z-index: 10;

  .key-setting-close {
    font-size: 24px;
    position: absolute;
    right: 10px;
    top: 10px;
  }
}
</style>

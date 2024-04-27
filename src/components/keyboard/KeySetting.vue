<script setup lang="ts">
import { Settings, CloseCircle } from "@vicons/ionicons5";
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
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { loadDefaultKeyconfig } from "../../invoke";
import { KeyMappingConfig } from "../../keyMappingConfig";

const store = useGlobalStore();
const localStore = new Store("store.bin");
const message = useMessage();

const showKeyInfoFlag = defineModel("showKeyInfoFlag", { required: true });
const showSettingFlag = defineModel("showSettingFlag", { required: true });
const edited = defineModel("edited", { required: true });

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
  store.resetEditKeyMappingList();
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
      showSettingFlag.value = !showSettingFlag.value;
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
    message.error("导入失败");
    return;
  }
  store.keyMappingConfigList.push(keyMappingConfig);
  store.setKeyMappingIndex(store.keyMappingConfigList.length - 1);
  showImportModal.value = false;
  localStore.set("keyMappingConfigList", store.keyMappingConfigList);
  message.success("按键方案已导入");
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
    message.error("默认按键方案导入失败");
    return;
  }

  localStore.set("keyMappingConfigList", store.keyMappingConfigList);
  message.success(`已导入${count}个默认方案`);
}

function createKeyMappingConfig() {
  const keyboardElement = document.getElementById(
    "keyboardElement"
  ) as HTMLElement;
  const newConfig: KeyMappingConfig = {
    title: "新方案",
    relativeSize: {
      w: keyboardElement.clientWidth,
      h: keyboardElement.clientHeight,
    },
    list: [],
  };
  store.keyMappingConfigList.push(newConfig);
  store.setKeyMappingIndex(store.keyMappingConfigList.length - 1);
  localStore.set("keyMappingConfigList", store.keyMappingConfigList);
  message.success("新方案已创建");
}

function copyCurKeyMappingConfig() {
  const curConfig = store.keyMappingConfigList[store.curKeyMappingIndex];
  const newConfig: KeyMappingConfig = {
    title: curConfig.title + "-副本",
    relativeSize: curConfig.relativeSize,
    list: curConfig.list,
  };
  store.keyMappingConfigList.push(newConfig);
  store.setKeyMappingIndex(store.keyMappingConfigList.length - 1);
  localStore.set("keyMappingConfigList", store.keyMappingConfigList);
  message.success("方案已复制为：" + curConfig.title + "-副本");
}

function delCurKeyMappingConfig() {
  if (store.keyMappingConfigList.length <= 1) {
    message.error("至少保留一个方案");
    return;
  }
  const title = store.keyMappingConfigList[store.curKeyMappingIndex].title;
  store.keyMappingConfigList.splice(store.curKeyMappingIndex, 1);
  store.setKeyMappingIndex(
    store.curKeyMappingIndex > 0 ? store.curKeyMappingIndex - 1 : 0
  );
  localStore.set("keyMappingConfigList", store.keyMappingConfigList);
  message.success("方案已删除：" + title);
}

function renameKeyMappingConfig() {
  const newTitle = renameModalInputValue.value;
  showRenameModal.value = false;
  if (newTitle !== "") {
    store.keyMappingConfigList[store.curKeyMappingIndex].title = newTitle;
    localStore.set("keyMappingConfigList", store.keyMappingConfigList);
    message.success("方案已重命名为：" + newTitle);
  } else {
    message.error("方案名不能为空");
  }
}

function exportKeyMappingConfig() {
  const config = store.keyMappingConfigList[store.curKeyMappingIndex];
  const data = JSON.stringify(config, null, 2);
  writeText(data)
    .then(() => {
      message.success("当前按键方案已导出到剪切板");
    })
    .catch((e) => {
      console.error(e);
      message.error("按键方案导出失败");
    });
}

function saveKeyMappingConfig() {
  store.applyEditKeyMappingList();
  edited.value = false;
}

function resetKeyMappingConfig() {
  store.resetEditKeyMappingList();
  edited.value = false;
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
      `请注意当前按键方案"${curKeyMappingConfig.title}"与蒙版尺寸不一致，若有需要可进行迁移`
    );
  }
}

function migrateKeyMappingConfig() {
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
    newConfig.title += "-迁移";

    store.keyMappingConfigList.splice(
      store.curKeyMappingIndex + 1,
      0,
      newConfig
    );
    store.setKeyMappingIndex(store.curKeyMappingIndex + 1);
    message.success("已迁移到新方案：" + newConfig.title);
  } else {
    message.info("当前方案符合蒙版尺寸，无需迁移");
  }
}
</script>

<template>
  <NButton
    circle
    type="info"
    size="large"
    class="key-setting-btn"
    id="keySettingBtn"
    title="长按可拖动"
    @mousedown="dragHandler"
    :style="{
      left: keySettingPos.x + 'px',
      top: keySettingPos.y + 'px',
    }"
  >
    <template #icon>
      <NIcon><Settings /></NIcon>
    </template>
  </NButton>
  <div class="key-setting" v-show="showSettingFlag">
    <NButton text class="key-setting-close" @click="showSettingFlag = false">
      <NIcon><CloseCircle></CloseCircle></NIcon>
    </NButton>
    <NH4 prefix="bar">按键方案</NH4>
    <NSelect
      :value="store.curKeyMappingIndex"
      @update:value="(value: number)=>store.setKeyMappingIndex(value)"
      :options="keyMappingNameOptions"
    />
    <NP> Relative Size:{{ curRelativeSize.w }}x{{ curRelativeSize.h }} </NP>
    <NFlex style="margin-top: 20px">
      <template v-if="edited">
        <NButton type="success" @click="saveKeyMappingConfig">保存方案</NButton>
        <NButton type="error" @click="resetKeyMappingConfig">还原方案</NButton>
      </template>
      <NButton @click="createKeyMappingConfig">新建方案</NButton>
      <NButton @click="copyCurKeyMappingConfig">复制方案</NButton>
      <NButton @click="migrateKeyMappingConfig">迁移方案</NButton>
      <NButton @click="delCurKeyMappingConfig">删除方案</NButton>
      <NButton
        @click="
          showRenameModal = true;
          renameModalInputValue =
            store.keyMappingConfigList[store.curKeyMappingIndex].title;
        "
        >重命名</NButton
      >
    </NFlex>
    <NH4 prefix="bar">其他</NH4>
    <NFlex>
      <NButton
        @click="
          showImportModal = true;
          importModalInputValue = '';
        "
        >导入方案</NButton
      >
      <NButton @click="exportKeyMappingConfig">导出方案</NButton>
      <NButton @click="importDefaultKeyMappingConfig">导入默认</NButton>
      <NButton @click="showKeyInfoFlag = !showKeyInfoFlag">按键信息</NButton>
    </NFlex>
  </div>
  <NModal v-model:show="showImportModal">
    <NCard style="width: 40%; height: 50%" title="导入按键方案">
      <NFlex vertical style="height: 100%">
        <NInput
          type="textarea"
          style="flex-grow: 1"
          placeholder="粘贴单个按键方案的JSON文本 (此处无法对按键方案的合法性进行判断, 请确保JSON内容正确)"
          v-model:value="importModalInputValue"
          round
          clearable
        />
        <NButton round @click="importKeyMappingConfig">导入</NButton>
      </NFlex>
    </NCard>
  </NModal>
  <NModal v-model:show="showRenameModal">
    <NCard style="width: 40%" title="重命名按键方案">
      <NFlex vertical>
        <NInput v-model:value="renameModalInputValue" clearable />
        <NButton round @click="renameKeyMappingConfig">重命名</NButton>
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

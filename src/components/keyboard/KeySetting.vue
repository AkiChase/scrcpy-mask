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
import { computed, ref } from "vue";
import { useGlobalStore } from "../../store/global";

const store = useGlobalStore();
const message = useMessage();

const showKeyInfoFlag = defineModel({ required: true });

const showSetting = ref(false);
const showImportModal = ref(false);
const importModalInputValue = ref("");

const keyMappingNameOptions = computed(() => {
  return store.keyMappingConfigList.map((item, index) => {
    return {
      label: item.title,
      value: index,
    };
  });
});

const curKeyMappingrelativeSize = computed(() => {
  return store.keyMappingConfigList[store.curKeyMappingIndex].relativeSize;
});

function dragHandler(downEvent: MouseEvent) {
  const target = document.querySelector(
    ".keyboard .key-setting-btn"
  ) as HTMLElement;
  downEvent.preventDefault();
  const x = downEvent.clientX;
  const y = downEvent.clientY;

  let moveFlag = false;
  let lastPosX = 100;
  let lastPosY = 100;
  const moveHandler = (moveEvent: MouseEvent) => {
    let right = lastPosX + x - moveEvent.clientX;
    let bottom = lastPosY + y - moveEvent.clientY;
    target.style.setProperty("right", `${right < 0 ? 0 : right}px`);
    target.style.setProperty("bottom", `${bottom < 0 ? 0 : bottom}px`);
  };

  const timer = setTimeout(() => {
    moveFlag = true;
    target.style.setProperty("cursor", "grabbing");
    window.addEventListener("mousemove", moveHandler);
  }, 1000);

  const upHandler = (upEvent: MouseEvent) => {
    clearTimeout(timer);
    window.removeEventListener("mousemove", moveHandler);
    window.removeEventListener("mouseup", upHandler);
    if (moveFlag) {
      lastPosX = lastPosX + x - upEvent.clientX;
      lastPosY = lastPosY + y - upEvent.clientY;
      target.style.setProperty("cursor", "pointer");
    } else {
      showSetting.value = !showSetting.value;
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
  store.curKeyMappingIndex = store.keyMappingConfigList.length - 1;
  showImportModal.value = false;
}
</script>

<template>
  <NButton
    circle
    type="info"
    size="large"
    class="key-setting-btn"
    @mousedown="dragHandler"
  >
    <template #icon>
      <NIcon><Settings /></NIcon>
    </template>
  </NButton>
  <div class="key-setting" v-show="showSetting">
    <NButton text class="key-setting-close" @click="showSetting = false">
      <NIcon><CloseCircle></CloseCircle></NIcon>
    </NButton>
    <NH4 prefix="bar">按键方案</NH4>
    <NSelect
      v-model:value="store.curKeyMappingIndex"
      :options="keyMappingNameOptions"
    />
    <NP>
      Relative Size: {{ curKeyMappingrelativeSize.w }} x
      {{ curKeyMappingrelativeSize.h }}
    </NP>
    <NFlex style="margin-top: 20px">
      <NButton>新建方案</NButton>
      <NButton>复制方案</NButton>
      <NButton>删除方案</NButton>
      <NButton>重命名</NButton>
    </NFlex>
    <NH4 prefix="bar">其他</NH4>
    <NFlex>
      <NButton @click="showImportModal = true">导入</NButton>
      <NButton>导出</NButton>
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
</template>

<style scoped lang="scss">
.key-setting-btn {
  position: absolute;
  z-index: 9;
  right: 100px;
  bottom: 100px;
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

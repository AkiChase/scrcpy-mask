<script setup lang="ts">
import { onActivated, ref } from "vue";
import KeyInfo from "./KeyInfo.vue";
import KeySetting from "./KeySetting.vue";
import KeyCommon from "./KeyCommon.vue";
import { useGlobalStore } from "../../store/global";
import { useDialog, useMessage } from "naive-ui";
import { onBeforeRouteLeave } from "vue-router";

// TODO 左键空白区域，关闭设置或active设为-1
// TODO 右键空白区域添加按键
// TODO 左键可拖动按钮（并显示到顶部），右键按钮进行修改、删除
// TODO 设置界面添加清空本地数据的按钮
// TODO 添加开发者工具打开按钮

const keyInfoShowFlag = ref(false);
const store = useGlobalStore();
const message = useMessage();
const dialog = useDialog();

const activeButtonIndex = ref(-1);
let edited = ref(false);

onActivated(() => {
  // migrate keyMappingConfig if relativeSize does not match
  migrateKeyMappingConfig();
  // reset editKeyMappingList as the same as keyMappingList
  store.resetEditKeyMappingList();
});

onBeforeRouteLeave(() => {
  if (edited.value) {
    dialog.warning({
      title: "Warning",
      content: "当前方案尚未保存，是否保存？",
      positiveText: "保存",
      negativeText: "取消",
      onPositiveClick: () => {
        store.applyEditKeyMappingList();
        edited.value = false;
      },
      onNegativeClick: () => {
        store.resetEditKeyMappingList();
        edited.value = false;
      },
    });
  }
});

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
    const newConfig = {
      ...curKeyMappingConfig,
      relativeSize: {
        w: sizeW,
        h: sizeH,
      },
    };

    const relativePosToMaskPos = (x: number, y: number) => {
      return {
        x: Math.round((x / relativeSize.w) * sizeW),
        y: Math.round((y / relativeSize.h) * sizeH),
      };
    };

    for (let keyMapping of curKeyMappingConfig.list) {
      const { x, y } = relativePosToMaskPos(keyMapping.posX, keyMapping.posY);
      keyMapping.posX = x;
      keyMapping.posY = y;
    }
    newConfig.title += "-迁移";
    store.keyMappingConfigList.splice(store.curKeyMappingIndex, 1, newConfig);
    store.curKeyMappingIndex += 1;

    message.warning(
      "当前按键方案与蒙版尺寸不一致，已自动迁移到新方案：" + newConfig.title
    );
  }
}
</script>

<template>
  <div id="keyboardElement" class="keyboard">
    <KeySetting v-model="keyInfoShowFlag" />
    <KeyInfo v-model="keyInfoShowFlag" />
    <template v-for="(_, index) in store.editKeyMappingList">
      <KeyCommon
        @edit="edited = true"
        @active="activeButtonIndex = index"
        :index="index"
        :activeIndex="activeButtonIndex"
      />
    </template>
  </div>
</template>

<style scoped lang="scss">
.keyboard {
  color: var(--light-color);
  background-color: rgba(0, 0, 0, 0.5);
  overflow: hidden;
  position: relative;

  .keyboard-button {
    position: absolute;
    border-radius: 50%;
    width: 40px;
    height: 40px;
    border: 1px solid red;
    background-color: red;
  }
}
</style>

<script setup lang="ts">
import { onActivated, ref } from "vue";
import KeyInfo from "./KeyInfo.vue";
import KeySetting from "./KeySetting.vue";
import KeyCommon from "./KeyCommon.vue";
import { useGlobalStore } from "../../store/global";
import { useDialog } from "naive-ui";
import { onBeforeRouteLeave } from "vue-router";

// TODO 普通按钮 KeyMacro，KeyCancelSkill，KeyTap
//      单个键，KeyMacro有输入框
// TODO 方向轮盘按钮 KeySteeringWheel
//      有四个按钮+offset
// TODO 技能按钮 KeyDirectionalSkill，KeyDirectionlessSkill，KeyTriggerWhenPressedSkill（有区分directional)
//      单个键，靠两个flag来区分这四种情况，还有range或time要视情况
// TODO 添加视野按钮 KeyObservation
//      单个键，有灵敏度scale
// TODO 按钮齿轮图标可修改、删除
// TODO 右键空白区域添加按键
// TODO 设置界面添加本地数据编辑器（类似utools）
// TODO 添加开发者工具打开按钮

const showKeyInfoFlag = ref(false);
const showSettingFlag = ref(false);
const store = useGlobalStore();
const dialog = useDialog();

const activeButtonIndex = ref(-1);
const activeSteeringWheelButtonKeyIndex = ref(-1);
let edited = ref(false);

function setCurButtonKey(curKey: string) {
  const keyMapping = store.editKeyMappingList[activeButtonIndex.value];
  if (keyMapping.type === "SteeringWheel") {
    const keyObject = keyMapping.key as {
      left: string;
      right: string;
      up: string;
      down: string;
    };
    switch (activeSteeringWheelButtonKeyIndex.value) {
      case 0:
        keyObject.up = curKey;
        break;
      case 1:
        keyObject.down = curKey;
        break;
      case 2:
        keyObject.left = curKey;
        break;
      case 3:
        keyObject.right = curKey;
        break;
    }
  } else {
    keyMapping.key = curKey;
  }
}

function handleClick(event: MouseEvent) {
  if (event.button === 0) {
    // left click
    if (event.target === document.getElementById("keyboardElement")) {
      if (showSettingFlag.value) {
        showSettingFlag.value = false;
        return;
      }
      activeButtonIndex.value = -1;
    }
  } else if (event.button === 2) {
    // right click
    if (event.target === document.getElementById("keyboardElement")) {
      // add button
      if (showSettingFlag.value) showSettingFlag.value = false;
      activeButtonIndex.value = -1;

      console.log("弹出新增");
    } else if (
      // modify key
      activeButtonIndex.value !== -1 &&
      activeButtonIndex.value < store.editKeyMappingList.length
    ) {
      event.preventDefault();
      const curKey = `M${event.button}`;
      setCurButtonKey(curKey);
    }
  } else {
    // other click
    if (
      activeButtonIndex.value !== -1 &&
      activeButtonIndex.value < store.editKeyMappingList.length
    ) {
      event.preventDefault();
      const curKey = `M${event.button}`;
      setCurButtonKey(curKey);
    }
  }
}

function handleKeyUp(event: KeyboardEvent) {
  if (
    activeButtonIndex.value !== -1 &&
    activeButtonIndex.value < store.editKeyMappingList.length
  ) {
    event.preventDefault();
    const curKey = event.code;
    setCurButtonKey(curKey);
  }
}

function handleMouseWheel(event: WheelEvent) {
  if (
    activeButtonIndex.value !== -1 &&
    activeButtonIndex.value < store.editKeyMappingList.length
  ) {
    event.preventDefault();
    if (event.deltaY > 0) {
      // WheelDown
      setCurButtonKey("WheelDown");
    } else if (event.deltaY < 0) {
      // WheelUp
      setCurButtonKey("WheelUp");
    }
  }
}

onActivated(() => {
  document.addEventListener("keyup", handleKeyUp);
  document.addEventListener("wheel", handleMouseWheel);
});

onBeforeRouteLeave(() => {
  document.removeEventListener("keyup", handleKeyUp);
  document.removeEventListener("wheel", handleMouseWheel);
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
</script>

<template>
  <div
    id="keyboardElement"
    class="keyboard"
    @mousedown="handleClick"
    @contextmenu.prevent
  >
    <KeySetting
      v-model:showKeyInfoFlag="showKeyInfoFlag"
      v-model:showSettingFlag="showSettingFlag"
      v-model:edited="edited"
    />
    <KeyInfo v-model:showKeyInfoFlag="showKeyInfoFlag" />
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

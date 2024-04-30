<script setup lang="ts">
import { onActivated } from "vue";
import KeyInfo from "./KeyInfo.vue";
import KeySetting from "./KeySetting.vue";
import KeyCommon from "./KeyCommon.vue";
import KeySteeringWheel from "./KeySteeringWheel.vue";
import KeySkill from "./KeySkill.vue";
import { KeySteeringWheel as KeyMappingSteeringWheel } from "../../keyMappingConfig";
import { useGlobalStore } from "../../store/global";
import { useDialog, useMessage } from "naive-ui";
import { onBeforeRouteLeave } from "vue-router";
import KeyObservation from "./KeyObservation.vue";
import { useKeyboardStore } from "../../store/keyboard";

// TODO 打开设置时要关闭active，建议将各种数据打包到一个对象中共享，省的麻烦
// TODO 切换按键方案时提示未保存，然后修改edit
// TODO 方向轮盘具体按键还没激活时也会触发按键检查
// TODO 右键空白区域添加按键
// TODO 设置界面添加本地数据编辑器（类似utools）
// TODO 添加开发者工具打开按钮
// TODO 添加息屏按键

const store = useGlobalStore();
const keyboardStore = useKeyboardStore();
const dialog = useDialog();
const message = useMessage();

function isKeyUnique(curKey: string): boolean {
  const set = new Set<string>();
  for (const keyMapping of store.editKeyMappingList) {
    if (keyMapping.type === "SteeringWheel") {
      const nameList: ["up", "down", "left", "right"] = [
        "up",
        "down",
        "left",
        "right",
      ];
      for (const name of nameList) {
        if (set.has((keyMapping as KeyMappingSteeringWheel).key[name]))
          return false;
        set.add((keyMapping as KeyMappingSteeringWheel).key[name]);
      }
    } else {
      if (set.has(keyMapping.key as string)) return false;
      set.add(keyMapping.key as string);
    }
  }
  if (set.has(curKey)) return false;
  return true;
}

function setCurButtonKey(curKey: string) {
  if (!isKeyUnique(curKey)) {
    message.error("按键重复：" + curKey);
    return;
  }

  const keyMapping = store.editKeyMappingList[keyboardStore.activeButtonIndex];
  if (keyMapping.type === "SteeringWheel") {
    const keyObject = keyMapping.key as {
      left: string;
      right: string;
      up: string;
      down: string;
    };
    const nameList: ["up", "down", "left", "right"] = [
      "up",
      "down",
      "left",
      "right",
    ];
    const activeSteeringWheelButtonKeyIndex =
      keyboardStore.activeSteeringWheelButtonKeyIndex;
    if (
      activeSteeringWheelButtonKeyIndex >= 0 &&
      activeSteeringWheelButtonKeyIndex <= 3
    ) {
      const curName = nameList[activeSteeringWheelButtonKeyIndex];
      keyObject[curName] = curKey;
    }
  } else {
    keyMapping.key = curKey;
  }
  keyboardStore.edited = true;
}

function handleClick(event: MouseEvent) {
  if (event.button === 0) {
    // left click
    if (event.target === document.getElementById("keyboardElement")) {
      if (keyboardStore.showSettingFlag) {
        keyboardStore.showSettingFlag = false;
      } else {
        keyboardStore.activeButtonIndex = -1;
        keyboardStore.activeSteeringWheelButtonKeyIndex = -1;
        keyboardStore.showButtonSettingFlag = false;
      }
    }
  } else if (event.button === 2) {
    // right click
    if (event.target === document.getElementById("keyboardElement")) {
      // add button
      if (keyboardStore.showSettingFlag) keyboardStore.showSettingFlag = false;
      keyboardStore.activeButtonIndex = -1;
      keyboardStore.activeSteeringWheelButtonKeyIndex = -1;

      console.log("弹出新增");
    } else if (
      // modify key
      keyboardStore.activeButtonIndex !== -1 &&
      keyboardStore.activeButtonIndex < store.editKeyMappingList.length &&
      !keyboardStore.showButtonSettingFlag
    ) {
      const curKey = `M${event.button}`;
      setCurButtonKey(curKey);
    }
  } else {
    // other click
    event.preventDefault();
    if (
      keyboardStore.activeButtonIndex !== -1 &&
      keyboardStore.activeButtonIndex < store.editKeyMappingList.length &&
      !keyboardStore.showButtonSettingFlag
    ) {
      const curKey = `M${event.button}`;
      setCurButtonKey(curKey);
    }
  }
}

function handleKeyUp(event: KeyboardEvent) {
  if (
    keyboardStore.activeButtonIndex !== -1 &&
    keyboardStore.activeButtonIndex < store.editKeyMappingList.length &&
    !keyboardStore.showButtonSettingFlag
  ) {
    const curKey = event.code;
    setCurButtonKey(curKey);
  }
}

function handleMouseWheel(event: WheelEvent) {
  if (
    keyboardStore.activeButtonIndex !== -1 &&
    keyboardStore.activeButtonIndex < store.editKeyMappingList.length &&
    !keyboardStore.showButtonSettingFlag
  ) {
    if (event.deltaY > 0) {
      // WheelDown
      setCurButtonKey("WheelDown");
    } else if (event.deltaY < 0) {
      // WheelUp
      setCurButtonKey("WheelUp");
    }
  }
}

function resetEditKeyMappingList() {
  keyboardStore.showSettingFlag = false;
  keyboardStore.activeButtonIndex = -1;
  keyboardStore.activeSteeringWheelButtonKeyIndex = -1;
  store.resetEditKeyMappingList();
  keyboardStore.edited = false;
}

onActivated(() => {
  document.addEventListener("keyup", handleKeyUp);
  document.addEventListener("wheel", handleMouseWheel);
});

onBeforeRouteLeave(() => {
  document.removeEventListener("keyup", handleKeyUp);
  document.removeEventListener("wheel", handleMouseWheel);
  if (keyboardStore.edited) {
    dialog.warning({
      title: "Warning",
      content: "当前方案尚未保存，是否保存？",
      positiveText: "保存",
      negativeText: "取消",
      onPositiveClick: () => {
        if (store.applyEditKeyMappingList()) {
          keyboardStore.edited = false;
        } else {
          message.error("存在重复按键，无法保存");
        }
      },
      onNegativeClick: () => {
        resetEditKeyMappingList();
      },
    });
  }
});
</script>

<template>
  <div
    v-if="store.keyMappingConfigList.length"
    id="keyboardElement"
    class="keyboard"
    @mousedown="handleClick"
    @contextmenu.prevent
  >
    <KeySetting @reset-edit-key-mapping-list="resetEditKeyMappingList" />
    <KeyInfo />
    <template v-for="(_, index) in store.editKeyMappingList">
      <KeySteeringWheel
        v-if="store.editKeyMappingList[index].type === 'SteeringWheel'"
        :index="index"
      />
      <KeySkill
        v-else-if="
          store.editKeyMappingList[index].type === 'DirectionalSkill' ||
          store.editKeyMappingList[index].type === 'DirectionlessSkill' ||
          store.editKeyMappingList[index].type === 'TriggerWhenPressedSkill'
        "
        :index="index"
      />
      <KeyObservation
        v-else-if="store.editKeyMappingList[index].type === 'Observation'"
        :index="index"
      />
      <KeyCommon v-else :index="index" />
    </template>
  </div>
</template>

<style scoped lang="scss">
.keyboard {
  color: var(--light-color);
  background-color: rgba(0, 0, 0, 0.5);
  overflow: hidden;
  position: relative;
  user-select: none;
  -webkit-user-select: none;

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

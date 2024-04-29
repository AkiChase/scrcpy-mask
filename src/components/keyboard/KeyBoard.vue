<script setup lang="ts">
import { onActivated, ref } from "vue";
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

// TODO 按钮齿轮图标可修改
// TODO 普通按钮 KeyMacro，KeyCancelSkill，KeyTap
//      KeyMacro有输入框
// TODO 方向轮盘按钮 KeySteeringWheel
//      offset
// TODO 技能按钮 KeyDirectionalSkill，KeyDirectionlessSkill，KeyTriggerWhenPressedSkill（有区分directional)
//      靠两个flag来区分这四种情况，还有range或time要视情况
// TODO 添加视野按钮 KeyObservation
//      有灵敏度scale

// TODO 右键空白区域添加按键
// TODO 设置界面添加本地数据编辑器（类似utools）
// TODO 添加开发者工具打开按钮

const showKeyInfoFlag = ref(false);
const showSettingFlag = ref(false);
const showButtonSettingFlag = ref(false);
const store = useGlobalStore();
const dialog = useDialog();
const message = useMessage();

const activeButtonIndex = ref(-1);
const activeSteeringWheelButtonKeyIndex = ref(-1);
let edited = ref(false);

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

  const keyMapping = store.editKeyMappingList[activeButtonIndex.value];
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
    if (
      activeSteeringWheelButtonKeyIndex.value >= 0 &&
      activeSteeringWheelButtonKeyIndex.value <= 3
    ) {
      const curName = nameList[activeSteeringWheelButtonKeyIndex.value];
      keyObject[curName] = curKey;
    }
  } else {
    keyMapping.key = curKey;
  }
  edited.value = true;
}

function handleClick(event: MouseEvent) {
  if (event.button === 0) {
    // left click
    if (event.target === document.getElementById("keyboardElement")) {
      if (showSettingFlag.value) {
        showSettingFlag.value = false;
      } else {
        activeButtonIndex.value = -1;
        activeSteeringWheelButtonKeyIndex.value = -1;
        showButtonSettingFlag.value = false;
      }
    }
  } else if (event.button === 2) {
    // right click
    if (event.target === document.getElementById("keyboardElement")) {
      // add button
      if (showSettingFlag.value) showSettingFlag.value = false;
      activeButtonIndex.value = -1;
      activeSteeringWheelButtonKeyIndex.value = -1;

      console.log("弹出新增");
    } else if (
      // modify key
      activeButtonIndex.value !== -1 &&
      activeButtonIndex.value < store.editKeyMappingList.length &&
      !showButtonSettingFlag.value
    ) {
      const curKey = `M${event.button}`;
      setCurButtonKey(curKey);
    }
  } else {
    // other click
    event.preventDefault();
    if (
      activeButtonIndex.value !== -1 &&
      activeButtonIndex.value < store.editKeyMappingList.length &&
      !showButtonSettingFlag.value
    ) {
      const curKey = `M${event.button}`;
      setCurButtonKey(curKey);
    }
  }
}

function handleKeyUp(event: KeyboardEvent) {
  if (
    activeButtonIndex.value !== -1 &&
    activeButtonIndex.value < store.editKeyMappingList.length &&
    !showButtonSettingFlag.value
  ) {
    const curKey = event.code;
    setCurButtonKey(curKey);
  }
}

function handleMouseWheel(event: WheelEvent) {
  if (
    activeButtonIndex.value !== -1 &&
    activeButtonIndex.value < store.editKeyMappingList.length &&
    !showButtonSettingFlag.value
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
  showSettingFlag.value = false;
  activeButtonIndex.value = -1;
  activeSteeringWheelButtonKeyIndex.value = -1;
  store.resetEditKeyMappingList();
  edited.value = false;
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
        if (store.applyEditKeyMappingList()) {
          edited.value = false;
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
    <KeySetting
      v-model:show-key-info-flag="showKeyInfoFlag"
      v-model:show-setting-flag="showSettingFlag"
      v-model:edited="edited"
      @reset-edit-key-mapping-list="resetEditKeyMappingList"
    />
    <KeyInfo v-model:showKeyInfoFlag="showKeyInfoFlag" />
    <template v-for="(_, index) in store.editKeyMappingList">
      <KeySteeringWheel
        v-if="store.editKeyMappingList[index].type === 'SteeringWheel'"
        @edit="edited = true"
        :index="index"
        v-model:active-index="activeButtonIndex"
        v-model:active-steering-wheel-button-key-index="
          activeSteeringWheelButtonKeyIndex
        "
        v-model:show-button-setting-flag="showButtonSettingFlag"
      />
      <KeySkill
        v-else-if="
          store.editKeyMappingList[index].type === 'DirectionalSkill' ||
          store.editKeyMappingList[index].type === 'DirectionlessSkill' ||
          store.editKeyMappingList[index].type === 'TriggerWhenPressedSkill'
        "
        @edit="edited = true"
        :index="index"
        v-model:active-index="activeButtonIndex"
        v-model:show-button-setting-flag="showButtonSettingFlag"
      />
      <KeyObservation
        v-else-if="store.editKeyMappingList[index].type === 'Observation'"
        @edit="edited = true"
        :index="index"
        v-model:active-index="activeButtonIndex"
        v-model:show-button-setting-flag="showButtonSettingFlag"
      />
      <KeyCommon
        v-else
        @edit="edited = true"
        :index="index"
        v-model:active-index="activeButtonIndex"
        v-model:show-button-setting-flag="showButtonSettingFlag"
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

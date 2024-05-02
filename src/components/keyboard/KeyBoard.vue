<script setup lang="ts">
import { nextTick, onActivated, ref } from "vue";
import KeyInfo from "./KeyInfo.vue";
import KeySetting from "./KeySetting.vue";
import KeyCommon from "./KeyCommon.vue";
import KeySteeringWheel from "./KeySteeringWheel.vue";
import KeySkill from "./KeySkill.vue";
import KeyObservation from "./KeyObservation.vue";
import {
  KeyDirectionalSkill,
  KeySteeringWheel as KeyMappingSteeringWheel,
  KeyObservation as KeyMappingObservation,
  KeyTap,
  KeyMacro,
} from "../../keyMappingConfig";
import { useGlobalStore } from "../../store/global";
import { DropdownOption, NDropdown, useDialog, useMessage } from "naive-ui";
import { onBeforeRouteLeave } from "vue-router";
import { useKeyboardStore } from "../../store/keyboard";

const store = useGlobalStore();
const keyboardStore = useKeyboardStore();
const dialog = useDialog();
const message = useMessage();

const addButtonPos = ref({ x: 0, y: 0 });
const addButtonOptions: DropdownOption[] = [
  {
    label: "普通点击",
    key: "Tap",
  },
  {
    label: "键盘行走",
    key: "SteeringWheel",
  },
  {
    label: "技能",
    key: "DirectionalSkill",
  },
  {
    label: "技能取消",
    key: "CancelSkill",
  },
  {
    label: "观察视角",
    key: "Observation",
  },
  {
    label: "宏",
    key: "Macro",
  },
];

function onAddButtonSelect(
  type:
    | "Tap"
    | "SteeringWheel"
    | "DirectionalSkill"
    | "CancelSkill"
    | "Observation"
    | "Macro"
) {
  keyboardStore.showButtonAddFlag = false;
  const keyMapping = {
    type,
    key: "NONE",
    note: "",
    posX: addButtonPos.value.x - 70,
    posY: addButtonPos.value.y - 30,
    pointerId: 2, // default skill pointerId
  };
  if (type === "Tap") {
    (keyMapping as KeyTap).time = 80;
  } else if (type === "SteeringWheel") {
    (keyMapping as unknown as KeyMappingSteeringWheel).key = {
      left: "NONE",
      right: "NONE",
      up: "NONE",
      down: "NONE",
    };
  } else if (type === "DirectionalSkill") {
    (keyMapping as unknown as KeyDirectionalSkill).range = 30;
  } else if (type === "CancelSkill") {
    keyMapping.note = "取消技能";
  } else if (type === "Observation") {
    (keyMapping as unknown as KeyMappingObservation).scale = 0.6;
  } else if (type === "Macro") {
    (keyMapping as unknown as KeyMacro).macro = {
      down: null,
      loop: null,
      up: null,
    };
  } else return;
  keyboardStore.edited = true;
  store.editKeyMappingList.push(keyMapping);
}

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
  if (
    keyboardStore.activeButtonIndex === -1 ||
    keyboardStore.activeButtonIndex >= store.editKeyMappingList.length||
    keyboardStore.showButtonSettingFlag||
    keyboardStore.showButtonAddFlag
  )
    return;

  const keyMapping = store.editKeyMappingList[keyboardStore.activeButtonIndex];
  if (
    keyMapping.type === "SteeringWheel" &&
    keyboardStore.activeSteeringWheelButtonKeyIndex === -1
  )
    return;

  if (!isKeyUnique(curKey)) {
    message.error("按键重复：" + curKey);
    return;
  }

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
      keyboardStore.showSettingFlag = false;
      keyboardStore.activeButtonIndex = -1;
      keyboardStore.activeSteeringWheelButtonKeyIndex = -1;
      keyboardStore.showButtonAddFlag = true;

      keyboardStore.showButtonAddFlag = false;
      nextTick().then(() => {
        keyboardStore.showButtonAddFlag = true;
        addButtonPos.value.x = event.clientX;
        addButtonPos.value.y = event.clientY;
      });
    } else {
      setCurButtonKey(`M${event.button}`);
    }
  } else {
    // other click
    event.preventDefault();
    setCurButtonKey(`M${event.button}`);
  }
}

function handleKeyUp(event: KeyboardEvent) {
  setCurButtonKey(event.code);
}

function handleMouseWheel(event: WheelEvent) {
  if (event.deltaY > 0) {
    // WheelDown
    setCurButtonKey("WheelDown");
  } else if (event.deltaY < 0) {
    // WheelUp
    setCurButtonKey("WheelUp");
  }
}

function resetKeyMappingConfig() {
  keyboardStore.activeButtonIndex = -1;
  keyboardStore.activeSteeringWheelButtonKeyIndex = -1;
  keyboardStore.showSettingFlag = false;
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
        resetKeyMappingConfig();
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
    <KeySetting />
    <KeyInfo />
    <NDropdown
      :options="addButtonOptions"
      :show="keyboardStore.showButtonAddFlag"
      placement="bottom-start"
      trigger="manual"
      :x="addButtonPos.x"
      :y="addButtonPos.y"
      @clickoutside="keyboardStore.showButtonAddFlag = false"
      @select="onAddButtonSelect"
    />
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

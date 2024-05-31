<script setup lang="ts">
import { nextTick, onActivated, ref } from "vue";
import KeyInfo from "./KeyInfo.vue";
import KeySetting from "./KeySetting.vue";
import KeyCommon from "./KeyCommon.vue";
import KeySteeringWheel from "./KeySteeringWheel.vue";
import KeySkill from "./KeySkill.vue";
import KeyObservation from "./KeyObservation.vue";
import KeySight from "./KeySight.vue";
import KeyFire from "./KeyFire.vue";
import KeySwipe from "./KeySwipe.vue";
import ScreenStream from "../ScreenStream.vue";

import {
  KeyDirectionalSkill,
  KeySteeringWheel as KeyMappingSteeringWheel,
  KeyObservation as KeyMappingObservation,
  KeyTap,
  KeyMacro,
  KeyMapping,
  KeySwipe as KeyMappingKeySwipe,
  KeySight as KeyMappingKeySight,
  KeyFire as KeyMappingKeyFire,
} from "../../keyMappingConfig";
import { useGlobalStore } from "../../store/global";
import { DropdownOption, NDropdown, useDialog, useMessage } from "naive-ui";
import { onBeforeRouteLeave } from "vue-router";
import { useKeyboardStore } from "../../store/keyboard";
import { useI18n } from "vue-i18n";

const { t } = useI18n();
const store = useGlobalStore();
const keyboardStore = useKeyboardStore();
const dialog = useDialog();
const message = useMessage();

const curPageActive = ref(false);
const addButtonPos = ref({ x: 0, y: 0 });
const addButtonOptions: DropdownOption[] = [
  {
    label: () => t("pages.KeyBoard.addButton.Tap"),
    key: "Tap",
  },
  {
    label: () => t("pages.KeyBoard.addButton.SteeringWheel"),
    key: "SteeringWheel",
  },
  {
    label: () => t("pages.KeyBoard.addButton.Swipe"),
    key: "Swipe",
  },
  {
    label: () => t("pages.KeyBoard.addButton.Skill"),
    key: "DirectionalSkill",
  },
  {
    label: () => t("pages.KeyBoard.addButton.CancelSkill"),
    key: "CancelSkill",
  },
  {
    label: () => t("pages.KeyBoard.addButton.Observation"),
    key: "Observation",
  },
  {
    label: () => t("pages.KeyBoard.addButton.Macro"),
    key: "Macro",
  },
  {
    label: () => t("pages.KeyBoard.addButton.Sight"),
    key: "Sight",
  },
  {
    label: () => t("pages.KeyBoard.addButton.Fire"),
    key: "Fire",
  },
];

function onAddButtonSelect(
  type:
    | "Tap"
    | "Swipe"
    | "SteeringWheel"
    | "DirectionalSkill"
    | "CancelSkill"
    | "Observation"
    | "Macro"
    | "Sight"
    | "Fire"
) {
  keyboardStore.showButtonAddFlag = false;
  const keyMapping = {
    type,
    key: "NONE",
    note: "",
    posX: addButtonPos.value.x - 70,
    posY: addButtonPos.value.y - 30,
    pointerId: 2, // default skill and fire pointerId
  };
  if (type === "Tap") {
    keyMapping.pointerId = 3;
    (keyMapping as KeyTap).time = 80;
  } else if (type === "Swipe") {
    keyMapping.pointerId = 3;
    (keyMapping as KeyMappingKeySwipe).pos = [
      { x: keyMapping.posX, y: keyMapping.posY },
    ];
    (keyMapping as KeyMappingKeySwipe).intervalBetweenPos = 100;
  } else if (type === "SteeringWheel") {
    keyMapping.pointerId = 1;
    (keyMapping as unknown as KeyMappingSteeringWheel).key = {
      left: "NONE1",
      right: "NONE2",
      up: "NONE3",
      down: "NONE4",
    };
    (keyMapping as unknown as KeyMappingSteeringWheel).offset = 100;
  } else if (type === "DirectionalSkill") {
    (keyMapping as unknown as KeyDirectionalSkill).range = 30;
  } else if (type === "CancelSkill") {
    keyMapping.note = t("pages.KeyBoard.addButton.CancelSkill");
  } else if (type === "Observation") {
    keyMapping.pointerId = 4;
    (keyMapping as unknown as KeyMappingObservation).scale = 0.6;
  } else if (type === "Macro") {
    delete (keyMapping as any).pointerId;
    (keyMapping as unknown as KeyMacro).macro = {
      down: null,
      loop: null,
      up: null,
    };
  } else if (type === "Sight") {
    for (const mapping of store.editKeyMappingList) {
      if (mapping.type === "Sight") {
        message.error(t("pages.KeyBoard.addButton.existSight"));
        return;
      }
    }
    keyMapping.pointerId = 0;
    (keyMapping as unknown as KeyMappingKeySight).scaleX = 0.5;
    (keyMapping as unknown as KeyMappingKeySight).scaleY = 0.5;
  } else if (type === "Fire") {
    for (const mapping of store.editKeyMappingList) {
      if (mapping.type === "Fire") {
        message.error(t("pages.KeyBoard.addButton.existFire"));
        return;
      }
    }
    delete (keyMapping as any).key;
    (keyMapping as unknown as KeyMappingKeyFire).scaleX = 0.5;
    (keyMapping as unknown as KeyMappingKeyFire).scaleY = 0.5;
    (keyMapping as unknown as KeyMappingKeyFire).drag = false;
  } else return;
  keyboardStore.edited = true;
  store.editKeyMappingList.push(keyMapping as KeyMapping);
  keyboardStore.activeButtonIndex = store.editKeyMappingList.length - 1;
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
    } else if (keyMapping.type !== "Fire") {
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
    keyboardStore.activeButtonIndex >= store.editKeyMappingList.length ||
    keyboardStore.showButtonSettingFlag ||
    keyboardStore.activeButtonIndex >= store.editKeyMappingList.length ||
    keyboardStore.showButtonSettingFlag ||
    keyboardStore.editSwipePointsFlag ||
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
    message.error(t("pages.KeyBoard.buttonKeyRepeat", [curKey]));
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
    keyboardStore.edited = true;
  } else if (keyMapping.type !== "Fire") {
    keyMapping.key = curKey;
    keyboardStore.edited = true;
  }
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

function handleKeyDown(event: KeyboardEvent) {
  // prevent F1-F12
  if (/^F(1[0-2]|[1-9])$/.test(event.code)) event.preventDefault();
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
  curPageActive.value = true;
  document.addEventListener("keydown", handleKeyDown);
  document.addEventListener("keyup", handleKeyUp);
  document.addEventListener("wheel", handleMouseWheel);
});

onBeforeRouteLeave(() => {
  curPageActive.value = false;
  return new Promise((resolve, _) => {
    document.removeEventListener("keydown", handleKeyDown);
    document.removeEventListener("keyup", handleKeyUp);
    document.removeEventListener("wheel", handleMouseWheel);
    if (keyboardStore.edited) {
      dialog.warning({
        title: t("pages.KeyBoard.noSaveDialog.title"),
        content: t("pages.KeyBoard.noSaveDialog.content"),
        positiveText: t("pages.KeyBoard.noSaveDialog.positiveText"),
        negativeText: t("pages.KeyBoard.noSaveDialog.negativeText"),
        onPositiveClick: () => {
          if (store.applyEditKeyMappingList()) {
            keyboardStore.edited = false;
            resolve(true);
          } else {
            message.error(t("pages.KeyBoard.noSaveDialog.keyRepeat"));
            resolve(false);
          }
        },
        onNegativeClick: () => {
          resetKeyMappingConfig();
          resolve(true);
        },
      });
    } else resolve(true);
  });
});
</script>

<template>
  <ScreenStream
    :cid="store.screenStreamClientId"
    v-if="curPageActive && store.controledDevice && store.screenStream.enable"
  />
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
          store.editKeyMappingList[index].type === 'TriggerWhenPressedSkill' ||
          store.editKeyMappingList[index].type ===
            'TriggerWhenDoublePressedSkill'
        "
        :index="index"
      />
      <KeyObservation
        v-else-if="store.editKeyMappingList[index].type === 'Observation'"
        :index="index"
      />
      <KeySwipe
        v-else-if="store.editKeyMappingList[index].type === 'Swipe'"
        :index="index"
      />
      <KeySight
        v-else-if="store.editKeyMappingList[index].type === 'Sight'"
        :index="index"
      />
      <KeyFire
        v-else-if="store.editKeyMappingList[index].type === 'Fire'"
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

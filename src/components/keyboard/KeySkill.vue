<script setup lang="ts">
import { computed, ref } from "vue";
import { useGlobalStore } from "../../store/global";
import { Flash, CloseCircle, Settings } from "@vicons/ionicons5";
import {
  NIcon,
  NButton,
  NH4,
  NFormItem,
  NInput,
  NInputNumber,
  NCheckbox,
  NFlex,
} from "naive-ui";
import {
  KeyDirectionalSkill,
  KeySkill,
  KeyTriggerWhenPressedSkill,
} from "../../tools/keyMappingConfig";
import { useKeyboardStore } from "../../store/keyboard";
import { configKeySkill } from "./config";

const props = defineProps<{
  index: number;
}>();

const keyboardStore = useKeyboardStore();

const store = useGlobalStore();
const elementRef = ref<HTMLElement | null>(null);

const isActive = computed(
  () => props.index === keyboardStore.activeButtonIndex
);
const keyMapping = computed(
  () => store.editKeyMappingList[props.index] as KeySkill
);

function dragHandler(downEvent: MouseEvent) {
  keyboardStore.activeButtonIndex = props.index;
  keyboardStore.showButtonSettingFlag = false;
  const oldX = keyMapping.value.posX;
  const oldY = keyMapping.value.posY;
  const element = elementRef.value;
  if (element) {
    const keyboardElement = document.getElementById(
      "keyboardElement"
    ) as HTMLElement;
    const maxX = keyboardElement.clientWidth - 30;
    const maxY = keyboardElement.clientHeight - 30;

    const x = downEvent.clientX;
    const y = downEvent.clientY;
    const moveHandler = (moveEvent: MouseEvent) => {
      let newX = oldX + moveEvent.clientX - x;
      let newY = oldY + moveEvent.clientY - y;
      newX = Math.max(30, Math.min(newX, maxX));
      newY = Math.max(30, Math.min(newY, maxY));
      keyMapping.value.posX = newX;
      keyMapping.value.posY = newY;
    };
    window.addEventListener("mousemove", moveHandler);
    const upHandler = () => {
      window.removeEventListener("mousemove", moveHandler);
      window.removeEventListener("mouseup", upHandler);
      if (oldX !== keyMapping.value.posX || oldY !== keyMapping.value.posY) {
        keyboardStore.edited = true;
      }
    };
    window.addEventListener("mouseup", upHandler);
  }
}

function delCurKeyMapping() {
  keyboardStore.edited = true;

  keyboardStore.activeButtonIndex = -1;
  store.editKeyMappingList.splice(props.index, 1);
}

const isDirectionless = computed(
  () =>
    keyMapping.value.type === "DirectionlessSkill" ||
    (keyMapping.value.type === "TriggerWhenPressedSkill" &&
      !(keyMapping.value as KeyTriggerWhenPressedSkill).directional)
);

const isTriggerWhenPressed = computed(
  () => keyMapping.value.type === "TriggerWhenPressedSkill"
);

const isTriggerWhenDoublePressed = computed(
  () => keyMapping.value.type === "TriggerWhenDoublePressedSkill"
);

function changeSkillType(flag: string) {
  // the design of skill keymapping type is not good
  const t = keyMapping.value.type;
  if (flag === "direction") {
    keyboardStore.edited = true;
    if (t === "DirectionalSkill") {
      // to DirectionlessSkill
      const k = keyMapping.value as any;
      delete k.range;
      k.type = "DirectionlessSkill";
    } else if (t === "DirectionlessSkill") {
      // to DirectionalSkill
      const k = keyMapping.value as any;
      k.range = 0;
      k.type = "DirectionalSkill";
    } else if (t === "TriggerWhenPressedSkill") {
      // change directional flag
      const k = keyMapping.value as KeyTriggerWhenPressedSkill;
      k.directional = !k.directional;
      k.rangeOrTime = k.directional ? 0 : 80;
    } else if (t === "TriggerWhenDoublePressedSkill") {
      // to DirectionlessSkill
      const k = keyMapping.value as any;
      delete k.range;
      k.type = "DirectionlessSkill";
    }
  } else if (flag === "trigger") {
    keyboardStore.edited = true;
    if (t === "DirectionalSkill") {
      const k = keyMapping.value as any;
      k.directional = true;
      k.rangeOrTime = k.range;
      delete k.range;
      k.type = "TriggerWhenPressedSkill";
    } else if (t === "DirectionlessSkill") {
      const k = keyMapping.value as any;
      k.directional = false;
      k.rangeOrTime = 80; // touch time
      k.type = "TriggerWhenPressedSkill";
    } else if (t === "TriggerWhenPressedSkill") {
      // to DirectionalSkill or DirectionlessSkill
      const k = keyMapping.value as any;
      if (k.directional) {
        k.range = k.rangeOrTime;
        delete k.rangeOrTime;
        k.type = "DirectionalSkill";
      } else {
        delete k.rangeOrTime;
        k.type = "DirectionlessSkill";
      }
      delete k.directional;
    } else if (t === "TriggerWhenDoublePressedSkill") {
      // to TriggerWhenPressedSkill && directional
      const k = keyMapping.value as any;
      k.directional = true;
      k.rangeOrTime = k.range;
      delete k.range;
      k.type = "TriggerWhenPressedSkill";
    }
  } else if (flag === "trigger-double") {
    keyboardStore.edited = true;
    if (t === "DirectionalSkill") {
      // to TriggerWhenDoublePressedSkill
      const k = keyMapping.value as any;
      k.type = "TriggerWhenDoublePressedSkill";
    } else if (t === "DirectionlessSkill") {
      // to TriggerWhenDoublePressedSkill
      const k = keyMapping.value as any;
      k.range = 0;
      k.type = "TriggerWhenDoublePressedSkill";
    } else if (t === "TriggerWhenPressedSkill") {
      // to TriggerWhenDoublePressedSkill
      const k = keyMapping.value as any;
      k.range = k.directional ? k.rangeOrTime : 0;
      delete k.rangeOrTime;
      k.type = "TriggerWhenDoublePressedSkill";
    } else if (t === "TriggerWhenDoublePressedSkill") {
      // to DirectionalSkill
      const k = keyMapping.value as any;
      k.type = "DirectionalSkill";
    }
  }
}

const settingPosX = ref(0);
const settingPosY = ref(0);

function showSetting() {
  const keyboardElement = document.getElementById(
    "keyboardElement"
  ) as HTMLElement;
  // setting
  const maxWidth = keyboardElement.clientWidth - configKeySkill.settingW;
  const maxHeight = keyboardElement.clientHeight - configKeySkill.settingH;
  settingPosX.value = Math.min(keyMapping.value.posX + 40, maxWidth);
  settingPosY.value = Math.min(keyMapping.value.posY - 40, maxHeight);
  updateRangeIndicator(keyboardElement);
  keyboardStore.showButtonSettingFlag = true;
}

const rangeIndicatorTop = ref(0);
const indicatorLength = ref(0);
function updateRangeIndicator(element?: HTMLElement) {
  if (!element)
    element = document.getElementById("keyboardElement") as HTMLElement;

  if (!isDirectionless.value) {
    // indicator
    const range =
      keyMapping.value.type === "DirectionalSkill"
        ? (keyMapping.value as KeyDirectionalSkill).range
        : (keyMapping.value as KeyTriggerWhenPressedSkill).rangeOrTime;
    indicatorLength.value = Math.round(
      ((element.clientHeight * range) / 100) * 2
    );
    rangeIndicatorTop.value = Math.round(
      element.clientHeight / 2 - indicatorLength.value / 4
    );
  }
}
</script>

<template>
  <div
    :class="{ active: isActive }"
    :style="{
      left: `${keyMapping.posX - 30}px`,
      top: `${keyMapping.posY - 30}px`,
    }"
    @mousedown="dragHandler"
    class="key-skill"
    ref="elementRef"
  >
    <NIcon size="25"><Flash /></NIcon>
    <span>{{ keyMapping.key }}</span>
    <NButton
      class="key-close-btn"
      text
      @click="delCurKeyMapping"
      :type="isActive ? 'primary' : 'info'"
    >
      <template #icon>
        <NIcon size="15">
          <CloseCircle />
        </NIcon>
      </template>
    </NButton>
    <NButton
      class="key-setting-btn"
      text
      @click="showSetting"
      :type="isActive ? 'primary' : 'info'"
    >
      <template #icon>
        <NIcon size="15">
          <Settings />
        </NIcon>
      </template>
    </NButton>
  </div>
  <div
    class="key-setting"
    v-if="isActive && keyboardStore.showButtonSettingFlag"
    :style="{
      left: `${settingPosX}px`,
      top: `${settingPosY}px`,
      width: `${configKeySkill.settingW}px`,
      height: `${configKeySkill.settingH}px`,
    }"
  >
    <NH4 prefix="bar">{{ $t("pages.KeyBoard.KeySkill.skill") }}</NH4>
    <NFormItem :label="$t('pages.KeyBoard.KeySkill.options')">
      <NFlex vertical>
        <NCheckbox
          @click="changeSkillType('trigger-double')"
          :checked="isTriggerWhenDoublePressed"
          >{{ $t("pages.KeyBoard.KeySkill.double") }}</NCheckbox
        >
        <NCheckbox
          @click="changeSkillType('direction')"
          :checked="isDirectionless"
          >{{ $t("pages.KeyBoard.KeySkill.directionless") }}</NCheckbox
        >
        <NCheckbox
          @click="changeSkillType('trigger')"
          :checked="isTriggerWhenPressed"
          >{{ $t("pages.KeyBoard.KeySkill.triggerWhenPressed") }}</NCheckbox
        >
      </NFlex>
    </NFormItem>
    <NFormItem
      v-if="!isDirectionless"
      :label="$t('pages.KeyBoard.KeySkill.range')"
    >
      <NInputNumber
        v-if="keyMapping.type === 'DirectionalSkill'"
        v-model:value="keyMapping.range"
        placeholder="range"
        :min="0"
        :max="100"
        @update:value="
          keyboardStore.edited = true;
          updateRangeIndicator();
        "
      />
      <NInputNumber
        v-else-if="keyMapping.type === 'TriggerWhenPressedSkill'"
        v-model:value="keyMapping.rangeOrTime"
        placeholder="rangeOrTime"
        :min="0"
        :max="100"
        @update:value="
          keyboardStore.edited = true;
          updateRangeIndicator();
        "
      />
      <NInputNumber
        v-else-if="keyMapping.type === 'TriggerWhenDoublePressedSkill'"
        v-model:value="keyMapping.range"
        placeholder="range"
        :min="0"
        :max="100"
        @update:value="
          keyboardStore.edited = true;
          updateRangeIndicator();
        "
      />
    </NFormItem>
    <NFormItem
      v-if="(keyMapping.type==='TriggerWhenPressedSkill'&&!(keyMapping as KeyTriggerWhenPressedSkill).directional)"
      :label="$t('pages.KeyBoard.setting.touchTime')"
    >
      <NInputNumber
        v-model:value="keyMapping.rangeOrTime"
        :min="0"
        :placeholder="$t('pages.KeyBoard.setting.touchTimePlaceholder')"
        @update:value="keyboardStore.edited = true"
      />
    </NFormItem>
    <NFormItem :label="$t('pages.KeyBoard.setting.pointerID')">
      <NInputNumber
        v-model:value="keyMapping.pointerId"
        :min="0"
        :placeholder="$t('pages.KeyBoard.setting.pointerIDPlaceholder')"
        @update:value="keyboardStore.edited = true"
      />
    </NFormItem>
    <NFormItem :label="$t('pages.KeyBoard.setting.note')">
      <NInput
        v-model:value="keyMapping.note"
        :placeholder="$t('pages.KeyBoard.setting.notePlaceholder')"
        @update:value="keyboardStore.edited = true"
      />
    </NFormItem>
  </div>
  <div
    v-if="isActive && keyboardStore.showButtonSettingFlag"
    class="range-indicator"
    :style="{
      top: `${rangeIndicatorTop}px`,
      width: `${indicatorLength}px`,
      height: `${indicatorLength}px`,
    }"
  ></div>
</template>

<style scoped lang="scss">
.range-indicator {
  position: absolute;
  left: 0;
  right: 0;
  margin: auto;
  border-radius: 50%;
  background-color: var(--blue-color);
  clip-path: polygon(0 0, 100% 0, 50% 25%);
}

.key-setting {
  position: absolute;
  display: flex;
  flex-direction: column;
  padding: 10px 20px;
  box-sizing: border-box;
  border-radius: 5px;
  border: 2px solid var(--light-color);
  background-color: var(--bg-color);
  z-index: 3;
}

.key-skill {
  position: absolute;
  height: 60px;
  width: 60px;
  border-radius: 50%;
  box-sizing: border-box;
  border: 2px solid var(--blue-color);
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  font-size: 10px;
  font-weight: bold;
  cursor: pointer;

  .n-icon {
    color: var(--blue-color);
  }

  &:not(.active):hover {
    border: 2px solid var(--light-color);
    color: var(--light-color);

    .n-icon {
      color: var(--light-color);
    }
  }

  .key-close-btn {
    position: absolute;
    left: 65px;
    bottom: 45px;
  }

  .key-setting-btn {
    position: absolute;
    left: 65px;
    top: 45px;
  }
}

.active {
  border: 2px solid var(--primary-color);
  color: var(--primary-color);
  z-index: 2;

  .n-icon {
    color: var(--primary-color);
  }
}
</style>

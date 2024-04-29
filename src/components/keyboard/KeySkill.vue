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
  KeyTriggerWhenPressedSkill,
} from "../../keyMappingConfig";
const emit = defineEmits<{
  edit: [];
}>();

const props = defineProps<{
  index: number;
}>();

const activeIndex = defineModel("activeIndex", { required: true });
const showButtonSettingFlag = defineModel("showButtonSettingFlag", {
  required: true,
});

const store = useGlobalStore();
const elementRef = ref<HTMLElement | null>(null);

const isActive = computed(() => props.index === activeIndex.value);
const keyMapping = computed(() => store.editKeyMappingList[props.index]);

function dragHandler(downEvent: MouseEvent) {
  activeIndex.value = props.index;
  showButtonSettingFlag.value = false;
  const oldX = keyMapping.value.posX;
  const oldY = keyMapping.value.posY;
  const element = elementRef.value;
  if (element) {
    const keyboardElement = document.getElementById(
      "keyboardElement"
    ) as HTMLElement;
    const maxX = keyboardElement.clientWidth - 60;
    const maxY = keyboardElement.clientHeight - 60;

    const x = downEvent.clientX;
    const y = downEvent.clientY;
    const moveHandler = (moveEvent: MouseEvent) => {
      let newX = oldX + moveEvent.clientX - x;
      let newY = oldY + moveEvent.clientY - y;
      newX = Math.max(0, Math.min(newX, maxX));
      newY = Math.max(0, Math.min(newY, maxY));
      keyMapping.value.posX = newX;
      keyMapping.value.posY = newY;
    };
    window.addEventListener("mousemove", moveHandler);
    const upHandler = () => {
      window.removeEventListener("mousemove", moveHandler);
      window.removeEventListener("mouseup", upHandler);
      if (oldX !== keyMapping.value.posX || oldY !== keyMapping.value.posY) {
        emit("edit");
      }
    };
    window.addEventListener("mouseup", upHandler);
  }
}

function delCurKeyMapping() {
  emit("edit");
  activeIndex.value = -1;
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

function changeSkillType(flag: string) {
  // the design of skill keymapping type is not good
  const t = keyMapping.value.type;
  if (flag === "direction") {
    emit("edit");
    if (t === "DirectionalSkill") {
      delete (keyMapping.value as any).range;
      keyMapping.value.type = "DirectionlessSkill";
    } else if (t === "DirectionlessSkill") {
      (keyMapping.value as any).range = 0;
      keyMapping.value.type = "DirectionalSkill";
    } else {
      const k = keyMapping.value as KeyTriggerWhenPressedSkill;
      k.directional = !k.directional;
      k.rangeOrTime = k.directional ? 0 : 80;
    }
  } else if (flag === "trigger") {
    emit("edit");
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
    } else {
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
    }
  }
}

const settingPosX = ref(0);
const settingPosY = ref(0);
function showSetting() {
  const keyboardElement = document.getElementById(
    "keyboardElement"
  ) as HTMLElement;
  const maxWidth = keyboardElement.clientWidth - 200;
  const maxHeight = keyboardElement.clientHeight - 350;

  settingPosX.value = Math.min(keyMapping.value.posX + 40, maxWidth);
  settingPosY.value = Math.min(keyMapping.value.posY - 30, maxHeight);
  showButtonSettingFlag.value = true;
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
    v-if="isActive && showButtonSettingFlag"
    :style="{
      left: `${settingPosX}px`,
      top: `${settingPosY}px`,
    }"
  >
    <NH4 prefix="bar">技能</NH4>
    <NFormItem label="选项">
      <NFlex vertical>
        <NCheckbox
          @click="changeSkillType('direction')"
          :checked="isDirectionless"
          >无方向技能</NCheckbox
        >
        <NCheckbox
          @click="changeSkillType('trigger')"
          :checked="isTriggerWhenPressed"
          >按下时触发</NCheckbox
        >
      </NFlex>
    </NFormItem>
    <NFormItem v-if="!isDirectionless" label="范围">
      <NInputNumber
        v-if="keyMapping.type === 'DirectionalSkill'"
        v-model:value="(keyMapping as KeyDirectionalSkill).range"
        placeholder="请输入技能范围"
        :min="0"
        :max="100"
        @update:value="emit('edit')"
      />
      <NInputNumber
        v-else
        v-model:value="(keyMapping as KeyTriggerWhenPressedSkill).rangeOrTime"
        placeholder="请输入技能范围"
        :min="0"
        :max="100"
        @update:value="emit('edit')"
      />
    </NFormItem>
    <NFormItem
      v-if="(keyMapping.type==='TriggerWhenPressedSkill'&&!(keyMapping as KeyTriggerWhenPressedSkill).directional)"
      label="触摸时长"
    >
      <NInputNumber
        v-model:value="(keyMapping as KeyTriggerWhenPressedSkill).rangeOrTime"
        :min="0"
        placeholder="请输入触摸时长(ms)"
        @update:value="emit('edit')"
      />
    </NFormItem>
    <NFormItem label="备注">
      <NInput
        v-model:value="keyMapping.note"
        placeholder="请输入备注"
        @update:value="emit('edit')"
      />
    </NFormItem>
  </div>
</template>

<style scoped lang="scss">
.key-setting {
  position: absolute;
  display: flex;
  flex-direction: column;
  padding: 10px 20px;
  box-sizing: border-box;
  width: 200px;
  height: 350px;
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

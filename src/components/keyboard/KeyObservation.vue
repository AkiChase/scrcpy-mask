<script setup lang="ts">
import { computed, ref } from "vue";
import { useGlobalStore } from "../../store/global";
import { NIcon, NButton, NFormItem, NInput, NH4, NInputNumber } from "naive-ui";
import { Eye, CloseCircle, Settings } from "@vicons/ionicons5";
import { KeyObservation } from "../../keyMappingConfig";

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
const keyMapping = computed(
  () => store.editKeyMappingList[props.index] as KeyObservation
);

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

const settingPosX = ref(0);
const settingPosY = ref(0);
function showSetting() {
  const keyboardElement = document.getElementById(
    "keyboardElement"
  ) as HTMLElement;
  const maxWidth = keyboardElement.clientWidth - 150;
  const maxHeight = keyboardElement.clientHeight - 220;

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
    class="key-observation"
    ref="elementRef"
  >
    <NIcon size="25"><Eye /></NIcon>
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
    <NH4 prefix="bar">观察视角</NH4>
    <NFormItem label="灵敏度">
      <NInputNumber
        v-model:value="keyMapping.scale"
        placeholder="请输入灵敏度"
        :step="0.1"
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
  width: 150px;
  height: 220px;
  border-radius: 5px;
  border: 2px solid var(--light-color);
  background-color: var(--bg-color);
  z-index: 3;
}

.key-observation {
  position: absolute;
  height: 60px;
  width: 60px;
  box-sizing: border-box;
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

<script setup lang="ts">
import { computed, ref } from "vue";
import { useGlobalStore } from "../../store/global";
import { NIcon, NButton, NFormItem, NInput, NH4, NInputNumber } from "naive-ui";
import { CloseCircle, Settings } from "@vicons/ionicons5";
import { KeySight } from "../../keyMappingConfig";
import { useKeyboardStore } from "../../store/keyboard";

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
  () => store.editKeyMappingList[props.index] as KeySight
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

const settingPosX = ref(0);
const settingPosY = ref(0);
function showSetting() {
  const keyboardElement = document.getElementById(
    "keyboardElement"
  ) as HTMLElement;
  const maxWidth = keyboardElement.clientWidth - 150;
  const maxHeight = keyboardElement.clientHeight - 380;

  settingPosX.value = Math.min(keyMapping.value.posX + 40, maxWidth);
  settingPosY.value = Math.min(keyMapping.value.posY - 30, maxHeight);
  keyboardStore.showButtonSettingFlag = true;
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
    class="key-sight"
    ref="elementRef"
  >
    <NIcon size="25">
      <svg
        viewBox="0 0 1024 1024"
        version="1.1"
        xmlns="http://www.w3.org/2000/svg"
      >
        <path
          d="M65.472 479.232A448 448 0 0 1 481.28 64.32V32a32 32 0 1 1 64 0v32.448a448 448 0 0 1 413.952 415.808h33.152a32 32 0 1 1 0 64h-33.28a448.064 448.064 0 0 1-414.784 413.888v33.28a32 32 0 1 1-64 0v-33.28a448.064 448.064 0 0 1-414.912-414.912H32a32 32 0 1 1 0-64h33.472z m64.192 0h94.72a32 32 0 0 1 0 64h-94.72a384.064 384.064 0 0 0 350.656 350.72V800a32 32 0 0 1 64 0v93.952a384.128 384.128 0 0 0 350.592-349.632h-94.72a32 32 0 1 1 0-64h94.848A384 384 0 0 0 545.28 128.64v94.272a32 32 0 0 1-64 0V128.512a383.744 383.744 0 0 0-351.616 350.72z m318.656 32a64 64 0 1 1 128 0 64 64 0 0 1-128 0z"
        ></path>
      </svg>
    </NIcon>
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
    }"
  >
    <NH4 prefix="bar">{{ $t('pages.KeyBoard.KeySight.sight') }}</NH4>
    <NFormItem :label="$t('pages.KeyBoard.KeySight.scaleX')">
      <NInputNumber
        v-model:value="keyMapping.scaleX"
        :placeholder="$t('pages.KeyBoard.KeySight.scalePlaceholder')"
        :show-button="false"
        @update:value="keyboardStore.edited = true"
      />
    </NFormItem>
    <NFormItem :label="$t('pages.KeyBoard.KeySight.scaleY')">
      <NInputNumber
        v-model:value="keyMapping.scaleY"
        :placeholder="$t('pages.KeyBoard.KeySight.scalePlaceholder')"
        :show-button="false"
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
</template>

<style scoped lang="scss">
.key-setting {
  position: absolute;
  display: flex;
  flex-direction: column;
  padding: 10px 20px;
  box-sizing: border-box;
  width: 150px;
  height: 380px;
  border-radius: 5px;
  border: 2px solid var(--light-color);
  background-color: var(--bg-color);
  z-index: 3;
}

.key-sight {
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

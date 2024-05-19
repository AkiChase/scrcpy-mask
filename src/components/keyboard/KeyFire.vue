<script setup lang="ts">
import { computed, ref } from "vue";
import { useGlobalStore } from "../../store/global";
import {
  NIcon,
  NButton,
  NFormItem,
  NInput,
  NH4,
  NInputNumber,
  NCheckbox,
} from "naive-ui";
import { CloseCircle, Settings } from "@vicons/ionicons5";
import { KeyFire } from "../../keyMappingConfig";
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
  () => store.editKeyMappingList[props.index] as KeyFire
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
  const maxWidth = keyboardElement.clientWidth - 200;
  const maxHeight = keyboardElement.clientHeight - 430;

  settingPosX.value = Math.min(keyMapping.value.posX + 40, maxWidth);
  settingPosY.value = Math.min(keyMapping.value.posY - 40, maxHeight);
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
    class="key-fire"
    ref="elementRef"
  >
    <NIcon size="25">
      <svg
        viewBox="0 0 1024 1024"
        version="1.1"
        xmlns="http://www.w3.org/2000/svg"
      >
        <path
          d="M562.333538 310.665846a43.716923 43.716923 0 0 0-59.746461-1.181538l-301.528615 276.440615a39.424 39.424 0 0 0-1.181539 57.265231l190.070154 190.227692a39.345231 39.345231 0 0 0 57.225846-1.339077l276.558769-301.528615a43.598769 43.598769 0 0 0-1.260307-59.707077l-160.137847-160.177231zM149.385846 663.236923a41.550769 41.550769 0 0 0-58.564923 0 41.550769 41.550769 0 0 0 0 58.525539l222.641231 222.601846a41.432615 41.432615 0 0 0 58.525538-58.564923L149.385846 663.236923zM879.143385 118.350769c-63.015385-1.851077-195.465846 8.073846-281.796923 109.331693-1.457231 2.953846-15.596308 31.586462 6.222769 53.563076l150.173538 151.000616c5.710769 5.435077 24.339692 19.298462 53.326769 2.953846 40.093538-38.4 109.686154-127.606154 108.819693-282.269538-0.787692-28.750769-26.190769-33.831385-36.745846-34.579693z"
        ></path>
      </svg>
    </NIcon>
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
    <NH4 prefix="bar">{{ $t("pages.KeyBoard.KeyFire.fire") }}</NH4>
    <NCheckbox
      @click="keyMapping.drag = !keyMapping.drag"
      :checked="keyMapping.drag"
      style="margin-bottom: 20px"
      >{{ $t("pages.KeyBoard.KeyFire.drag") }}</NCheckbox
    >
    <NFormItem :label="$t('pages.KeyBoard.KeyFire.scaleX')">
      <NInputNumber
        v-model:value="keyMapping.scaleX"
        :placeholder="$t('pages.KeyBoard.KeyFire.scalePlaceholder')"
        :show-button="false"
        @update:value="keyboardStore.edited = true"
      />
    </NFormItem>
    <NFormItem :label="$t('pages.KeyBoard.KeyFire.scaleY')">
      <NInputNumber
        v-model:value="keyMapping.scaleY"
        :placeholder="$t('pages.KeyBoard.KeyFire.scalePlaceholder')"
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
  width: 200px;
  height: 430px;
  border-radius: 5px;
  border: 2px solid var(--light-color);
  background-color: var(--bg-color);
  z-index: 3;
}

.key-fire {
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

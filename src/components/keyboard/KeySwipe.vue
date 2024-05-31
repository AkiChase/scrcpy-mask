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
  useMessage,
} from "naive-ui";
import { Analytics, CloseCircle, Settings } from "@vicons/ionicons5";
import { useKeyboardStore } from "../../store/keyboard";
import { KeySwipe } from "../../keyMappingConfig";
import { useI18n } from "vue-i18n";

const props = defineProps<{
  index: number;
}>();

const keyboardStore = useKeyboardStore();
const message = useMessage();
const store = useGlobalStore();
const { t } = useI18n();

const elementRef = ref<HTMLElement | null>(null);

const isActive = computed(
  () => props.index === keyboardStore.activeButtonIndex
);
const keyMapping = computed(
  () => store.editKeyMappingList[props.index] as KeySwipe
);

const trackPoints = computed(() => {
  let s = "";
  if (isActive.value) {
    for (const point of keyMapping.value.pos) {
      s += `${point.x},${point.y} `;
    }
  }
  return s;
});

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
  settingPosY.value = Math.min(keyMapping.value.posY - 40, maxHeight);
  keyboardStore.showButtonSettingFlag = true;
}

function editSwipePoints() {
  message.info(t("pages.KeyBoard.Swipe.editTips"));
  keyboardStore.showButtonSettingFlag = false;
  keyboardStore.editSwipePointsFlag = true;
  keyboardStore.edited = true;
}

function swipePointDragHandlue(downEvent: MouseEvent, index: number) {
  if (downEvent.button === 2) {
    // del point
    keyMapping.value.pos.splice(index, 1);
    return;
  }
  if (downEvent.button !== 0) return;

  const oldX = keyMapping.value.pos[index].x;
  const oldY = keyMapping.value.pos[index].y;
  const keyboardElement = document.getElementById(
    "keyboardElement"
  ) as HTMLElement;
  const maxX = keyboardElement.clientWidth;
  const maxY = keyboardElement.clientHeight;

  const x = downEvent.clientX;
  const y = downEvent.clientY;
  const moveHandler = (moveEvent: MouseEvent) => {
    let newX = oldX + moveEvent.clientX - x;
    let newY = oldY + moveEvent.clientY - y;
    newX = Math.max(0, Math.min(newX, maxX));
    newY = Math.max(0, Math.min(newY, maxY));
    keyMapping.value.pos[index].x = newX;
    keyMapping.value.pos[index].y = newY;
  };
  const upHandler = () => {
    window.removeEventListener("mousemove", moveHandler);
    window.removeEventListener("mouseup", upHandler);
  };
  window.addEventListener("mousemove", moveHandler);
  window.addEventListener("mouseup", upHandler);
}

function swipeTrackClickHandler(event: MouseEvent) {
  if (event.button !== 0) return;
  console.log(event.target, event.currentTarget);
  if (event.target !== event.currentTarget) return;
  keyMapping.value.pos.push({ x: event.clientX - 70, y: event.clientY - 30 });
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
    class="key-swipe"
    ref="elementRef"
  >
    <NIcon size="30"><Analytics /></NIcon>
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
    <NH4 prefix="bar">{{ $t("pages.KeyBoard.Swipe.swipe") }}</NH4>
    <NFormItem :label="$t('pages.KeyBoard.Swipe.pos')">
      <NButton type="success" @click="editSwipePoints">{{
        $t("pages.KeyBoard.Swipe.editPos")
      }}</NButton>
    </NFormItem>
    <NFormItem :label="$t('pages.KeyBoard.Swipe.interval')">
      <NInputNumber
        v-model:value="keyMapping.intervalBetweenPos"
        :placeholder="$t('pages.KeyBoard.Swipe.intervalPlaceholder')"
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

  <template v-if="isActive">
    <div
      v-if="isActive"
      class="track"
      :class="{ 'edit-track': keyboardStore.editSwipePointsFlag }"
    >
      <svg @click="swipeTrackClickHandler">
        <polyline :points="trackPoints" />
        <circle
          v-for="(pos, i) in keyMapping.pos"
          :cx="pos.x"
          :cy="pos.y"
          r="5"
          @mousedown="(e) => swipePointDragHandlue(e, i)"
        />
        <text v-for="(pos, i) in keyMapping.pos" :x="pos.x + 5" :y="pos.y - 5">
          {{ i }}
        </text>
      </svg>
    </div>
  </template>
</template>

<style scoped lang="scss">
.track {
  position: absolute;
  left: 0;
  right: 0;
  top: 0;
  bottom: 0;
  z-index: -1;

  svg {
    height: 100%;
    width: 100%;

    polyline {
      fill: none;
      stroke: var(--primary-hover-color);
      stroke-width: 2;
    }

    circle {
      cursor: pointer;
      fill: var(--primary-color);

      &:hover {
        fill: var(--primary-pressed-color);
      }
    }

    text {
      cursor: default;
      fill: var(--primary-pressed-color);
      font-size: 15px;
      text-anchor: end-alignment;
    }
  }
}

.edit-track {
  z-index: 4;
  background-color: rgba(0, 0, 0, 0.6);
}

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

.key-swipe {
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

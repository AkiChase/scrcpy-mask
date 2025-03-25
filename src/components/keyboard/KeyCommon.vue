<script setup lang="ts">
import { computed, ref } from "vue";
import { useGlobalStore } from "../../store/global";
import {
  NButton,
  NFormItem,
  NH4,
  NIcon,
  NInput,
  NModal,
  NCard,
  useMessage,
  NFlex,
  NInputNumber,
} from "naive-ui";
import { CloseCircle, Settings } from "@vicons/ionicons5";
import {
  KeyCommon,
  KeyMacro,
  KeyMacroList,
} from "../../tools/keyMappingConfig";
import { useKeyboardStore } from "../../store/keyboard";
import { useI18n } from "vue-i18n";
import { configKeyCommon } from "./config";
import { error } from "@tauri-apps/plugin-log";

const props = defineProps<{
  index: number;
}>();

const keyboardStore = useKeyboardStore();

const { t } = useI18n();
const store = useGlobalStore();
const message = useMessage();
const elementRef = ref<HTMLElement | null>(null);

const isActive = computed(
  () => props.index === keyboardStore.activeButtonIndex
);
const keyMapping = computed(
  () => store.editKeyMappingList[props.index] as KeyCommon
);

const showMacroModal = ref(false);
const editedMacroRaw = ref({
  down: "",
  loop: "",
  up: "",
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
    const maxX = keyboardElement.clientWidth - 20;
    const maxY = keyboardElement.clientHeight - 20;

    const x = downEvent.clientX;
    const y = downEvent.clientY;
    const moveHandler = (moveEvent: MouseEvent) => {
      let newX = oldX + moveEvent.clientX - x;
      let newY = oldY + moveEvent.clientY - y;
      newX = Math.max(20, Math.min(newX, maxX));
      newY = Math.max(20, Math.min(newY, maxY));
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

function parseMacro(macroRaw: string): KeyMacroList {
  // simple parsing and possible to let the wrong code pass
  if (macroRaw === "") {
    return null;
  }
  const macro: KeyMacroList = JSON.parse(macroRaw);
  if (macro === null) return macro;
  for (const macroItem of macro) {
    if (typeof macroItem !== "object") {
      throw ["macroItem is not object", macroItem];
    }
    if (!("type" in macroItem)) {
      throw ["macroItem has no type attribute", macroItem];
    }
    if (!("args" in macroItem)) {
      throw ["macroItem has no args attribute", macroItem];
    }
  }
  return macro;
}

let macroEditedFlag = false;
function editMacro() {
  macroEditedFlag = false;
  keyboardStore.activeButtonIndex = -1;
  keyboardStore.showButtonSettingFlag = false;

  const macro = (keyMapping.value as KeyMacro).macro;
  editedMacroRaw.value = {
    down: macro.down === null ? "" : JSON.stringify(macro.down, null, 2),
    loop: macro.loop === null ? "" : JSON.stringify(macro.loop, null, 2),
    up: macro.up === null ? "" : JSON.stringify(macro.up, null, 2),
  };
  showMacroModal.value = true;
}

function saveMacro() {
  if (!macroEditedFlag) return;
  try {
    const macro: {
      down: KeyMacroList;
      loop: KeyMacroList;
      up: KeyMacroList;
    } = {
      down: null,
      loop: null,
      up: null,
    };
    const keyList: ["down", "loop", "up"] = ["down", "loop", "up"];
    for (const key of keyList) {
      const macroRaw = editedMacroRaw.value[key];
      macro[key] = parseMacro(macroRaw);
    }

    (keyMapping.value as KeyMacro).macro = macro;
    showMacroModal.value = false;
    keyboardStore.edited = true;
    message.success(t("pages.KeyBoard.KeyCommon.macroParseSuccess"));
  } catch (e) {
    message.error(t("pages.KeyBoard.KeyCommon.macroParseFailed"));
    error("Failed to save macro, " + e);
    console.error(e);
  }
}

const settingPosX = ref(0);
const settingPosY = ref(0);
function showSetting() {
  const keyboardElement = document.getElementById(
    "keyboardElement"
  ) as HTMLElement;
  const maxWidth = keyboardElement.clientWidth - configKeyCommon.settingW;
  const maxHeight = keyboardElement.clientHeight - configKeyCommon.settingH;

  settingPosX.value = Math.min(keyMapping.value.posX + 25, maxWidth);
  settingPosY.value = Math.min(keyMapping.value.posY - 25, maxHeight);
  keyboardStore.showButtonSettingFlag = true;
}
</script>

<template>
  <div
    :class="{ active: isActive }"
    :style="{
      left: `${keyMapping.posX - 20}px`,
      top: `${keyMapping.posY - 20}px`,
    }"
    @mousedown="dragHandler"
    class="key-common"
    ref="elementRef"
  >
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
      width: `${configKeyCommon.settingW}px`,
      height: `${configKeyCommon.settingH}px`,
    }"
  >
    <NH4 prefix="bar">{{
      keyMapping.type === "CancelSkill"
        ? t("pages.KeyBoard.KeyCommon.cancelSkill")
        : keyMapping.type === "Tap"
        ? t("pages.KeyBoard.KeyCommon.tap")
        : t("pages.KeyBoard.KeyCommon.macro")
    }}</NH4>
    <NFormItem
      v-if="keyMapping.type === 'Macro'"
      :label="$t('pages.KeyBoard.KeyCommon.macroCode')"
    >
      <NButton type="success" @click="editMacro">
        {{ $t("pages.KeyBoard.KeyCommon.editMacro") }}
      </NButton>
    </NFormItem>
    <NFormItem
      v-if="keyMapping.type === 'Tap'"
      :label="$t('pages.KeyBoard.setting.touchTime')"
    >
      <NInputNumber
        v-model:value="keyMapping.time"
        :min="0"
        :placeholder="$t('pages.KeyBoard.setting.touchTimePlaceholder')"
        @update:value="keyboardStore.edited = true"
      />
    </NFormItem>
    <NFormItem
      v-if="keyMapping.type !== 'Macro'"
      :label="$t('pages.KeyBoard.setting.pointerID')"
    >
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
  <NModal
    v-if="keyMapping.type === 'Macro'"
    v-model:show="showMacroModal"
    @before-leave="saveMacro"
  >
    <NCard
      style="width: 50%; height: 80%"
      :title="$t('pages.KeyBoard.KeyCommon.macroModal.title')"
    >
      <NFlex vertical style="height: 100%">
        <div>{{ $t("pages.KeyBoard.KeyCommon.macroModal.down") }}</div>
        <NInput
          type="textarea"
          style="flex-grow: 1"
          :placeholder="$t('pages.KeyBoard.KeyCommon.macroModal.placeholder')"
          v-model:value="editedMacroRaw.down"
          @update:value="macroEditedFlag = true"
          round
          clearable
        />
        <div>{{ $t("pages.KeyBoard.KeyCommon.macroModal.loop") }}</div>
        <NInput
          type="textarea"
          style="flex-grow: 1"
          :placeholder="$t('pages.KeyBoard.KeyCommon.macroModal.placeholder')"
          v-model:value="editedMacroRaw.loop"
          @update:value="macroEditedFlag = true"
          round
          clearable
        />
        <div>{{ $t("pages.KeyBoard.KeyCommon.macroModal.up") }}</div>
        <NInput
          type="textarea"
          style="flex-grow: 1"
          :placeholder="$t('pages.KeyBoard.KeyCommon.macroModal.placeholder')"
          v-model:value="editedMacroRaw.up"
          @update:value="macroEditedFlag = true"
          round
          clearable
        />
      </NFlex>
    </NCard>
  </NModal>
</template>

<style scoped lang="scss">
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

.key-common {
  position: absolute;
  height: 40px;
  width: 40px;
  box-sizing: border-box;
  border-radius: 50%;
  border: 2px solid var(--blue-color);
  display: flex;
  justify-content: center;
  align-items: center;
  font-size: 10px;
  font-weight: bold;
  cursor: pointer;

  &:not(.active):hover {
    border: 2px solid var(--light-color);
  }

  .key-close-btn {
    position: absolute;
    left: 45px;
    bottom: 25px;
  }

  .key-setting-btn {
    position: absolute;
    left: 45px;
    top: 25px;
  }
}
.active {
  border: 2px solid var(--primary-color);
  color: var(--primary-color);
  z-index: 2;
}
</style>

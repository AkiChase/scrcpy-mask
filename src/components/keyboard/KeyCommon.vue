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
import { KeyMacro, KeyMacroList, KeyTap } from "../../keyMappingConfig";
import { useKeyboardStore } from "../../store/keyboard";

const props = defineProps<{
  index: number;
}>();

const keyboardStore = useKeyboardStore();

const store = useGlobalStore();
const message = useMessage();
const elementRef = ref<HTMLElement | null>(null);

const isActive = computed(
  () => props.index === keyboardStore.activeButtonIndex
);
const keyMapping = computed(() => store.editKeyMappingList[props.index]);

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

let editedFlag = false;
function editMacro() {
  editedFlag = false;
  const macro = (keyMapping.value as KeyMacro).macro;
  editedMacroRaw.value = {
    down: macro.down === null ? "" : JSON.stringify(macro.down, null, 2),
    loop: macro.loop === null ? "" : JSON.stringify(macro.loop, null, 2),
    up: macro.up === null ? "" : JSON.stringify(macro.up, null, 2),
  };
  showMacroModal.value = true;
}

function saveMacro() {
  if (!editedFlag) return;
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
    message.success("宏代码解析成功，但不保证代码正确性，请自行测试");
  } catch (e) {
    console.error(e);
    message.error("宏代码保存失败，请检查代码格式是否正确");
  }
}

const settingPosX = ref(0);
const settingPosY = ref(0);
function showSetting() {
  const keyboardElement = document.getElementById(
    "keyboardElement"
  ) as HTMLElement;
  const maxWidth = keyboardElement.clientWidth - 150;
  const maxHeight = keyboardElement.clientHeight - 300;

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
    }"
  >
    <NH4 prefix="bar">{{
      keyMapping.type === "CancelSkill"
        ? "技能取消"
        : keyMapping.type === "Tap"
        ? "普通点击"
        : "宏"
    }}</NH4>
    <NFormItem v-if="keyMapping.type === 'Macro'" label="宏代码">
      <NButton type="success" @click="editMacro"> 编辑代码 </NButton>
    </NFormItem>
    <NFormItem v-if="keyMapping.type === 'Tap'" label="触摸时长">
      <NInputNumber
        v-model:value="(keyMapping as KeyTap).time"
        :min="0"
        placeholder="请输入触摸时长(ms)"
        @update:value="keyboardStore.edited = true"
      />
    </NFormItem>
    <NFormItem label="触点ID">
      <NInputNumber
        v-model:value="keyMapping.pointerId"
        :min="0"
        placeholder="请输入触点ID"
        @update:value="keyboardStore.edited = true"
      />
    </NFormItem>
    <NFormItem label="备注">
      <NInput
        v-model:value="keyMapping.note"
        placeholder="请输入备注"
        @update:value="keyboardStore.edited = true"
      />
    </NFormItem>
    <NModal
      v-if="keyMapping.type === 'Macro'"
      v-model:show="showMacroModal"
      @before-leave="saveMacro"
    >
      <NCard style="width: 50%; height: 80%" title="宏编辑">
        <NFlex vertical style="height: 100%">
          <div>按下按键执行</div>
          <NInput
            type="textarea"
            style="flex-grow: 1"
            placeholder="JSON宏代码, 可为空"
            v-model:value="editedMacroRaw.down"
            @update:value="editedFlag = true"
            round
            clearable
          />
          <div>按住执行</div>
          <NInput
            type="textarea"
            style="flex-grow: 1"
            placeholder="JSON宏代码, 可为空"
            v-model:value="editedMacroRaw.loop"
            @update:value="editedFlag = true"
            round
            clearable
          />
          <div>抬起执行</div>
          <NInput
            type="textarea"
            style="flex-grow: 1"
            placeholder="JSON宏代码, 可为空"
            v-model:value="editedMacroRaw.up"
            @update:value="editedFlag = true"
            round
            clearable
          />
        </NFlex>
      </NCard>
    </NModal>
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
  height: 300px;
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

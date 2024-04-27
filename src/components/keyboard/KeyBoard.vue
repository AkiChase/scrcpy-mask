<script setup lang="ts">
import { ref } from "vue";
import KeyInfo from "./KeyInfo.vue";
import KeySetting from "./KeySetting.vue";
import KeyCommon from "./KeyCommon.vue";
import { useGlobalStore } from "../../store/global";
import { useDialog } from "naive-ui";
import { onBeforeRouteLeave } from "vue-router";

// TODO 方向轮盘按钮 KeySteeringWheel
//      有四个按钮+offset
// TODO 技能按钮 KeyDirectionalSkill，KeyDirectionlessSkill，KeyTriggerWhenPressedSkill（有区分directional)
//      单个键，靠两个flag来区分这四种情况，还有range或time要视情况
// TODO 添加视野按钮 KeyObservation
//      单个键，有灵敏度scale
// TODO 普通按钮 KeyMacro，KeyCancelSkill，KeyTap
//      单个键，KeyMacro有输入框
// TODO 右键按钮打开菜单修改、删除
// TODO 右键空白区域添加按键
// TODO 设置界面添加本地数据编辑器（类似utools）
// TODO 添加开发者工具打开按钮

const showKeyInfoFlag = ref(false);
const showSettingFlag = ref(false);
const store = useGlobalStore();
const dialog = useDialog();

const activeButtonIndex = ref(-1);
let edited = ref(false);

function handleClick(event: MouseEvent) {
  if (
    event.button === 0 &&
    event.target === document.getElementById("keyboardElement")
  ) {
    if (showSettingFlag.value) {
      showSettingFlag.value = false;
      return;
    }
    activeButtonIndex.value = -1;
  }
}

onBeforeRouteLeave(() => {
  if (edited.value) {
    dialog.warning({
      title: "Warning",
      content: "当前方案尚未保存，是否保存？",
      positiveText: "保存",
      negativeText: "取消",
      onPositiveClick: () => {
        store.applyEditKeyMappingList();
        edited.value = false;
      },
      onNegativeClick: () => {
        store.resetEditKeyMappingList();
        edited.value = false;
      },
    });
  }
});
</script>

<template>
  <div id="keyboardElement" class="keyboard" @click="handleClick">
    <KeySetting
      v-model:showKeyInfoFlag="showKeyInfoFlag"
      v-model:showSettingFlag="showSettingFlag"
      v-model:edited="edited"
    />
    <KeyInfo v-model:showKeyInfoFlag="showKeyInfoFlag" />
    <template v-for="(_, index) in store.editKeyMappingList">
      <KeyCommon
        @edit="edited = true"
        @active="activeButtonIndex = index"
        :index="index"
        :activeIndex="activeButtonIndex"
      />
    </template>
  </div>
</template>

<style scoped lang="scss">
.keyboard {
  color: var(--light-color);
  background-color: rgba(0, 0, 0, 0.5);
  overflow: hidden;
  position: relative;

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

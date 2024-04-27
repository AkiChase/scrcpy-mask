<script setup lang="ts">
import { ref } from "vue";
import KeyInfo from "./KeyInfo.vue";
import KeySetting from "./KeySetting.vue";

// TODO 右键空白区域添加按键
// TODO 左键可拖动按钮（并显示到顶部），右键按钮进行修改、删除
// TODO 导入默认配置（重复的用新名称）
// TODO 设置界面添加清空本地数据的按钮

const keyInfoShowFlag = ref(false);
</script>

<template>
  <div id="keyboardElement" class="keyboard">
    <KeySetting v-model="keyInfoShowFlag" />
    <KeyInfo v-model="keyInfoShowFlag" />
  </div>
</template>

<style scoped>
.keyboard {
  color: var(--light-color);
  background-color: rgba(0, 0, 0, 0.5);
  overflow: hidden;
  position: relative;
}
</style>

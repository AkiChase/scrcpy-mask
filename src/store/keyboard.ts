import { defineStore } from "pinia";
import { ref } from "vue";

export const useKeyboardStore = defineStore("keyboard", () => {
  const showKeyInfoFlag = ref(false);
  const showSettingFlag = ref(false);
  const showButtonSettingFlag = ref(false);
  const showButtonAddFlag = ref(false);
  const editSwipePointsFlag = ref(false);
  const activeButtonIndex = ref(-1);
  const activeSteeringWheelButtonKeyIndex = ref(-1);
  const edited = ref(false);

  return {
    showKeyInfoFlag,
    showSettingFlag,
    showButtonSettingFlag,
    showButtonAddFlag,
    editSwipePointsFlag,
    activeButtonIndex,
    activeSteeringWheelButtonKeyIndex,
    edited,
  };
});

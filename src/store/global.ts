import { defineStore } from "pinia";
import { Ref, ref } from "vue";
import { Device } from "../invoke";

export const useGlobalStore = defineStore("counter", () => {
  const showLoadingRef = ref(false);
  function showLoading() {
    showLoadingRef.value = true;
  }
  function hideLoading() {
    showLoadingRef.value = false;
  }

  interface ControledDevice {
    scid: string;
    deviceName: string;
    device: Device;
  }

  const screenSizeW: Ref<number> = ref(0);
  const screenSizeH: Ref<number> = ref(0);

  const controledDevice: Ref<ControledDevice | null> = ref(null);

  const curKeyMappingConfig: any = {
    relativeSize: { w: 1280, h: 720 },
    title: "王者荣耀-暃",
    list: [
      {
        type: "SteeringWheel",
        note: "方向轮盘",
        key: {
          left: "KeyA",
          right: "KeyD",
          up: "KeyW",
          down: "KeyS",
        },
        posX: 180,
        posY: 560,
        offset: 100,
        pointerId: 1,
      },
      {
        type: "DirectionalSkill",
        note: "技能1",
        key: "KeyQ",
        posX: 950,
        posY: 610,
        range: 200,
        pointerId: 2,
      },
      {
        type: "DirectionalSkill",
        note: "技能2",
        key: "AltLeft",
        posX: 1025,
        posY: 500,
        range: 200,
        pointerId: 2,
      },
      {
        type: "DirectionalSkill",
        note: "技能3",
        key: "KeyE",
        posX: 1160,
        posY: 420,
        range: 200,
        pointerId: 2,
      },
      {
        type: "TriggerWhenPressedSkill",
        note: "技能3",
        key: "M4",
        posX: 1160,
        posY: 420,
        directional: true,
        rangeOrTime: 0,
        pointerId: 2,
      },
      {
        type: "DirectionlessSkill",
        note: "无方向装备技能",
        key: "M1",
        posX: 1150,
        posY: 280,
        pointerId: 2,
      },
      {
        type: "CancelSkill",
        note: "取消技能",
        key: "Space",
        posX: 1160,
        posY: 140,
        pointerId: 2,
      },
      {
        type: "Tap",
        note: "回城",
        key: "KeyB",
        time: 80,
        posX: 650,
        posY: 650,
        pointerId: 3,
      },
      {
        type: "Tap",
        note: "回复",
        key: "KeyC",
        time: 80,
        posX: 740,
        posY: 650,
        pointerId: 3,
      },
      {
        type: "DirectionalSkill",
        note: "召唤师技能",
        key: "KeyF",
        posX: 840,
        posY: 650,
        range: 200,
        pointerId: 2,
      },
      {
        type: "TriggerWhenPressedSkill",
        note: "无方向召唤师技能",
        key: "ControlLeft",
        posX: 840,
        posY: 650,
        directional: false,
        rangeOrTime: 80,
        pointerId: 3,
      },
      {
        type: "Tap",
        note: "攻击",
        key: "M2",
        time: 80,
        posX: 1165,
        posY: 620,
        pointerId: 3,
      },
      {
        type: "Tap",
        note: "技能1升级",
        key: "Digit1",
        time: 80,
        posX: 880,
        posY: 560,
        pointerId: 3,
      },
      {
        type: "Tap",
        note: "技能2升级",
        key: "Digit2",
        time: 80,
        posX: 960,
        posY: 430,
        pointerId: 3,
      },
      {
        type: "Tap",
        note: "技能3升级",
        key: "Digit3",
        time: 80,
        posX: 1090,
        posY: 350,
        pointerId: 3,
      },
      {
        type: "Tap",
        note: "快速购买1",
        key: "Digit5",
        time: 80,
        posX: 130,
        posY: 300,
        pointerId: 3,
      },
      {
        type: "Tap",
        note: "快速购买2",
        key: "Digit6",
        time: 80,
        posX: 130,
        posY: 370,
        pointerId: 3,
      },
      {
        type: "TriggerWhenPressedSkill",
        note: "装备技能",
        key: "WheelDown",
        posX: 1150,
        posY: 280,
        directional: false,
        rangeOrTime: 80,
        pointerId: 3,
      },
      {
        type: "Observation",
        note: "观察",
        key: "M3",
        posX: 1000,
        posY: 200,
        scale: 0.5,
        pointerId: 4,
      },
      {
        type: "Macro",
        note: "战绩面板",
        key: "Tab",
        macro: {
          down: [
            {
              type: "touch",
              args: ["default", 5, 1185, 40, 80],
            },
          ],
          loop: null,
          up: [
            {
              type: "touch",
              args: ["default", 5, 1220, 100, 80],
            },
          ],
        },
        posX: 1185,
        posY: 40,
        pointerId: 5,
      },
      {
        type: "Macro",
        note: "商店",
        key: "ShiftLeft",
        macro: {
          down: [
            {
              type: "touch",
              args: ["default", 5, 40, 300, 80],
            },
          ],
          loop: null,
          up: [
            {
              type: "touch",
              args: ["default", 5, 1200, 60, 80],
            },
          ],
        },
        posX: 40,
        posY: 300,
        pointerId: 5,
      },
      {
        type: "Macro",
        note: "地图",
        key: "KeyZ",
        macro: {
          down: [
            {
              type: "touch",
              args: ["default", 5, 250, 230, 80],
            },
          ],
          loop: null,
          up: [
            {
              type: "touch",
              args: ["default", 5, 640, 150, 80],
            },
          ],
        },
        posX: 250,
        posY: 230,
        pointerId: 5,
      },
    ],
  };

  return {
    showLoading,
    hideLoading,
    showLoadingRef,
    controledDevice,
    screenSizeW,
    screenSizeH,
    curKeyMappingConfig,
  };
});

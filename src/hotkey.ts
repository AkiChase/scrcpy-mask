// https://github.com/jamiebuilds/tinykeys/pull/193/commits/2598ecb3db6b3948c7acbf0e7bd8b0674961ad61
import { MessageReactive, useMessage } from "naive-ui";
import {
  SwipeAction,
  TouchAction,
  swipe,
  touch,
} from "./frontcommand/scrcpyMaskCmd";
import {
  KeyCancelSkill,
  KeyDirectionalSkill,
  KeyDirectionlessSkill,
  KeyFire,
  KeyMacro,
  KeyMacroList,
  KeyMappingConfig,
  KeyObservation,
  KeySight,
  KeySteeringWheel,
  KeySwipe,
  KeyTap,
  KeyTriggerWhenDoublePressedSkill,
  KeyTriggerWhenPressedSkill,
} from "./keyMappingConfig";
import { useGlobalStore } from "./store/global";
import { LogicalPosition, getCurrent } from "@tauri-apps/api/window";
import { readText } from "@tauri-apps/plugin-clipboard-manager";
import { useI18n } from "vue-i18n";
import { KeyToCodeMap } from "./frontcommand/KeyToCodeMap";
import {
  AndroidKeyEventAction,
  AndroidMetastate,
} from "./frontcommand/android";
import { UIEventsCode } from "./frontcommand/UIEventsCode";
import { sendInjectKeycode, sendSetClipboard } from "./frontcommand/controlMsg";

function clientxToPosx(clientx: number) {
  return clientx < 70
    ? 0
    : Math.floor((clientx - 70) * (store.screenSizeW / store.maskSizeW));
}

function clientyToPosy(clienty: number) {
  return clienty < 30
    ? 0
    : Math.floor((clienty - 30) * (store.screenSizeH / store.maskSizeH));
}

function clientxToPosOffsetx(clientx: number, posx: number, scale = 1) {
  let offsetX = clientxToPosx(clientx) - posx;
  return Math.round(offsetX * scale);
}

function clientyToPosOffsety(clienty: number, posy: number, scale = 1) {
  let offsetY = clientyToPosy(clienty) - posy;
  return Math.round(offsetY * scale);
}

function clientPosToSkillOffset(
  clientPos: { x: number; y: number },
  range: number
): { offsetX: number; offsetY: number } {
  const maskSizeH = store.maskSizeH;
  const maskSizeW = store.maskSizeW;

  const maxLength = (120 / maskSizeH) * store.screenSizeH;
  const centerX = maskSizeW * 0.5;
  const centerY = maskSizeH * 0.5;

  // The center of the game display is higher than the center of the mask
  clientPos.y -= maskSizeH * 0.066;

  // w450 : h315 = 100 : 70, so the true offsetX is 0.7 * cOffsetX
  const cOffsetX = (clientPos.x - 70 - centerX) * 0.7;
  const cOffsetY = clientPos.y - 30 - centerY;
  const offsetD = Math.sqrt(cOffsetX ** 2 + cOffsetY ** 2);
  if (offsetD == 0) {
    return {
      offsetX: 0,
      offsetY: 0,
    };
  }

  const rangeD = (maskSizeH - centerY) * range * 0.01;
  if (offsetD >= rangeD) {
    // include the case of rangeD == 0
    return {
      offsetX: Math.round((maxLength / offsetD) * cOffsetX),
      offsetY: Math.round((maxLength / offsetD) * cOffsetY),
    };
  } else {
    const factor = offsetD / rangeD;
    return {
      offsetX: Math.round((cOffsetX / rangeD) * maxLength * factor),
      offsetY: Math.round((cOffsetY / rangeD) * maxLength * factor),
    };
  }
}

async function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function calculateMacroPosX(
  // pos relative to the mask
  posX: [string, number] | number,
  relativeSizeW: number
): number {
  if (typeof posX === "number") {
    return Math.round(posX * (store.screenSizeW / relativeSizeW));
  }
  if (typeof posX === "string") {
    return clientxToPosx(mouseX);
  } else {
    if (posX[0] === "mouse") {
      return (
        clientxToPosx(mouseX) +
        Math.round(posX[1] * (store.screenSizeW / relativeSizeW))
      );
    } else {
      throw new Error("Invalid pos");
    }
  }
}

function calculateMacroPosY(
  // pos relative to the mask
  posY: [string, number] | number,
  relativeSizeH: number
): number {
  if (typeof posY === "number") {
    return Math.round(posY * (store.screenSizeH / relativeSizeH));
  }
  if (typeof posY === "string") {
    return clientyToPosy(mouseY);
  } else {
    if (posY[0] === "mouse") {
      return (
        clientyToPosy(mouseY) +
        Math.round(posY[1] * (store.screenSizeH / relativeSizeH))
      );
    } else {
      throw new Error("Invalid pos");
    }
  }
}

function calculateMacroPosList(
  posList: [[string, number] | number, [string, number] | number][],
  relativeSize: { w: number; h: number }
): { x: number; y: number }[] {
  return posList.map((posPair) => {
    return {
      x: calculateMacroPosX(posPair[0], relativeSize.w),
      y: calculateMacroPosY(posPair[1], relativeSize.h),
    };
  });
}

// add shortcuts for observation
function addObservationShortcuts(
  key: string,
  relativeSize: { w: number; h: number },
  // pos relative to the mask
  posX: number,
  posY: number,
  scale: number,
  pointerId: number
) {
  let observationMouseX = 0;
  let observationMouseY = 0;
  posX = Math.round((posX / relativeSize.w) * store.screenSizeW);
  posY = Math.round((posY / relativeSize.h) * store.screenSizeH);
  addShortcut(
    key,
    async () => {
      observationMouseX = clientxToPosx(mouseX);
      observationMouseY = clientyToPosy(mouseY);
      await touchX(TouchAction.Down, pointerId, posX, posY);
    },
    async () => {
      await touchX(
        TouchAction.Move,
        pointerId,
        posX + clientxToPosOffsetx(mouseX, observationMouseX, scale),
        posY + clientyToPosOffsety(mouseY, observationMouseY, scale)
      );
    },
    async () => {
      await touchX(
        TouchAction.Up,
        pointerId,
        posX + clientxToPosOffsetx(mouseX, observationMouseY, scale),
        posY + clientyToPosOffsety(mouseY, observationMouseY, scale)
      );
    }
  );
}

// add shortcuts for simple tap (touch when press down)
function addTapShortcuts(
  key: string,
  relativeSize: { w: number; h: number },
  time: number,
  // pos relative to the mask
  posX: number,
  posY: number,
  pointerId: number
) {
  posX = Math.round((posX / relativeSize.w) * store.screenSizeW);
  posY = Math.round((posY / relativeSize.h) * store.screenSizeH);
  addShortcut(
    key,
    async () => {
      await touchX(TouchAction.Default, pointerId, posX, posY, time);
    },
    undefined,
    undefined
  );
}

// add shortcuts for cancel skill
function addCancelSkillShortcuts(
  key: string,
  relativeSize: { w: number; h: number },
  // pos relative to the mask
  posX: number,
  posY: number,
  pointerId: number
) {
  posX = Math.round((posX / relativeSize.w) * store.screenSizeW);
  posY = Math.round((posY / relativeSize.h) * store.screenSizeH);
  addShortcut(
    key,
    async () => {
      // delete the callback
      for (const cancelAbleKey of cancelAbleKeyList) {
        loopDownKeyCBMap.delete(cancelAbleKey);
        upKeyCBMap.delete(cancelAbleKey);
      }
      // special case for double press skill
      for (const [key, val] of doublePressedDownKey) {
        if (val) {
          loopDownKeyCBMap.delete(key);
          doublePressedDownKey.set(key, false);
        }
      }
      let distance = 0;
      while (distance <= 20) {
        await touchX(TouchAction.Move, pointerId, posX + distance, posY);
        await sleep(5);
        distance += 1;
      }
      await touchX(TouchAction.Up, pointerId, posX, posY);
    },
    undefined,
    undefined
  );
}

// add shortcuts for trigger when pressed skill
function addTriggerWhenPressedSkillShortcuts(
  key: string,
  relativeSize: { w: number; h: number },
  // pos relative to the mask
  posX: number,
  posY: number,
  directional: boolean,
  // range: when directional is true
  // time: when directional is false
  rangeOrTime: number,
  pointerId: number
) {
  posX = Math.round((posX / relativeSize.w) * store.screenSizeW);
  posY = Math.round((posY / relativeSize.h) * store.screenSizeH);
  if (directional) {
    addShortcut(
      key,
      // down
      async () => {
        // up doublepress skill
        await upDoublePressedKey();
        const skillOffset = clientPosToSkillOffset(
          { x: mouseX, y: mouseY },
          rangeOrTime
        );
        await swipe({
          action: SwipeAction.Default,
          pointerId,
          screen: {
            w: store.screenSizeW,
            h: store.screenSizeH,
          },
          pos: [
            { x: posX, y: posY },
            {
              x: posX + skillOffset.offsetX,
              y: posY + skillOffset.offsetY,
            },
          ],
          intervalBetweenPos: 0,
        });
      },
      undefined,
      undefined
    );
  } else {
    addShortcut(
      key,
      async () => {
        await upDoublePressedKey();
        await touchX(TouchAction.Default, pointerId, posX, posY, rangeOrTime);
      },
      undefined,
      undefined
    );
  }
}

// add shortcuts for trigger when double pressed skill (cancelable, but in another way)
const doublePressedDownKey = new Map<string, boolean>();
function addTriggerWhenDoublePressedSkillShortcuts(
  key: string,
  relativeSize: { w: number; h: number },
  // pos relative to the mask
  posX: number,
  posY: number,
  range: number,
  pointerId: number
) {
  posX = Math.round((posX / relativeSize.w) * store.screenSizeW);
  posY = Math.round((posY / relativeSize.h) * store.screenSizeH);
  doublePressedDownKey.set(key, false);
  addShortcut(
    key,
    // down
    async () => {
      if (doublePressedDownKey.get(key) === false) {
        // first press: touch down
        const skillOffset = clientPosToSkillOffset(
          { x: mouseX, y: mouseY },
          range
        );
        await swipe({
          action: SwipeAction.NoUp,
          pointerId,
          screen: {
            w: store.screenSizeW,
            h: store.screenSizeH,
          },
          pos: [
            { x: posX, y: posY },
            {
              x: posX + skillOffset.offsetX,
              y: posY + skillOffset.offsetY,
            },
          ],
          intervalBetweenPos: 0,
        });
        // set the flag to true
        doublePressedDownKey.set(key, true);
        // add loop CB
        loopDownKeyCBMap.set(key, async () => {
          const loopSkillOffset = clientPosToSkillOffset(
            { x: mouseX, y: mouseY },
            range
          );
          await touchX(
            TouchAction.Move,
            pointerId,
            posX + loopSkillOffset.offsetX,
            posY + loopSkillOffset.offsetY
          );
        });
      } else {
        // second press: touch up
        // delete the loop CB
        loopDownKeyCBMap.delete(key);
        const skillOffset = clientPosToSkillOffset(
          { x: mouseX, y: mouseY },
          range
        );
        await touchX(
          TouchAction.Up,
          pointerId,
          posX + skillOffset.offsetX,
          posY + skillOffset.offsetY
        );
        // set the flag to false
        doublePressedDownKey.set(key, false);
      }
    },
    undefined,
    undefined
  );
}

// add shortcuts for directionless skill (cancelable)
function addDirectionlessSkillShortcuts(
  key: string,
  relativeSize: { w: number; h: number },
  // pos relative to the mask
  posX: number,
  posY: number,
  pointerId: number
) {
  posX = Math.round((posX / relativeSize.w) * store.screenSizeW);
  posY = Math.round((posY / relativeSize.h) * store.screenSizeH);
  addShortcut(
    key,
    // down
    async () => {
      // up doublepress skill
      await upDoublePressedKey();
      await touchX(TouchAction.Down, pointerId, posX, posY);
    },
    // loop
    undefined,
    // up
    async () => {
      await touch({
        action: TouchAction.Up,
        pointerId,
        screen: {
          w: store.screenSizeW,
          h: store.screenSizeH,
        },
        pos: {
          x: posX,
          y: posY,
        },
      });
    },
    true
  );
}

// up all double pressed key
async function upDoublePressedKey() {
  for (const [key, val] of doublePressedDownKey) {
    if (val) {
      await downKeyCBMap.get(key)?.();
    }
  }
}

// add shortcuts for directional skill (cancelable)
function addDirectionalSkillShortcuts(
  key: string,
  relativeSize: { w: number; h: number },
  // pos relative to the mask
  posX: number,
  posY: number,
  range: number,
  pointerId: number
) {
  posX = Math.round((posX / relativeSize.w) * store.screenSizeW);
  posY = Math.round((posY / relativeSize.h) * store.screenSizeH);
  addShortcut(
    key,
    // down
    async () => {
      // up doublepress skill
      await upDoublePressedKey();
      const skillOffset = clientPosToSkillOffset(
        { x: mouseX, y: mouseY },
        range
      );
      await swipe({
        action: SwipeAction.NoUp,
        pointerId,
        screen: {
          w: store.screenSizeW,
          h: store.screenSizeH,
        },
        pos: [
          { x: posX, y: posY },
          {
            x: posX + skillOffset.offsetX,
            y: posY + skillOffset.offsetY,
          },
        ],
        intervalBetweenPos: 0,
      });
    },
    // loop
    async () => {
      const skillOffset = clientPosToSkillOffset(
        { x: mouseX, y: mouseY },
        range
      );
      await touchX(
        TouchAction.Move,
        pointerId,
        posX + skillOffset.offsetX,
        posY + skillOffset.offsetY
      );
    },
    // up
    async () => {
      const skillOffset = clientPosToSkillOffset(
        { x: mouseX, y: mouseY },
        range
      );
      await touchX(
        TouchAction.Up,
        pointerId,
        posX + skillOffset.offsetX,
        posY + skillOffset.offsetY
      );
    },
    true
  );
}

// add shortcuts for steering wheel
function addSteeringWheelKeyboardShortcuts(
  key: wheelKey,
  relativeSize: { w: number; h: number },
  // pos relative to the mask
  posX: number,
  posY: number,
  offset: number,
  pointerId: number
) {
  let loopFlag = false;
  let curPosX = 0;
  let curPosY = 0;
  posX = Math.round((posX / relativeSize.w) * store.screenSizeW);
  posY = Math.round((posY / relativeSize.h) * store.screenSizeH);

  // calculate the end coordinates of the eight directions of the direction wheel
  let offsetHalf = Math.round(offset / 1.414);
  const calculatedPosList = [
    { x: posX - offset, y: posY }, // left
    { x: posX + offset, y: posY }, // right
    { x: posX, y: posY - offset }, // up
    { x: posX, y: posY + offset }, // down
    { x: posX - offsetHalf, y: posY - offsetHalf }, // left up
    { x: posX + offsetHalf, y: posY - offsetHalf }, // right up
    { x: posX - offsetHalf, y: posY + offsetHalf }, // left down
    { x: posX + offsetHalf, y: posY + offsetHalf }, // right down
  ];

  // united loop callback for all directions
  const unitedloopCB = async () => {
    let newPosIndex;
    if (downKeyMap.get(key.left)) {
      newPosIndex = downKeyMap.get(key.up)
        ? 4
        : downKeyMap.get(key.down)
        ? 6
        : 0;
    } else if (downKeyMap.get(key.right)) {
      newPosIndex = downKeyMap.get(key.up)
        ? 5
        : downKeyMap.get(key.down)
        ? 7
        : 1;
    } else if (downKeyMap.get(key.up)) {
      newPosIndex = 2;
    } else if (downKeyMap.get(key.down)) {
      newPosIndex = 3;
    } else {
      // all keys released
      await unitedUpCB();
      return;
    }
    // if newPos is the same as curPos, return
    let newPos = calculatedPosList[newPosIndex];
    if (newPos.x === curPosX && newPos.y === curPosY) return;

    curPosX = newPos.x;
    curPosY = newPos.y;
    // move to the direction
    await touchX(TouchAction.Move, pointerId, newPos.x, newPos.y);
  };

  const unitedUpCB = async () => {
    if (
      downKeyMap.get(key.left) === false &&
      downKeyMap.get(key.right) === false &&
      downKeyMap.get(key.up) === false &&
      downKeyMap.get(key.down) === false
    ) {
      // all wheel keys has been released
      for (const k of ["left", "right", "up", "down"]) {
        // only delete the valid key
        loopDownKeyCBMap.delete(key[k]);
        upKeyCBMap.delete(key[k]);
      }
      // touch up
      await touchX(TouchAction.Up, pointerId, curPosX, curPosY);
      // recover the status
      curPosX = 0;
      curPosY = 0;
      loopFlag = false;
    }
  };

  for (const k of ["left", "right", "up", "down"]) {
    addShortcut(
      key[k],
      async () => {
        if (loopFlag) return;
        loopFlag = true;
        // add upCB
        upKeyCBMap.set(key[k], unitedUpCB);

        // touch down on the center position
        await touchX(TouchAction.Down, pointerId, posX, posY);
        // add loopCB
        loopDownKeyCBMap.set(key[k], unitedloopCB);
      },
      undefined,
      undefined
    );
  }
}

interface wheelKey {
  left: string;
  right: string;
  up: string;
  down: string;
  [key: string]: string;
}

// add baisc click shortcuts
function addClickShortcuts(key: string, pointerId: number) {
  addShortcut(
    key,
    async () => {
      await touchX(
        TouchAction.Down,
        pointerId,
        clientxToPosx(mouseX),
        clientyToPosy(mouseY)
      );
    },
    async () => {
      await touchX(
        TouchAction.Move,
        pointerId,
        clientxToPosx(mouseX),
        clientyToPosy(mouseY)
      );
    },
    async () => {
      await touchX(
        TouchAction.Up,
        pointerId,
        clientxToPosx(mouseX),
        clientyToPosy(mouseY)
      );
    }
  );
}

// add shortcut for sight mode
function addSightShortcuts(
  relativeSize: { w: number; h: number },
  sightKeyMapping: KeySight,
  fireKeyMapping?: KeyFire
) {
  const appWindow = getCurrent();

  let mouseLock = false;
  let msgReactive: ReturnType<typeof message.info> | null = null;
  const sightClientX = 70 + sightKeyMapping.posX;
  const sightClientY = 30 + sightKeyMapping.posY;
  const sightDeviceX = Math.round(
    (sightKeyMapping.posX / relativeSize.w) * store.screenSizeW
  );
  const sightDeviceY = Math.round(
    (sightKeyMapping.posY / relativeSize.h) * store.screenSizeH
  );

  const fireDeviceX = fireKeyMapping
    ? Math.round((fireKeyMapping.posX / relativeSize.w) * store.screenSizeW)
    : 0;

  const fireDeviceY = fireKeyMapping
    ? Math.round((fireKeyMapping.posY / relativeSize.h) * store.screenSizeH)
    : 0;

  const removeShortcut = (key: string) => {
    loopDownKeyCBMap.delete(key);
    downKeyCBMap.delete(key);
    upKeyCBMap.delete(key);
    downKeyMap.delete(key);
  };

  const touchRelateToSight = async (action: TouchAction) => {
    await touchX(
      action,
      sightKeyMapping.pointerId,
      sightDeviceX +
        clientxToPosOffsetx(mouseX, sightDeviceX, sightKeyMapping.scaleX),
      sightDeviceY +
        clientyToPosOffsety(mouseY, sightDeviceY, sightKeyMapping.scaleY)
    );
  };

  const sightLoopCB = async () => {
    await touchRelateToSight(TouchAction.Move);
  };

  // only scaleX and scaleY are different from sightLoopCB
  const fireNoDragLoopCB = fireKeyMapping
    ? async () => {
        await touchX(
          TouchAction.Move,
          sightKeyMapping.pointerId,
          sightDeviceX +
            clientxToPosOffsetx(mouseX, sightDeviceX, fireKeyMapping.scaleX),
          sightDeviceY +
            clientyToPosOffsety(mouseY, sightDeviceY, fireKeyMapping.scaleY)
        );
      }
    : undefined;

  const fireDragLoopCB = fireKeyMapping
    ? async () => {
        await touchX(
          TouchAction.Move,
          fireKeyMapping.pointerId,
          fireDeviceX +
            accOffsetX +
            clientxToPosOffsetx(mouseX, sightDeviceX, fireKeyMapping.scaleX),
          fireDeviceY +
            accOffsetY +
            clientyToPosOffsety(mouseY, sightDeviceY, fireKeyMapping.scaleY)
        );
      }
    : undefined;

  let accOffsetX = 0;
  let accOffsetY = 0;
  const moveLeaveHandler = async () => {
    if (fireKeyMapping && fireKeyMapping.drag && downKeyMap.get("M0")) {
      // fire drag mode
      // stop fireDragLoopCB
      loopDownKeyCBMap.delete(sightKeyMapping.key);
      // cal accOffset
      accOffsetX += clientxToPosOffsetx(
        mouseX,
        sightDeviceX,
        fireKeyMapping.scaleX
      );
      accOffsetY += clientyToPosOffsety(
        mouseY,
        sightDeviceY,
        fireKeyMapping.scaleY
      );
      // move mouse
      await appWindow.setCursorPosition(
        new LogicalPosition(sightClientX, sightClientY)
      );
      mouseX = sightClientX;
      mouseY = sightClientY;
      // start fireDragLoopCB
      loopDownKeyCBMap.set(sightKeyMapping.key, fireDragLoopCB!);
    } else {
      // sight mode or fire without drag mode
      const fireFlag =
        fireKeyMapping && !fireKeyMapping.drag && downKeyMap.get("M0");
      // stop sightLoopCB or fireNoDragLoopCB
      loopDownKeyCBMap.delete(sightKeyMapping.key);
      // touch up
      if (fireFlag) {
        await touchX(
          TouchAction.Move,
          sightKeyMapping.pointerId,
          sightDeviceX +
            clientxToPosOffsetx(mouseX, sightDeviceX, fireKeyMapping.scaleX),
          sightDeviceY +
            clientyToPosOffsety(mouseY, sightDeviceY, fireKeyMapping.scaleY)
        );
      } else {
        await touchRelateToSight(TouchAction.Up);
      }
      // move mouse
      await appWindow.setCursorPosition(
        new LogicalPosition(sightClientX, sightClientY)
      );
      mouseX = sightClientX;
      mouseY = sightClientY;
      // touch down
      await touchX(
        TouchAction.Down,
        sightKeyMapping.pointerId,
        sightDeviceX,
        sightDeviceY
      );
      // start sightLoopCB or fireNoDragLoopCB
      loopDownKeyCBMap.set(
        sightKeyMapping.key,
        fireFlag ? fireNoDragLoopCB! : sightLoopCB
      );
    }
  };

  // add sight shortcut
  addShortcut(sightKeyMapping.key, async () => {
    if (mouseLock) {
      // stop sight mode

      // remove box element
      const mouseRangeBoxElement = document.getElementById("mouseRangeBox");
      if (mouseRangeBoxElement) {
        mouseRangeBoxElement.removeEventListener(
          "mouseleave",
          moveLeaveHandler
        );
        mouseRangeBoxElement.remove();
      }
      maskElement.style.cursor = "pointer";

      mouseLock = false;
      loopDownKeyCBMap.delete(sightKeyMapping.key);
      await touchRelateToSight(TouchAction.Up);
      await appWindow.setCursorVisible(true);

      if (msgReactive) {
        msgReactive.destroy();
        msgReactive = null;
      }
      // remove fire key
      if (fireKeyMapping) {
        removeShortcut("M0");
      }
      // add click
      addClickShortcuts("M0", 0);
    } else {
      // start sight mode

      // create box element
      maskElement.style.cursor = "none";
      const mouseRangeBoxElement = createMouseRangeBox();
      mouseRangeBoxElement.addEventListener("contextmenu", (e) =>
        e.preventDefault()
      );
      mouseRangeBoxElement.addEventListener("mouseleave", moveLeaveHandler);
      document.body.appendChild(mouseRangeBoxElement);

      mouseLock = true;
      await appWindow.setCursorVisible(false);
      msgReactive = message.info(
        t("pages.Mask.sightMode", [sightKeyMapping.key]),
        {
          duration: 0,
        }
      );

      await appWindow.setCursorPosition(
        new LogicalPosition(sightClientX, sightClientY)
      );
      mouseX = sightClientX;
      mouseY = sightClientY;
      await touchX(
        TouchAction.Down,
        sightKeyMapping.pointerId,
        sightDeviceX,
        sightDeviceY
      );
      loopDownKeyCBMap.set(sightKeyMapping.key, sightLoopCB);
      // remove click
      removeShortcut("M0");
      // add fire key
      if (fireKeyMapping) {
        // fire with drag
        addShortcut(
          "M0",
          async () => {
            // stop sightLoopCB
            loopDownKeyCBMap.delete(sightKeyMapping.key);
            // touch up sight
            await touchRelateToSight(TouchAction.Up);
            if (!fireKeyMapping.drag) {
              // touch down sight
              await touchX(
                TouchAction.Down,
                sightKeyMapping.pointerId,
                sightDeviceX,
                sightDeviceY
              );
            } else {
              // clear accumulated offset
              accOffsetX = 0;
              accOffsetY = 0;
            }
            // move cursor
            await appWindow.setCursorPosition(
              new LogicalPosition(sightClientX, sightClientY)
            );
            mouseX = sightClientX;
            mouseY = sightClientY;
            // touch down fire
            await touchX(
              TouchAction.Down,
              fireKeyMapping.pointerId,
              fireDeviceX,
              fireDeviceY
            );

            // start fireDragLoopCB or fireNoDragLoopCB
            loopDownKeyCBMap.set(
              sightKeyMapping.key,
              fireKeyMapping.drag ? fireDragLoopCB! : fireNoDragLoopCB!
            );
          },
          undefined,
          async () => {
            // stop fireDragLoopCB or fireNoDragLoopCB
            loopDownKeyCBMap.delete(sightKeyMapping.key);
            // touch up fire
            await touchX(
              TouchAction.Up,
              fireKeyMapping.pointerId,
              fireDeviceX +
                clientxToPosOffsetx(
                  mouseX,
                  sightDeviceX,
                  fireKeyMapping.scaleX
                ),
              fireDeviceY +
                clientyToPosOffsety(mouseY, sightDeviceY, fireKeyMapping.scaleY)
            );
            // touch down sight
            await touchX(
              TouchAction.Down,
              sightKeyMapping.pointerId,
              sightDeviceX,
              sightDeviceY
            );
            // move cursor
            await appWindow.setCursorPosition(
              new LogicalPosition(sightClientX, sightClientY)
            );
            mouseX = sightClientX;
            mouseY = sightClientY;
            // start sightLoopCB
            loopDownKeyCBMap.set(sightKeyMapping.key, sightLoopCB);
          }
        );
      }
    }
  });
}

function addSwipeShortcuts(
  key: string,
  relativeSize: { w: number; h: number },
  // pos relative to the mask
  pos: { x: number; y: number }[],
  pointerId: number,
  intervalBetweenPos: number
) {
  const newPosList = pos.map((posObj) => {
    return {
      x: Math.round((posObj.x / relativeSize.w) * store.screenSizeW),
      y: Math.round((posObj.y / relativeSize.h) * store.screenSizeH),
    };
  });

  addShortcut(
    key,
    async () => {
      await swipe({
        action: SwipeAction.Default,
        pointerId,
        screen: {
          w: store.screenSizeW,
          h: store.screenSizeH,
        },
        pos: newPosList,
        intervalBetweenPos,
      });
    },
    undefined,
    undefined
  );
}

function createMouseRangeBox(): HTMLElement {
  const box = document.createElement("div");
  box.id = "mouseRangeBox";
  box.style.position = "absolute";
  box.style.top = "40px";
  box.style.bottom = "40px";
  box.style.left = "100px";
  box.style.right = "100px";
  box.style.zIndex = "9999";
  box.style.backgroundColor = "transparent";
  box.style.cursor = "none";
  return box;
}

function handleKeydown(event: KeyboardEvent) {
  event.preventDefault();
  if (event.repeat) return;
  if (downKeyMap.has(event.code)) {
    downKeyMap.set(event.code, true);
    // execute the down callback (if there is) asyncily
    let cb = downKeyCBMap.get(event.code);
    if (cb) cb();
  }
}

function handleKeyup(event: KeyboardEvent) {
  event.preventDefault();
  if (downKeyMap.has(event.code)) {
    downKeyMap.set(event.code, false);
    // execute the up callback (if there is) asyncily
    let cb = upKeyCBMap.get(event.code);
    if (cb) cb();
  }
}

function handleMouseDown(event: MouseEvent) {
  const target = event.target;
  if (!(target instanceof HTMLElement)) return;
  if (target.id !== "maskElement" && target.id !== "mouseRangeBox") return;

  mouseX = event.clientX;
  mouseY = event.clientY;
  event.preventDefault();
  let key = "M" + event.button.toString();
  if (downKeyMap.has(key)) {
    downKeyMap.set(key, true);
    // execute the down callback asyncily
    let cb = downKeyCBMap.get(key);
    if (cb) cb();
  }
}

function handleMouseUp(event: MouseEvent) {
  mouseX = event.clientX;
  mouseY = event.clientY;
  event.preventDefault();
  let key = "M" + event.button.toString();
  if (downKeyMap.has(key) && downKeyMap.get(key)) {
    downKeyMap.set(key, false);
    // execute the up callback asyncily
    let cb = upKeyCBMap.get(key);
    if (cb) cb();
  }
}

function handleMouseMove(event: MouseEvent) {
  mouseX = event.clientX;
  mouseY = event.clientY;
}

let lastWheelDownTime: number = 0;
let lastWheelUpTime: number = 0;
function handleMouseWheel(event: WheelEvent) {
  event.preventDefault();
  // trigger interval is 50ms
  if (event.deltaY > 0 && event.timeStamp - lastWheelDownTime > 50) {
    lastWheelDownTime = event.timeStamp;
    // WheelDown
    downKeyCBMap.get("WheelDown")?.();
    upKeyCBMap.get("WheelDown")?.();
  } else if (event.deltaY < 0 && event.timeStamp - lastWheelUpTime > 50) {
    lastWheelUpTime = event.timeStamp;
    // WheelUp
    downKeyCBMap.get("WheelUp")?.();
    upKeyCBMap.get("WheelUp")?.();
  }
}

export class KeyInputHandler {
  private static readonly repeatCounter: Map<number, number> = new Map();
  private static msgReactive: MessageReactive | null = null;
  private static handler(event: KeyboardEvent) {
    const keycode = KeyToCodeMap.get(event.code as UIEventsCode);
    if (!keycode) {
      return;
    }
    let action: AndroidKeyEventAction;
    let repeatCount = 0;
    if (event.type === "keydown") {
      if (event.getModifierState("Control") && event.code === "KeyV") return;

      action = AndroidKeyEventAction.AKEY_EVENT_ACTION_DOWN;
      if (event.repeat) {
        let count = KeyInputHandler.repeatCounter.get(keycode);
        if (typeof count !== "number") {
          count = 1;
        } else {
          count++;
        }
        repeatCount = count;
        KeyInputHandler.repeatCounter.set(keycode, count);
      }
    } else if (event.type === "keyup") {
      if (event.getModifierState("Control") && event.code === "KeyV") {
        (async () => {
          const text = await readText();
          await sendSetClipboard({
            sequence: Math.floor(Math.random() * 10000),
            text,
            paste: true,
          });
        })();
        return;
      }

      action = AndroidKeyEventAction.AKEY_EVENT_ACTION_UP;
      KeyInputHandler.repeatCounter.delete(keycode);
    } else {
      return;
    }
    const metaState =
      (event.getModifierState("Alt") ? AndroidMetastate.AMETA_ALT_ON : 0) |
      (event.getModifierState("Shift") ? AndroidMetastate.AMETA_SHIFT_ON : 0) |
      (event.getModifierState("Control") ? AndroidMetastate.AMETA_CTRL_ON : 0) |
      (event.getModifierState("Meta") ? AndroidMetastate.AMETA_META_ON : 0) |
      (event.getModifierState("CapsLock")
        ? AndroidMetastate.AMETA_CAPS_LOCK_ON
        : 0) |
      (event.getModifierState("ScrollLock")
        ? AndroidMetastate.AMETA_SCROLL_LOCK_ON
        : 0) |
      (event.getModifierState("NumLock")
        ? AndroidMetastate.AMETA_NUM_LOCK_ON
        : 0);

    sendInjectKeycode({
      action,
      keycode,
      repeat: repeatCount,
      metastate: metaState,
    });
    event.preventDefault();
  }
  public static addEventListener() {
    KeyInputHandler.msgReactive = message.info(t("pages.Mask.keyInputMode"), {
      duration: 0,
      closable: true,
      onClose: () => {
        KeyInputHandler.removeEventListener();
        listenToEvent(true);
        store.keyInputFlag = false;
      },
    });
    window.addEventListener("keydown", KeyInputHandler.handler);
    window.addEventListener("keyup", KeyInputHandler.handler);
  }
  public static removeEventListener() {
    if (KeyInputHandler.msgReactive) {
      KeyInputHandler.msgReactive.destroy();
      KeyInputHandler.msgReactive = null;
    }
    window.removeEventListener("keydown", KeyInputHandler.handler);
    window.removeEventListener("keyup", KeyInputHandler.handler);
  }
}

function addShortcut(
  key: string,
  downCB?: () => Promise<void>,
  loopCB?: () => Promise<void>,
  upCB?: () => Promise<void>,
  cancelAble = false // only work with downCB && upCB
) {
  downKeyMap.set(key, false);

  if (cancelAble && downCB && upCB) {
    cancelAbleKeyList.push(key);
    const cancelAbleUpCB = async () => {
      loopDownKeyCBMap.delete(key);
      upKeyCBMap.delete(key);
      await upCB();
    };
    if (loopCB)
      downKeyCBMap.set(key, async () => {
        loopDownKeyCBMap.set(key, loopCB);
        upKeyCBMap.set(key, cancelAbleUpCB);
        await downCB();
      });
    else {
      // no loopCB
      downKeyCBMap.set(key, async () => {
        upKeyCBMap.set(key, cancelAbleUpCB);
        await downCB();
      });
    }
  } else {
    if (downCB && loopCB)
      downKeyCBMap.set(key, async () => {
        downCB();
        loopDownKeyCBMap.set(key, loopCB);
      });
    else if (downCB) {
      // no loopCB
      downKeyCBMap.set(key, downCB);
    }

    if (upCB) {
      upKeyCBMap.set(key, async () => {
        loopDownKeyCBMap.delete(key);
        upCB();
      });
    }
  }
}

/**
 * execute the json object macro
 * @param macro
 * @example
 * await execMacro([
 *  // touch down
 *   {
 *    type: "touch",
 *    // op, pointerId, posX, posY
 *    args: ["down", 5, ["mouse", -10], 600]
 *  },
 *  // sleep 1000ms
 *  {
 *   type: "sleep",
 *  // time(ms)
 *   args: [1000]
 *  },
 *  // touch up
 *  {
 *  type: "touch",
 *  args: ["up", 5, ["mouse", 10], 600]
 *  },
 *  // touch 1000ms
 *  {
 *  type: "touch",
 *  args: ["default", 5, ["mouse", 10], 600, 1000]
 *  },
 *  // swipe
 *  {
 *   type: "swipe",
 *   // op, pointerId, posList, intervalBetweenPos
 *   args: [
 *      "default", 5,
 *     [
 *       [
 *         ["mouse", 100],
 *         ["mouse", -100]
 *       ],
 *       ["mouse", "mouse"]
 *     ],
 *     1000
 *   ]
 *  },
 *  // key-input-mode
 *  {
 *    type: "key-input-mode",
 *    args: []
 *  }
 * ]);
 */
async function execMacro(
  relativeSize: { w: number; h: number },
  macro: KeyMacroList
) {
  if (macro === null) return;
  for (const cmd of macro) {
    if (!cmd.hasOwnProperty("type") || !cmd.hasOwnProperty("args")) {
      console.error("Invalid command: ", cmd);
      return;
    }
    try {
      switch (cmd.type) {
        case "sleep":
          await sleep(cmd.args[0]);
          break;
        case "touch":
          let touchAction;
          switch (cmd.args[0]) {
            case "default":
              touchAction = TouchAction.Default;
              break;
            case "down":
              touchAction = TouchAction.Down;
              break;
            case "up":
              touchAction = TouchAction.Up;
              break;
            case "move":
              touchAction = TouchAction.Move;
              break;
            default:
              console.error("Invalid touch action: ", cmd.args[0]);
              return;
          }
          await touchX(
            touchAction,
            cmd.args[1],
            calculateMacroPosX(cmd.args[2], relativeSize.w),
            calculateMacroPosY(cmd.args[3], relativeSize.h),
            cmd.args.length > 4 ? cmd.args[4] : undefined
          );
          break;
        case "swipe":
          let swipeAction;
          switch (cmd.args[0]) {
            case "default":
              swipeAction = SwipeAction.Default;
              break;
            case "noUp":
              swipeAction = SwipeAction.NoUp;
              break;
            case "noDown":
              swipeAction = SwipeAction.NoDown;
              break;
            default:
              console.error("Invalid swipe action: ", cmd.args[0]);
              return;
          }
          await swipe({
            action: swipeAction,
            pointerId: cmd.args[1],
            screen: {
              w: store.screenSizeW,
              h: store.screenSizeH,
            },
            pos: calculateMacroPosList(cmd.args[2], relativeSize),
            intervalBetweenPos: cmd.args[3],
          });
          break;
        case "key-input-mode":
          if (!store.keyInputFlag) {
            // on
            unlistenToEvent(true);
            KeyInputHandler.addEventListener();
            store.keyInputFlag = true;
          } else {
            // off
            KeyInputHandler.removeEventListener();
            listenToEvent(true);
            store.keyInputFlag = false;
          }
          break;
        default:
          console.error("Invalid command: ", cmd);
          return;
      }
    } catch (e) {
      console.error("Invalid command: ", cmd, e);
      return;
    }
  }
}

let loopFlag = false;
function execLoopCB() {
  loopDownKeyCBMap.forEach((cb) => {
    cb();
  });
  if (loopFlag) requestAnimationFrame(execLoopCB);
}

// change ts type
function asType<T>(_val: any): asserts _val is T {}

function applyKeyMappingConfigShortcuts(
  keyMappingConfig: KeyMappingConfig
): boolean {
  try {
    const relativeSize = keyMappingConfig.relativeSize;
    for (const item of keyMappingConfig.list) {
      switch (item.type) {
        case "SteeringWheel":
          asType<KeySteeringWheel>(item);
          addSteeringWheelKeyboardShortcuts(
            item.key,
            relativeSize,
            item.posX,
            item.posY,
            item.offset,
            item.pointerId
          );
          break;
        case "DirectionalSkill":
          asType<KeyDirectionalSkill>(item);
          addDirectionalSkillShortcuts(
            item.key,
            relativeSize,
            item.posX,
            item.posY,
            item.range,
            item.pointerId
          );
          break;
        case "DirectionlessSkill":
          asType<KeyDirectionlessSkill>(item);
          addDirectionlessSkillShortcuts(
            item.key,
            relativeSize,
            item.posX,
            item.posY,
            item.pointerId
          );
          break;
        case "CancelSkill":
          asType<KeyCancelSkill>(item);
          addCancelSkillShortcuts(
            item.key,
            relativeSize,
            item.posX,
            item.posY,
            item.pointerId
          );
          break;
        case "Tap":
          asType<KeyTap>(item);
          addTapShortcuts(
            item.key,
            relativeSize,
            item.time,
            item.posX,
            item.posY,
            item.pointerId
          );
          break;
        case "Swipe":
          asType<KeySwipe>(item);
          addSwipeShortcuts(
            item.key,
            relativeSize,
            item.pos,
            item.pointerId,
            item.intervalBetweenPos
          );
          break;
        case "TriggerWhenPressedSkill":
          asType<KeyTriggerWhenPressedSkill>(item);
          addTriggerWhenPressedSkillShortcuts(
            item.key,
            relativeSize,
            item.posX,
            item.posY,
            item.directional,
            item.rangeOrTime,
            item.pointerId
          );
          break;
        case "TriggerWhenDoublePressedSkill":
          asType<KeyTriggerWhenDoublePressedSkill>(item);
          addTriggerWhenDoublePressedSkillShortcuts(
            item.key,
            relativeSize,
            item.posX,
            item.posY,
            item.range,
            item.pointerId
          );
          break;
        case "Observation":
          asType<KeyObservation>(item);
          addObservationShortcuts(
            item.key,
            relativeSize,
            item.posX,
            item.posY,
            item.scale,
            item.pointerId
          );
          break;
        case "Macro":
          asType<KeyMacro>(item);
          addShortcut(
            item.key,
            item.macro.down === null
              ? undefined
              : async () => {
                  await execMacro(relativeSize, item.macro.down);
                },
            item.macro.loop === null
              ? undefined
              : async () => {
                  await execMacro(relativeSize, item.macro.loop);
                },
            item.macro.up === null
              ? undefined
              : async () => {
                  await execMacro(relativeSize, item.macro.up);
                }
          );
          break;
        case "Sight":
          asType<KeySight>(item);
          let fireKeyMapping;
          for (const keyMapping of keyMappingConfig.list) {
            if (keyMapping.type === "Fire") {
              fireKeyMapping = keyMapping;
              break;
            }
          }
          addSightShortcuts(relativeSize, item, fireKeyMapping);
          break;
        case "Fire":
          break;
        default:
          console.error("Invalid item type: ", item);
          break;
      }
    }
    return true;
  } catch (e) {
    console.error("Invalid keyMappingConfig: ", keyMappingConfig, e);
    clearShortcuts();
    return false;
  }
}

async function touchX(
  action: TouchAction,
  pointerId: number,
  posX: number,
  posY: number,
  time?: number
) {
  await touch({
    action,
    pointerId,
    screen: {
      w: store.screenSizeW,
      h: store.screenSizeH,
    },
    pos: {
      x: posX,
      y: posY,
    },
    time,
  });
}

export function listenToEvent(onlyKeyEvent = false) {
  window.addEventListener("keydown", handleKeydown);
  window.addEventListener("keyup", handleKeyup);
  if (onlyKeyEvent) return;

  window.addEventListener("mousedown", handleMouseDown);
  window.addEventListener("mousemove", handleMouseMove);
  window.addEventListener("mouseup", handleMouseUp);
  window.addEventListener("wheel", handleMouseWheel);
  loopFlag = true;
  execLoopCB();
}

export function unlistenToEvent(onlyKeyEvent = false) {
  window.removeEventListener("keydown", handleKeydown);
  window.removeEventListener("keyup", handleKeyup);
  if (onlyKeyEvent) return;

  window.removeEventListener("mousedown", handleMouseDown);
  window.removeEventListener("mousemove", handleMouseMove);
  window.removeEventListener("mouseup", handleMouseUp);
  window.removeEventListener("wheel", handleMouseWheel);

  loopFlag = false;
}

export function clearShortcuts() {
  downKeyMap.clear();
  downKeyCBMap.clear();
  loopDownKeyCBMap.clear();
  upKeyCBMap.clear();
  cancelAbleKeyList.length = 0;
}

export function applyShortcuts(
  keyMappingConfig: KeyMappingConfig,
  globalStore: ReturnType<typeof useGlobalStore>,
  messageAPI: ReturnType<typeof useMessage>,
  i18nT: ReturnType<typeof useI18n>["t"]
) {
  store = globalStore;
  maskElement = document.getElementById("maskElement") as HTMLElement;
  message = messageAPI;
  t = i18nT;

  addClickShortcuts("M0", 0);

  return applyKeyMappingConfigShortcuts(keyMappingConfig);
}

let mouseX = 0;
let mouseY = 0;
let store: ReturnType<typeof useGlobalStore>;
let maskElement: HTMLElement;
let message: ReturnType<typeof useMessage>;
let t: ReturnType<typeof useI18n>["t"];

const downKeyMap: Map<string, boolean> = new Map();
const downKeyCBMap: Map<string, () => Promise<void>> = new Map();
const loopDownKeyCBMap: Map<string, () => Promise<void>> = new Map();
const upKeyCBMap: Map<string, () => Promise<void>> = new Map();
const cancelAbleKeyList: string[] = [];

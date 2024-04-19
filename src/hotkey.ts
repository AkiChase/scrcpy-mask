// https://github.com/jamiebuilds/tinykeys/pull/193/commits/2598ecb3db6b3948c7acbf0e7bd8b0674961ad61
import {
  SwipeAction,
  TouchAction,
  swipe,
  touch,
} from "./frontcommand/scrcpyMaskCmd";

function clientxToPosx(clientx: number) {
  return clientx < 70
    ? 0
    : Math.floor((clientx - 70) * (screenSizeW / maskSizeW));
}

function clientyToPosy(clienty: number) {
  return clienty < 30
    ? 0
    : Math.floor((clienty - 30) * (screenSizeH / maskSizeH));
}

function clientxToPosOffsetx(clientx: number, posx: number, scale: number) {
  let offsetX = clientxToPosx(clientx) - posx;
  return Math.round(offsetX * scale);
}

function clientyToPosOffsety(clienty: number, posy: number, scale: number) {
  let offsetY = clientyToPosy(clienty) - posy;
  return Math.round(offsetY * scale);
}

function clientxToCenterOffsetx(clientx: number, range: number, scale = 0.5) {
  return Math.max(
    -range,
    Math.min(range, clientxToPosOffsetx(clientx, screenSizeW / 2, scale))
  );
}

function clientyToCenterOffsety(clienty: number, range: number, scale = 0.5) {
  return Math.max(
    -range,
    Math.min(range, clientyToPosOffsety(clienty, screenSizeH * 0.55, scale))
  );
}

async function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function calculateMacroPosX(
  posX: [string, number] | number,
  relativeSizeW: number
): number {
  if (typeof posX === "number") {
    return Math.round(posX * (screenSizeW / relativeSizeW));
  }
  if (typeof posX === "string") {
    return clientxToPosx(mouseX);
  } else {
    if (posX[0] === "mouse") {
      return (
        clientxToPosx(mouseX) +
        Math.round(posX[1] * (screenSizeW / relativeSizeW))
      );
    } else {
      throw new Error("Invalid pos");
    }
  }
}

function calculateMacroPosY(
  posY: [string, number] | number,
  relativeSizeH: number
): number {
  if (typeof posY === "number") {
    return Math.round(posY * (screenSizeH / relativeSizeH));
  }
  if (typeof posY === "string") {
    return clientyToPosy(mouseY);
  } else {
    if (posY[0] === "mouse") {
      return (
        clientyToPosy(mouseY) +
        Math.round(posY[1] * (screenSizeH / relativeSizeH))
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

// TODO 2. 根据配置初始化快捷键
// TODO ? 技能界面实际上是有投影变换的，需要一定的算法，不能仅仅相对坐标 （640,400）

// add shortcuts for observation
function addObservationShortcuts(
  key: string,
  relativeSize: { w: number; h: number },
  posX: number,
  posY: number,
  scale: number,
  pointerId: number
) {
  let observationMouseX = 0;
  let observationMouseY = 0;
  posX = Math.round((posX / relativeSize.w) * screenSizeW);
  posY = Math.round((posY / relativeSize.h) * screenSizeH);
  addShortcut(
    key,
    async () => {
      observationMouseX = clientxToPosx(mouseX);
      observationMouseY = clientyToPosy(mouseY);
      await touch({
        action: TouchAction.Down,
        pointerId,
        screen: {
          w: screenSizeW,
          h: screenSizeH,
        },
        pos: {
          x: posX,
          y: posY,
        },
      });
    },
    async () => {
      await touch({
        action: TouchAction.Move,
        pointerId,
        screen: {
          w: screenSizeW,
          h: screenSizeH,
        },
        pos: {
          x: posX + clientxToPosOffsetx(mouseX, observationMouseX, scale),
          y: posY + clientyToPosOffsety(mouseY, observationMouseY, scale),
        },
      });
    },
    async () => {
      await touch({
        action: TouchAction.Up,
        pointerId,
        screen: {
          w: screenSizeW,
          h: screenSizeH,
        },
        pos: {
          x: posX + clientxToPosOffsetx(mouseX, observationMouseY, scale),
          y: posY + clientyToPosOffsety(mouseY, observationMouseY, scale),
        },
      });
    }
  );
}

// add shortcuts for simple tap (touch for 100 ms when pressed)
function addTapShortcuts(
  key: string,
  relativeSize: { w: number; h: number },
  time: number,
  posX: number,
  posY: number,
  pointerId: number
) {
  posX = Math.round((posX / relativeSize.w) * screenSizeW);
  posY = Math.round((posY / relativeSize.h) * screenSizeH);
  addShortcut(
    key,
    async () => {
      await touch({
        action: TouchAction.Default,
        pointerId,
        screen: {
          w: screenSizeW,
          h: screenSizeH,
        },
        pos: {
          x: posX,
          y: posY,
        },
        time,
      });
    },
    undefined,
    undefined
  );
}

// add shortcuts for cancel skill
function addCancelSkillShortcuts(
  key: string,
  relativeSize: { w: number; h: number },
  posX: number,
  posY: number,
  pointerId: number
) {
  posX = Math.round((posX / relativeSize.w) * screenSizeW);
  posY = Math.round((posY / relativeSize.h) * screenSizeH);
  addShortcut(
    key,
    async () => {
      // delete the callback
      for (const cancelAbleKey of cancelAbleKeyList) {
        loopDownKeyCBMap.delete(cancelAbleKey);
        upKeyCBMap.delete(cancelAbleKey);
      }

      let distance = 0;
      while (distance <= 20) {
        await touch({
          action: TouchAction.Move,
          pointerId,
          screen: {
            w: screenSizeW,
            h: screenSizeH,
          },
          pos: {
            x: posX + distance,
            y: posY,
          },
        });
        await sleep(5);
        distance += 1;
      }
      await touch({
        action: TouchAction.Up,
        pointerId,
        screen: {
          w: screenSizeW,
          h: screenSizeH,
        },
        pos: {
          x: posX,
          y: posY,
        },
      });
    },
    undefined,
    undefined
  );
}

// add shortcuts for trigger when pressed skill
function addTriggerWhenPressedSkillShortcuts(
  key: string,
  relativeSize: { w: number; h: number },
  // pos relative to the device
  posX: number,
  posY: number,
  directional: boolean,
  // range is needed when directional is true
  range: number,
  pointerId: number
) {
  if (directional) {
    posX = Math.round((posX / relativeSize.w) * screenSizeW);
    posY = Math.round((posY / relativeSize.h) * screenSizeH);
    addShortcut(
      key,
      // down
      async () => {
        await swipe({
          action: SwipeAction.Default,
          pointerId,
          screen: {
            w: screenSizeW,
            h: screenSizeH,
          },
          pos: [
            { x: posX, y: posY },
            {
              x: posX + clientxToCenterOffsetx(mouseX, range),
              y: posY + clientyToCenterOffsety(mouseY, range),
            },
          ],
          intervalBetweenPos: 0,
        });
      },
      undefined,
      undefined
    );
  } else {
    addTapShortcuts(key, relativeSize, 80, posX, posY, pointerId);
  }
}

// add shortcuts for directionless skill (cancelable)
function addDirectionlessSkillShortcuts(
  key: string,
  relativeSize: { w: number; h: number },
  // pos relative to the device
  posX: number,
  posY: number,
  pointerId: number
) {
  posX = Math.round((posX / relativeSize.w) * screenSizeW);
  posY = Math.round((posY / relativeSize.h) * screenSizeH);
  addShortcut(
    key,
    // down
    async () => {
      await touch({
        action: TouchAction.Down,
        pointerId,
        screen: {
          w: screenSizeW,
          h: screenSizeH,
        },
        pos: {
          x: posX,
          y: posY,
        },
      });
    },
    // loop
    undefined,
    // up
    async () => {
      await touch({
        action: TouchAction.Up,
        pointerId,
        screen: {
          w: screenSizeW,
          h: screenSizeH,
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

// add shortcuts for directional skill (cancelable)
function addDirectionalSkillShortcuts(
  key: string,
  relativeSize: { w: number; h: number },
  // pos relative to the device
  posX: number,
  posY: number,
  range: number,
  pointerId: number
) {
  posX = Math.round((posX / relativeSize.w) * screenSizeW);
  posY = Math.round((posY / relativeSize.h) * screenSizeH);
  addShortcut(
    key,
    // down
    async () => {
      await swipe({
        action: SwipeAction.NoUp,
        pointerId,
        screen: {
          w: screenSizeW,
          h: screenSizeH,
        },
        pos: [
          { x: posX, y: posY },
          {
            x: posX + clientxToCenterOffsetx(mouseX, range),
            y: posY + clientyToCenterOffsety(mouseY, range),
          },
        ],
        intervalBetweenPos: 0,
      });
    },
    // loop
    async () => {
      await touch({
        action: TouchAction.Move,
        pointerId,
        screen: {
          w: screenSizeW,
          h: screenSizeH,
        },
        pos: {
          x: posX + clientxToCenterOffsetx(mouseX, range),
          y: posY + clientyToCenterOffsety(mouseY, range),
        },
      });
    },
    // up
    async () => {
      await touch({
        action: TouchAction.Up,
        pointerId,
        screen: {
          w: screenSizeW,
          h: screenSizeH,
        },
        pos: {
          x: posX + clientxToCenterOffsetx(mouseX, range),
          y: posY + clientyToCenterOffsety(mouseY, range),
        },
      });
    },
    true
  );
}

// add shortcuts for steering wheel
function addSteeringWheelKeyboardShortcuts(
  key: wheelKey,
  relativeSize: { w: number; h: number },
  // pos relative to the device
  posX: number,
  posY: number,
  offset: number,
  pointerId: number
) {
  let loopFlag = false;
  let curPosX = 0;
  let curPosY = 0;
  posX = Math.round((posX / relativeSize.w) * screenSizeW);
  posY = Math.round((posY / relativeSize.h) * screenSizeH);

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
    await touch({
      action: TouchAction.Move,
      pointerId,
      screen: {
        w: screenSizeW,
        h: screenSizeH,
      },
      pos: newPos,
    });
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
      await touch({
        action: TouchAction.Up,
        pointerId,
        screen: {
          w: screenSizeW,
          h: screenSizeH,
        },
        pos: {
          x: curPosX,
          y: curPosY,
        },
      });
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
        await touch({
          action: TouchAction.Down,
          pointerId,
          screen: {
            w: screenSizeW,
            h: screenSizeH,
          },
          pos: {
            x: posX,
            y: posY,
          },
        });
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
      await touch({
        action: TouchAction.Down,
        pointerId,
        screen: {
          w: screenSizeW,
          h: screenSizeH,
        },
        pos: {
          x: clientxToPosx(mouseX),
          y: clientyToPosy(mouseY),
        },
      });
    },
    async () => {
      await touch({
        action: TouchAction.Move,
        pointerId,
        screen: {
          w: screenSizeW,
          h: screenSizeH,
        },
        pos: {
          x: clientxToPosx(mouseX),
          y: clientyToPosy(mouseY),
        },
      });
    },
    async () => {
      await touch({
        action: TouchAction.Up,
        pointerId,
        screen: {
          w: screenSizeW,
          h: screenSizeH,
        },
        pos: {
          x: clientxToPosx(mouseX),
          y: clientyToPosy(mouseY),
        },
      });
    }
  );
}

let screenSizeW: number;
let screenSizeH: number;
let maskSizeW: number;
let maskSizeH: number;
let mouseX = 0;
let mouseY = 0;

const downKeyMap: Map<string, boolean> = new Map();
const downKeyCBMap: Map<string, () => Promise<void>> = new Map();
const loopDownKeyCBMap: Map<string, () => Promise<void>> = new Map();
const upKeyCBMap: Map<string, () => Promise<void>> = new Map();
const cancelAbleKeyList: string[] = [];

function keydownHandler(event: KeyboardEvent) {
  if (event.repeat) return;
  event.preventDefault();
  if (downKeyMap.has(event.code)) {
    downKeyMap.set(event.code, true);
    // execute the down callback (if there is) asyncily
    let cb = downKeyCBMap.get(event.code);
    if (cb) cb();
  }
}

function keyupHandler(event: KeyboardEvent) {
  event.preventDefault();
  if (downKeyMap.has(event.code)) {
    downKeyMap.set(event.code, false);
    // execute the up callback (if there is) asyncily
    let cb = upKeyCBMap.get(event.code);
    if (cb) cb();
  }
}

function handleMouseDown(event: MouseEvent) {
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
  if (downKeyMap.has(key)) {
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
  } else if (event.deltaY < 0 && event.timeStamp - lastWheelUpTime > 50) {
    lastWheelUpTime = event.timeStamp;
    // WheelUp
    downKeyCBMap.get("WheelUp")?.();
  }
}

function addShortcut(
  key: string,
  downCB: () => Promise<void>,
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
 *    args: ["down", 5, ["mouse", -10], 600],
 *  },
 *  // sleep 1000ms
 *  {
 *   type: "sleep",
 *  // time(ms)
 *   args: [1000],
 *  },
 *  // touch up
 *  {
 *  type: "touch",
 *  args: ["up", 5, ["mouse", 10], 600],
 *  },
 *  // touch 1000ms
 *  {
 *  type: "touch",
 *  args: ["default", 5, ["mouse", 10], 600, 1000],
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
 *         ["mouse", -100],
 *       ],
 *       ["mouse", "mouse"],
 *     ],
 *     1000,
 *   ],
 *  },
 * ]);
 */

async function execMacro(relativeSize: { w: number; h: number }, macro: any[]) {
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
          await touch({
            action: touchAction,
            pointerId: cmd.args[1],
            screen: {
              w: screenSizeW,
              h: screenSizeH,
            },
            pos: {
              x: calculateMacroPosX(cmd.args[2], relativeSize.w),
              y: calculateMacroPosY(cmd.args[3], relativeSize.h),
            },
            time: cmd.args.length > 4 ? cmd.args[4] : undefined,
          });
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
              w: screenSizeW,
              h: screenSizeH,
            },
            pos: calculateMacroPosList(cmd.args[2], relativeSize),
            intervalBetweenPos: cmd.args[3],
          });
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

export function listenToKeyEvent() {
  document.addEventListener("keydown", keydownHandler);
  document.addEventListener("keyup", keyupHandler);
  loopFlag = true;
  execLoopCB();
  // setInterval(()=>console.log(loopDownKeyCBMap), 3000);
}

export function unlistenToKeyEvent() {
  document.removeEventListener("keydown", keydownHandler);
  document.removeEventListener("keyup", keyupHandler);
  loopFlag = false;
}

export function clearShortcuts() {
  downKeyMap.clear();
  downKeyCBMap.clear();
  loopDownKeyCBMap.clear();
  upKeyCBMap.clear();
  cancelAbleKeyList.length = 0;
}

export function updateScreenSizeAndMaskArea(
  screenSize: [number, number],
  maskArea: [number, number]
) {
  screenSizeW = screenSize[0];
  screenSizeH = screenSize[1];
  maskSizeW = maskArea[0];
  maskSizeH = maskArea[1];
}

export function applyShortcuts(element: HTMLElement) {
  element.addEventListener("mousedown", handleMouseDown);
  element.addEventListener("mousemove", handleMouseMove);
  element.addEventListener("mouseup", handleMouseUp);
  element.addEventListener("wheel", handleMouseWheel);
  // TODO 1.2 使用setCursorGrab相关来限制移出，而不是使用下面的方法
  // TODO 任何down的时候都要限制移出
  element.addEventListener("mouseout", handleMouseUp); // mouse out of the element as mouse up

  // 读取按键配置文件时获取
  const relativeSize = { w: 1280, h: 720 };

  addClickShortcuts("M0", 0);
  addSteeringWheelKeyboardShortcuts(
    {
      left: "KeyA",
      right: "KeyD",
      up: "KeyW",
      down: "KeyS",
    },
    relativeSize,
    180,
    560,
    100,
    1
  );
  addDirectionalSkillShortcuts("KeyQ", relativeSize, 950, 610, 200, 2); // skill 1
  addDirectionalSkillShortcuts("AltLeft", relativeSize, 1025, 500, 200, 2); // skill 2
  addDirectionalSkillShortcuts("KeyE", relativeSize, 1160, 420, 200, 2); // skill 3
  addTriggerWhenPressedSkillShortcuts(
    "M4",
    relativeSize,
    1160,
    420,
    false,
    0,
    2
  ); // skill 3 (no direction and trigger when pressed)
  addDirectionlessSkillShortcuts("M1", relativeSize, 1150, 280, 2); // equipment skill (middle mouse click)
  addCancelSkillShortcuts("Space", relativeSize, 1160, 140, 2); // cancel skill

  addTapShortcuts("KeyB", relativeSize, 80, 650, 650, 3); // home
  addTapShortcuts("KeyC", relativeSize, 80, 740, 650, 3); // recover
  addDirectionalSkillShortcuts("KeyF", relativeSize, 840, 650, 200, 2); // summoner skills
  addTriggerWhenPressedSkillShortcuts(
    "ControlLeft",
    relativeSize,
    840,
    650,
    false,
    0,
    3
  ); // summoner skills (no direction and trigger when pressed)
  addTapShortcuts("M2", relativeSize, 80, 1165, 620, 3); // attack (right click)
  addTapShortcuts("Digit1", relativeSize, 80, 880, 560, 3); // skill 1 upgrade
  addTapShortcuts("Digit2", relativeSize, 80, 960, 430, 3); // skill 2 upgrade
  addTapShortcuts("Digit3", relativeSize, 80, 1090, 350, 3); // skill 3 upgrade
  addTapShortcuts("Digit5", relativeSize, 80, 130, 300, 3); // quick buy 1
  addTapShortcuts("Digit6", relativeSize, 80, 130, 370, 3); // quick buy 2

  addTapShortcuts("WheelDown", relativeSize, 80, 130, 440, 3); // equipment skill

  addObservationShortcuts("M3", relativeSize, 1000, 200, 0.5, 4); // observation

  // panel
  addShortcut(
    "Tab",
    async () => {
      await execMacro(relativeSize, [
        {
          type: "touch",
          args: ["default", 5, 1185, 40, 80],
        },
      ]);
    },
    undefined,
    async () => {
      await execMacro(relativeSize, [
        {
          type: "touch",
          args: ["default", 5, 1220, 100, 80],
        },
      ]);
    }
  );

  // shop
  addShortcut(
    "ShiftLeft",
    async () => {
      await execMacro(relativeSize, [
        {
          type: "touch",
          args: ["default", 5, 40, 300, 80],
        },
      ]);
    },
    undefined,
    async () => {
      await execMacro(relativeSize, [
        {
          type: "touch",
          args: ["default", 5, 1200, 60, 80],
        },
      ]);
    }
  );

  // map
  addShortcut(
    "KeyZ",
    async () => {
      await execMacro(relativeSize, [
        {
          type: "touch",
          args: ["default", 5, 250, 230, 80],
        },
      ]);
    },
    undefined,
    async () => {
      await execMacro(relativeSize, [
        {
          type: "touch",
          args: ["default", 5, 640, 150, 80],
        },
      ]);
    }
  );
}

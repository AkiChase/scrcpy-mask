// https://github.com/jamiebuilds/tinykeys/pull/193/commits/2598ecb3db6b3948c7acbf0e7bd8b0674961ad61
import {
  SwipeAction,
  TouchAction,
  swipe,
  touch,
} from "./frontcommand/scrcpyMaskCmd";

function clientxToPosx(clientx: number) {
  return clientx < 70 ? 0 : Math.floor(clientx - 70);
}

function clientyToPosy(clienty: number) {
  return clienty < 30 ? 0 : Math.floor(clienty - 30);
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
    Math.min(range, clientyToPosOffsety(clienty, screenSizeH * 0.57, scale))
  );
}

async function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function calculateMacroPosX(pos: [string, number] | number): number {
  if (typeof pos === "number") {
    return pos;
  }
  if (typeof pos === "string") {
    return clientxToPosx(mouseX);
  } else {
    if (pos[0] === "mouse") {
      return clientxToPosx(mouseX) + pos[1];
    } else {
      throw new Error("Invalid pos");
    }
  }
}

function calculateMacroPosY(pos: [string, number] | number): number {
  if (typeof pos === "number") {
    return pos;
  }
  if (typeof pos === "string") {
    return clientyToPosy(mouseY);
  } else {
    if (pos[0] === "mouse") {
      return clientyToPosy(mouseY) + pos[1];
    } else {
      throw new Error("Invalid pos");
    }
  }
}

function calculateMacroPosList(
  posList: [[string, number] | number, [string, number] | number][]
): { x: number; y: number }[] {
  return posList.map((posPair) => {
    return {
      x: calculateMacroPosX(posPair[0]),
      y: calculateMacroPosY(posPair[1]),
    };
  });
}

// TODO 方向轮盘也放到loop内吧，两个独立的循环不太好。但至少要将轮盘的循环间隔调低。
// TODO 偶尔不定时抽风（但等待一会就能恢复正常），表现为setinterval中的回调函数没有执行

// add shortcuts for observation
function addObservationShortcuts(
  key: string,
  posX: number,
  posY: number,
  scale: number,
  pointerId: number
) {
  let observationMouseX = 0;
  let observationMouseY = 0;
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
  posX: number,
  posY: number,
  pointerId: number
) {
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
      });
    },
    undefined,
    undefined
  );
}

// add shortcuts for cancel skill
function addCancelSkillShortcuts(
  key: string,
  posX: number,
  posY: number,
  pointerId: number
) {
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
  // pos relative to the device
  posX: number,
  posY: number,
  directional: boolean,
  // range is needed when directional is true
  range: number,
  pointerId: number
) {
  if (directional) {
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
    addTapShortcuts(key, posX, posY, pointerId);
  }
}

// add shortcuts for directionless skill (cancelable)
function addDirectionlessSkillShortcuts(
  key: string,
  // pos relative to the device
  posX: number,
  posY: number,
  pointerId: number
) {
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
  // pos relative to the device
  posX: number,
  posY: number,
  range: number,
  pointerId: number
) {
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
  // pos relative to the device
  posX: number,
  posY: number,
  offset: number,
  pointerId: number
) {
  for (const k of ["left", "right", "up", "down"]) {
    if (key[k])
      addShortcut(
        key[k],
        async () => {
          _keyDownFlag[k] = true;
          await _wheelKeyLoop(pointerId, posX, posY, offset);
        },
        undefined,
        async () => {
          _keyDownFlag[k] = false;
        }
      );
  }
}

let _keyDownFlag: stringKeyFlag = {
  left: false,
  right: false,
  up: false,
  down: false,
};
let _isWheelKeyLoopRunning = false;
// single loop for the steering wheel
async function _wheelKeyLoop(
  pointerId: number,
  // pos relative to the device
  posX: number,
  posY: number,
  offset: number
) {
  if (_isWheelKeyLoopRunning) return;
  _isWheelKeyLoopRunning = true;
  // calculate the end coordinates of the eight directions of the direction wheel
  let offsetHalf = Math.round(offset / 1.414);

  const pos = [
    { x: posX - offset, y: posY }, // left
    { x: posX + offset, y: posY }, // right
    { x: posX, y: posY - offset }, // up
    { x: posX, y: posY + offset }, // down
    { x: posX - offsetHalf, y: posY - offsetHalf }, // left up
    { x: posX + offsetHalf, y: posY - offsetHalf }, // right up
    { x: posX - offsetHalf, y: posY + offsetHalf }, // left down
    { x: posX + offsetHalf, y: posY + offsetHalf }, // right down
  ];

  // touch down on the center position
  await touch({
    action: TouchAction.Down,
    pointerId: pointerId,
    screen: {
      w: screenSizeW,
      h: screenSizeH,
    },
    pos: {
      x: posX,
      y: posY,
    },
  });

  // move to the direction
  let curPos;
  let _lastKeyDownFlag: stringKeyFlag = {
    left: true,
    right: true,
    up: true,
    down: true,
  };
  while (
    _keyDownFlag.left ||
    _keyDownFlag.right ||
    _keyDownFlag.up ||
    _keyDownFlag.down
  ) {
    // if key down not changed
    if (
      _keyDownFlag.left === _lastKeyDownFlag.left &&
      _keyDownFlag.right === _lastKeyDownFlag.right &&
      _keyDownFlag.up === _lastKeyDownFlag.up &&
      _keyDownFlag.down === _lastKeyDownFlag.down
    ) {
      await sleep(50);
      continue;
    }
    // record the last key down flag
    _lastKeyDownFlag = { ..._keyDownFlag };
    // key down changed
    if (_keyDownFlag.left) {
      curPos = _keyDownFlag.up ? pos[4] : _keyDownFlag.down ? pos[6] : pos[0];
    } else if (_keyDownFlag.right) {
      curPos = _keyDownFlag.up ? pos[5] : _keyDownFlag.down ? pos[7] : pos[1];
    } else if (_keyDownFlag.up) {
      curPos = pos[2];
    } else if (_keyDownFlag.down) {
      curPos = pos[3];
    } else {
      curPos = { x: posX, y: posY };
    }
    await touch({
      action: TouchAction.Move,
      pointerId: pointerId,
      screen: {
        w: screenSizeW,
        h: screenSizeH,
      },
      pos: curPos,
    });
    await sleep(100);
  }
  // touch up
  await touch({
    action: TouchAction.Up,
    pointerId: pointerId,
    screen: {
      w: screenSizeW,
      h: screenSizeH,
    },
    pos: curPos ? curPos : { x: posX, y: posY },
  });
  _isWheelKeyLoopRunning = false;
}
interface wheelKey {
  left: string;
  right: string;
  up: string;
  down: string;
  [key: string]: string;
}
type stringKeyFlag = Record<string, boolean>;

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
    // execute the down callback (if there is) asyncily
    let cb = downKeyCBMap.get(event.code);
    if (cb) cb();
    downKeyMap.set(event.code, true);
  }
}

function keyupHandler(event: KeyboardEvent) {
  event.preventDefault();
  if (downKeyMap.has(event.code)) {
    // execute the up callback (if there is) asyncily
    let cb = upKeyCBMap.get(event.code);
    if (cb) cb();
    downKeyMap.set(event.code, false);
  }
}

function handleMouseDown(event: MouseEvent) {
  mouseX = event.clientX;
  mouseY = event.clientY;
  event.preventDefault();
  let key = "M" + event.button.toString();
  if (downKeyMap.has(key)) {
    // execute the down callback asyncily
    let cb = downKeyCBMap.get(key);
    if (cb) cb();
    downKeyMap.set(key, true);
  }
}

function handleMouseUp(event: MouseEvent) {
  mouseX = event.clientX;
  mouseY = event.clientY;
  event.preventDefault();
  let key = "M" + event.button.toString();
  if (downKeyMap.has(key)) {
    // execute the up callback asyncily
    let cb = upKeyCBMap.get(key);
    if (cb) cb();
    downKeyMap.set(key, false);
  }
}

function handleMouseMove(event: MouseEvent) {
  mouseX = event.clientX;
  mouseY = event.clientY;
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
 *   {
 *    type: "touch",
 *    // op, pointerId, posX, posY
 *    args: ["down", 5, ["mouse", -10], 600],
 *  },
 *  {
 *   type: "sleep",
 *   // time(ms)
 *   args: [1000],
 *  },
 *  {
 *  type: "touch",
 *  args: ["up", 5, ["mouse", 10], 600],
 *  },
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
async function execMacro(macro: any[]) {
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
              x: calculateMacroPosX(cmd.args[2]),
              y: calculateMacroPosY(cmd.args[3]),
            },
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
            pos: calculateMacroPosList(cmd.args[2]),
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

let timmerId: number = 0;
export function listenToKeyEvent() {
  document.addEventListener("keydown", keydownHandler);
  document.addEventListener("keyup", keyupHandler);
  timmerId = setInterval(() => {
    loopDownKeyCBMap.forEach((cb) => {
      cb();
    });
  }, 50);
}

export function unlistenToKeyEvent() {
  document.removeEventListener("keydown", keydownHandler);
  document.removeEventListener("keyup", keyupHandler);
  clearInterval(timmerId);
}

export function initShortcuts(
  screenSize: [number, number],
  element: HTMLElement
) {
  screenSizeW = screenSize[0];
  screenSizeH = screenSize[1];

  element.addEventListener("mousedown", handleMouseDown);
  element.addEventListener("mousemove", handleMouseMove);
  element.addEventListener("mouseup", handleMouseUp);
  element.addEventListener("mouseout", handleMouseUp); // mouse out of the element as mouse up

  addClickShortcuts("M0", 0);
  addSteeringWheelKeyboardShortcuts(
    {
      left: "KeyA",
      right: "KeyD",
      up: "KeyW",
      down: "KeyS",
    },
    180,
    560,
    100,
    1
  );
  addDirectionalSkillShortcuts("KeyQ", 950, 610, 100, 2); // skill 1
  addDirectionalSkillShortcuts("AltLeft", 1025, 500, 100, 2); // skill 2
  addDirectionalSkillShortcuts("KeyE", 1160, 420, 100, 2); // skill 3
  addTriggerWhenPressedSkillShortcuts("M4", 1160, 420, false, 0, 2); // skill 3 (no direction and trigger when pressed)
  addDirectionlessSkillShortcuts("M1", 1150, 280, 2); // equipment skill (middle mouse click)
  addCancelSkillShortcuts("Space", 1160, 140, 2); // cancel skill

  addTapShortcuts("KeyB", 650, 650, 3); // home
  addTapShortcuts("KeyC", 740, 650, 3); // recover
  addDirectionalSkillShortcuts("KeyF", 840, 650, 100, 2); // summoner skills
  addTriggerWhenPressedSkillShortcuts("ControlLeft", 840, 650, false, 0, 3); // summoner skills (no direction and trigger when pressed)
  addTapShortcuts("M2", 1165, 620, 3); // attack (right click)

  addObservationShortcuts("M3", 1000, 200, 0.5, 4); // observation

  // panel
  addShortcut(
    "Tab",
    async () => {
      await execMacro([
        {
          type: "touch",
          args: ["default", 5, 1185, 40],
        },
      ]);
    },
    undefined,
    async () => {
      await execMacro([
        {
          type: "touch",
          args: ["default", 5, 1220, 100],
        },
      ]);
    }
  );

  // shop
  addShortcut(
    "ShiftLeft",
    async () => {
      await execMacro([
        {
          type: "touch",
          args: ["default", 5, 40, 300],
        },
      ]);
    },
    undefined,
    async () => {
      await execMacro([
        {
          type: "touch",
          args: ["default", 5, 1200, 60],
        },
      ]);
    }
  );
}

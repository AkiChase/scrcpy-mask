// https://github.com/jamiebuilds/tinykeys/pull/193/commits/2598ecb3db6b3948c7acbf0e7bd8b0674961ad61
import {
  SwipeAction,
  TouchAction,
  swipe,
  touch,
} from "./frontcommand/scrcpyMaskCmd";

let posFactor = 1; // it will be replaced in initMouseShortcuts
function clientxToPosx(clientx: number) {
  return clientx < 70 ? 0 : Math.floor((clientx - 70) * posFactor);
}

function clientyToPosy(clienty: number) {
  return clienty < 30 ? 0 : Math.floor((clienty - 30) * posFactor);
}

function clientxToSkillOffsetx(clientx: number, range: number, scale = 0.5) {
  // Get the offset relative to the center of the screen
  let offsetX = clientxToPosx(clientx) - screenSizeW / 2;
  return Math.max(-range, Math.min(range, Math.round(offsetX * scale)));
}

function clientyToSkillOffsety(clienty: number, range: number, scale = 0.5) {
  // Get the offset relative to the center of the screen
  let offsetY = clientyToPosy(clienty) - screenSizeH / 2;
  return Math.max(-range, Math.min(range, Math.round(offsetY * scale)));
}

async function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// TODO 取消技能
// 仅仅针对技能快捷键，要求传入pointer_id（所有技能键的都一样）, 取消位置
// 在down时就执行
// 执行时直接删除对应的loopDownKeyCBMap, upKeyCBMap键值对，恢复cursor，然后将触点使用touch-move到取消位置然后up
// TODO 普通点击
// 直接用default的touch，按下时长是写死的100ms
// TODO 视野移动
// 关键在于中心位置应该是传入的坐标，要重新计算相对偏移
// TODO 宏
// 宏也是分为down，loop，up三个阶段
// 目前只需要支持sleep, touch, swipe三个指令即可

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
  skillKeyList.push(key);
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
              x: posX + clientxToSkillOffsetx(mouseX, range),
              y: posY + clientyToSkillOffsety(mouseY, range),
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
      undefined,
      undefined
    );
  }
}

// add shortcuts for directionless skill
function addDirectionlessSkillShortcuts(
  key: string,
  // pos relative to the device
  posX: number,
  posY: number,
  pointerId: number
) {
  skillKeyList.push(key);
  addShortcut(
    key,
    // down
    async () => {
      document.body.style.cursor = "pointer";
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
      document.body.style.cursor = "";
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
    }
  );
}

// add shortcuts for directional skill
function addDirectionalSkillShortcuts(
  key: string,
  // pos relative to the device
  posX: number,
  posY: number,
  range: number,
  pointerId: number
) {
  skillKeyList.push(key);
  addShortcut(
    key,
    // down
    async () => {
      document.body.style.cursor = "pointer";
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
            x: posX + clientxToSkillOffsetx(mouseX, range),
            y: posY + clientyToSkillOffsety(mouseY, range),
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
          x: posX + clientxToSkillOffsetx(mouseX, range),
          y: posY + clientyToSkillOffsety(mouseY, range),
        },
      });
    },
    // up
    async () => {
      document.body.style.cursor = "";
      await touch({
        action: TouchAction.Up,
        pointerId,
        screen: {
          w: screenSizeW,
          h: screenSizeH,
        },
        pos: {
          x: posX + clientxToSkillOffsetx(mouseX, range),
          y: posY + clientyToSkillOffsety(mouseY, range),
        },
      });
    }
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

// add baisc left click shortcuts
function addLeftClickShortcuts(pointerId: number) {
  addShortcut(
    "M0",
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
let oldMouseX = 0;
let oldMouseY = 0;

const downKeyMap: Map<string, boolean> = new Map();
const downKeyCBMap: Map<string, () => Promise<void>> = new Map();
const loopDownKeyCBMap: Map<string, () => Promise<void>> = new Map();
const upKeyCBMap: Map<string, () => Promise<void>> = new Map();

const skillKeyList: string[] = [];

function keydownHandler(event: KeyboardEvent) {
  if (event.repeat) return;
  if (downKeyMap.has(event.key)) {
    event.preventDefault();
    // execute the down callback (if there is) asyncily
    let cb = downKeyCBMap.get(event.key);
    if (cb) cb();
    downKeyMap.set(event.key, true);
  }
}

function keyupHandler(event: KeyboardEvent) {
  if (downKeyMap.has(event.key)) {
    event.preventDefault();
    // execute the up callback (if there is) asyncily
    let cb = upKeyCBMap.get(event.key);
    if (cb) cb();
    downKeyMap.set(event.key, false);
  }
}

function handleMouseDown(event: MouseEvent) {
  let key = "M" + event.button.toString();
  if (downKeyMap.has(key)) {
    event.preventDefault();
    // execute the down callback asyncily
    let cb = downKeyCBMap.get(key);
    if (cb) cb();
    downKeyMap.set(key, true);
  }
}

function handleMouseUp(event: MouseEvent) {
  let key = "M" + event.button.toString();
  if (downKeyMap.has(key)) {
    event.preventDefault();
    // execute the up callback asyncily
    let cb = upKeyCBMap.get(key);
    if (cb) cb();
    downKeyMap.set(key, false);
  }
}

function handleMouseMove(event: MouseEvent) {
  oldMouseX = mouseX;
  oldMouseY = mouseY;
  mouseX = event.clientX;
  mouseY = event.clientY;
}

function addShortcut(
  key: string,
  downCB: () => Promise<void>,
  loopCB?: () => Promise<void>,
  upCB?: () => Promise<void>
) {
  downKeyMap.set(key, false);

  if (downCB && loopCB)
    downKeyCBMap.set(key, async () => {
      downCB();
      loopDownKeyCBMap.set(key, loopCB);
    });
  else if (downCB) {
    downKeyCBMap.set(key, downCB);
  }

  if (upCB) {
    if (loopCB)
      upKeyCBMap.set(key, async () => {
        loopDownKeyCBMap.delete(key);
        upCB();
      });
    else upKeyCBMap.set(key, upCB);
  }
}

export function initShortcuts(
  element: HTMLElement,
  factor: number,
  screenSize: [number, number]
) {
  posFactor = factor;
  screenSizeW = screenSize[0];
  screenSizeH = screenSize[1];
  element.addEventListener("mousedown", handleMouseDown);
  element.addEventListener("mousemove", handleMouseMove);
  element.addEventListener("mouseup", handleMouseUp);
  element.addEventListener("keydown", keydownHandler);
  element.addEventListener("keyup", keyupHandler);

  addLeftClickShortcuts(0);
  addSteeringWheelKeyboardShortcuts(
    {
      left: "a",
      right: "d",
      up: "w",
      down: "s",
    },
    180,
    560,
    100,
    1
  );
  addDirectionalSkillShortcuts("Alt", 1025, 500, 200, 2);
  addDirectionalSkillShortcuts("q", 950, 610, 200, 2);
  addDirectionalSkillShortcuts("e", 1160, 420, 200, 2);
  addTriggerWhenPressedSkillShortcuts("M4", 1160, 420, false, 0, 2);
  setInterval(() => {
    loopDownKeyCBMap.forEach((cb) => {
      if (oldMouseX !== mouseX && oldMouseY !== mouseY) {
        cb();
      }
    });
  }, 50);
}

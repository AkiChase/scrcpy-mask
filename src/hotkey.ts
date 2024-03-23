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

function clientxToSkillOffsetx(clientx: number) {
  // Get the offset relative to the center of the screen
  let offsetX = clientxToPosx(clientx) - screenSizeW / 2;
  return Math.max(-100, Math.min(100, offsetX));
}

function clientyToSkillOffsety(clienty: number) {
  // Get the offset relative to the center of the screen
  let offsetY = clientyToPosy(clienty) - screenSizeH / 2;
  return Math.max(-100, Math.min(100, offsetY));
}

async function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

//#region mouse shortcuts
let screenSizeW: number;
let screenSizeH: number;
let mouseX = 0;
let mouseY = 0;
let mouseDownFlag = [false, false, false, false, false];
// left right mid x1 x2

export function initMouseShortcuts(
  element: HTMLElement,
  factor: number,
  screenSize: [number, number]
) {
  posFactor = factor;
  screenSizeW = screenSize[0];
  screenSizeH = screenSize[1];
  element.addEventListener("mousedown", handleMouseDown);
  // mouse move/up should be window global
  window.addEventListener("mousemove", handleMouseMove);
  window.addEventListener("mouseup", handleMouseUp);
}

async function handleMouseDown(event: MouseEvent) {
  if (event.button >= 0 && event.button <= 4) {
    mouseDownFlag[event.button] = true;
    // left
    if (event.button === 0) {
      await touch({
        action: TouchAction.Down,
        pointerId: 0,
        screen: {
          w: screenSizeW,
          h: screenSizeH,
        },
        pos: {
          x: clientxToPosx(event.clientX),
          y: clientyToPosy(event.clientY),
        },
      });
    }
  }
}

async function handleMouseUp(event: MouseEvent) {
  if (event.button >= 0 && event.button <= 4) {
    // if pressed
    if (mouseDownFlag[event.button]) {
      mouseDownFlag[event.button] = false;
      // left
      if (event.button === 0) {
        await touch({
          action: TouchAction.Up,
          pointerId: 0,
          screen: {
            w: screenSizeW,
            h: screenSizeH,
          },
          pos: {
            x: clientxToPosx(event.clientX),
            y: clientyToPosy(event.clientY),
          },
        });
      }
    }
  }
}

let mousemoveCbMap: Map<string, () => Promise<void>> = new Map();
async function handleMouseMove(event: MouseEvent) {
  event.preventDefault();
  mouseX = event.clientX;
  mouseY = event.clientY;

  // execute all tasks
  mousemoveCbMap.forEach(async (cb) => await cb());

  // left
  if (mouseDownFlag[0]) {
    await touch({
      action: TouchAction.Move,
      pointerId: 0,
      screen: {
        w: screenSizeW,
        h: screenSizeH,
      },
      pos: {
        x: clientxToPosx(event.clientX),
        y: clientyToPosy(event.clientY),
      },
    });
  }
}
//#endregion

//#region keyboardShortcuts
// add keyboard shortcuts for directional skill
function addDirectionalSkillKeyboardShortcuts(
  key: string,
  // pos relative to the device
  posX: number,
  posY: number,
  pointerId: number
) {
  addKeyboardShortcut(
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
            x: posX + clientxToSkillOffsetx(mouseX),
            y: posY + clientyToSkillOffsety(mouseY),
          },
        ],
        intervalBetweenPos: 0,
      });
    },
    // move
    async () => {
      await touch({
        action: TouchAction.Move,
        pointerId: 1,
        screen: {
          w: screenSizeW,
          h: screenSizeH,
        },
        pos: {
          x: posX + clientxToSkillOffsetx(mouseX),
          y: posY + clientyToSkillOffsety(mouseY),
        },
      });
    },
    // up
    async () => {
      await touch({
        action: TouchAction.Up,
        pointerId: 1,
        screen: {
          w: screenSizeW,
          h: screenSizeH,
        },
        pos: {
          x: posX + clientxToSkillOffsetx(mouseX),
          y: posY + clientyToSkillOffsety(mouseY),
        },
      });
    }
  );
}
let _keyDownFlag: stringKeyFlag = {
  left: false,
  right: false,
  up: false,
  down: false,
};
let _isWheelKeyLoopRunning = false;
async function _wheelKeyLoop(
  pId: number,
  // pos relative to the device
  posX: number,
  posY: number,
  offset: number
) {
  if (_isWheelKeyLoopRunning) return;
  _isWheelKeyLoopRunning = true;
  // calculate the end coordinates of the eight directions of the direction wheel
  let offsetHalf = Math.round(offset / 1.414);

  let pos = [
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
    pointerId: pId,
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
      pointerId: pId,
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
    pointerId: pId,
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

// add keyboard shortcuts for steering wheel
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
      addKeyboardShortcut(
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

const downKeyMap: Map<string, boolean> = new Map();
const downKeyCBMap: Map<string, () => Promise<void>> = new Map();
const upKeyCBMap: Map<string, () => Promise<void>> = new Map();

async function keydownHandler(event: KeyboardEvent) {
  if (event.repeat) return;
  downKeyMap.set(event.key, true);
  let cb = downKeyCBMap.get(event.key);
  if (cb) await cb();
}

async function keyupHandler(event: KeyboardEvent) {
  downKeyMap.set(event.key, false);
  let cb = upKeyCBMap.get(event.key);
  if (cb) await cb();
}

function addKeyboardShortcut(
  key: string,
  downCB: () => Promise<void>,
  moveCB?: () => Promise<void>,
  upCB?: () => Promise<void>
) {
  if (moveCB)
    downKeyCBMap.set(key, async () => {
      mousemoveCbMap.set(key, moveCB);
      await downCB();
    });
  else downKeyCBMap.set(key, downCB);

  if (upCB) {
    if (moveCB)
      upKeyCBMap.set(key, async () => {
        mousemoveCbMap.delete(key);
        await upCB();
      });
    else upKeyCBMap.set(key, upCB);
  }
}

export function initKeyboardShortcuts(element: HTMLElement) {
  element.addEventListener("keydown", keydownHandler);
  element.addEventListener("keyup", keyupHandler);
  addDirectionalSkillKeyboardShortcuts("q", 1025, 500, 1);
  addSteeringWheelKeyboardShortcuts(
    {
      left: "a",
      right: "d",
      up: "w",
      down: "s",
    },
    230,
    384,
    100,
    2
  );
}
//#endregion

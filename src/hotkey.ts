// https://github.com/jamiebuilds/tinykeys/pull/193/commits/2598ecb3db6b3948c7acbf0e7bd8b0674961ad61
import { tinykeys } from "tinykeys";
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

// reduce the frequency of sending
// let ignoreMoveFlag = true;
let mousemoveTask: Map<string, (x: number, y: number) => Promise<void>> =
  new Map();
async function handleMouseMove(event: MouseEvent) {
  // ignoreMoveFlag = !ignoreMoveFlag;
  // if (ignoreMoveFlag) return;
  event.preventDefault();
  mouseX = event.clientX;
  mouseY = event.clientY;

  // execute all tasks
  mousemoveTask.forEach(async (task) => await task(mouseX, mouseY));

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
function addKeyboardShortcuts(
  element: HTMLElement,
  key: string,
  downCB?: (x: number, y: number) => Promise<void>,
  moveCB?: (x: number, y: number) => Promise<void>,
  upCB?: (x: number, y: number) => Promise<void>
) {
  tinykeys(
    element,
    {
      [key]: async (event) => {
        if (event.repeat) return;
        if (downCB) {
          await downCB(mouseX, mouseY);
        }
        // add move event task
        if (moveCB) mousemoveTask.set(key, moveCB);
      },
    },
    {
      event: "keydown",
    }
  );
  if (upCB) {
    tinykeys(
      element,
      {
        [key]: async () => {
          // delete move event task
          if (moveCB) mousemoveTask.delete(key);
          await upCB(mouseX, mouseY);
        },
      },
      {
        event: "keyup",
      }
    );
  }
}

// add keyboard shortcuts for directional skill
function addDirectionalSkillKeyboardShortcuts(
  element: HTMLElement,
  key: string,
  // pos relative to the device
  posX: number,
  posY: number,
  pointerId: number
) {
  addKeyboardShortcuts(
    element,
    key,
    // down
    async (x: number, y: number) => {
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
            x: posX + clientxToSkillOffsetx(x),
            y: posY + clientyToSkillOffsety(y),
          },
        ],
        intervalBetweenPos: 0,
      });
    },
    // move
    async (x: number, y: number) => {
      await touch({
        action: TouchAction.Move,
        pointerId: 1,
        screen: {
          w: screenSizeW,
          h: screenSizeH,
        },
        pos: {
          x: posX + clientxToSkillOffsetx(x),
          y: posY + clientyToSkillOffsety(y),
        },
      });
    },
    // up
    async (x: number, y: number) => {
      await touch({
        action: TouchAction.Up,
        pointerId: 1,
        screen: {
          w: screenSizeW,
          h: screenSizeH,
        },
        pos: {
          x: posX + clientxToSkillOffsetx(x),
          y: posY + clientyToSkillOffsety(y),
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
  element: HTMLElement,
  // pos relative to the device
  posX: number,
  posY: number,
  offset: number,
  pointerId: number,
  key: wheelKey
) {
  for (const k of ["left", "right", "up", "down"]) {
    if (key[k])
      addKeyboardShortcuts(
        element,
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

export function initKeyboardShortcuts(element: HTMLElement) {
  addDirectionalSkillKeyboardShortcuts(element, "q", 1025, 500, 1);
  addSteeringWheelKeyboardShortcuts(element, 230, 384, 100, 2, {
    left: "a",
    right: "d",
    up: "w",
    down: "s",
  });
}
//#endregion

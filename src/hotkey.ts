// https://github.com/jamiebuilds/tinykeys/pull/193/commits/2598ecb3db6b3948c7acbf0e7bd8b0674961ad61
import { tinykeys } from "tinykeys";
import {
  SwipeAction,
  TouchAction,
  swipe,
  touch,
} from "./frontcommand/scrcpyMaskCmd";

let posFactor = 1;
function clientxToPosx(clientx: number) {
  return clientx < 70 ? 0 : Math.floor((clientx - 70) * posFactor);
}

function clientyToPosy(clienty: number) {
  return clienty < 30 ? 0 : Math.floor((clienty - 30) * posFactor);
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
    console.log("down", event.button);
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
      console.log("up", event);
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
let ignoreMoveFlag = true;
let mousemoveTask: Map<string, (x: number, y: number) => Promise<void>> =
  new Map();
async function handleMouseMove(event: MouseEvent) {
  ignoreMoveFlag = !ignoreMoveFlag;
  if (ignoreMoveFlag) return;
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

export function initKeyboardShortcuts(element: HTMLElement) {
  tinykeys(
    element,
    {
      Control: async (event) => {
        if (event.repeat) return;
        // down and move to mouse
        await swipe({
          action: SwipeAction.NoUp,
          pointerId: 1,
          screen: {
            w: screenSizeW,
            h: screenSizeH,
          },
          pos: [
            { x: clientxToPosx(100 + 70), y: clientyToPosy(100 + 30) },
            { x: clientxToPosx(mouseX), y: clientyToPosy(mouseY) },
          ],
          intervalBetweenPos: 0,
        });
        // add move event task
        mousemoveTask.set("Control", async (x: number, y: number) => {
          await touch({
            action: TouchAction.Move,
            pointerId: 1,
            screen: {
              w: screenSizeW,
              h: screenSizeH,
            },
            pos: {
              x: clientxToPosx(x),
              y: clientyToPosy(y),
            },
          });
        });
      },
    },
    {
      event: "keydown",
    }
  );
  tinykeys(
    element,
    {
      Control: async () => {
        // delete move event task
        mousemoveTask.delete("Control");
        // up
        await touch({
          action: TouchAction.Up,
          pointerId: 1,
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
    },
    {
      event: "keyup",
    }
  );
}
//#endregion

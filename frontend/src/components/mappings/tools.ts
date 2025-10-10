export function mappingButtonPresetStyle(
  radiusX: number,
  radiusY?: number
): React.CSSProperties {
  radiusY = radiusY ?? radiusX;

  return {
    width: radiusX,
    height: radiusY,
    left: -radiusX / 2,
    top: -radiusY / 2,
  };
}

export function clientPositionToMappingPosition(
  cX: number,
  cY: number,
  maskArea: { width: number; height: number; left: number; top: number },
  oW: number,
  oH: number
) {
  const mX = Math.max(0, Math.min(maskArea.width, cX - maskArea.left));
  const mY = Math.max(0, Math.min(maskArea.height, cY - maskArea.top));
  return {
    x: Math.round((mX / maskArea.width) * oW),
    y: Math.round((mY / maskArea.height) * oH),
  };
}

export function mappingButtonPosition(
  oX: number,
  oY: number,
  scale: { x: number; y: number }
) {
  return {
    x: Math.round(oX * scale.x),
    y: Math.round(oY * scale.y),
  };
}

export function mappingButtonTransformStyle(
  oX: number,
  oY: number,
  scale: { x: number; y: number }
): string {
  const x = Math.round(oX * scale.x);
  const y = Math.round(oY * scale.y);
  return `translate(${x}px, ${y}px)`;
}

export function mappingButtonDragFactory(
  maskArea: { width: number; height: number; left: number; top: number },
  originalSize: { width: number; height: number },
  onMouseUp: ({ x, y }: { x: number; y: number }) => void,
  delay?: number
) {
  delay = delay ?? 500;
  const handleDrag = (downEvent: React.MouseEvent) => {
    if (downEvent.button !== 0) return;

    const { width, height, left, top } = maskArea;
    const element = downEvent.currentTarget as HTMLElement;

    let dragStarted = false;
    let longPressTimer = 0;
    let curMaskX = 0;
    let curMaskY = 0;

    const handleMouseMove = (moveEvent: MouseEvent) => {
      curMaskX = Math.max(0, Math.min(moveEvent.clientX - left, width));
      curMaskY = Math.max(0, Math.min(moveEvent.clientY - top, height));
      if (!dragStarted) return;
      element.style.transform = `translate(${curMaskX}px, ${curMaskY}px)`;
    };

    const handleMouseUp = (upEvent: MouseEvent) => {
      clearTimeout(longPressTimer);
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
      if (!dragStarted) return;

      curMaskX = Math.max(0, Math.min(upEvent.clientX - left, width));
      curMaskY = Math.max(0, Math.min(upEvent.clientY - top, height));
      element.style.transform = `translate(${curMaskX}px, ${curMaskY}px)`;

      onMouseUp({
        x: Math.round((curMaskX / width) * originalSize.width),
        y: Math.round((curMaskY / height) * originalSize.height),
      });
    };

    curMaskX = Math.max(0, Math.min(downEvent.clientX - left, width));
    curMaskY = Math.max(0, Math.min(downEvent.clientY - top, height));
    window.addEventListener("mousemove", handleMouseMove);

    longPressTimer = setTimeout(() => {
      dragStarted = true;
      element.style.transform = `translate(${curMaskX}px, ${curMaskY}px)`;
    }, delay);
    window.addEventListener("mouseup", handleMouseUp);
  };

  return handleDrag;
}

export function mappingModalDragFactory(
  delay?: number
) {
  delay = delay ?? 150;
  const handleDrag = (downEvent: React.MouseEvent) => {
    if (downEvent.button !== 0) return;
    const element = document.querySelector(".setting-modal") as HTMLElement;

    let dragStarted = false;
    let longPressTimer = 0;
    let oX = downEvent.clientX;
    let oY = downEvent.clientY;
    let cX = oX;
    let cY = oY;
    let match = element.style.transform.match(/translate\((-?\d+)px,\s*(-?\d+)px\)/);
    let oldTX = match ? parseInt(match[1]) : 0;
    let oldTY = match ? parseInt(match[2]) : 0;

    const handleMouseMove = (moveEvent: MouseEvent) => {
      cX = moveEvent.clientX
      cY = moveEvent.clientY
      if (!dragStarted) return;
      element.style.transform = `translate(${oldTX + cX - oX}px, ${oldTY + cY - oY}px)`;
    };

    const handleMouseUp = (upEvent: MouseEvent) => {
      clearTimeout(longPressTimer);
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
      if (!dragStarted) return;

      cX = upEvent.clientX
      cY = upEvent.clientY
      element.style.transform = `translate(${oldTX + cX - oX}px, ${oldTY + cY - oY}px)`;
    };

    cX = downEvent.clientX
    cY = downEvent.clientY
    window.addEventListener("mousemove", handleMouseMove);

    longPressTimer = setTimeout(() => {
      dragStarted = true;
      element.style.transform = `translate(${oldTX + cX - oX}px, ${oldTY + cY - oY}px)`;
    }, delay);
    window.addEventListener("mouseup", handleMouseUp);
  };

  return handleDrag;
}
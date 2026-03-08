export function mappingButtonPresetStyle(
  radiusX: number,
  radiusY?: number,
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
  oH: number,
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
  scale: { x: number; y: number },
) {
  return {
    x: Math.round(oX * scale.x),
    y: Math.round(oY * scale.y),
  };
}

export function mappingButtonTransformStyle(
  oX: number,
  oY: number,
  scale: { x: number; y: number },
): string {
  const x = Math.round(oX * scale.x);
  const y = Math.round(oY * scale.y);
  return `translate(${x}px, ${y}px)`;
}

export function mappingButtonDragFactory(
  maskArea: { width: number; height: number; left: number; top: number },
  originalSize: { width: number; height: number },
  onMouseUp: ({ x, y }: { x: number; y: number }) => void,
  delay?: number,
) {
  delay = delay ?? 500;
  const handleDrag = (downEvent: React.MouseEvent) => {
    if (downEvent.button !== 0) return;

    const mappingContainer = document.getElementById("mappings-container") as HTMLElement;
    const scrollX = mappingContainer.scrollLeft;
    const scrollY = mappingContainer.scrollTop;
    
    const { width, height, left, top } = maskArea;
    const element = downEvent.currentTarget as HTMLElement;

    let dragStarted = false;
    let longPressTimer = 0;
    let curMaskX = 0;
    let curMaskY = 0;

    const updateCurMaskPos = (e: MouseEvent | React.MouseEvent) => {
      curMaskX = Math.max(0, Math.min(e.clientX + scrollX - left, width));
      curMaskY = Math.max(0, Math.min(e.clientY + scrollY - top, height));
    };

    const handleMouseMove = (moveEvent: MouseEvent) => {
      updateCurMaskPos(moveEvent);
      if (!dragStarted) return;
      element.style.transform = `translate(${curMaskX}px, ${curMaskY}px)`;
    };

    const handleMouseUp = (upEvent: MouseEvent) => {
      clearTimeout(longPressTimer);
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
      if (!dragStarted) return;

      updateCurMaskPos(upEvent);
      element.style.transform = `translate(${curMaskX}px, ${curMaskY}px)`;

      onMouseUp({
        x: Math.round((curMaskX / width) * originalSize.width),
        y: Math.round((curMaskY / height) * originalSize.height),
      });
    };

    updateCurMaskPos(downEvent);
    window.addEventListener("mousemove", handleMouseMove);

    longPressTimer = setTimeout(() => {
      dragStarted = true;
      element.style.transform = `translate(${curMaskX}px, ${curMaskY}px)`;
    }, delay);
    window.addEventListener("mouseup", handleMouseUp);
  };

  return handleDrag;
}

export function mappingModalDragFactory(delay?: number) {
  delay = delay ?? 150;
  const handleDrag = (downEvent: React.MouseEvent) => {
    if (downEvent.button !== 0) return;
    const element = document.querySelector(".setting-modal") as HTMLElement;

    const oX = downEvent.pageX;
    const oY = downEvent.pageY;
    let dragStarted = false;
    let longPressTimer = 0;
    let cX = oX;
    let cY = oY;
    const match = element.style.transform.match(
      /translate\((-?\d+)px,\s*(-?\d+)px\)/,
    );
    const oldTX = match ? parseInt(match[1]) : 0;
    const oldTY = match ? parseInt(match[2]) : 0;

    const handleMouseMove = (moveEvent: MouseEvent) => {
      cX = moveEvent.pageX;
      cY = moveEvent.pageY;
      if (!dragStarted) return;
      element.style.transform = `translate(${oldTX + cX - oX}px, ${oldTY + cY - oY}px)`;
    };

    const handleMouseUp = (upEvent: MouseEvent) => {
      clearTimeout(longPressTimer);
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
      if (!dragStarted) return;
      cX = upEvent.pageX;
      cY = upEvent.pageY;
      element.style.transform = `translate(${oldTX + cX - oX}px, ${oldTY + cY - oY}px)`;
    };
    cX = downEvent.pageX;
    cY = downEvent.pageY;
    window.addEventListener("mousemove", handleMouseMove);
    longPressTimer = window.setTimeout(() => {
      dragStarted = true;
      element.style.transform = `translate(${oldTX + cX - oX}px, ${oldTY + cY - oY}px)`;
    }, delay);
    window.addEventListener("mouseup", handleMouseUp);
  };
  return handleDrag;
}

import { type CSSProperties, type PropsWithChildren } from "react";
import { MappingOverlayContext } from "./MappingOverlayContext";

type MappingOverlayTone = "boundary" | "observation" | "cast" | "drag";

const toneColors: Record<MappingOverlayTone, { border: string; fill: string }> =
  {
    boundary: {
      border: "rgba(245, 158, 11, 0.9)",
      fill: "rgba(245, 158, 11, 0.08)",
    },
    observation: {
      border: "rgba(14, 165, 233, 0.9)",
      fill: "rgba(14, 165, 233, 0.08)",
    },
    cast: {
      border: "rgba(168, 85, 247, 0.9)",
      fill: "rgba(168, 85, 247, 0.07)",
    },
    drag: {
      border: "rgba(34, 197, 94, 0.9)",
      fill: "rgba(34, 197, 94, 0.08)",
    },
  };

function overlayStyle(tone: MappingOverlayTone): CSSProperties {
  const color = toneColors[tone];
  return {
    position: "absolute",
    border: `1px dashed ${color.border}`,
    backgroundColor: color.fill,
    pointerEvents: "none",
    boxSizing: "border-box",
  };
}

type MappingOverlayProviderProps = PropsWithChildren<{
  showAllGuides: boolean;
}>;

export function MappingOverlayProvider({
  showAllGuides,
  children,
}: MappingOverlayProviderProps) {
  return (
    <MappingOverlayContext.Provider value={{ showAllGuides }}>
      {children}
    </MappingOverlayContext.Provider>
  );
}

export type MappingOverlayRectShape = {
  left: number;
  top: number;
  width: number;
  height: number;
};

type MappingOverlayRectProps = {
  shape: MappingOverlayRectShape;
  visible: boolean;
  tone?: MappingOverlayTone;
};

export function MappingOverlayRect({
  shape,
  visible,
  tone = "boundary",
}: MappingOverlayRectProps) {
  if (!visible || shape.width <= 0 || shape.height <= 0) {
    return null;
  }

  const style: CSSProperties = {
    ...overlayStyle(tone),
    left: shape.left,
    top: shape.top,
    width: shape.width,
    height: shape.height,
  };

  return <div style={style} />;
}

export type MappingOverlayEllipseShape = {
  centerX: number;
  centerY: number;
  radiusX: number;
  radiusY: number;
};

type MappingOverlayEllipseProps = {
  shape: MappingOverlayEllipseShape;
  visible: boolean;
  tone: MappingOverlayTone;
};

export function MappingOverlayEllipse({
  shape,
  visible,
  tone,
}: MappingOverlayEllipseProps) {
  if (!visible || shape.radiusX <= 0 || shape.radiusY <= 0) {
    return null;
  }

  const style: CSSProperties = {
    ...overlayStyle(tone),
    left: shape.centerX - shape.radiusX,
    top: shape.centerY - shape.radiusY,
    width: shape.radiusX * 2,
    height: shape.radiusY * 2,
    borderRadius: "9999px",
  };

  return <div style={style} />;
}

export type MappingOverlayCircleShape = {
  centerX: number;
  centerY: number;
  radius: number;
};

type MappingOverlayCircleProps = {
  shape: MappingOverlayCircleShape;
  visible: boolean;
  tone: MappingOverlayTone;
};

export function MappingOverlayCircle({
  shape,
  visible,
  tone,
}: MappingOverlayCircleProps) {
  return (
    <MappingOverlayEllipse
      shape={{
        centerX: shape.centerX,
        centerY: shape.centerY,
        radiusX: shape.radius,
        radiusY: shape.radius,
      }}
      visible={visible}
      tone={tone}
    />
  );
}

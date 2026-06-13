import {
  type CSSProperties,
  type PropsWithChildren,
} from "react";
import { MappingOverlayContext } from "./MappingOverlayContext";

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
};

export function MappingOverlayRect({
  shape,
  visible,
}: MappingOverlayRectProps) {
  if (!visible || shape.width <= 0 || shape.height <= 0) {
    return null;
  }

  const style: CSSProperties = {
    position: "absolute",
    left: shape.left,
    top: shape.top,
    width: shape.width,
    height: shape.height,
    border: "1px dashed rgba(245, 158, 11, 0.9)",
    backgroundColor: "rgba(245, 158, 11, 0.08)",
    pointerEvents: "none",
    boxSizing: "border-box",
  };

  return <div style={style} />;
}

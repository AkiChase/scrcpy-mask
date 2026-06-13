import { useContext, type CSSProperties, type PropsWithChildren } from "react";
import { MappingOverlayContext } from "./MappingOverlayContext";

type MappingOverlayTone =
  | "boundary"
  | "observation"
  | "cast"
  | "drag"
  | "trace";

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
    trace: {
      border: "rgba(6, 182, 212, 0.9)",
      fill: "rgba(6, 182, 212, 0.12)",
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

function overlayColor(tone: MappingOverlayTone) {
  return toneColors[tone];
}

type MappingOverlayProviderProps = PropsWithChildren<{
  showAllGuides: boolean;
  viewportOrigin: { left: number; top: number } | null;
  viewportSize: { width: number; height: number };
}>;

export function MappingOverlayProvider({
  showAllGuides,
  viewportOrigin,
  viewportSize,
  children,
}: MappingOverlayProviderProps) {
  return (
    <MappingOverlayContext.Provider
      value={{ showAllGuides, viewportOrigin, viewportSize }}
    >
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
  const { viewportOrigin } = useContext(MappingOverlayContext);

  if (!visible || shape.width <= 0 || shape.height <= 0) {
    return null;
  }

  const style: CSSProperties = {
    ...overlayStyle(tone),
    position: viewportOrigin ? "fixed" : "absolute",
    left: (viewportOrigin?.left ?? 0) + shape.left,
    top: (viewportOrigin?.top ?? 0) + shape.top,
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
  const { viewportOrigin } = useContext(MappingOverlayContext);

  if (!visible || shape.radiusX <= 0 || shape.radiusY <= 0) {
    return null;
  }

  const style: CSSProperties = {
    ...overlayStyle(tone),
    position: viewportOrigin ? "fixed" : "absolute",
    left: (viewportOrigin?.left ?? 0) + shape.centerX - shape.radiusX,
    top: (viewportOrigin?.top ?? 0) + shape.centerY - shape.radiusY,
    width: shape.radiusX * 2,
    height: shape.radiusY * 2,
    borderRadius: "50%",
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

export type MappingOverlayPathGroupShape = {
  centerX: number;
  centerY: number;
  paths: {
    d: string;
    opacity: number;
  }[];
};

type MappingOverlayPathGroupProps = {
  shape: MappingOverlayPathGroupShape;
  visible: boolean;
  tone: MappingOverlayTone;
};

export function MappingOverlayPathGroup({
  shape,
  visible,
  tone,
}: MappingOverlayPathGroupProps) {
  const { viewportOrigin, viewportSize } = useContext(MappingOverlayContext);

  if (
    !visible ||
    shape.paths.length === 0 ||
    viewportSize.width <= 0 ||
    viewportSize.height <= 0
  ) {
    return null;
  }

  const color = overlayColor(tone);
  const style: CSSProperties = {
    position: viewportOrigin ? "fixed" : "absolute",
    left: viewportOrigin?.left ?? 0,
    top: viewportOrigin?.top ?? 0,
    width: viewportSize.width,
    height: viewportSize.height,
    pointerEvents: "none",
  };

  return (
    <svg
      style={style}
      viewBox={`0 0 ${viewportSize.width} ${viewportSize.height}`}
    >
      <g transform={`translate(${shape.centerX}, ${shape.centerY})`}>
        {shape.paths.map((path, index) => (
          <path
            key={index}
            d={path.d}
            fill={color.border}
            fillOpacity={path.opacity}
          />
        ))}
      </g>
    </svg>
  );
}

export type MappingOverlayPoint = {
  x: number;
  y: number;
};

type MappingOverlayPolylineProps = {
  points: MappingOverlayPoint[];
  visible: boolean;
  tone: MappingOverlayTone;
  showLabels?: boolean;
};

export function MappingOverlayPolyline({
  points,
  visible,
  tone,
  showLabels = false,
}: MappingOverlayPolylineProps) {
  const { viewportOrigin, viewportSize } = useContext(MappingOverlayContext);

  if (
    !visible ||
    points.length <= 1 ||
    viewportSize.width <= 0 ||
    viewportSize.height <= 0
  ) {
    return null;
  }

  const color = overlayColor(tone);
  const markerId = `mapping-overlay-arrow-${tone}`;
  const style: CSSProperties = {
    position: viewportOrigin ? "fixed" : "absolute",
    left: viewportOrigin?.left ?? 0,
    top: viewportOrigin?.top ?? 0,
    width: viewportSize.width,
    height: viewportSize.height,
    pointerEvents: "none",
  };

  return (
    <svg
      style={style}
      viewBox={`0 0 ${viewportSize.width} ${viewportSize.height}`}
    >
      <defs>
        <marker
          id={markerId}
          markerWidth="8"
          markerHeight="7"
          refX="8"
          refY="3.5"
          orient="auto"
          markerUnits="strokeWidth"
        >
          <path d="M0,0 L8,3.5 L0,7 Z" fill={color.border} />
        </marker>
      </defs>
      {points.map((point, index) => {
        if (index === points.length - 1) {
          return null;
        }

        const next = points[index + 1];
        return (
          <line
            key={index}
            x1={point.x}
            y1={point.y}
            x2={next.x}
            y2={next.y}
            stroke={color.border}
            strokeWidth="2"
            markerEnd={`url(#${markerId})`}
          />
        );
      })}
      {points.map((point, index) => (
        <g key={index} transform={`translate(${point.x}, ${point.y})`}>
          <circle r="6" fill={color.border} fillOpacity="0.9" />
          {showLabels && (
            <text
              y="-12"
              textAnchor="middle"
              fill={color.border}
              fontSize="12"
              fontWeight="700"
            >
              {index + 1}
            </text>
          )}
        </g>
      ))}
    </svg>
  );
}

import { Tooltip } from "antd";

type IconButtonProps = {
  icon: React.ReactNode;
  size?: number;
  color?: "default" | "info" | "error" | "primary" | "success" | "warning";
  tooltip?: string;
} & React.ComponentProps<"a">;

export default function IconButton({
  icon,
  size,
  color,
  tooltip,
  ...rest
}: IconButtonProps) {
  color = color ?? "default";
  size = size ?? 16;

  const className =
    (color === "default"
      ? "color-text hover:color-text-hover active:color-text-active"
      : `color-${color}-text hover:color-${color}-text-hover active:color-${color}-text-active`) +
    " block active:transform-scale-95 transition-duration-300 ease-in-out";

  if (tooltip !== undefined) {
    return (
      <Tooltip title={tooltip}>
        <a style={{ fontSize: size }} className={className} {...rest}>
          {icon}
        </a>
      </Tooltip>
    );
  }

  return (
    <a style={{ fontSize: size }} className={className} {...rest}>
      {icon}
    </a>
  );
}

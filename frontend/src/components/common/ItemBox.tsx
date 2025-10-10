import { QuestionCircleOutlined } from "@ant-design/icons";
import { Flex, Tooltip } from "antd";
import type { ComponentProps, PropsWithChildren, ReactNode } from "react";

type ItemBoxContainerProps = PropsWithChildren<{
  gap?: number;
}> &
  ComponentProps<"div">;

export function ItemBoxContainer({
  children,
  gap,
  ...rest
}: ItemBoxContainerProps) {
  gap = gap ?? 24;

  return (
    <Flex {...rest} vertical gap={gap}>
      {children}
    </Flex>
  );
}

type ItemBoxProps = PropsWithChildren<{
  label?: ReactNode;
  tooltip?: ReactNode;
}> &
  ComponentProps<"div">;

export function ItemBox({ label, tooltip, children, ...rest }: ItemBoxProps) {
  return (
    <div {...rest}>
      {label && (
        <Flex
          align="center"
          gap="small"
          className="color-text font-bold pb-2 pl-1 pr-1"
        >
          <div>{label}</div>
          {tooltip && (
            <Tooltip title={tooltip}>
              <QuestionCircleOutlined />
            </Tooltip>
          )}
        </Flex>
      )}
      <div>{children}</div>
    </div>
  );
}

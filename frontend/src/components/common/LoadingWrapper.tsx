import React, { Suspense } from "react";
import { Spin } from "antd";
import { LoadingOutlined } from "@ant-design/icons";

export default function LoadingWrapper({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <Suspense
      fallback={
        <Spin indicator={<LoadingOutlined spin />} size="large" fullscreen />
      }
    >
      {children}
    </Suspense>
  );
}

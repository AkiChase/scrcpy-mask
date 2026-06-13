import { useEffect, useMemo, useState } from "react";
import type { FpsConfig, FpsTouchMode, MappingUpdater } from "./mapping";
import { Flex, InputNumber, Select, Space, Tooltip, Typography } from "antd";
import {
  mappingButtonDragFactory,
  mappingButtonScaledPresetStyle,
  mappingButtonTransformStyle,
} from "./tools";
import { useAppSelector } from "../../store/store";
import { ItemBox, ItemBoxContainer } from "../common/ItemBox";
import {
  SettingBind,
  SettingFooter,
  SettingMappingId,
  SettingModal,
  SettingNote,
  SettingPointerId,
} from "./Common";
import { useTranslation } from "react-i18next";
import { IconFont } from "../../hooks";
import {
  MappingOverlayRect,
  type MappingOverlayRectShape,
} from "./MappingOverlay";
import { useMappingGuideState } from "./MappingOverlayContext";

export default function ButtonFps({
  index,
  config,
  originalSize,
  onConfigChange,
  onConfigDelete,
  onConfigCopy,
  getAvailablePointerId,
}: {
  index: number;
  config: FpsConfig;
  originalSize: { width: number; height: number };
  onConfigChange: MappingUpdater<FpsConfig>;
  onConfigDelete: () => void;
  onConfigCopy: () => void;
  getAvailablePointerId: (reserved?: number[]) => number;
}) {
  const id = `mapping-fps-${index}`;
  const bindText = config.bind.length > 0 ? config.bind.join("+") : "???";
  const className =
    "rounded-full absolute box-border border-solid border-2 color-text " +
    (config.bind.length > 0
      ? "border-text-secondary hover:border-text"
      : "border-primary hover:border-primary-hover");

  const maskArea = useAppSelector((state) => state.other.maskArea);
  const [showSetting, setShowSetting] = useState(false);
  const mappingGuide = useMappingGuideState(showSetting);

  const scale = useMemo(() => {
    return {
      x: maskArea.width / originalSize.width,
      y: maskArea.height / originalSize.height,
    };
  }, [originalSize, maskArea]);

  const buttonStyle = useMemo(
    () => mappingButtonScaledPresetStyle(52, maskArea),
    [maskArea],
  );

  const boundaryShape = useMemo<MappingOverlayRectShape | null>(() => {
    if (config.max_offset_x <= 0 && config.max_offset_y <= 0) {
      return null;
    }
    const centerX = config.position.x * scale.x;
    const centerY = config.position.y * scale.y;
    const rawLeft =
      config.max_offset_x > 0 ? centerX - config.max_offset_x * scale.x : 0;
    const rawRight =
      config.max_offset_x > 0
        ? centerX + config.max_offset_x * scale.x
        : maskArea.width;
    const rawTop =
      config.max_offset_y > 0 ? centerY - config.max_offset_y * scale.y : 0;
    const rawBottom =
      config.max_offset_y > 0
        ? centerY + config.max_offset_y * scale.y
        : maskArea.height;
    const left = Math.max(0, rawLeft);
    const top = Math.max(0, rawTop);
    const right = Math.min(maskArea.width, rawRight);
    const bottom = Math.min(maskArea.height, rawBottom);
    return {
      left,
      top,
      width: Math.max(0, right - left),
      height: Math.max(0, bottom - top),
    };
  }, [config.max_offset_x, config.max_offset_y, config.position, maskArea, scale]);

  useEffect(() => {
    const element = document.getElementById(id);
    if (element) {
      element.style.transform = mappingButtonTransformStyle(
        config.position.x,
        config.position.y,
        scale
      );
    }
  }, [index, config, scale]);

  const handleDrag = mappingButtonDragFactory(
    maskArea,
    originalSize,
    ({ x, y }) => {
      onConfigChange({
        ...config,
        position: {
          x,
          y,
        },
      });
    }
  );

  const handleMouseDown = (e: React.MouseEvent) => {
    mappingGuide.startPointerDown(e);
    handleDrag(e);
  };

  const handleSetting = (e: React.MouseEvent) => {
    e.preventDefault();
    setShowSetting(true);
  };

  return (
    <>
      <SettingModal open={showSetting} onClose={() => setShowSetting(false)}>
        <Setting
          config={config}
          onConfigChange={onConfigChange}
          getAvailablePointerId={getAvailablePointerId}
          onConfigDelete={() => {
            setShowSetting(false);
            onConfigDelete();
          }}
          onConfigCopy={() => {
            setShowSetting(false);
            onConfigCopy();
          }}
        />
      </SettingModal>
      {boundaryShape && (
        <MappingOverlayRect
          shape={boundaryShape}
          visible={mappingGuide.visible}
          tone="boundary"
        />
      )}
      <Flex
        id={id}
        style={buttonStyle}
        className={className}
        onMouseDown={handleMouseDown}
        onContextMenu={handleSetting}
        {...mappingGuide.interactionProps}
        justify="center"
        align="center"
        vertical
      >
        <Tooltip trigger="click" title={`${config.type}: ${bindText}`}>
          <Typography.Text ellipsis={true} className="text-2.5 font-bold">
            {bindText}
          </Typography.Text>
        </Tooltip>
        <IconFont type="icon-aim" className="text-4" />
      </Flex>
    </>
  );
}

function Setting({
  config,
  onConfigChange,
  getAvailablePointerId,
  onConfigDelete,
  onConfigCopy,
}: {
  config: FpsConfig;
  onConfigChange: MappingUpdater<FpsConfig>;
  getAvailablePointerId: (reserved?: number[]) => number;
  onConfigDelete: () => void;
  onConfigCopy: () => void;
}) {
  const { t } = useTranslation();

  function withTouchModePointer(
    touchMode: FpsTouchMode,
    anotherPointerId: number,
  ): FpsTouchMode {
    switch (touchMode.type) {
      case "clean":
        return { type: "clean", another_pointer_id: anotherPointerId };
      case "delayed":
        return {
          type: "delayed",
          interval: touchMode.interval,
          another_pointer_id: anotherPointerId,
        };
      case "overlap":
        return { type: "overlap", another_pointer_id: anotherPointerId };
      default:
        return touchMode;
    }
  }

  function handlePrimaryPointerChange(pointerId: number) {
    if (
      config.touch_mode.type !== "none" &&
      config.touch_mode.another_pointer_id === pointerId
    ) {
      onConfigChange({
        ...config,
        pointer_id: pointerId,
        touch_mode: withTouchModePointer(
          config.touch_mode,
          getAvailablePointerId([pointerId]),
        ),
      });
      return;
    }
    onConfigChange({ ...config, pointer_id: pointerId });
  }

  function handleTouchModeChange(type: FpsTouchMode["type"]) {
    if (type === "none") {
      onConfigChange({ ...config, touch_mode: { type: "none" } });
      return;
    }
    const anotherPointerId =
      config.touch_mode.type === "none"
        ? getAvailablePointerId([config.pointer_id])
        : config.touch_mode.another_pointer_id;
    const touchMode: FpsTouchMode =
      type === "delayed"
        ? {
            type,
            interval:
              config.touch_mode.type === "delayed"
                ? config.touch_mode.interval
                : 16,
            another_pointer_id: anotherPointerId,
          }
        : { type, another_pointer_id: anotherPointerId };
    onConfigChange({ ...config, touch_mode: touchMode });
  }

  return (
    <div>
      <h1 className="title-with-line">{t("mappings.fps.setting.title")}</h1>
      <ItemBoxContainer className="max-h-70vh overflow-y-auto pr-2 scrollbar">
        <SettingMappingId id={config.id} />
        <SettingBind
          bind={config.bind}
          onBindChange={(bind) => onConfigChange((pre) => ({ ...pre, bind }))}
        />
        <SettingPointerId
          pointerId={config.pointer_id}
          onPointerIdChange={handlePrimaryPointerChange}
        />
        <ItemBox label={t("mappings.fps.setting.maxOffset")} tooltip={t("mappings.fps.setting.maxOffsetHint")}>
          <Space.Compact className="w-full">
            <InputNumber
              className="w-full"
              prefix="X:"
              value={config.max_offset_x}
              min={0}
              onChange={(v) =>
                v !== null && onConfigChange({ ...config, max_offset_x: v })
              }
            />
            <InputNumber
              className="w-full"
              prefix="Y:"
              value={config.max_offset_y}
              min={0}
              onChange={(v) =>
                v !== null && onConfigChange({ ...config, max_offset_y: v })
              }
            />
          </Space.Compact>
        </ItemBox>
        <ItemBox label={t("mappings.fps.setting.touchMode")} tooltip={t("mappings.fps.setting.touchModeHint")}>
          <Select
            className="w-full"
            value={config.touch_mode.type}
            onChange={handleTouchModeChange}
            options={[
              { label: t("mappings.fps.setting.touchModeNone"), value: "none" },
              { label: t("mappings.fps.setting.touchModeClean"), value: "clean" },
              { label: t("mappings.fps.setting.touchModeDelayed"), value: "delayed" },
              { label: t("mappings.fps.setting.touchModeOverlap"), value: "overlap" },
            ]}
          />
        </ItemBox>
        {config.touch_mode.type !== "none" && (
          <ItemBox label={t("mappings.fps.setting.anotherPointerId")} tooltip={t("mappings.fps.setting.anotherPointerIdHint")}>
            <InputNumber
              className="w-full"
              value={config.touch_mode.another_pointer_id}
              min={0}
              step={1}
              onChange={(v) =>
                v !== null &&
                v !== config.pointer_id &&
                onConfigChange({
                  ...config,
                  touch_mode: withTouchModePointer(config.touch_mode, v),
                })
              }
            />
          </ItemBox>
        )}
        {config.touch_mode.type === "delayed" && (
          <ItemBox label={t("mappings.fps.setting.touchModeInterval")} tooltip={t("mappings.fps.setting.touchModeIntervalHint")}>
            <InputNumber
              className="w-full"
              value={config.touch_mode.interval}
              min={0}
              step={1}
              onChange={(v) => {
                if (v === null || config.touch_mode.type !== "delayed") {
                  return;
                }
                onConfigChange({
                  ...config,
                  touch_mode: {
                    type: "delayed",
                    interval: v,
                    another_pointer_id: config.touch_mode.another_pointer_id,
                  },
                });
              }}
            />
          </ItemBox>
        )}
        <ItemBox label={t("mappings.fps.setting.sensitivity")} tooltip={t("mappings.fps.setting.sensitivityHint")}>
          <Space.Compact className="w-full">
            <InputNumber
              className="w-full"
              prefix="X:"
              value={config.sensitivity_x}
              min={0}
              onChange={(v) =>
                v !== null &&
                onConfigChange({
                  ...config,
                  sensitivity_x: v,
                })
              }
            />
            <InputNumber
              className="w-full"
              prefix="Y:"
              value={config.sensitivity_y}
              min={0}
              onChange={(v) =>
                v !== null &&
                onConfigChange({
                  ...config,
                  sensitivity_y: v,
                })
              }
            />
          </Space.Compact>
        </ItemBox>
        <SettingNote
          note={config.note}
          onNoteChange={(note) => onConfigChange({ ...config, note })}
        />
        <SettingFooter onDelete={onConfigDelete} onCopy={onConfigCopy} />
      </ItemBoxContainer>
    </div>
  );
}

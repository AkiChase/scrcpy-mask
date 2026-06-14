import { useEffect, useMemo, useRef, useState } from "react";
import type { MappingUpdater, MouseCastSpellConfig, Position } from "./mapping";
import {
  Button,
  Flex,
  InputNumber,
  Popover,
  Select,
  Slider,
  Space,
  Switch,
  Tooltip,
  Typography,
} from "antd";
import {
  clientPositionToMappingPosition,
  mappingButtonDragFactory,
  mappingButtonScaledPresetStyle,
  mappingButtonTransformStyle,
} from "./tools";
import { useAppSelector } from "../../store/store";
import { ItemBoxContainer, ItemBox } from "../common/ItemBox";
import {
  CursorPos,
  DeviceBackground,
  RefreshImageButton,
  SettingBind,
  SettingFooter,
  SettingMappingId,
  SettingModal,
  SettingNote,
  SettingPointerId,
  SettingScriptHooks,
} from "./Common";
import { useTranslation } from "react-i18next";
import { IconFont, useMessageContext } from "../../hooks";
import { RollbackOutlined } from "@ant-design/icons";
import { throttle } from "../../utils";
import {
  MappingOverlayCircle,
  type MappingOverlayCircleShape,
  MappingOverlayPathGroup,
  type MappingOverlayPathGroupShape,
} from "./MappingOverlay";
import { useMappingGuideState } from "./MappingOverlayContext";

function projectedCastRadii(
  radius: number,
  horizontalFactor: number,
  verticalFactor: number,
  originalSize: { height: number },
  maskArea: { height: number },
) {
  // the smaller factor means stronger projection compression, so the visible
  // projected range needs a longer axis in that direction.
  const hF =
    horizontalFactor < verticalFactor ? verticalFactor / horizontalFactor : 1;
  const vF =
    verticalFactor < horizontalFactor ? horizontalFactor / verticalFactor : 1;
  const maskRadius = (radius / originalSize.height) * maskArea.height;

  return {
    rx: Math.round(maskRadius * hF),
    ry: Math.round(maskRadius * vF),
  };
}

function projectedCastSectorPaths(
  radius: number,
  horizontalFactor: number,
  verticalFactor: number,
  originalSize: { height: number },
  maskArea: { height: number },
) {
  const { rx, ry } = projectedCastRadii(
    radius,
    horizontalFactor,
    verticalFactor,
    originalSize,
    maskArea,
  );

  const rad = (deg: number) => (deg * Math.PI) / 180;
  const sectorPath = (angle1: number, angle2: number) => {
    const x1 = rx * Math.cos(rad(angle1));
    const y1 = -ry * Math.sin(rad(angle1));
    const x2 = rx * Math.cos(rad(angle2));
    const y2 = -ry * Math.sin(rad(angle2));
    return `M0,0 L${x1},${y1} A${rx},${ry} 0 0,0 ${x2},${y2} Z`;
  };

  return {
    d1: sectorPath(30, 150),
    d2: sectorPath(60, 120),
  };
}

export default function ButtonMouseCastSpell({
  index,
  config,
  originalSize,
  onConfigChange,
  onConfigDelete,
  onConfigCopy,
}: {
  index: number;
  config: MouseCastSpellConfig;
  originalSize: { width: number; height: number };
  onConfigChange: MappingUpdater<MouseCastSpellConfig>;
  onConfigDelete: () => void;
  onConfigCopy: () => void;
}) {
  const id = `mapping-mouse-cast-spell-${index}`;
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
    () => mappingButtonScaledPresetStyle(64, maskArea),
    [maskArea],
  );

  const dragRadiusShape = useMemo<MappingOverlayCircleShape>(() => {
    return {
      centerX: config.position.x * scale.x,
      centerY: config.position.y * scale.y,
      radius: config.drag_radius * scale.y,
    };
  }, [config.drag_radius, config.position, scale]);

  const castProjectionShape = useMemo<MappingOverlayPathGroupShape | null>(() => {
    if (
      config.cast_no_direction ||
      config.cast_radius <= 0 ||
      config.horizontal_scale_factor <= 0 ||
      config.vertical_scale_factor <= 0
    ) {
      return null;
    }

    const { d1, d2 } = projectedCastSectorPaths(
      config.cast_radius,
      config.horizontal_scale_factor,
      config.vertical_scale_factor,
      originalSize,
      maskArea,
    );

    return {
      centerX: config.center.x * scale.x,
      centerY: config.center.y * scale.y,
      paths: [
        { d: d2, opacity: 0.18 },
        { d: d1, opacity: 0.12 },
      ],
    };
  }, [
    config.cast_no_direction,
    config.cast_radius,
    config.center,
    config.horizontal_scale_factor,
    config.vertical_scale_factor,
    maskArea,
    originalSize,
    scale,
  ]);

  useEffect(() => {
    const element = document.getElementById(id);
    if (element) {
      element.style.transform = mappingButtonTransformStyle(
        config.position.x,
        config.position.y,
        scale,
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
    },
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
          originalSize={originalSize}
          onConfigChange={onConfigChange}
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
      {castProjectionShape && (
        <MappingOverlayPathGroup
          shape={castProjectionShape}
          visible={mappingGuide.visible}
          tone="cast"
        />
      )}
      <MappingOverlayCircle
        shape={dragRadiusShape}
        visible={mappingGuide.visible}
        tone="drag"
      />
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
        gap={4}
      >
        <Tooltip trigger="click" title={`${config.type}: ${bindText}`}>
          <Typography.Text ellipsis={true} className="text-2.5 font-bold">
            {bindText}
          </Typography.Text>
        </Tooltip>
        <IconFont type="icon-lightning" />
      </Flex>
    </>
  );
}

type CastCenterProps = {
  center: Position;
  radius: number;
  horizontalFactor: number;
  verticalFactor: number;
  maskArea: { width: number; height: number; left: number; top: number };
  originalSize: { width: number; height: number };
  onCenterChange: (pos: Position) => void;
  onRadiusChange: (r: number) => void;
  onFactorChange: (key: "h" | "v", factor: number) => void;
};

function CastCenter({
  center,
  radius,
  horizontalFactor,
  verticalFactor,
  maskArea,
  originalSize,
  onCenterChange,
  onRadiusChange,
  onFactorChange,
}: CastCenterProps) {
  const { t } = useTranslation();
  const handleDrag = mappingButtonDragFactory(
    maskArea,
    originalSize,
    (pos) => {
      onCenterChange(pos);
    },
    200,
  );

  const maxRadius = Math.min(center.x, center.y);
  const { d1, d2, transform } = (() => {
    const { d1, d2 } = projectedCastSectorPaths(
      radius,
      horizontalFactor,
      verticalFactor,
      originalSize,
      maskArea,
    );

    const scale = {
      x: maskArea.width / originalSize.width,
      y: maskArea.height / originalSize.height,
    };

    const transform = mappingButtonTransformStyle(center.x, center.y, scale);

    return { d1, d2, transform };
  })();

  return (
    <Popover
      trigger="contextMenu"
      content={
        <ItemBoxContainer>
          <ItemBox label={t("mappings.mouseCastSpell.setting.center")} tooltip={t("mappings.mouseCastSpell.setting.centerHint")}>
            <Space.Compact className="w-full">
              <InputNumber
                className="w-full"
                prefix="X:"
                value={center.x}
                min={0}
                onChange={(v) =>
                  v !== null &&
                  onCenterChange({
                    x: v,
                    y: center.y,
                  })
                }
              />
              <InputNumber
                className="w-full"
                prefix="Y:"
                value={center.y}
                min={0}
                onChange={(v) =>
                  v !== null &&
                  onCenterChange({
                    x: center.x,
                    y: v,
                  })
                }
              />
              <Button
                type="primary"
                onClick={() =>
                  onCenterChange({
                    x: originalSize.width / 2,
                    y: originalSize.height / 2,
                  })
                }
              >
                {t("mappings.mouseCastSpell.setting.setCenter")}
              </Button>
            </Space.Compact>
          </ItemBox>
          <ItemBox label={t("mappings.mouseCastSpell.setting.scaleFactor")} tooltip={t("mappings.mouseCastSpell.setting.scaleFactorHint")}>
            <Space.Compact className="w-full">
              <InputNumber
                className="w-full"
                prefix="X:"
                value={horizontalFactor}
                min={0.01}
                onChange={(v) => v !== null && onFactorChange("h", v)}
              />
              <InputNumber
                className="w-full"
                prefix="Y:"
                value={verticalFactor}
                min={0.01}
                onChange={(v) => v !== null && onFactorChange("v", v)}
              />
            </Space.Compact>
          </ItemBox>
          <ItemBox label={t("mappings.mouseCastSpell.setting.castRadius")} tooltip={t("mappings.mouseCastSpell.setting.castRadiusHint")}>
            <Slider
              min={0}
              max={maxRadius}
              onChange={(v) => onRadiusChange(v)}
              value={radius}
            />
          </ItemBox>
        </ItemBoxContainer>
      }
    >
      <g onMouseDown={handleDrag} style={{ transform }}>
        <path
          d={d2}
          fill="var(--ant-color-primary)"
          style={{
            opacity: 0.6,
          }}
        />
        <path
          d={d1}
          fill="var(--ant-color-primary)"
          style={{
            opacity: 0.4,
          }}
        />
      </g>
    </Popover>
  );
}

type SkillButtonProps = {
  position: Position;
  radius: number;
  maskArea: { width: number; height: number; left: number; top: number };
  originalSize: { width: number; height: number };
  onPositionChange: (pos: Position) => void;
  onRadiusChange: (r: number) => void;
};

function SkillButton({
  position,
  radius,
  maskArea,
  originalSize,
  onPositionChange,
  onRadiusChange,
}: SkillButtonProps) {
  const { t } = useTranslation();
  const handleDrag = mappingButtonDragFactory(
    maskArea,
    originalSize,
    (pos) => {
      onPositionChange(pos);
    },
    200,
  );

  const scale = {
    x: maskArea.width / originalSize.width,
    y: maskArea.height / originalSize.height,
  };
  const transform = mappingButtonTransformStyle(position.x, position.y, scale);
  const maskRadius = (radius / originalSize.height) * maskArea.height;

  return (
    <Popover
      trigger="contextMenu"
      content={
        <ItemBoxContainer>
          <ItemBox label={t("mappings.mouseCastSpell.setting.dragRadius")} tooltip={t("mappings.mouseCastSpell.setting.dragRadiusHint")}>
            <InputNumber
              className="w-full"
              value={radius}
              min={1}
              onChange={(v) => v !== null && onRadiusChange(v)}
            />
          </ItemBox>
        </ItemBoxContainer>
      }
    >
      <g onMouseDown={handleDrag} style={{ transform }}>
        <circle
          style={{ opacity: 0.6 }}
          cx="0"
          cy="0"
          r="48"
          fill="var(--ant-color-primary)"
        />
        <circle
          cx="0"
          cy="0"
          r={maskRadius}
          stroke="var(--ant-color-primary)"
          fill="none"
          strokeWidth="2"
          strokeDasharray="10,10"
          strokeLinecap="round"
        />
      </g>
    </Popover>
  );
}

function SkillEditor({
  config,
  originalSize,
  onExit,
  onChange,
}: {
  config: MouseCastSpellConfig;
  originalSize: { width: number; height: number };
  onExit: () => void;
  onChange: (config: MouseCastSpellConfig) => void;
}) {
  const { t } = useTranslation();
  const handleMouseMove = throttle((e: React.MouseEvent) => {
    if (cursorPosRef.current) {
      const { x, y } = clientPositionToMappingPosition(
        e.clientX,
        e.clientY,
        maskArea,
        originalSize.width,
        originalSize.height,
      );
      cursorPosRef.current.innerText = `(${x},${y})`;
    }
  }, 100);

  const maskArea = useAppSelector((state) => state.other.maskArea);
  const cursorPosRef = useRef<HTMLDivElement>(null);

  return (
    <div className="select-none fixed left-0 top-0 right-0 bottom-0 bg-[var(--ant-color-bg-mask)] z-2000">
      <Space.Compact className="absolute top-8 right-8 z--1">
        <RefreshImageButton />
        <Button
          type="primary"
          icon={<RollbackOutlined />}
          onClick={() => onExit()}
        >
          {t("mappings.mouseCastSpell.setting.back")}
        </Button>
      </Space.Compact>
      <div
        className="absolute border border-solid border-primary"
        style={{
          left: maskArea.left - 1,
          top: maskArea.top - 1,
          width: maskArea.width,
          height: maskArea.height,
        }}
      >
        <DeviceBackground alpha={0} />
        <div className="w-full h-full absolute" onMouseMove={handleMouseMove}>
          <CursorPos ref={cursorPosRef} className="absolute top--6" />
          <div className="color-text-secondary font-bold absolute top--6 right-0">
            {`[${originalSize.width} x ${originalSize.height}]`}
          </div>
          <svg className="w-full h-full">
            {!config.cast_no_direction && (
              <CastCenter
                center={config.center}
                radius={config.cast_radius}
                horizontalFactor={config.horizontal_scale_factor}
                verticalFactor={config.vertical_scale_factor}
                maskArea={maskArea}
                originalSize={originalSize}
                onCenterChange={(pos) => onChange({ ...config, center: pos })}
                onRadiusChange={(r) => onChange({ ...config, cast_radius: r })}
                onFactorChange={(type, factor) =>
                  type === "h"
                    ? onChange({ ...config, horizontal_scale_factor: factor })
                    : onChange({ ...config, vertical_scale_factor: factor })
                }
              />
            )}
            <SkillButton
              position={config.position}
              radius={config.drag_radius}
              maskArea={maskArea}
              originalSize={originalSize}
              onPositionChange={(pos) => onChange({ ...config, position: pos })}
              onRadiusChange={(r) => onChange({ ...config, drag_radius: r })}
            />
          </svg>
        </div>
      </div>
    </div>
  );
}

function Setting({
  config,
  originalSize,
  onConfigChange,
  onConfigDelete,
  onConfigCopy,
}: {
  config: MouseCastSpellConfig;
  originalSize: { width: number; height: number };
  onConfigChange: MappingUpdater<MouseCastSpellConfig>;
  onConfigDelete: () => void;
  onConfigCopy: () => void;
}) {
  const { t } = useTranslation();
  const messageApi = useMessageContext();
  const [isEditing, setIsEditing] = useState(false);

  return (
    <div>
      <h1 className="title-with-line">
        {t("mappings.mouseCastSpell.setting.title")}
      </h1>
      {isEditing && (
        <SkillEditor
          config={config}
          originalSize={originalSize}
          onExit={() => setIsEditing(false)}
          onChange={(c) => onConfigChange(c)}
        />
      )}
      <ItemBoxContainer className="max-h-70vh overflow-y-auto pr-2 scrollbar">
        <SettingMappingId id={config.id} />
        <SettingBind
          bind={config.bind}
          onBindChange={(bind) => onConfigChange((pre) => ({ ...pre, bind }))}
        />
        <SettingPointerId
          pointerId={config.pointer_id}
          onPointerIdChange={(pointerId) =>
            onConfigChange({ ...config, pointer_id: pointerId })
          }
        />
        <ItemBox label={t("mappings.directionPad.setting.initDuration")} tooltip={t("mappings.directionPad.setting.initDurationHint")}>
          <InputNumber
            className="w-full"
            value={config.initial_duration}
            min={0}
            onChange={(v) =>
              v !== null && onConfigChange({ ...config, initial_duration: v })
            }
          />
        </ItemBox>
        <ItemBox label={t("mappings.mouseCastSpell.setting.enableInitialSwipeRandomization")} tooltip={t("mappings.mouseCastSpell.setting.enableInitialSwipeRandomizationHint")}>
          <Switch
            checked={config.enable_initial_swipe_randomization}
            onChange={(enable_initial_swipe_randomization) =>
              onConfigChange({ ...config, enable_initial_swipe_randomization })
            }
          />
        </ItemBox>
        <ItemBox label={t("mappings.common.randomOffsetX")} tooltip={t("mappings.common.randomOffsetXHint")}>
          <InputNumber
            className="w-full"
            value={config.random_offset_x}
            min={0}
            onChange={(v) =>
              v !== null && onConfigChange({ ...config, random_offset_x: v })
            }
          />
        </ItemBox>
        <ItemBox label={t("mappings.common.randomOffsetY")} tooltip={t("mappings.common.randomOffsetYHint")}>
          <InputNumber
            className="w-full"
            value={config.random_offset_y}
            min={0}
            onChange={(v) =>
              v !== null && onConfigChange({ ...config, random_offset_y: v })
            }
          />
        </ItemBox>
        <ItemBox label={t("mappings.mouseCastSpell.setting.releaseMode.label")} tooltip={t("mappings.mouseCastSpell.setting.releaseMode.hint")}>
          <Select
            className="w-full"
            value={config.release_mode}
            onChange={(v) => onConfigChange({ ...config, release_mode: v })}
            options={[
              {
                label: t(
                  "mappings.mouseCastSpell.setting.releaseMode.onRelease",
                ),
                value: "OnRelease",
              },
              {
                label: t("mappings.mouseCastSpell.setting.releaseMode.onPress"),
                value: "OnPress",
              },
              {
                label: t(
                  "mappings.mouseCastSpell.setting.releaseMode.onSecondPress",
                ),
                value: "OnSecondPress",
              },
            ]}
          />
        </ItemBox>
        <ItemBox label={t("mappings.mouseCastSpell.setting.castNoDirection")} tooltip={t("mappings.mouseCastSpell.setting.castNoDirectionHint")}>
          <Switch
            checked={config.cast_no_direction}
            onChange={(v) => {
              onConfigChange({ ...config, cast_no_direction: v });
            }}
          />
        </ItemBox>
        <ItemBox label={t("mappings.mouseCastSpell.setting.editLabel")}>
          <Button
            type="dashed"
            onClick={() => {
              messageApi?.info(t("mappings.mouseCastSpell.setting.editHelp"));
              setIsEditing(true);
            }}
          >
            {t("mappings.mouseCastSpell.setting.edit")}
          </Button>
        </ItemBox>
        <SettingNote
          note={config.note}
          onNoteChange={(note) => onConfigChange({ ...config, note })}
        />
        <SettingScriptHooks
          scriptHooks={config.script_hooks}
          onScriptHooksChange={(script_hooks) =>
            onConfigChange({ ...config, script_hooks })
          }
        />
        <SettingFooter onDelete={onConfigDelete} onCopy={onConfigCopy} />
      </ItemBoxContainer>
    </div>
  );
}

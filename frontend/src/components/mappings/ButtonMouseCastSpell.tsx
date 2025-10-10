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
  mappingButtonPresetStyle,
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
  SettingModal,
  SettingNote,
  SettingPointerId,
} from "./Common";
import { useTranslation } from "react-i18next";
import { useMessageContext } from "../../hooks";
import { RollbackOutlined } from "@ant-design/icons";
import { throttle } from "../../utils";

const PRESET_STYLE = mappingButtonPresetStyle(64);

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
  const bindText = config.bind.join("+");
  const className =
    "rounded-full absolute box-border border-solid border-2 color-text " +
    (config.bind.length > 0
      ? "border-text-secondary hover:border-text"
      : "border-primary hover:border-primary-hover");

  const maskArea = useAppSelector((state) => state.other.maskArea);
  const [showSetting, setShowSetting] = useState(false);

  const scale = useMemo(() => {
    return {
      x: maskArea.width / originalSize.width,
      y: maskArea.height / originalSize.height,
    };
  }, [originalSize, maskArea]);

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
      <Flex
        id={id}
        style={PRESET_STYLE}
        className={className}
        onMouseDown={handleDrag}
        onContextMenu={handleSetting}
        justify="center"
        align="center"
      >
        <Tooltip trigger="click" title={`${config.type}: ${bindText}`}>
          <Typography.Text ellipsis={true} className="text-2.5 font-bold">
            {bindText}
          </Typography.Text>
        </Tooltip>
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
    200
  );

  const maxRadius = Math.min(center.x, center.y);
  const { d1, d2, transform } = (() => {
    // the smaller the factor, the longer the axis
    const hF =
      horizontalFactor < verticalFactor ? verticalFactor / horizontalFactor : 1;
    const vF =
      verticalFactor < horizontalFactor ? horizontalFactor / verticalFactor : 1;

    const maskRadius = (radius / originalSize.height) * maskArea.height;
    const rx = Math.round(maskRadius * hF);
    const ry = Math.round(maskRadius * vF);

    const rad = (deg: number) => (deg * Math.PI) / 180;
    const d = (angle1: number, angle2: number) => {
      const cx = 0;
      const cy = 0;
      const x1 = cx + rx * Math.cos(rad(angle1));
      const y1 = cy - ry * Math.sin(rad(angle1));
      const x2 = cx + rx * Math.cos(rad(angle2));
      const y2 = cy - ry * Math.sin(rad(angle2));
      return `M${cx},${cy} L${x1},${y1} A${rx},${ry} 0 0,0 ${x2},${y2} Z`;
    };

    const d1 = d(30, 150);
    const d2 = d(60, 120);
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
          <ItemBox label={t("mappings.mouseCastSpell.setting.center")}>
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
          <ItemBox label={t("mappings.mouseCastSpell.setting.scaleFactor")}>
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
          <ItemBox label={t("mappings.mouseCastSpell.setting.castRadius")}>
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
    200
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
          <ItemBox label={t("mappings.mouseCastSpell.setting.dragRadius")}>
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
        originalSize.height
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
        <ItemBox label={t("mappings.mouseCastSpell.setting.releaseMode.label")}>
          <Select
            className="w-full"
            value={config.release_mode}
            onChange={(v) => onConfigChange({ ...config, release_mode: v })}
            options={[
              {
                label: t(
                  "mappings.mouseCastSpell.setting.releaseMode.onRelease"
                ),
                value: "OnRelease",
              },
              {
                label: t("mappings.mouseCastSpell.setting.releaseMode.onPress"),
                value: "OnPress",
              },
              {
                label: t(
                  "mappings.mouseCastSpell.setting.releaseMode.onSecondPress"
                ),
                value: "OnSecondPress",
              },
            ]}
          />
        </ItemBox>
        <ItemBox label={t("mappings.mouseCastSpell.setting.castNoDirection")}>
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
        <SettingFooter onDelete={onConfigDelete} onCopy={onConfigCopy} />
      </ItemBoxContainer>
    </div>
  );
}

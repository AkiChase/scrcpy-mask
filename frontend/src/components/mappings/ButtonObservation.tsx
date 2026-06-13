import { useEffect, useMemo, useState } from "react";
import type { MappingUpdater, ObservationConfig } from "./mapping";
import { Flex, InputNumber, Space, Tooltip, Typography } from "antd";
import {
  mappingButtonDragFactory,
  mappingButtonPresetStyle,
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
  SettingScriptHooks,
} from "./Common";
import { useTranslation } from "react-i18next";
import { IconFont } from "../../hooks";
import {
  MappingOverlayCircle,
  type MappingOverlayCircleShape,
} from "./MappingOverlay";
import { useMappingGuideState } from "./MappingOverlayContext";

const PRESET_STYLE = mappingButtonPresetStyle(52);

export default function ButtonObservation({
  index,
  config,
  originalSize,
  onConfigChange,
  onConfigDelete,
  onConfigCopy,
}: {
  index: number;
  config: ObservationConfig;
  originalSize: { width: number; height: number };
  onConfigChange: MappingUpdater<ObservationConfig>;
  onConfigDelete: () => void;
  onConfigCopy: () => void;
}) {
  const id = `mapping-single-tap-${index}`;
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

  const maxRadiusShape = useMemo<MappingOverlayCircleShape | null>(() => {
    if (config.max_radius <= 0) {
      return null;
    }

    return {
      centerX: config.position.x * scale.x,
      centerY: config.position.y * scale.y,
      radius: config.max_radius * scale.y,
    };
  }, [config.max_radius, config.position, scale]);

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
      {maxRadiusShape && (
        <MappingOverlayCircle
          shape={maxRadiusShape}
          visible={mappingGuide.visible}
          tone="observation"
        />
      )}
      <Flex
        id={id}
        style={PRESET_STYLE}
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
        <IconFont type="icon-eye" className="text-4" />
      </Flex>
    </>
  );
}

function Setting({
  config,
  onConfigChange,
  onConfigDelete,
  onConfigCopy,
}: {
  config: ObservationConfig;
  onConfigChange: MappingUpdater<ObservationConfig>;
  onConfigDelete: () => void;
  onConfigCopy: () => void;
}) {
  const { t } = useTranslation();

  return (
    <div>
      <h1 className="title-with-line">
        {t("mappings.observation.setting.title")}
      </h1>
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
        <ItemBox label={t("mappings.observation.setting.maxRadius")} tooltip={t("mappings.observation.setting.maxRadiusHint")}>
          <InputNumber
            className="w-full"
            value={config.max_radius ?? 0}
            min={0}
            onChange={(v) =>
              v !== null && onConfigChange({ ...config, max_radius: v })
            }
          />
        </ItemBox>
        <ItemBox label={t("mappings.observation.setting.sensitivity")} tooltip={t("mappings.observation.setting.sensitivityHint")}>
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

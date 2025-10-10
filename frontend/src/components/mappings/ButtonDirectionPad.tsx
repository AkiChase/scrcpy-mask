import { useEffect, useMemo, useRef, useState } from "react";
import type {
  DirectionBinding,
  DirectionButtonBinding,
  DirectionJoyStickBinding,
  DirectionPadConfig,
  MappingUpdater,
} from "./mapping";
import {
  Flex,
  InputNumber,
  Select,
  Space,
  Switch,
  Tooltip,
  Typography,
} from "antd";
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
  SettingModal,
  SettingNote,
  SettingPointerId,
} from "./Common";
import { useTranslation } from "react-i18next";
import { AXIS_NAMES } from "./keyCode";

type DirectionPadContentProps = {
  bind: DirectionBinding;
};

function BindText({ text }: { text: string }) {
  return (
    <Tooltip trigger="click" title={text}>
      <Typography.Text ellipsis={true} className="text-2.5 font-bold">
        {text}
      </Typography.Text>
    </Tooltip>
  );
}

function DirectionPadContent({ bind }: DirectionPadContentProps) {
  if (bind.type === "Button") {
    const bindTexts = {
      up: bind.up.join("+"),
      down: bind.down.join("+"),
      left: bind.left.join("+"),
      right: bind.right.join("+"),
    };

    return (
      <>
        <Flex className="flex-1" align="center">
          <BindText text={bindTexts.up} />
        </Flex>
        <Flex className="w-full" justify="space-around" align="center">
          <BindText text={bindTexts.left} />
          <BindText text={bindTexts.right} />
        </Flex>
        <Flex className="flex-1" align="center">
          <BindText text={bindTexts.down} />
        </Flex>
      </>
    );
  } else {
    return (
      <>
        <Flex vertical gap={8}>
          <BindText text={bind.x} />
          <BindText text={bind.y} />
        </Flex>
      </>
    );
  }
}

export default function ButtonDirectionPad({
  index,
  config,
  originalSize,
  onConfigChange,
  onConfigDelete,
  onConfigCopy,
}: {
  index: number;
  config: DirectionPadConfig;
  originalSize: { width: number; height: number };
  onConfigChange: MappingUpdater<DirectionPadConfig>;
  onConfigDelete: () => void;
  onConfigCopy: () => void;
}) {
  const id = `mapping-direction-pad-${index}`;
  const className = useMemo(() => {
    const base =
      "rounded-full absolute box-border border-solid border-2 color-text ";
    if (config.bind.type === "Button") {
      const { up, down, left, right } = config.bind;
      if (
        up.length === 0 ||
        down.length === 0 ||
        left.length === 0 ||
        right.length === 0
      ) {
        return base + "border-primary hover:border-primary-hover";
      } else {
        return base + "border-text-secondary hover:border-text";
      }
    } else {
      return base + "border-text-secondary hover:border-text";
    }
  }, [config.bind]);

  const maskArea = useAppSelector((state) => state.other.maskArea);
  const [showSetting, setShowSetting] = useState(false);

  const scale = useMemo(() => {
    return {
      x: maskArea.width / originalSize.width,
      y: maskArea.height / originalSize.height,
    };
  }, [originalSize, maskArea]);

  const buttonStyle = useMemo(
    () =>
      mappingButtonPresetStyle(
        Math.round(config.max_offset_x * scale.x),
        Math.round(config.max_offset_y * scale.y)
      ),
    [config.max_offset_x, config.max_offset_y, scale]
  );

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
        style={buttonStyle}
        className={className}
        onMouseDown={handleDrag}
        onContextMenu={handleSetting}
        justify="center"
        align="center"
        vertical
      >
        <DirectionPadContent bind={config.bind} />
      </Flex>
    </>
  );
}

const gamepadAxisOptions = AXIS_NAMES.map((axis) => ({
  label: axis,
  value: axis,
}));

function Setting({
  config,
  onConfigChange,
  onConfigDelete,
  onConfigCopy,
}: {
  config: DirectionPadConfig;
  onConfigChange: MappingUpdater<DirectionPadConfig>;
  onConfigDelete: () => void;
  onConfigCopy: () => void;
}) {
  const { t } = useTranslation();

  const isJoyStick = config.bind.type === "JoyStick";

  const bindValueRef = useRef<{
    Button: DirectionButtonBinding;
    JoyStick: DirectionJoyStickBinding;
  }>(
    config.bind.type === "Button"
      ? {
          Button: config.bind,
          JoyStick: {
            type: "JoyStick",
            x: "LeftStickX",
            y: "LeftStickY",
          },
        }
      : {
          Button: {
            type: "Button",
            up: ["KeyW"],
            down: ["KeyS"],
            left: ["KeyA"],
            right: ["KeyD"],
          },
          JoyStick: config.bind,
        }
  );

  function toggleBindMode(toJoyStick: boolean) {
    if (toJoyStick) {
      onConfigChange({ ...config, bind: bindValueRef.current.JoyStick });
    } else {
      onConfigChange({ ...config, bind: bindValueRef.current.Button });
    }
  }

  function handleBindChange(
    type: "up" | "down" | "left" | "right" | "x" | "y",
    value: string | string[]
  ) {
    if (type === "x" || type === "y") {
      bindValueRef.current.JoyStick[type] = value as string;
    } else {
      bindValueRef.current.Button[type] = value as string[];
    }
    onConfigChange((pre) => ({
      ...pre,
      bind: {
        ...config.bind,
        [type]: value,
      },
    }));
  }

  return (
    <div>
      <h1 className="title-with-line">
        {t("mappings.directionPad.setting.title")}
      </h1>
      <ItemBoxContainer className="max-h-70vh overflow-y-auto pr-2 scrollbar">
        <ItemBox
          label={
            <Flex className="w-full" align="center" justify="space-between">
              <span>{t("mappings.common.bind.settingLabel")}</span>
              <Switch
                size="small"
                checkedChildren={t("mappings.directionPad.setting.joyStick")}
                unCheckedChildren={t("mappings.directionPad.setting.button")}
                checked={isJoyStick}
                onChange={toggleBindMode}
              />
            </Flex>
          }
        >
          {isJoyStick ? (
            <ItemBoxContainer gap={12} className="pl-8">
              <ItemBox label={t("mappings.directionPad.setting.xAxis")}>
                <Select
                  className="w-full"
                  value={(config.bind as DirectionJoyStickBinding).x}
                  onChange={(v) => handleBindChange("x", v)}
                  options={gamepadAxisOptions}
                />
              </ItemBox>
              <ItemBox label={t("mappings.directionPad.setting.yAxis")}>
                <Select
                  className="w-full"
                  value={(config.bind as DirectionJoyStickBinding).y}
                  onChange={(v) => handleBindChange("y", v)}
                  options={gamepadAxisOptions}
                />
              </ItemBox>
            </ItemBoxContainer>
          ) : (
            <ItemBoxContainer gap={12} className="pl-8">
              <SettingBind
                label={t("mappings.directionPad.setting.up")}
                bind={(config.bind as DirectionButtonBinding).up}
                onBindChange={(bind) => handleBindChange("up", bind)}
              />
              <SettingBind
                label={t("mappings.directionPad.setting.down")}
                bind={(config.bind as DirectionButtonBinding).down}
                onBindChange={(bind) => handleBindChange("down", bind)}
              />
              <SettingBind
                label={t("mappings.directionPad.setting.left")}
                bind={(config.bind as DirectionButtonBinding).left}
                onBindChange={(bind) => handleBindChange("left", bind)}
              />
              <SettingBind
                label={t("mappings.directionPad.setting.right")}
                bind={(config.bind as DirectionButtonBinding).right}
                onBindChange={(bind) => handleBindChange("right", bind)}
              />
            </ItemBoxContainer>
          )}
        </ItemBox>
        <SettingPointerId
          pointerId={config.pointer_id}
          onPointerIdChange={(pointerId) =>
            onConfigChange({ ...config, pointer_id: pointerId })
          }
        />
        <ItemBox label={t("mappings.directionPad.setting.maxOffset")}>
          <Space.Compact className="w-full">
            <InputNumber
              className="w-full"
              prefix="X:"
              value={config.max_offset_x}
              min={1}
              onChange={(v) =>
                v !== null && onConfigChange({ ...config, max_offset_x: v })
              }
            />
            <InputNumber
              className="w-full"
              prefix="Y:"
              value={config.max_offset_y}
              min={1}
              onChange={(v) =>
                v !== null && onConfigChange({ ...config, max_offset_y: v })
              }
            />
          </Space.Compact>
        </ItemBox>
        <ItemBox label={t("mappings.directionPad.setting.initDuration")}>
          <InputNumber
            className="w-full"
            value={config.initial_duration}
            min={0}
            onChange={(v) =>
              v !== null && onConfigChange({ ...config, initial_duration: v })
            }
          />
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

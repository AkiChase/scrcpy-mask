import { useEffect, useMemo, useRef, useState } from "react";
import type {
  ButtonBinding,
  DirectionBinding,
  DirectionButtonBinding,
  DirectionJoyStickBinding,
  MappingUpdater,
  PadCastSpellConfig,
} from "./mapping";
import { Flex, InputNumber, Select, Switch, Tooltip, Typography } from "antd";
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

type PadCastSpellContentProps = {
  padBind: DirectionBinding;
  bind: ButtonBinding;
};

function BindText({ text }: { text: string }) {
  return (
    <Tooltip trigger="click" title={text}>
      <Typography.Text
        ellipsis={true}
        className="text-2.5 font-bold text-center"
      >
        {text}
      </Typography.Text>
    </Tooltip>
  );
}

function PadCastSpellContent({ padBind, bind }: PadCastSpellContentProps) {
  const bindText = bind.join("+");
  if (padBind.type === "Button") {
    const padBindTexts = {
      up: padBind.up.join("+"),
      down: padBind.down.join("+"),
      left: padBind.left.join("+"),
      right: padBind.right.join("+"),
    };

    return (
      <>
        <Flex className="flex-1" align="center">
          <BindText text={padBindTexts.up} />
        </Flex>
        <Flex className="w-full" justify="space-around" align="center">
          <BindText text={padBindTexts.left} />
          <BindText text={bindText} />
          <BindText text={padBindTexts.right} />
        </Flex>
        <Flex className="flex-1" align="center">
          <BindText text={padBindTexts.down} />
        </Flex>
      </>
    );
  } else {
    return (
      <>
        <Flex vertical gap={8}>
          <BindText text={padBind.x} />
          <BindText text={bindText} />
          <BindText text={padBind.y} />
        </Flex>
      </>
    );
  }
}

export default function ButtonPadCastSpell({
  index,
  config,
  originalSize,
  onConfigChange,
  onConfigDelete,
  onConfigCopy,
}: {
  index: number;
  config: PadCastSpellConfig;
  originalSize: { width: number; height: number };
  onConfigChange: MappingUpdater<PadCastSpellConfig>;
  onConfigDelete: () => void;
  onConfigCopy: () => void;
}) {
  const id = `mapping-direction-pad-${index}`;
  const className = useMemo(() => {
    const base =
      "rounded-full absolute box-border border-solid border-2 color-text ";
    if (config.pad_bind.type === "Button") {
      const { up, down, left, right } = config.pad_bind;
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
  }, [config.pad_bind]);

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

  const buttonStyle = useMemo(
    () =>
      mappingButtonPresetStyle(
        Math.round(config.drag_radius * scale.y),
        Math.round(config.drag_radius * scale.y)
      ),
    [config.drag_radius, scale]
  );

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
        <PadCastSpellContent padBind={config.pad_bind} bind={config.bind} />
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
  config: PadCastSpellConfig;
  onConfigChange: MappingUpdater<PadCastSpellConfig>;
  onConfigDelete: () => void;
  onConfigCopy: () => void;
}) {
  const { t } = useTranslation();

  const isJoyStick = config.pad_bind.type === "JoyStick";

  const padBindValueRef = useRef<{
    Button: DirectionButtonBinding;
    JoyStick: DirectionJoyStickBinding;
  }>(
    config.pad_bind.type === "Button"
      ? {
          Button: config.pad_bind,
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
          JoyStick: config.pad_bind,
        }
  );

  function toggleBindMode(toJoyStick: boolean) {
    if (toJoyStick) {
      onConfigChange({ ...config, pad_bind: padBindValueRef.current.JoyStick });
    } else {
      onConfigChange({ ...config, pad_bind: padBindValueRef.current.Button });
    }
  }

  function handlePadBindChange(
    type: "up" | "down" | "left" | "right" | "x" | "y",
    value: string | string[]
  ) {
    if (type === "x" || type === "y") {
      padBindValueRef.current.JoyStick[type] = value as string;
    } else {
      padBindValueRef.current.Button[type] = value as string[];
    }
    onConfigChange((pre) => ({
      ...pre,
      pad_bind: {
        ...config.pad_bind,
        [type]: value,
      },
    }));
  }

  return (
    <div>
      <h1 className="title-with-line">
        {t("mappings.padCastSpell.setting.title")}
      </h1>
      <ItemBoxContainer className="max-h-70vh overflow-y-auto pr-2 scrollbar">
        <SettingBind
          bind={config.bind}
          onBindChange={(bind) => onConfigChange((pre) => ({ ...pre, bind }))}
        />
        <ItemBox
          label={
            <Flex className="w-full" align="center" justify="space-between">
              <span>{t("mappings.padCastSpell.setting.padBindLabel")}</span>
              <Switch
                size="small"
                checkedChildren={t("mappings.padCastSpell.setting.joyStick")}
                unCheckedChildren={t("mappings.padCastSpell.setting.button")}
                checked={isJoyStick}
                onChange={toggleBindMode}
              />
            </Flex>
          }
        >
          {isJoyStick ? (
            <ItemBoxContainer gap={12} className="pl-8">
              <ItemBox label={t("mappings.padCastSpell.setting.xAxis")}>
                <Select
                  className="w-full"
                  value={(config.pad_bind as DirectionJoyStickBinding).x}
                  onChange={(v) => handlePadBindChange("x", v)}
                  options={gamepadAxisOptions}
                />
              </ItemBox>
              <ItemBox label={t("mappings.padCastSpell.setting.yAxis")}>
                <Select
                  className="w-full"
                  value={(config.pad_bind as DirectionJoyStickBinding).y}
                  onChange={(v) => handlePadBindChange("y", v)}
                  options={gamepadAxisOptions}
                />
              </ItemBox>
            </ItemBoxContainer>
          ) : (
            <ItemBoxContainer gap={12} className="pl-8">
              <SettingBind
                label={t("mappings.padCastSpell.setting.up")}
                bind={(config.pad_bind as DirectionButtonBinding).up}
                onBindChange={(bind) => handlePadBindChange("up", bind)}
              />
              <SettingBind
                label={t("mappings.padCastSpell.setting.down")}
                bind={(config.pad_bind as DirectionButtonBinding).down}
                onBindChange={(bind) => handlePadBindChange("down", bind)}
              />
              <SettingBind
                label={t("mappings.padCastSpell.setting.left")}
                bind={(config.pad_bind as DirectionButtonBinding).left}
                onBindChange={(bind) => handlePadBindChange("left", bind)}
              />
              <SettingBind
                label={t("mappings.padCastSpell.setting.right")}
                bind={(config.pad_bind as DirectionButtonBinding).right}
                onBindChange={(bind) => handlePadBindChange("right", bind)}
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
        <ItemBox label={t("mappings.padCastSpell.setting.dragRadius")}>
          <InputNumber
            className="w-full"
            value={config.drag_radius}
            min={1}
            onChange={(v) =>
              v !== null && onConfigChange({ ...config, drag_radius: v })
            }
          />
        </ItemBox>
        <ItemBox label={t("mappings.padCastSpell.setting.block")}>
          <Switch
            checked={config.block_direction_pad}
            onChange={(v) => {
              onConfigChange({ ...config, block_direction_pad: v });
            }}
          />
        </ItemBox>
        <ItemBox label={t("mappings.padCastSpell.setting.releaseMode.label")}>
          <Select
            className="w-full"
            value={config.release_mode}
            onChange={(v) => onConfigChange({ ...config, release_mode: v })}
            options={[
              {
                label: t("mappings.padCastSpell.setting.releaseMode.onRelease"),
                value: "OnRelease",
              },
              {
                label: t(
                  "mappings.padCastSpell.setting.releaseMode.onSecondPress"
                ),
                value: "OnSecondPress",
              },
            ]}
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

import { useEffect, useMemo, useState } from "react";
import type { MappingUpdater, ScriptConfig } from "./mapping";
import { Flex, InputNumber, Tooltip, Typography } from "antd";
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
} from "./Common";
import { useTranslation } from "react-i18next";
import { IconFont } from "../../hooks";
import { ScriptEditor } from "./ScriptEditor";

const PRESET_STYLE = mappingButtonPresetStyle(52);

export default function ButtonScript({
  index,
  config,
  originalSize,
  onConfigChange,
  onConfigDelete,
  onConfigCopy,
}: {
  index: number;
  config: ScriptConfig;
  originalSize: { width: number; height: number };
  onConfigChange: MappingUpdater<ScriptConfig>;
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
        style={PRESET_STYLE}
        className={className}
        onMouseDown={handleDrag}
        onContextMenu={handleSetting}
        justify="center"
        align="center"
        vertical
      >
        <Tooltip trigger="click" title={`${config.type}: ${bindText}`}>
          <Typography.Text ellipsis={true} className="text-2.5 font-bold">
            {bindText}
          </Typography.Text>
        </Tooltip>
        <IconFont type="icon-code" className="text-4"/>
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
  config: ScriptConfig;
  onConfigChange: MappingUpdater<ScriptConfig>;
  onConfigDelete: () => void;
  onConfigCopy: () => void;
}) {
  const { t } = useTranslation();

  return (
    <div>
      <h1 className="title-with-line">{t("mappings.script.setting.title")}</h1>
      <ItemBoxContainer className="max-h-70vh overflow-y-auto pr-2 scrollbar">
        <SettingMappingId id={config.id} />
        <SettingBind
          bind={config.bind}
          onBindChange={(bind) => onConfigChange((pre) => ({ ...pre, bind }))}
        />
        <ItemBox label={t("mappings.script.setting.interval")} tooltip={t("mappings.script.setting.intervalHint")}>
          <InputNumber
            className="w-full"
            value={config.interval}
            min={0}
            onChange={(v) =>
              v !== null && onConfigChange({ ...config, interval: v })
            }
          />
        </ItemBox>
        <ItemBox
          label={t("mappings.script.setting.pressed_script")}
          tooltip={t("mappings.script.setting.pressedScriptHint")}
        >
          <ScriptEditor
            minRows={1}
            maxRows={6}
            value={config.pressed_script}
            placeholder={t(
              "mappings.script.setting.pressed_script_placeholder",
            )}
            onChange={(value) =>
              onConfigChange({ ...config, pressed_script: value })
            }
          />
        </ItemBox>
        <ItemBox
          label={t("mappings.script.setting.held_script")}
          tooltip={t("mappings.script.setting.heldScriptHint")}
        >
          <ScriptEditor
            minRows={1}
            maxRows={6}
            value={config.held_script}
            placeholder={t("mappings.script.setting.held_script_placeholder")}
            onChange={(value) =>
              onConfigChange({ ...config, held_script: value })
            }
          />
        </ItemBox>
        <ItemBox
          label={t("mappings.script.setting.released_script")}
          tooltip={t("mappings.script.setting.releasedScriptHint")}
        >
          <ScriptEditor
            minRows={1}
            maxRows={6}
            value={config.released_script}
            placeholder={t(
              "mappings.script.setting.released_script_placeholder",
            )}
            onChange={(value) =>
              onConfigChange({ ...config, released_script: value })
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

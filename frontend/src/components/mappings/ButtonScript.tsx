import { useEffect, useMemo, useState } from "react";
import type { MappingUpdater, ScriptConfig } from "./mapping";
import { Flex, Input, InputNumber, Modal, Tooltip, Typography } from "antd";
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
} from "./Common";
import { useTranslation } from "react-i18next";
import IconButton from "../common/IconButton";
import { PlayCircleOutlined } from "@ant-design/icons";
import { useDispatch } from "react-redux";
import { setIsLoading } from "../../store/other";
import { requestPost } from "../../utils";
import { useMessageContext } from "../../hooks";

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
  const dispatch = useDispatch();
  const messageApi = useMessageContext();

  const [errorMsg, setErrorMsg] = useState("");
  const [open, setOpen] = useState(false);

  async function run_script(script: string) {
    dispatch(setIsLoading(true));
    try {
      const res = await requestPost("/api/device/control/eval_script", {
        script,
      });
      messageApi?.success(res.message);
    } catch (error: any) {
      setErrorMsg(error);
      setOpen(true);
    }
    dispatch(setIsLoading(false));
  }

  return (
    <div>
      <Modal
        title={t("mappings.script.setting.result")}
        className="min-w-50vw"
        open={open}
        onCancel={() => setOpen(false)}
        footer={null}
      >
        <Input.TextArea
          className="font-mono"
          value={errorMsg}
          readOnly
          autoSize
        />
      </Modal>
      <h1 className="title-with-line">{t("mappings.script.setting.title")}</h1>
      <ItemBoxContainer className="max-h-70vh overflow-y-auto pr-2 scrollbar">
        <SettingBind
          bind={config.bind}
          onBindChange={(bind) => onConfigChange((pre) => ({ ...pre, bind }))}
        />
        <ItemBox label={t("mappings.script.setting.interval")}>
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
          label={
            <Flex className="w-full" align="center" justify="space-between">
              <span>{t("mappings.script.setting.pressed_script")}</span>
              <IconButton
                tooltip={t("mappings.script.setting.run_script")}
                icon={<PlayCircleOutlined />}
                onClick={() => run_script(config.pressed_script)}
              />
            </Flex>
          }
        >
          <Input.TextArea
            className="w-full font-mono"
            value={config.pressed_script}
            placeholder={t(
              "mappings.script.setting.pressed_script_placeholder"
            )}
            autoSize={{ minRows: 1, maxRows: 10 }}
            onChange={(e) =>
              onConfigChange({ ...config, pressed_script: e.target.value })
            }
          />
        </ItemBox>
        <ItemBox
          label={
            <Flex className="w-full" align="center" justify="space-between">
              <span>{t("mappings.script.setting.held_script")}</span>
              <IconButton
                tooltip={t("mappings.script.setting.run_script")}
                icon={<PlayCircleOutlined />}
                onClick={() => run_script(config.held_script)}
              />
            </Flex>
          }
        >
          <Input.TextArea
            className="w-full font-mono"
            value={config.held_script}
            placeholder={t("mappings.script.setting.held_script_placeholder")}
            autoSize={{ minRows: 1, maxRows: 10 }}
            onChange={(e) =>
              onConfigChange({ ...config, held_script: e.target.value })
            }
          />
        </ItemBox>
        <ItemBox
          label={
            <Flex className="w-full" align="center" justify="space-between">
              <span>{t("mappings.script.setting.released_script")}</span>
              <IconButton
                tooltip={t("mappings.script.setting.run_script")}
                icon={<PlayCircleOutlined />}
                onClick={() => run_script(config.released_script)}
              />
            </Flex>
          }
        >
          <Input.TextArea
            className="w-full font-mono"
            value={config.released_script}
            placeholder={t(
              "mappings.script.setting.released_script_placeholder"
            )}
            autoSize={{ minRows: 1, maxRows: 10 }}
            onChange={(e) =>
              onConfigChange({ ...config, released_script: e.target.value })
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

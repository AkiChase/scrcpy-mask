import { useEffect, useMemo, useState } from "react";
import type {
  MappingUpdater,
  MultipleTapConfig,
  MultipleTapItem,
} from "./mapping";
import {
  Button,
  Flex,
  InputNumber,
  Popover,
  Space,
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
  DeviceBackground,
  RefreshImageButton,
  SettingBind,
  SettingFooter,
  SettingModal,
  SettingNote,
  SettingPointerId,
} from "./Common";
import { useTranslation } from "react-i18next";
import { RollbackOutlined } from "@ant-design/icons";
import { useMessageContext } from "../../hooks";

const PRESET_STYLE = mappingButtonPresetStyle(52);

export default function ButtonMultipleTap({
  index,
  config,
  originalSize,
  onConfigChange,
  onConfigDelete,
  onConfigCopy,
}: {
  index: number;
  config: MultipleTapConfig;
  originalSize: { width: number; height: number };
  onConfigChange: MappingUpdater<MultipleTapConfig>;
  onConfigDelete: () => void;
  onConfigCopy: () => void;
}) {
  const id = `mapping-multiple-tap-${index}`;
  const bindText = config.bind.join("+");
  const className =
    "rounded-full absolute box-border border-solid border-2 color-text " +
    (config.bind.length > 0
      ? "border-text-secondary hover:border-text"
      : "border-primary hover:border-primary-hover");

  const maskArea = useAppSelector((state) => state.other.maskArea);
  const [showSetting, setShowSetting] = useState(false);
  const [isEditingPos, setIsEditingPos] = useState(false);

  const scale = useMemo(() => {
    return {
      x: maskArea.width / originalSize.width,
      y: maskArea.height / originalSize.height,
    };
  }, [originalSize, maskArea]);

  useEffect(() => {
    const element = document.getElementById(id);
    if (element) {
      const position = config.items[0].position;
      element.style.transform = mappingButtonTransformStyle(
        position.x,
        position.y,
        scale
      );
    }
  }, [index, config, scale]);

  const handleDrag = mappingButtonDragFactory(
    maskArea,
    originalSize,
    ({ x, y }) => {
      const newConfig = {
        ...config,
      };
      newConfig.items[0].position = {
        x,
        y,
      };
      onConfigChange(newConfig);
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
          originalSize={originalSize}
          isEditing={isEditingPos}
          onIsEditingChange={(v) => setIsEditingPos(v)}
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
      {showSetting && !isEditingPos && (
        <Background items={config.items} originalSize={originalSize} />
      )}
    </>
  );
}

function Background({
  items,
  originalSize,
}: {
  items: MultipleTapItem[];
  originalSize: { width: number; height: number };
}) {
  const maskArea = useAppSelector((state) => state.other.maskArea);
  const scale = useMemo(() => {
    return {
      x: maskArea.width / originalSize.width,
      y: maskArea.height / originalSize.height,
    };
  }, [originalSize, maskArea]);

  return (
    <div
      className="fixed bg-transparent"
      style={{
        left: maskArea.left,
        top: maskArea.top,
        width: maskArea.width,
        height: maskArea.height,
      }}
    >
      {items.map((item, index) => {
        return (
          <div
            key={index}
            className="rounded-full w-3 h-3 bg-primary absolute left--1.5 top--1.5 text-center text-bold"
            style={{
              transform: mappingButtonTransformStyle(
                item.position.x,
                item.position.y,
                scale
              ),
            }}
          >
            <span className="relative bottom-5">{index + 1}</span>
          </div>
        );
      })}
    </div>
  );
}

type PositonEditorItemProps = {
  maskArea: { width: number; height: number; left: number; top: number };
  originalSize: { width: number; height: number };
  item: MultipleTapItem;
  index: number;
  onItemChange: (index: number, item: MultipleTapItem) => void;
  onItemDelete: (index: number) => void;
};

function PositonEditorItem({
  maskArea,
  originalSize,
  item,
  index,
  onItemChange,
  onItemDelete,
}: PositonEditorItemProps) {
  const { t } = useTranslation();

  const [open, setOpen] = useState(false);

  const handleDrag = mappingButtonDragFactory(
    maskArea,
    originalSize,
    (pos) => onItemChange(index, { ...item, position: pos }),
    100
  );

  const scale = useMemo(() => {
    return {
      x: maskArea.width / originalSize.width,
      y: maskArea.height / originalSize.height,
    };
  }, [originalSize, maskArea]);

  return (
    <Popover
      destroyOnHidden
      trigger="contextMenu"
      open={open}
      onOpenChange={(open) => setOpen(open)}
      content={
        <ItemBoxContainer gap={12}>
          <ItemBox label={t("mappings.multipleTap.setting.wait")}>
            <InputNumber
              className="w-full"
              value={item.wait}
              min={0}
              onChange={(v) =>
                v !== null && onItemChange(index, { ...item, wait: v })
              }
            />
          </ItemBox>
          <ItemBox label={t("mappings.multipleTap.setting.duration")}>
            <InputNumber
              className="w-full"
              value={item.duration}
              min={0}
              onChange={(v) =>
                v !== null && onItemChange(index, { ...item, duration: v })
              }
            />
          </ItemBox>
          <ItemBox>
            <Button
              block
              type="primary"
              onClick={() => {
                setOpen(false);
                onItemDelete(index);
              }}
            >
              {t("mappings.multipleTap.setting.delete")}
            </Button>
          </ItemBox>
        </ItemBoxContainer>
      }
    >
      <div
        className="rounded-full w-3 h-3 bg-primary absolute left--1.5 top--1.5 text-center text-bold hover:bg-primary-hover active:bg-primary-active"
        style={{
          transform: mappingButtonTransformStyle(
            item.position.x,
            item.position.y,
            scale
          ),
        }}
        onMouseDown={handleDrag}
      >
        <span className="relative bottom-5 whitespace-nowrap">{index + 1}</span>
      </div>
    </Popover>
  );
}

function PositonEditor({
  items,
  originalSize,
  onExit,
  onChange,
}: {
  items: MultipleTapItem[];
  originalSize: { width: number; height: number };
  onExit: () => void;
  onChange: (items: MultipleTapItem[]) => void;
}) {
  const maskArea = useAppSelector((state) => state.other.maskArea);
  const messageApi = useMessageContext();
  const { t } = useTranslation();

  function handleItemDelete(index: number) {
    if (items.length === 1) {
      messageApi?.warning(t("mappings.multipleTap.setting.keepLastOne"));
      return;
    }
    onChange(items.filter((_, i) => i !== index));
  }

  function handleItemChange(index: number, item: MultipleTapItem) {
    onChange([...items.slice(0, index), item, ...items.slice(index + 1)]);
  }

  function handleEditorClick(e: React.MouseEvent) {
    if (e.target === e.currentTarget && e.button === 2) {
      onChange([
        ...items,
        {
          duration: 50,
          position: clientPositionToMappingPosition(
            e.clientX,
            e.clientY,
            maskArea,
            originalSize.width,
            originalSize.height
          ),
          wait: 50,
        },
      ]);
    }
  }

  return (
    <div className="select-none fixed left-0 top-0 right-0 bottom-0 bg-[var(--ant-color-bg-mask)] z-2000">
      <Space.Compact className="absolute top-8 right-8 z--1">
        <RefreshImageButton />
        <Button
          type="primary"
          icon={<RollbackOutlined />}
          onClick={() => onExit()}
        >
          {t("mappings.multipleTap.setting.back")}
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
        <div
          className="w-full h-full absolute"
          onMouseDown={handleEditorClick}
          onContextMenu={(e) => e.preventDefault()}
        >
          {items.map((item, index) => (
            <PositonEditorItem
              key={index}
              item={item}
              index={index}
              onItemChange={handleItemChange}
              onItemDelete={handleItemDelete}
              maskArea={maskArea}
              originalSize={originalSize}
            />
          ))}
        </div>
      </div>
    </div>
  );
}

function Setting({
  config,
  onConfigChange,
  onConfigDelete,
  onConfigCopy,
  originalSize,
  isEditing,
  onIsEditingChange,
}: {
  config: MultipleTapConfig;
  onConfigChange: MappingUpdater<MultipleTapConfig>;
  onConfigDelete: () => void;
  onConfigCopy: () => void;
  originalSize: { width: number; height: number };
  isEditing: boolean;
  onIsEditingChange: (v: boolean) => void;
}) {
  const { t } = useTranslation();
  const messageApi = useMessageContext();

  return (
    <div>
      <h1 className="title-with-line">
        {t("mappings.multipleTap.setting.title")}
      </h1>
      {isEditing && (
        <PositonEditor
          items={config.items}
          originalSize={originalSize}
          onExit={() => onIsEditingChange(false)}
          onChange={(items) => onConfigChange({ ...config, items })}
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
        <ItemBox label={t("mappings.multipleTap.setting.operations")}>
          <Button
            type="dashed"
            onClick={() => {
              messageApi?.info(
                t("mappings.multipleTap.setting.operationsHelp")
              );
              onIsEditingChange(true);
            }}
          >
            {t("mappings.multipleTap.setting.edit")}
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

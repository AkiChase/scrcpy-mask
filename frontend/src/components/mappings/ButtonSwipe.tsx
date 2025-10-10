import { useEffect, useMemo, useState } from "react";
import type { MappingUpdater, SwipeConfig } from "./mapping";
import { Button, Flex, Popover, Space, Tooltip, Typography } from "antd";
import {
  clientPositionToMappingPosition,
  mappingButtonDragFactory,
  mappingButtonPosition,
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

type Position = { x: number; y: number };

export default function ButtonSwipe({
  index,
  config,
  originalSize,
  onConfigChange,
  onConfigDelete,
  onConfigCopy,
}: {
  index: number;
  config: SwipeConfig;
  originalSize: { width: number; height: number };
  onConfigChange: MappingUpdater<SwipeConfig>;
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
      const position = config.positions[0];
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
      newConfig.positions[0] = {
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
        <Background positions={config.positions} originalSize={originalSize} />
      )}
    </>
  );
}

function Background({
  positions,
  originalSize,
}: {
  positions: Position[];
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
      <svg className="w-full h-full absolute color-primary">
        <defs>
          <marker
            id="arrow"
            markerWidth="8"
            markerHeight="7"
            refX="8"
            refY="3.5"
            orient="auto"
            markerUnits="strokeWidth"
          >
            <path d="M0,0 L8,3.5 L0,7 Z" fill="currentColor" />
          </marker>
        </defs>
        {positions.map((pos, index) => {
          if (index === positions.length - 1) return null;
          const { x: x1, y: y1 } = mappingButtonPosition(pos.x, pos.y, scale);
          const { x: x2, y: y2 } = mappingButtonPosition(
            positions[index + 1].x,
            positions[index + 1].y,
            scale
          );

          return (
            <line
              key={index}
              x1={x1}
              y1={y1}
              x2={x2}
              y2={y2}
              stroke="currentColor"
              strokeWidth="2"
              markerEnd="url(#arrow)"
            />
          );
        })}
      </svg>
      {positions.map((position, index) => {
        return (
          <div
            key={index}
            className="rounded-full w-3 h-3 bg-primary absolute left--1.5 top--1.5 text-center text-bold"
            style={{
              transform: mappingButtonTransformStyle(
                position.x,
                position.y,
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
  position: Position;
  index: number;
  onItemChange: (index: number, position: Position) => void;
  onItemDelete: (index: number) => void;
};

function PositonEditorItem({
  maskArea,
  originalSize,
  position,
  index,
  onItemChange,
  onItemDelete,
}: PositonEditorItemProps) {
  const { t } = useTranslation();

  const [open, setOpen] = useState(false);

  const handleDrag = mappingButtonDragFactory(
    maskArea,
    originalSize,
    (pos) => onItemChange(index, pos),
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
          <ItemBox>
            <Button
              block
              type="primary"
              onClick={() => {
                setOpen(false);
                onItemDelete(index);
              }}
            >
              {t("mappings.swipe.setting.delete")}
            </Button>
          </ItemBox>
        </ItemBoxContainer>
      }
    >
      <div
        className="rounded-full w-3 h-3 bg-primary absolute left--1.5 top--1.5 text-center text-bold hover:bg-primary-hover active:bg-primary-active"
        style={{
          transform: mappingButtonTransformStyle(position.x, position.y, scale),
        }}
        onMouseDown={handleDrag}
      >
        <span className="relative bottom-5 whitespace-nowrap">{index + 1}</span>
      </div>
    </Popover>
  );
}

function PositonEditor({
  positions,
  originalSize,
  onExit,
  onChange,
}: {
  positions: Position[];
  originalSize: { width: number; height: number };
  onExit: () => void;
  onChange: (positions: Position[]) => void;
}) {
  const maskArea = useAppSelector((state) => state.other.maskArea);
  const messageApi = useMessageContext();
  const { t } = useTranslation();

  const scale = useMemo(() => {
    return {
      x: maskArea.width / originalSize.width,
      y: maskArea.height / originalSize.height,
    };
  }, [originalSize, maskArea]);

  function handleItemDelete(index: number) {
    if (positions.length === 1) {
      messageApi?.warning(t("mappings.swipe.setting.keepLastOne"));
      return;
    }
    onChange(positions.filter((_, i) => i !== index));
  }

  function handleItemChange(index: number, position: Position) {
    onChange([
      ...positions.slice(0, index),
      position,
      ...positions.slice(index + 1),
    ]);
  }

  function handleEditorClick(e: React.MouseEvent) {
    if (e.target === e.currentTarget && e.button === 2) {
      onChange([
        ...positions,
        clientPositionToMappingPosition(
          e.clientX,
          e.clientY,
          maskArea,
          originalSize.width,
          originalSize.height
        ),
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
          {t("mappings.swipe.setting.back")}
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
        <svg
          className="w-full h-full absolute color-primary"
          onMouseDown={handleEditorClick}
          onContextMenu={(e) => e.preventDefault()}
        >
          <defs>
            <marker
              id="arrow"
              markerWidth="8"
              markerHeight="7"
              refX="8"
              refY="3.5"
              orient="auto"
              markerUnits="strokeWidth"
            >
              <path d="M0,0 L8,3.5 L0,7 Z" fill="currentColor" />
            </marker>
          </defs>
          {positions.map((pos, index) => {
            if (index === positions.length - 1) return null;
            const { x: x1, y: y1 } = mappingButtonPosition(pos.x, pos.y, scale);
            const { x: x2, y: y2 } = mappingButtonPosition(
              positions[index + 1].x,
              positions[index + 1].y,
              scale
            );

            return (
              <line
                key={index}
                x1={x1}
                y1={y1}
                x2={x2}
                y2={y2}
                stroke="currentColor"
                strokeWidth="2"
                markerEnd="url(#arrow)"
              />
            );
          })}
        </svg>
        {positions.map((position, index) => (
          <PositonEditorItem
            key={index}
            position={position}
            index={index}
            onItemChange={handleItemChange}
            onItemDelete={handleItemDelete}
            maskArea={maskArea}
            originalSize={originalSize}
          />
        ))}
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
  config: SwipeConfig;
  onConfigChange: MappingUpdater<SwipeConfig>;
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
      <h1 className="title-with-line">{t("mappings.swipe.setting.title")}</h1>
      {isEditing && (
        <PositonEditor
          positions={config.positions}
          originalSize={originalSize}
          onExit={() => onIsEditingChange(false)}
          onChange={(positions) => onConfigChange({ ...config, positions })}
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
        <ItemBox label={t("mappings.swipe.setting.positions")}>
          <Button
            type="dashed"
            onClick={() => {
              messageApi?.info(t("mappings.swipe.setting.positonsHelp"));
              onIsEditingChange(true);
            }}
          >
            {t("mappings.swipe.setting.edit")}
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

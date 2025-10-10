import {
  EditOutlined,
  CloseCircleOutlined,
  SyncOutlined,
  DeleteOutlined,
  CopyOutlined,
} from "@ant-design/icons";
import {
  Input,
  Badge,
  Space,
  type InputRef,
  Flex,
  Modal,
  Switch,
  InputNumber,
  Button,
  Select,
  type SelectProps,
} from "antd";
import {
  useState,
  useRef,
  useEffect,
  type PropsWithChildren,
  forwardRef,
  type ComponentPropsWithoutRef,
} from "react";
import IconButton from "../common/IconButton";

import type { ButtonBinding } from "./mapping";
import { EVENT_CODE_TO_KEY_CODE, KEY_NAMES } from "./keyCode";
import { debounce } from "../../utils";
import { useTranslation } from "react-i18next";
import { ItemBox } from "../common/ItemBox";
import { useAppSelector } from "../../store/store";
import { useRefreshBackgroundImage } from "../../hooks";
import { mappingModalDragFactory } from "./tools";

const MOUSE_BUTTONS = ["M-Left", "M-Middle", "M-Right", "M-Forward", "M-Back"];

type SettingModalProps = PropsWithChildren<{
  open: boolean;
  onClose: () => void;
}>;

export function SettingModal({ children, open, onClose }: SettingModalProps) {
  const handleDrag = mappingModalDragFactory()
  return (
    <Modal
      footer={null}
      open={open}
      onCancel={onClose}
      destroyOnHidden={true}
      keyboard={false}
      className="w-min-50vw setting-modal"
    >
      <div className="mx-auto mb-3 mt--3 h-1.5 w-50% rounded-full bg-text-secondary transition-colors duration-400
      hover:bg-text-tertiary cursor-grab active:cursor-grabbing select-none"
        onMouseDown={handleDrag}
      />
      {children}
    </Modal>
  );
}

type SettingBindProps = {
  bind: ButtonBinding;
  onBindChange: (bind: ButtonBinding) => void;
  label?: string;
};

export function SettingBind({ bind, onBindChange, label }: SettingBindProps) {
  const { t } = useTranslation();
  const [isManualInput, setIsManualInput] = useState(false);

  label = label ?? t("mappings.common.bind.settingLabel");

  return (
    <ItemBox
      label={
        <Flex className="w-full" align="center" justify="space-between">
          <span>{label}</span>
          <Switch
            size="small"
            checkedChildren={t("mappings.common.bind.settingManual")}
            unCheckedChildren={t("mappings.common.bind.settingAuto")}
            checked={isManualInput}
            onChange={(checked) => setIsManualInput(checked)}
          />
        </Flex>
      }
    >
      <InputBinding
        manual={isManualInput}
        bind={bind}
        onBindChange={onBindChange}
      />
    </ItemBox>
  );
}

type SettingPointerIdProps = {
  pointerId: number;
  onPointerIdChange: (pointerId: number) => void;
};

export function SettingPointerId({
  pointerId,
  onPointerIdChange,
}: SettingPointerIdProps) {
  const { t } = useTranslation();

  return (
    <ItemBox label={t("mappings.common.pointerId.label")}>
      <InputNumber
        className="w-full"
        value={pointerId}
        min={0}
        step={1}
        onChange={(v) => v !== null && onPointerIdChange(v)}
      />
    </ItemBox>
  );
}

type SettingNoteProps = {
  note: string;
  onNoteChange: (note: string) => void;
};

export function SettingNote({ note, onNoteChange }: SettingNoteProps) {
  const { t } = useTranslation();

  return (
    <ItemBox label={t("mappings.common.note.label")}>
      <Input value={note} onChange={(e) => onNoteChange(e.target.value)} />
    </ItemBox>
  );
}

export function SettingDelete({ onDelete }: { onDelete: () => void }) {
  const { t } = useTranslation();

  return (
    <ItemBox>
      <Button block type="primary" onClick={onDelete}>
        {t("mappings.common.delete.label")}
      </Button>
    </ItemBox>
  );
}

export function SettingFooter({
  onDelete,
  onCopy,
}: {
  onDelete: () => void;
  onCopy: () => void;
}) {
  const { t } = useTranslation();
  return (
    <Flex align="center" justify="end" gap={12}>
      <Button type="dashed" onClick={onCopy} icon={<CopyOutlined />}>
        {t("mappings.common.copy.label")}
      </Button>
      <Button type="dashed" onClick={onDelete} icon={<DeleteOutlined />}>
        {t("mappings.common.delete.label")}
      </Button>
    </Flex>
  );
}

function mappingButtonBindFactory(
  inputElement: HTMLElement,
  onBindChange: (bind: ButtonBinding) => void,
  onIsRecordingChange: (isRecording: boolean) => void
) {
  const pressedKeys = new Set<string>();

  const handleKeyDown = (e: KeyboardEvent) => {
    e.preventDefault();
    if (!pressedKeys.has(e.code)) {
      if (e.code in EVENT_CODE_TO_KEY_CODE) {
        pressedKeys.add(
          EVENT_CODE_TO_KEY_CODE[e.code as keyof typeof EVENT_CODE_TO_KEY_CODE]
        );
        onBindChange([...pressedKeys]);
      } else {
        console.warn("Unknow keycode: ", e.code);
      }
    }
  };

  const handleKeyUp = (e: KeyboardEvent) => {
    e.preventDefault();
    if (e.code in EVENT_CODE_TO_KEY_CODE) {
      pressedKeys.delete(
        EVENT_CODE_TO_KEY_CODE[e.code as keyof typeof EVENT_CODE_TO_KEY_CODE]
      );
    }
  };

  const handleMouseDown = (e: MouseEvent) => {
    if (!inputElement.contains(e.target as Node) && e.button === 0) {
      stopRecord();
      return;
    }
    e.preventDefault();

    const key =
      e.button >= 0 && e.button < MOUSE_BUTTONS.length
        ? MOUSE_BUTTONS[e.button]
        : `M-Other-${e.button}`;
    pressedKeys.add(key);
    onBindChange([...pressedKeys]);
  };

  const handleMouseUp = (e: MouseEvent) => {
    e.preventDefault();
    const key =
      e.button >= 0 && e.button < MOUSE_BUTTONS.length
        ? MOUSE_BUTTONS[e.button]
        : `M-Other-${e.button}`;

    pressedKeys.delete(key);
  };

  const handleWheel = (() => {
    const debounced = debounce((deltaY: number) => {
      const key = deltaY > 0 ? "ScrollDown" : "ScrollUp";
      pressedKeys.add(key);
      onBindChange([...pressedKeys]);
      pressedKeys.delete(key);
    }, 50);

    return (e: WheelEvent) => {
      e.preventDefault();

      if (e.deltaY === 0) return;
      debounced(e.deltaY);
    };
  })();

  const handleBlur = () => {
    pressedKeys.clear();
  };

  const handleContextMenu = (e: MouseEvent) => {
    e.preventDefault();
  };

  const startRecord = () => {
    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);
    window.addEventListener("blur", handleBlur);
    window.addEventListener("mousedown", handleMouseDown);
    window.addEventListener("mouseup", handleMouseUp);
    window.addEventListener("contextmenu", handleContextMenu);
    window.addEventListener("wheel", handleWheel, { passive: false });
    onIsRecordingChange(true);
  };

  const stopRecord = () => {
    window.removeEventListener("keydown", handleKeyDown);
    window.removeEventListener("keyup", handleKeyUp);
    window.removeEventListener("blur", handleBlur);
    window.removeEventListener("mousedown", handleMouseDown);
    window.removeEventListener("mouseup", handleMouseUp);
    window.removeEventListener("contextmenu", handleContextMenu);
    window.removeEventListener("wheel", handleWheel);
    onIsRecordingChange(false);
  };

  return {
    startRecord,
    stopRecord,
  };
}

function AutoInputBinding({
  bind,
  onBindChange,
}: {
  bind: ButtonBinding;
  onBindChange: (bind: ButtonBinding) => void;
}) {
  const { t } = useTranslation();
  const [isRecording, setIsRecording] = useState(false);
  const inputRef = useRef<InputRef>(null);
  const startRecord = useRef(() => { });

  useEffect(() => {
    startRecord.current = mappingButtonBindFactory(
      inputRef.current!.nativeElement as HTMLElement,
      onBindChange,
      setIsRecording
    ).startRecord;
  }, []);

  return (
    <Input
      ref={inputRef}
      value={bind.join("+")}
      placeholder={t("mappings.common.bind.autoInputPlaceholder")}
      readOnly
      onDoubleClick={() => {
        return !isRecording && startRecord.current();
      }}
      suffix={
        isRecording ? (
          <Badge color="red" text="Recording..." />
        ) : (
          <Space>
            <IconButton
              size={14}
              icon={<EditOutlined />}
              onClick={startRecord.current}
            />
            <IconButton
              size={14}
              icon={<CloseCircleOutlined />}
              onClick={() => onBindChange([])}
            />
          </Space>
        )
      }
    />
  );
}

const KeyNameOptions: SelectProps["options"] = KEY_NAMES.map((v) => ({
  value: v,
  label: v,
}));

function ManualInputBinding({
  bind,
  onBindChange,
}: {
  bind: ButtonBinding;
  onBindChange: (bind: ButtonBinding) => void;
}) {
  const { t } = useTranslation();
  return (
    <Select
      mode="multiple"
      allowClear
      className="w-full"
      placeholder={t("mappings.common.bind.manualSelectPlaceholder")}
      value={bind}
      onChange={(v) => onBindChange(v)}
      options={KeyNameOptions}
    />
  );
}

export function InputBinding({
  bind,
  onBindChange,
  manual,
}: {
  bind: ButtonBinding;
  onBindChange: (bind: ButtonBinding) => void;
  manual: boolean;
}) {
  return manual ? (
    <ManualInputBinding bind={bind} onBindChange={onBindChange} />
  ) : (
    <AutoInputBinding bind={bind} onBindChange={onBindChange} />
  );
}

export function DeviceBackground({ alpha }: { alpha?: number }) {
  const backgroundImage = useAppSelector(
    (state) => state.other.backgroundImage
  );

  alpha = alpha ?? 0.4;

  return (
    <div
      className="absolute w-full h-full bg-[length:100%_100%] bg-origin-content bg-no-repeat"
      style={{
        backgroundImage: `url(${backgroundImage})`,
      }}
    >
      <div
        className="w-full h-full"
        style={{ backgroundColor: `rgba(0,0,0,${alpha})` }}
      ></div>
    </div>
  );
}

export const CursorPos = forwardRef<
  HTMLDivElement,
  ComponentPropsWithoutRef<"div">
>((props, ref) => {
  const className =
    "cursor-default color-text-secondary font-bold z-10 " +
    (props.className ?? "");

  return <div style={props.style} ref={ref} className={className} />;
});

export function RefreshImageButton() {
  const { t } = useTranslation();
  const refreshBackground = useRefreshBackgroundImage();

  return (
    <Button
      type="primary"
      icon={<SyncOutlined />}
      onClick={() => refreshBackground()}
    >
      {t("mappings.common.refreshBackground")}
    </Button>
  );
}

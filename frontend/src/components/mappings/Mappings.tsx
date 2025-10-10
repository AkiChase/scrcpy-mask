import {
  type DirectionBinding,
  type MappingConfig,
  type MappingType,
} from "./mapping";
import * as MappingConstructor from "./mapping";

import {
  Badge,
  Button,
  Dropdown,
  Flex,
  Input,
  InputNumber,
  Modal,
  Popconfirm,
  Select,
  Space,
  Splitter,
  Table,
  type TableProps,
} from "antd";
import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type PropsWithChildren,
  type ReactNode,
} from "react";
import { useAppDispatch, useAppSelector } from "../../store/store";
import {
  CheckCircleOutlined,
  CopyOutlined,
  DeleteOutlined,
  EditOutlined,
  FileAddOutlined,
  FileSyncOutlined,
  FileTextOutlined,
  RollbackOutlined,
  SaveOutlined,
  SettingOutlined,
  SnippetsOutlined,
} from "@ant-design/icons";
import IconButton from "../common/IconButton";
import { deepClone, requestGet, requestPost, throttle } from "../../utils";
import { useMessageContext, useRefreshBackgroundImage } from "../../hooks";
import ButtonSingleTap from "./ButtonSingleTap";
import { setIsLoading, setMaskArea } from "../../store/other";
import ButtonRepeatTap from "./ButtonRepeatTap";
import ButtonMultipleTap from "./ButtonMultipleTap";
import { clientPositionToMappingPosition } from "./tools";
import ButtonSwipe from "./ButtonSwipe";
import ButtonDirectionPad from "./ButtonDirectionPad";
import ButtonMouseCastSpell from "./ButtonMouseCastSpell";
import { CursorPos, DeviceBackground, RefreshImageButton } from "./Common";
import ButtonPadCastSpell from "./ButtonPadCastSpell";
import ButtonCancelCast from "./ButtonCancelCast";
import ButtonObservation from "./ButtonObservation";
import ButtonFps from "./ButtonFps";
import ButtonRawInput from "./ButtonRawInput";
import { setActiveMappingFile } from "../../store/localConfig";
import { useTranslation } from "react-i18next";
import { ItemBox, ItemBoxContainer } from "../common/ItemBox";
import ButtonFire from "./ButtonFire";
import ButtonScript from "./ButtonScript";

type MappingFileTabelItem = {
  file: string;
  active: boolean;
  displayed: boolean;
};

type ConfirmProps = PropsWithChildren<{
  title: string;
  defaultValue: string;
  extral?: ReactNode;
  onConfirm: (value: string) => void;
}>;

function Confirm({
  title,
  defaultValue,
  extral,
  onConfirm,
  children,
}: ConfirmProps) {
  const { t } = useTranslation();
  defaultValue = defaultValue ?? "";
  const [newFile, setNewFile] = useState(defaultValue);

  return (
    <Popconfirm
      title={title}
      destroyOnHidden
      description={
        <ItemBoxContainer gap={8}>
          <ItemBox label={t("mappings.home.file")}>
            <Input
              placeholder={t("mappings.home.fileInputPlaceholder")}
              value={newFile}
              onChange={(e) => setNewFile(e.target.value)}
            />
          </ItemBox>
          {extral}
        </ItemBoxContainer>
      }
      onConfirm={() => onConfirm(newFile)}
      okText={t("mappings.home.confirmYes")}
      cancelText={t("mappings.home.confirmNo")}
    >
      {children}
    </Popconfirm>
  );
}

function Manager({
  open,
  onCancel,
  mappingList,
  displayedMapping,
  onActiveAction,
  onDisplayAction,
  onDuplicateAction,
  onDeleteAction,
  onCreateAction,
  onRenameAction,
  onMigrateAction,
}: {
  open: boolean;
  onCancel: () => void;
  mappingList: string[];
  displayedMapping: string;
  onActiveAction: (file: string) => void;
  onDisplayAction: (file: string) => void;
  onDuplicateAction: (file: string, newFile: string) => void;
  onDeleteAction: (file: string) => void;
  onCreateAction: (
    file: string,
    size: { width: number; height: number }
  ) => void;
  onRenameAction: (file: string, newFile: string) => void;
  onMigrateAction: (
    file: string,
    newFile: string,
    size: { width: number; height: number }
  ) => void;
}) {
  const { t } = useTranslation();
  const messageApi = useMessageContext();
  const activeMappingFile = useAppSelector(
    (state) => state.localConfig.activeMappingFile
  );
  const controlledDevices = useAppSelector(
    (state) => state.other.controlledDevices
  );

  const [newSize, setNewSize] = useState<{ width: number; height: number }>({
    width: 1280,
    height: 720,
  });

  const mappingFiles = useMemo<MappingFileTabelItem[]>(() => {
    return mappingList.map((file) => {
      return {
        file,
        active: file === activeMappingFile,
        displayed: file === displayedMapping,
      };
    });
  }, [mappingList, activeMappingFile, displayedMapping]);

  const columns: TableProps<MappingFileTabelItem>["columns"] = [
    {
      title: (
        <Space size="large">
          {t("mappings.home.file")}
          <Confirm
            title={t("mappings.home.createTitle")}
            onConfirm={(newFile) => onCreateAction(newFile, newSize)}
            defaultValue=""
            extral={
              <ItemBox label={t("mappings.home.size")}>
                <Space.Compact className="w-full">
                  <InputNumber
                    className="w-full"
                    prefix="W:"
                    value={newSize.width}
                    min={1}
                    onChange={(v) =>
                      v !== null && setNewSize({ ...newSize, width: v })
                    }
                  />
                  <InputNumber
                    className="w-full"
                    prefix="H:"
                    value={newSize.height}
                    min={1}
                    onChange={(v) =>
                      v !== null && setNewSize({ ...newSize, height: v })
                    }
                  />
                </Space.Compact>
              </ItemBox>
            }
          >
            <IconButton
              color="info"
              tooltip={t("mappings.home.create")}
              icon={<FileAddOutlined />}
              onClick={() => {
                const mainDevice = controlledDevices.find((d) => d.main);
                if (mainDevice) {
                  setNewSize({
                    width: mainDevice.device_size[0],
                    height: mainDevice.device_size[1],
                  });
                }
              }}
            />
          </Confirm>
        </Space>
      ),
      dataIndex: "file",
      key: "file",
      render: (_, record) => (
        <Flex align="center" justify="space-between" className="p-r-3">
          <span>{record.file}</span>
          <Space size={32}>
            {record.displayed && (
              <Badge status="processing" text={t("mappings.home.editing")} />
            )}
            {record.active && (
              <Badge status="success" text={t("mappings.home.active")} />
            )}
          </Space>
        </Flex>
      ),
    },
    {
      title: t("mappings.home.action"),
      key: "action",
      align: "center",
      width: 1,
      render: (_, record) => (
        <Space size="middle" className="text-4">
          <IconButton
            color="info"
            icon={<FileTextOutlined />}
            tooltip={t("mappings.home.edit")}
            onClick={() => onDisplayAction(record.file)}
          />
          <IconButton
            color="success"
            tooltip={t("mappings.home.activate")}
            icon={<CheckCircleOutlined />}
            onClick={() => onActiveAction(record.file)}
          />
          <Confirm
            title={t("mappings.home.renameTitle")}
            onConfirm={(newFile) => {
              if (newFile === record.file) {
                messageApi?.warning(t("mappings.home.differentName"));
              } else {
                onRenameAction(record.file, newFile);
              }
            }}
            defaultValue={record.file}
          >
            <IconButton
              color="warning"
              icon={<EditOutlined />}
              tooltip={t("mappings.home.rename")}
            />
          </Confirm>
          <Popconfirm
            title={t("mappings.home.deleteTitle")}
            destroyOnHidden
            description={t("mappings.home.deletePrompt")}
            onConfirm={() => onDeleteAction(record.file)}
            okText={t("mappings.home.confirmYes")}
            cancelText={t("mappings.home.confirmNo")}
          >
            <IconButton
              color="error"
              tooltip={t("mappings.home.delete")}
              icon={<DeleteOutlined />}
            />
          </Popconfirm>
          <Confirm
            title={t("mappings.home.duplicateTitle")}
            onConfirm={(newFile) => {
              if (newFile === record.file) {
                messageApi?.warning(t("mappings.home.differentName"));
              } else {
                onDuplicateAction(record.file, newFile);
              }
            }}
            defaultValue={record.file}
          >
            <IconButton
              color="info"
              tooltip={t("mappings.home.duplicate")}
              icon={<CopyOutlined />}
            />
          </Confirm>
          <Confirm
            title={t("mappings.home.migrationTitle")}
            onConfirm={(newFile) => {
              if (newFile === record.file) {
                messageApi?.warning(t("mappings.home.differentName"));
              } else {
                onMigrateAction(record.file, newFile, newSize);
              }
            }}
            defaultValue={record.file}
            extral={
              <ItemBox label={t("mappings.home.size")}>
                <Space.Compact className="w-full">
                  <InputNumber
                    className="w-full"
                    prefix="W:"
                    value={newSize.width}
                    min={1}
                    onChange={(v) =>
                      v !== null && setNewSize({ ...newSize, width: v })
                    }
                  />
                  <InputNumber
                    className="w-full"
                    prefix="H:"
                    value={newSize.height}
                    min={1}
                    onChange={(v) =>
                      v !== null && setNewSize({ ...newSize, height: v })
                    }
                  />
                </Space.Compact>
              </ItemBox>
            }
          >
            <IconButton
              color="warning"
              tooltip={t("mappings.home.migration")}
              icon={<SnippetsOutlined />}
              onClick={() => {
                const mainDevice = controlledDevices.find((d) => d.main);
                if (mainDevice) {
                  setNewSize({
                    width: mainDevice.device_size[0],
                    height: mainDevice.device_size[1],
                  });
                } else {
                  messageApi?.warning(t("mappings.common.noMainDevice"));
                }
              }}
            />
          </Confirm>
        </Space>
      ),
    },
  ];

  return (
    <Modal
      title={t("mappings.home.manager")}
      className="min-w-50vw"
      open={open}
      onCancel={onCancel}
      footer={null}
    >
      <Table<MappingFileTabelItem>
        size="small"
        rowKey={(record) => record.file}
        pagination={{ pageSize: 7 }}
        columns={columns}
        dataSource={mappingFiles}
      />
    </Modal>
  );
}

type EditState = {
  file: string;
  edited: boolean;
  current: MappingConfig;
  old: MappingConfig;
};

const buttonTypes = [
  "SingleTap",
  "RepeatTap",
  "MultipleTap",
  "Swipe",
  "DirectionPad",
  "MouseCastSpell",
  "PadCastSpell",
  "CancelCast",
  "Observation",
  "Fps",
  "Fire",
  "RawInput",
  "Script",
];

const mappingButtonMap = {
  SingleTap: ButtonSingleTap,
  RepeatTap: ButtonRepeatTap,
  MultipleTap: ButtonMultipleTap,
  Swipe: ButtonSwipe,
  DirectionPad: ButtonDirectionPad,
  MouseCastSpell: ButtonMouseCastSpell,
  PadCastSpell: ButtonPadCastSpell,
  CancelCast: ButtonCancelCast,
  Observation: ButtonObservation,
  Fps: ButtonFps,
  Fire: ButtonFire,
  RawInput: ButtonRawInput,
  Script: ButtonScript,
};

const mappingConstructorMap: any = Object.fromEntries(
  buttonTypes.map((key) => [
    key,
    MappingConstructor[`new${key}` as keyof typeof MappingConstructor],
  ])
);

const menuItems = buttonTypes.map((key) => [
  key,
  `mappings.${key.charAt(0).toLowerCase() + key.slice(1)}.name`,
]);

function Displayer({
  state,
  setState,
}: {
  state: EditState;
  setState: React.Dispatch<React.SetStateAction<EditState | null>>;
}) {
  const dispatch = useAppDispatch();
  const maskArea = useAppSelector((state) => state.other.maskArea);
  const { t } = useTranslation();

  const cursorPosRef = useRef<HTMLDivElement>(null);
  const displayerRef = useRef<HTMLDivElement>(null);
  const contextMenuPosRef = useRef<{ x: number; y: number }>({ x: 0, y: 0 });

  useEffect(() => {
    const displayerElement = displayerRef.current;
    if (!displayerElement) return;

    const observer = new ResizeObserver(() => {
      const rect = displayerElement.getBoundingClientRect();
      dispatch(
        setMaskArea({
          width: rect.width - 2,
          height: rect.height - 2,
          left: rect.left + 1,
          top: rect.top + 1,
        })
      );
    });
    observer.observe(displayerElement);

    return () => {
      observer.disconnect();
    };
  }, [displayerRef.current]);

  const { ratioStyle, originalSize } = useMemo(() => {
    return {
      originalSize: state.current.original_size,
      ratioStyle: {
        width: "100%",
        aspectRatio: `${state.current.original_size.width} / ${state.current.original_size.height}`,
      },
    };
  }, [state.current.original_size.width, state.current.original_size.height]);

  function updateMapping(
    index: number,
    updater: MappingType | ((prev: any) => any)
  ) {
    setState((prev) => {
      if (prev === null) return null;
      const newState = { ...prev };
      newState.edited = true;
      newState.current.mappings[index] =
        typeof updater === "function"
          ? updater(newState.current.mappings[index])
          : updater;

      return newState;
    });
  }

  function deleteMappingButton(index: number) {
    setState((prev) => {
      if (prev === null) return null;
      const newState = { ...prev };
      newState.edited = true;
      newState.current.mappings.splice(index, 1);

      return newState;
    });
  }

  function copyMappingButton(index: number) {
    setState((prev) => {
      if (prev === null) return null;
      const newState = { ...prev };
      newState.edited = true;
      newState.current.mappings.push(newState.current.mappings[index]);

      return newState;
    });
  }

  const handleMouseMove = throttle((e: React.MouseEvent) => {
    if (cursorPosRef.current) {
      const { x, y } = clientPositionToMappingPosition(
        e.clientX,
        e.clientY,
        maskArea,
        state.current.original_size.width,
        state.current.original_size.height
      );
      cursorPosRef.current.innerText = `(${x},${y})`;
    }
  }, 100);

  return (
    <div className="w-full">
      <Flex justify="space-between">
        <CursorPos ref={cursorPosRef} />
        <div className="color-text-secondary font-bold">
          {`[${originalSize.width} x ${originalSize.height}]`}
        </div>
      </Flex>
      <div
        ref={displayerRef}
        className="w-full border-text-quaternary border-solid border relative select-none"
        style={ratioStyle}
        onMouseMove={handleMouseMove}
      >
        <DeviceBackground />
        <Dropdown
          menu={{
            items: menuItems.map(([key, tID]) => ({
              key,
              label: t(tID),
            })),
            onClick({ key }) {
              let config: MappingType;
              if (key === "MouseCastSpell") {
                config = mappingConstructorMap.MouseCastSpell(
                  contextMenuPosRef.current,
                  {
                    x: originalSize.width / 2,
                    y: Math.round(originalSize.height * 0.566),
                  }
                );
              } else {
                config = mappingConstructorMap[key](contextMenuPosRef.current);
              }
              const newState = { ...state, edited: true };
              newState.current.mappings.push(config);
              setState(newState);
            },
          }}
          trigger={["contextMenu"]}
        >
          <div
            onContextMenu={(e) => {
              contextMenuPosRef.current = clientPositionToMappingPosition(
                e.clientX,
                e.clientY,
                maskArea,
                originalSize.width,
                originalSize.height
              );
            }}
            className="w-full h-full absolute bg-transparent"
          />
        </Dropdown>
        {state.current.mappings.map((mapping, index) => {
          const props: any = {
            originalSize,
            index,
            config: mapping,
            onConfigChange: (updater: any | ((prev: any) => any)) =>
              updateMapping(index, updater),
            onConfigDelete: () => deleteMappingButton(index),
            onConfigCopy: () => copyMappingButton(index),
          };

          if (mapping.type in mappingButtonMap) {
            const ButtonComponent =
              mappingButtonMap[mapping.type as keyof typeof mappingButtonMap];
            return <ButtonComponent key={index} {...props} />;
          }

          return <div key={index}></div>;
        })}
      </div>
    </div>
  );
}

export default function Mappings() {
  const messageApi = useMessageContext();
  const activeMappingFile = useAppSelector(
    (state) => state.localConfig.activeMappingFile
  );
  const refreshBackground = useRefreshBackgroundImage();
  const dispatch = useAppDispatch();
  const { t } = useTranslation();

  const [displayedMappingFile, setDisplayedMappingFile] = useState("");
  const [isManagerOpen, setIsManagerOpen] = useState(false);
  const [editState, setEditState] = useState<EditState | null>(null);
  const [mappingList, setMappingList] = useState<string[]>([]);

  const mappingListOptions = useMemo(() => {
    return mappingList.map((item) => ({
      label: (
        <Flex justify="space-between" align="center">
          <span>{item}</span>
          {activeMappingFile === item && (
            <Badge color="green" text={t("mappings.home.active")} />
          )}
        </Flex>
      ),
      value: item,
    }));
  }, [mappingList, activeMappingFile]);

  useEffect(() => {
    loadMappingList();
    refreshBackground(true);
  }, []);

  useEffect(() => {
    if (displayedMappingFile === "" && activeMappingFile !== "") {
      changeDisplayedMapping(activeMappingFile);
    }
  }, [activeMappingFile]);

  async function loadMappingList(silent: boolean = false) {
    if (!silent) dispatch(setIsLoading(true));
    try {
      const res = await requestGet<{
        mapping_list: string[];
        active_mapping: string;
      }>("/api/mapping/get_mapping_list");
      setMappingList(res.data.mapping_list);
      if (activeMappingFile !== res.data.active_mapping)
        dispatch(setActiveMappingFile(res.data.active_mapping));

      // current displayed file is not in the list
      if (
        res.data.mapping_list.findIndex(
          (file) => file === displayedMappingFile
        ) == -1
      ) {
        setDisplayedMappingFile(res.data.active_mapping);
      }
    } catch (error: any) {
      if (!silent) messageApi?.error(error);
    }
    if (!silent) dispatch(setIsLoading(false));
  }

  async function changeDisplayedMapping(file: string) {
    if (!file) return;
    dispatch(setIsLoading(true));
    try {
      const res = await requestPost<{ mapping_config: MappingConfig }>(
        "/api/mapping/read_mapping",
        {
          file,
        }
      );
      const mappingConfig = res.data.mapping_config;
      setDisplayedMappingFile(file);
      setEditState({
        file,
        edited: false,
        current: mappingConfig,
        old: deepClone(mappingConfig),
      });
    } catch (error: any) {
      messageApi?.error(error);
    }
    dispatch(setIsLoading(false));
  }

  async function changeActiveMapping(file: string) {
    dispatch(setIsLoading(true));
    try {
      const res = await requestPost("/api/mapping/change_active_mapping", {
        file,
      });
      dispatch(setActiveMappingFile(file));
      messageApi?.success(res.message);
    } catch (error: any) {
      messageApi?.error(error);
    }
    dispatch(setIsLoading(false));
  }

  async function updateMappingFile() {
    if (editState) {
      const errorMag = t("mappings.home.emptyBind");
      const curConfig = editState.current;
      const validateDirectionBind = (bind: DirectionBinding) => {
        if (bind.type === "Button") {
          for (const b of [bind.up, bind.down, bind.left, bind.right]) {
            if (b.length === 0) {
              messageApi?.error(errorMag);
              return false;
            }
          }
        } else {
          if (bind.x === "" || bind.y === "") {
            messageApi?.error(errorMag);
            return false;
          }
        }
        return true;
      };
      for (const mapping of curConfig.mappings) {
        if (Array.isArray(mapping.bind)) {
          if (mapping.bind.length === 0) {
            messageApi?.error(errorMag);
            return;
          }
        } else {
          if (!validateDirectionBind(mapping.bind)) {
            return;
          }
        }

        if ("pad_bind" in mapping) {
          if (!validateDirectionBind(mapping.pad_bind)) {
            return;
          }
        }
      }

      dispatch(setIsLoading(true));
      try {
        const res = await requestPost("/api/mapping/update_mapping", {
          file: editState.file,
          config: curConfig,
        });
        messageApi?.success(res.message);
        setEditState({
          file: editState.file,
          edited: false,
          current: curConfig,
          old: deepClone(curConfig),
        });
      } catch (error) {
        messageApi?.error(error as string);
      }
      dispatch(setIsLoading(false));
    }
  }

  async function restoreMappingFile() {
    if (editState) {
      setEditState({
        old: editState.old,
        current: deepClone(editState.old),
        edited: false,
        file: editState.file,
      });
    }
  }

  async function duplicateMappingFile(file: string, newFile: string) {
    dispatch(setIsLoading(true));
    try {
      const res = await requestPost("/api/mapping/duplicate_mapping", {
        file,
        new_file: newFile,
      });
      await loadMappingList(true);
      messageApi?.success(res.message);
    } catch (error) {
      messageApi?.error(error as string);
    }
    dispatch(setIsLoading(false));
  }

  async function deleteMappingFile(file: string) {
    dispatch(setIsLoading(true));
    try {
      const res = await requestPost("/api/mapping/delete_mapping", {
        file,
      });
      await loadMappingList(true);
      messageApi?.success(res.message);
    } catch (error) {
      messageApi?.error(error as string);
    }
    dispatch(setIsLoading(false));
  }

  async function createMappingFile(
    file: string,
    size: { width: number; height: number }
  ) {
    dispatch(setIsLoading(true));
    try {
      const res = await requestPost("/api/mapping/create_mapping", {
        file,
        config: {
          version: "0.0.1",
          original_size: size,
          mappings: [],
        },
      });
      await loadMappingList(true);
      messageApi?.success(res.message);
    } catch (error) {
      messageApi?.error(error as string);
    }
    dispatch(setIsLoading(false));
  }

  async function renameMappingFile(file: string, newFile: string) {
    dispatch(setIsLoading(true));
    try {
      const res = await requestPost("/api/mapping/rename_mapping", {
        file,
        new_file: newFile,
      });
      await loadMappingList(true);
      messageApi?.success(res.message);
    } catch (error) {
      messageApi?.error(error as string);
    }
    dispatch(setIsLoading(false));
  }

  async function migrateMappingFile(
    file: string,
    newFile: string,
    size: {
      width: number;
      height: number;
    }
  ) {
    dispatch(setIsLoading(true));
    try {
      const res = await requestPost("/api/mapping/migrate_mapping", {
        file,
        new_file: newFile,
        width: size.width,
        height: size.height,
      });
      await loadMappingList(true);
      messageApi?.success(res.message);
    } catch (error) {
      messageApi?.error(error as string);
    }
    dispatch(setIsLoading(false));
  }

  return (
    <Flex vertical gap={32} className="page-container hide-scrollbar">
      <Manager
        open={isManagerOpen}
        onCancel={() => setIsManagerOpen(false)}
        mappingList={mappingList}
        displayedMapping={displayedMappingFile}
        onActiveAction={changeActiveMapping}
        onDisplayAction={changeDisplayedMapping}
        onDuplicateAction={duplicateMappingFile}
        onDeleteAction={deleteMappingFile}
        onCreateAction={createMappingFile}
        onRenameAction={renameMappingFile}
        onMigrateAction={migrateMappingFile}
      />
      <section>
        <Flex justify="space-between" align="center">
          <Space.Compact>
            <Select
              className="w-80"
              showSearch
              value={displayedMappingFile}
              onChange={(value) => changeDisplayedMapping(value)}
              options={mappingListOptions}
            />
            <Button
              type="primary"
              disabled={editState === null || editState.edited === false}
              icon={<SaveOutlined />}
              onClick={updateMappingFile}
            >
              {t("mappings.home.save")}
            </Button>
            <Button
              type="primary"
              disabled={editState === null || editState.edited === false}
              icon={<RollbackOutlined />}
              onClick={restoreMappingFile}
            >
              {t("mappings.home.restore")}
            </Button>
            <Button
              disabled={activeMappingFile === displayedMappingFile}
              type="primary"
              icon={<CheckCircleOutlined />}
              onClick={() => changeActiveMapping(displayedMappingFile)}
            >
              {t("mappings.home.activate")}
            </Button>
            <Button
              type="primary"
              icon={<FileSyncOutlined />}
              onClick={() => loadMappingList()}
            >
              {t("mappings.home.refresh")}
            </Button>
            <Button
              type="primary"
              icon={<SettingOutlined />}
              onClick={() => setIsManagerOpen(true)}
            >
              {t("mappings.home.manage")}
            </Button>
          </Space.Compact>
          <RefreshImageButton />
        </Flex>
      </section>
      <section className="flex-grow-1 flex-shrink-0 pb-4">
        {editState && (
          <Splitter className="w-full h-full">
            <Splitter.Panel
              className="flex justify-center items-center"
              defaultSize="95%"
              min="5%"
              max="99%"
            >
              <Displayer state={editState} setState={setEditState} />
            </Splitter.Panel>
            <Splitter.Panel />
          </Splitter>
        )}
      </section>
    </Flex>
  );
}

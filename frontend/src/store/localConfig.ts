import { createSlice, type PayloadAction } from "@reduxjs/toolkit";
import { requestPost, toCamelCase } from "../utils";
import i18n from "../i18n";
import { staticStore } from "./store";

async function _updateLocalConfig(key: string, value: any) {
  try {
    const res = await requestPost("/api/config/update_config", {
      key,
      value,
    });
    staticStore.messageApi?.success(res.message);
  } catch (error: any) {
    staticStore.messageApi?.error(error);
  }
}

const debounceMap = new Map<string, ReturnType<typeof setTimeout>>();

function updateLocalConfig(key: string, value: any, time = 500) {
  if (debounceMap.has(key)) {
    clearTimeout(debounceMap.get(key)!);
  }

  const timeout = setTimeout(() => {
    _updateLocalConfig(key, value);
    debounceMap.delete(key);
  }, time);

  debounceMap.set(key, timeout);
}

export interface LocalConfigState {
  // port
  webPort: number;
  controllerPort: number;
  // adb
  adbPath: string;
  // mask area
  alwaysOnTop: boolean;
  verticalMaskHeight: number;
  horizontalMaskWidth: number;
  verticalPosition: [number, number];
  horizontalPosition: [number, number];
  // mapping
  activeMappingFile: string;
  mappingLabelOpacity: number;
  // language
  language: string;
  // clipboard sync
  clipboardSync: boolean;
  // video
  videoCodec: string;
  videoBitRate: number;
  videoMaxSize: number;
  videoMaxFps: number;
}

const initialState: LocalConfigState = {
  webPort: 0,
  controllerPort: 0,
  adbPath: "",
  alwaysOnTop: true,
  verticalMaskHeight: 0,
  horizontalMaskWidth: 0,
  verticalPosition: [0, 0],
  horizontalPosition: [0, 0],
  activeMappingFile: "",
  mappingLabelOpacity: 0,
  language: "en-US",
  clipboardSync: true,
  videoCodec: "H264",
  videoBitRate: 8000000,
  videoMaxSize: 0,
  videoMaxFps: 0
};

const localConfigSlice = createSlice({
  name: "localConfig",
  initialState,
  reducers: {
    forceSetLocalConfig: (state, action: PayloadAction<any>) => {
      for (const [key, value] of Object.entries(action.payload)) {
        const curKey = toCamelCase(key);
        if (curKey in state) {
          (state as any)[curKey] = value;
        }
      }
    },
    setWebPort: (state, action: PayloadAction<number>) => {
      state.webPort = action.payload;
      updateLocalConfig("web_port", action.payload);
    },
    setControllerPort: (state, action: PayloadAction<number>) => {
      state.controllerPort = action.payload;
      updateLocalConfig("controller_port", action.payload);
    },
    setAdbPath: (state, action: PayloadAction<string>) => {
      state.adbPath = action.payload;
      updateLocalConfig("adb_path", action.payload, 1000);
    },
    setAlwaysOnTop: (state, action: PayloadAction<boolean>) => {
      state.alwaysOnTop = action.payload;
      updateLocalConfig("always_on_top", action.payload);
    },
    setverticalMaskHeight: (state, action: PayloadAction<number>) => {
      state.verticalMaskHeight = action.payload;
      updateLocalConfig("vertical_screen_height", action.payload, 1000);
    },
    sethorizontalMaskWidth: (state, action: PayloadAction<number>) => {
      state.horizontalMaskWidth = action.payload;
      updateLocalConfig("horizontal_screen_width", action.payload, 1000);
    },
    setVerticalPosition: (state, action: PayloadAction<[number, number]>) => {
      state.verticalPosition = action.payload;
      updateLocalConfig("vertical_position", action.payload, 1000);
    },
    setHorizontalPosition: (state, action: PayloadAction<[number, number]>) => {
      state.horizontalPosition = action.payload;
      updateLocalConfig("horizontal_position", action.payload, 1000);
    },
    setActiveMappingFile: (state, action: PayloadAction<string>) => {
      state.activeMappingFile = action.payload;
      // already updated by change_active_mapping
    },
    setMappingLabelOpacity: (state, action: PayloadAction<number>) => {
      state.mappingLabelOpacity = action.payload;
      updateLocalConfig("mapping_label_opacity", action.payload);
    },
    setLanguage: (state, action: PayloadAction<string>) => {
      state.language = action.payload;
      i18n.changeLanguage(action.payload);
      updateLocalConfig("language", action.payload);
    },
    setClipboardSync: (state, action: PayloadAction<boolean>) => {
      state.clipboardSync = action.payload;
      updateLocalConfig("clipboard_sync", action.payload);
    },
    setVideoCodec: (state, action: PayloadAction<string>) => {
      state.videoCodec = action.payload;
      updateLocalConfig("video_codec", action.payload);
    },
    setVideoBitRate: (state, action: PayloadAction<number>) => {
      state.videoBitRate = action.payload;
      updateLocalConfig("video_bit_rate", action.payload);
    },
    setVideoMaxSize: (state, action: PayloadAction<number>) => {
      state.videoMaxSize = action.payload;
      updateLocalConfig("video_max_size", action.payload);
    },
    setVideoMaxFps: (state, action: PayloadAction<number>) => {
      state.videoMaxFps = action.payload;
      updateLocalConfig("video_max_fps", action.payload);
    },
  },
});

export const {
  forceSetLocalConfig,
  setWebPort,
  setControllerPort,
  setAdbPath,
  setAlwaysOnTop,
  setverticalMaskHeight,
  sethorizontalMaskWidth,
  setVerticalPosition,
  setHorizontalPosition,
  setActiveMappingFile,
  setMappingLabelOpacity,
  setLanguage,
  setClipboardSync,
  setVideoCodec,
  setVideoBitRate,
  setVideoMaxSize,
  setVideoMaxFps,
} = localConfigSlice.actions;

export default localConfigSlice.reducer;

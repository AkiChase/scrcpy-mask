import { createSlice, type PayloadAction } from "@reduxjs/toolkit";
import type { AdbDevice, ControlledDevice } from "../utils";

export interface DeviceRotation {
  rotation: number;
  width: number;
  height: number;
}

export interface OtherState {
  isLoading: boolean;
  maskArea: {
    width: number;
    height: number;
    left: number;
    top: number;
  };
  backgroundImage: string;
  controlledDevices: ControlledDevice[];
  adbDevices: AdbDevice[];
  deviceRotations: Record<string, DeviceRotation>;
  updateInfo: {
    hasUpdate: boolean;
    currentVersion: string;
    latestVersion: string;
    title: string;
    body: string;
    time: string;
  };
  showUpdateDialog: boolean;
}

const initialState: OtherState = {
  isLoading: false,
  maskArea: {
    width: 1,
    height: 1,
    left: 0,
    top: 0,
  },
  backgroundImage: "",
  controlledDevices: [],
  adbDevices: [],
  deviceRotations: {},
  updateInfo: {
    hasUpdate: false,
    currentVersion: "Unknown",
    latestVersion: "Unknown",
    title: "Unknown",
    body: "Unknown",
    time: "",
  },
  showUpdateDialog: false,
};

const otherSlice = createSlice({
  name: "other",
  initialState,
  reducers: {
    setIsLoading: (state, action: PayloadAction<OtherState["isLoading"]>) => {
      state.isLoading = action.payload;
    },
    setMaskArea: (state, action: PayloadAction<OtherState["maskArea"]>) => {
      state.maskArea = action.payload;
    },
    setBackgroundImage: (
      state,
      action: PayloadAction<OtherState["backgroundImage"]>
    ) => {
      if (state.backgroundImage) {
        URL.revokeObjectURL(state.backgroundImage);
      }
      state.backgroundImage = action.payload;
    },
    setControlledDevices: (
      state,
      action: PayloadAction<OtherState["controlledDevices"]>
    ) => {
      state.controlledDevices = action.payload;
    },
    setAdbDevices: (state, action: PayloadAction<OtherState["adbDevices"]>) => {
      state.adbDevices = action.payload;
    },
    setUpdateInfo: (state, action: PayloadAction<OtherState["updateInfo"]>) => {
      state.updateInfo = action.payload;
    },
    setShowUpdateDialog: (
      state,
      action: PayloadAction<OtherState["showUpdateDialog"]>
    ) => {
      state.showUpdateDialog = action.payload;
    },
    setDeviceRotation: (
      state,
      action: PayloadAction<{ scid: string } & DeviceRotation>
    ) => {
      const { scid, rotation, width, height } = action.payload;
      state.deviceRotations[scid] = { rotation, width, height };
      const device = state.controlledDevices.find((d) => d.scid === scid);
      if (device) {
        device.device_size = [width, height];
      }
    },
  },
});

export const {
  setIsLoading,
  setMaskArea,
  setBackgroundImage,
  setControlledDevices,
  setAdbDevices,
  setDeviceRotation,
  setUpdateInfo,
  setShowUpdateDialog,
} = otherSlice.actions;

export default otherSlice.reducer;

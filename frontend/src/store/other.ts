import { createSlice, type PayloadAction } from "@reduxjs/toolkit";
import type { ControlledDevice } from "../utils";

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
    setUpdateInfo: (state, action: PayloadAction<OtherState["updateInfo"]>) => {
      state.updateInfo = action.payload;
    },
    setShowUpdateDialog: (
      state,
      action: PayloadAction<OtherState["showUpdateDialog"]>
    ) => {
      state.showUpdateDialog = action.payload;
    },
  },
});

export const {
  setIsLoading,
  setMaskArea,
  setBackgroundImage,
  setControlledDevices,
  setUpdateInfo,
  setShowUpdateDialog,
} = otherSlice.actions;

export default otherSlice.reducer;

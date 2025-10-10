export interface MappingConfig {
  version: string;
  original_size: {
    width: number;
    height: number;
  };
  mappings: MappingType[];
}

export type MappingType =
  | SingleTapConfig
  | RepeatTapConfig
  | MultipleTapConfig
  | SwipeConfig
  | DirectionPadConfig
  | MouseCastSpellConfig
  | PadCastSpellConfig
  | CancelCastConfig
  | ObservationConfig
  | FpsConfig
  | FireConfig
  | RawInputConfig
  | ScriptConfig;

export type Position = {
  x: number;
  y: number;
};

export type ButtonBinding = string[];

export interface SingleTapConfig {
  bind: ButtonBinding;
  duration: number;
  note: string;
  pointer_id: number;
  position: Position;
  sync: boolean;
  type: "SingleTap";
}

export function newSingleTap(position: Position): SingleTapConfig {
  return {
    bind: [],
    duration: 50,
    note: "",
    pointer_id: 1,
    position,
    sync: false,
    type: "SingleTap",
  };
}

export interface RepeatTapConfig {
  bind: ButtonBinding;
  duration: number;
  interval: number;
  note: string;
  pointer_id: number;
  position: Position;
  type: "RepeatTap";
}

export function newRepeatTap(position: Position): RepeatTapConfig {
  return {
    bind: [],
    duration: 50,
    interval: 100,
    note: "",
    pointer_id: 1,
    position,
    type: "RepeatTap",
  };
}

export interface MultipleTapItem {
  duration: number;
  position: Position;
  wait: number;
}

export interface MultipleTapConfig {
  bind: ButtonBinding;
  items: MultipleTapItem[];
  note: string;
  pointer_id: number;
  type: "MultipleTap";
}

export function newMultipleTap(position: Position): MultipleTapConfig {
  return {
    bind: [],
    items: [
      {
        duration: 50,
        position,
        wait: 0,
      },
    ],
    note: "",
    pointer_id: 1,
    type: "MultipleTap",
  };
}

export interface SwipeConfig {
  bind: ButtonBinding;
  interval: number;
  note: string;
  pointer_id: number;
  positions: Position[];
  type: "Swipe";
}

export function newSwipe(position: Position): SwipeConfig {
  return {
    bind: [],
    interval: 100,
    note: "",
    pointer_id: 1,
    positions: [position],
    type: "Swipe",
  };
}

export interface DirectionButtonBinding {
  type: "Button";
  up: ButtonBinding;
  down: ButtonBinding;
  left: ButtonBinding;
  right: ButtonBinding;
}

export interface DirectionJoyStickBinding {
  type: "JoyStick";
  x: string;
  y: string;
}

export type DirectionBinding =
  | DirectionButtonBinding
  | DirectionJoyStickBinding;

export interface DirectionPadConfig {
  bind: DirectionBinding;
  initial_duration: number;
  max_offset_x: number;
  max_offset_y: number;
  note: string;
  pointer_id: number;
  position: Position;
  type: "DirectionPad";
}

export function newDirectionPad(position: Position): DirectionPadConfig {
  return {
    bind: {
      type: "Button",
      up: [],
      down: [],
      left: [],
      right: [],
    },
    initial_duration: 0,
    max_offset_x: 200,
    max_offset_y: 200,
    note: "",
    pointer_id: 2,
    position,
    type: "DirectionPad",
  };
}

export type MouseCastReleaseMode = "OnPress" | "OnRelease" | "OnSecondPress";

export interface MouseCastSpellConfig {
  bind: ButtonBinding;
  cast_no_direction: boolean;
  cast_radius: number;
  center: Position;
  drag_radius: number;
  horizontal_scale_factor: number;
  vertical_scale_factor: number;
  note: string;
  pointer_id: number;
  position: Position;
  release_mode: MouseCastReleaseMode;
  type: "MouseCastSpell";
}

export function newMouseCastSpell(
  position: Position,
  center: Position
): MouseCastSpellConfig {
  return {
    bind: [],
    cast_no_direction: false,
    cast_radius: 200,
    center,
    drag_radius: 150,
    horizontal_scale_factor: 7,
    vertical_scale_factor: 10,
    note: "",
    pointer_id: 3,
    position,
    release_mode: "OnRelease",
    type: "MouseCastSpell",
  };
}

export type PadCastReleaseMode = "OnRelease" | "OnSecondPress";

export interface PadCastSpellConfig {
  bind: ButtonBinding;
  block_direction_pad: boolean;
  drag_radius: number;
  note: string;
  pad_bind: DirectionBinding;
  pointer_id: number;
  position: Position;
  release_mode: PadCastReleaseMode;
  type: "PadCastSpell";
}

export function newPadCastSpell(position: Position): PadCastSpellConfig {
  return {
    bind: [],
    block_direction_pad: false,
    drag_radius: 150,
    note: "",
    pad_bind: {
      type: "Button",
      up: [],
      down: [],
      left: [],
      right: [],
    },
    pointer_id: 3,
    position,
    release_mode: "OnRelease",
    type: "PadCastSpell",
  };
}

export interface CancelCastConfig {
  bind: ButtonBinding;
  note: string;
  position: Position;
  type: "CancelCast";
}

export function newCancelCast(position: Position): CancelCastConfig {
  return {
    bind: [],
    note: "",
    position,
    type: "CancelCast",
  };
}

export interface ObservationConfig {
  bind: ButtonBinding;
  note: string;
  pointer_id: number;
  position: Position;
  sensitivity_x: number;
  sensitivity_y: number;
  type: "Observation";
}

export function newObservation(position: Position): ObservationConfig {
  return {
    bind: [],
    note: "",
    pointer_id: 4,
    position,
    sensitivity_x: 1,
    sensitivity_y: 1,
    type: "Observation",
  };
}

export interface FpsConfig {
  bind: ButtonBinding;
  note: string;
  pointer_id: number;
  position: Position;
  sensitivity_x: number;
  sensitivity_y: number;
  type: "Fps";
}

export function newFps(position: Position): FpsConfig {
  return {
    bind: [],
    note: "",
    pointer_id: 0,
    position,
    sensitivity_x: 1,
    sensitivity_y: 1,
    type: "Fps",
  };
}

export interface FireConfig {
  bind: ButtonBinding;
  note: string;
  pointer_id: number;
  position: Position;
  sensitivity_x: number;
  sensitivity_y: number;
  type: "Fire";
}

export function newFire(position: Position): FireConfig {
  return {
    bind: [],
    note: "",
    pointer_id: 0,
    position,
    sensitivity_x: 1,
    sensitivity_y: 1,
    type: "Fire",
  };
}

export interface RawInputConfig {
  bind: ButtonBinding;
  note: string;
  position: Position;
  type: "RawInput";
}

export function newRawInput(position: Position): RawInputConfig {
  return {
    bind: [],
    note: "",
    position,
    type: "RawInput",
  };
}

export interface ScriptConfig {
  bind: ButtonBinding;
  note: string;
  position: Position;
  pressed_script: string;
  released_script: string;
  held_script: string;
  interval: number;
  type: "Script";
}

export function newScript(position: Position): ScriptConfig {
  return {
    bind: [],
    note: "",
    position,
    pressed_script: "",
    released_script: "",
    held_script: "",
    interval: 300,
    type: "Script",
  };
}

export type MappingUpdater<T> = (updater: T | ((pre: T) => T)) => void;

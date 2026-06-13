import {
  default_jitter_offset,
  default_random_distance_max_scale,
  default_random_distance_min_scale,
  default_random_offset,
} from "../../utils";

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

export interface MappingScriptHooks {
  before_script: string;
  after_script: string;
}

export function defaultScriptHooks(): MappingScriptHooks {
  return {
    before_script: "",
    after_script: "",
  };
}

export function newMappingId(): string {
  if (typeof crypto !== "undefined" && "getRandomValues" in crypto) {
    const value = crypto.getRandomValues(new Uint32Array(1))[0];
    return value.toString(16).padStart(8, "0");
  }
  return Math.floor(Math.random() * 0x100000000)
    .toString(16)
    .padStart(8, "0");
}

export interface SingleTapConfig {
  id: string;
  bind: ButtonBinding;
  duration: number;
  note: string;
  pointer_id: number;
  position: Position;
  random_offset_x: number;
  random_offset_y: number;
  script_hooks: MappingScriptHooks;
  sync: boolean;
  type: "SingleTap";
}

export function newSingleTap(position: Position): SingleTapConfig {
  return {
    id: newMappingId(),
    bind: [],
    duration: 50,
    note: "",
    pointer_id: 1,
    position,
    random_offset_x: default_random_offset,
    random_offset_y: default_random_offset,
    script_hooks: defaultScriptHooks(),
    sync: false,
    type: "SingleTap",
  };
}

export interface RepeatTapConfig {
  id: string;
  bind: ButtonBinding;
  duration: number;
  interval: number;
  note: string;
  pointer_id: number;
  position: Position;
  random_offset_x: number;
  random_offset_y: number;
  script_hooks: MappingScriptHooks;
  type: "RepeatTap";
}

export function newRepeatTap(position: Position): RepeatTapConfig {
  return {
    id: newMappingId(),
    bind: [],
    duration: 50,
    interval: 100,
    note: "",
    pointer_id: 1,
    position,
    random_offset_x: default_random_offset,
    random_offset_y: default_random_offset,
    script_hooks: defaultScriptHooks(),
    type: "RepeatTap",
  };
}

export interface MultipleTapItem {
  duration: number;
  position: Position;
  wait: number;
}

export interface MultipleTapConfig {
  id: string;
  bind: ButtonBinding;
  items: MultipleTapItem[];
  note: string;
  pointer_id: number;
  random_offset_x: number;
  random_offset_y: number;
  script_hooks: MappingScriptHooks;
  type: "MultipleTap";
}

export function newMultipleTap(position: Position): MultipleTapConfig {
  return {
    id: newMappingId(),
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
    random_offset_x: default_random_offset,
    random_offset_y: default_random_offset,
    script_hooks: defaultScriptHooks(),
    type: "MultipleTap",
  };
}

export interface SwipeConfig {
  id: string;
  bind: ButtonBinding;
  enable_randomization: boolean;
  duration: number;
  note: string;
  pointer_id: number;
  positions: Position[];
  script_hooks: MappingScriptHooks;
  type: "Swipe";
}

export function newSwipe(position: Position): SwipeConfig {
  return {
    id: newMappingId(),
    bind: [],
    enable_randomization: false,
    duration: 100,
    note: "",
    pointer_id: 1,
    positions: [position],
    script_hooks: defaultScriptHooks(),
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
  id: string;
  bind: DirectionBinding;
  enable_randomization: boolean;
  initial_duration: number;
  max_offset_x: number;
  max_offset_y: number;
  note: string;
  pointer_id: number;
  position: Position;
  random_distance_max_scale: number;
  random_distance_min_scale: number;
  random_offset_x: number;
  random_offset_y: number;
  jitter_offset_x: number;
  jitter_offset_y: number;
  script_hooks: MappingScriptHooks;
  type: "DirectionPad";
  up_boost_key: ButtonBinding | null;
  up_boost_scale: number;
}

export function newDirectionPad(position: Position): DirectionPadConfig {
  return {
    id: newMappingId(),
    bind: {
      type: "Button",
      up: [],
      down: [],
      left: [],
      right: [],
    },
    enable_randomization: false,
    initial_duration: 0,
    max_offset_x: 200,
    max_offset_y: 200,
    note: "",
    pointer_id: 2,
    position,
    random_distance_max_scale: default_random_distance_max_scale,
    random_distance_min_scale: default_random_distance_min_scale,
    random_offset_x: default_random_offset,
    random_offset_y: default_random_offset,
    jitter_offset_x: default_jitter_offset,
    jitter_offset_y: default_jitter_offset,
    script_hooks: defaultScriptHooks(),
    type: "DirectionPad",
    up_boost_key: null,
    up_boost_scale: 2.0,
  };
}

export type MouseCastReleaseMode = "OnPress" | "OnRelease" | "OnSecondPress";

export interface MouseCastSpellConfig {
  id: string;
  bind: ButtonBinding;
  cast_no_direction: boolean;
  cast_radius: number;
  center: Position;
  drag_radius: number;
  horizontal_scale_factor: number;
  initial_duration: number;
  note: string;
  pointer_id: number;
  position: Position;
  random_offset_x: number;
  random_offset_y: number;
  release_mode: MouseCastReleaseMode;
  script_hooks: MappingScriptHooks;
  type: "MouseCastSpell";
  vertical_scale_factor: number;
}

export function newMouseCastSpell(
  position: Position,
  center: Position
): MouseCastSpellConfig {
  return {
    id: newMappingId(),
    bind: [],
    cast_no_direction: false,
    cast_radius: 200,
    center,
    drag_radius: 150,
    horizontal_scale_factor: 7,
    note: "",
    pointer_id: 3,
    position,
    random_offset_x: default_random_offset,
    random_offset_y: default_random_offset,
    initial_duration: 0,
    release_mode: "OnRelease",
    script_hooks: defaultScriptHooks(),
    type: "MouseCastSpell",
    vertical_scale_factor: 10,
  };
}

export type PadCastReleaseMode = "OnRelease" | "OnSecondPress";

export interface PadCastSpellConfig {
  id: string;
  bind: ButtonBinding;
  block_direction_pad: boolean;
  drag_radius: number;
  enable_randomization: boolean;
  note: string;
  pad_bind: DirectionBinding;
  pointer_id: number;
  position: Position;
  random_offset_x: number;
  random_offset_y: number;
  release_mode: PadCastReleaseMode;
  script_hooks: MappingScriptHooks;
  type: "PadCastSpell";
}

export function newPadCastSpell(position: Position): PadCastSpellConfig {
  return {
    id: newMappingId(),
    bind: [],
    block_direction_pad: false,
    drag_radius: 150,
    enable_randomization: false,
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
    random_offset_x: default_random_offset,
    random_offset_y: default_random_offset,
    release_mode: "OnRelease",
    script_hooks: defaultScriptHooks(),
    type: "PadCastSpell",
  };
}

export interface CancelCastConfig {
  id: string;
  bind: ButtonBinding;
  note: string;
  position: Position;
  script_hooks: MappingScriptHooks;
  type: "CancelCast";
}

export function newCancelCast(position: Position): CancelCastConfig {
  return {
    id: newMappingId(),
    bind: [],
    note: "",
    position,
    script_hooks: defaultScriptHooks(),
    type: "CancelCast",
  };
}

export interface ObservationConfig {
  id: string;
  bind: ButtonBinding;
  max_radius: number;
  note: string;
  pointer_id: number;
  position: Position;
  random_offset_x: number;
  random_offset_y: number;
  sensitivity_x: number;
  sensitivity_y: number;
  script_hooks: MappingScriptHooks;
  type: "Observation";
}

export function newObservation(position: Position): ObservationConfig {
  return {
    id: newMappingId(),
    bind: [],
    max_radius: 0,
    note: "",
    pointer_id: 4,
    position,
    random_offset_x: default_random_offset,
    random_offset_y: default_random_offset,
    sensitivity_x: 0.8,
    sensitivity_y: 0.8,
    script_hooks: defaultScriptHooks(),
    type: "Observation",
  };
}

export interface FpsConfig {
  id: string;
  bind: ButtonBinding;
  note: string;
  pointer_id: number;
  position: Position;
  sensitivity_x: number;
  sensitivity_y: number;
  max_offset_x: number;
  max_offset_y: number;
  touch_mode: FpsTouchMode;
  type: "Fps";
}

export type FpsTouchMode =
  | { type: "none" }
  | { type: "clean"; another_pointer_id: number }
  | { type: "delayed"; interval: number; another_pointer_id: number }
  | { type: "overlap"; another_pointer_id: number };

export function newFps(position: Position): FpsConfig {
  return {
    id: newMappingId(),
    bind: [],
    note: "",
    pointer_id: 0,
    position,
    sensitivity_x: 0.8,
    sensitivity_y: 0.8,
    max_offset_x: -1,
    max_offset_y: -1,
    touch_mode: { type: "none" },
    type: "Fps",
  };
}

export interface FireConfig {
  id: string;
  bind: ButtonBinding;
  note: string;
  pointer_id: number;
  position: Position;
  random_offset_x: number;
  random_offset_y: number;
  sensitivity_x: number;
  sensitivity_y: number;
  script_hooks: MappingScriptHooks;
  type: "Fire";
}

export function newFire(position: Position): FireConfig {
  return {
    id: newMappingId(),
    bind: [],
    note: "",
    pointer_id: 0,
    position,
    random_offset_x: default_random_offset,
    random_offset_y: default_random_offset,
    sensitivity_x: 0.8,
    sensitivity_y: 0.8,
    script_hooks: defaultScriptHooks(),
    type: "Fire",
  };
}

export interface RawInputConfig {
  id: string;
  bind: ButtonBinding;
  note: string;
  position: Position;
  type: "RawInput";
}

export function newRawInput(position: Position): RawInputConfig {
  return {
    id: newMappingId(),
    bind: [],
    note: "",
    position,
    type: "RawInput",
  };
}

export interface ScriptConfig {
  id: string;
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
    id: newMappingId(),
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

function withDefaultRandomOffset(value?: number): number {
  return value ?? default_random_offset;
}

function withDefaultJitterOffset(value?: number): number {
  return value ?? default_jitter_offset;
}

function withDefaultScriptHooks(
  value?: Partial<MappingScriptHooks>
): MappingScriptHooks {
  return {
    before_script: value?.before_script ?? "",
    after_script: value?.after_script ?? "",
  };
}

function normalizeFpsTouchMode(value?: Partial<FpsTouchMode>): FpsTouchMode {
  switch (value?.type) {
    case "clean":
      return {
        type: "clean",
        another_pointer_id: value.another_pointer_id ?? 1,
      };
    case "delayed":
      return {
        type: "delayed",
        interval: value.interval ?? 16,
        another_pointer_id: value.another_pointer_id ?? 1,
      };
    case "overlap":
      return {
        type: "overlap",
        another_pointer_id: value.another_pointer_id ?? 1,
      };
    default:
      return { type: "none" };
  }
}

export function normalizeMappingConfig(config: MappingConfig): MappingConfig {
  const usedIds = new Set<string>();
  return {
    ...config,
    mappings: config.mappings.map((mapping) => {
      const currentId = (mapping as { id?: string }).id;
      const id = currentId && !usedIds.has(currentId) ? currentId : newMappingId();
      usedIds.add(id);
      switch (mapping.type) {
        case "SingleTap":
          return {
            ...mapping,
            id,
            random_offset_x: withDefaultRandomOffset(mapping.random_offset_x),
            random_offset_y: withDefaultRandomOffset(mapping.random_offset_y),
            script_hooks: withDefaultScriptHooks(mapping.script_hooks),
          };
        case "RepeatTap":
          return {
            ...mapping,
            id,
            random_offset_x: withDefaultRandomOffset(mapping.random_offset_x),
            random_offset_y: withDefaultRandomOffset(mapping.random_offset_y),
            script_hooks: withDefaultScriptHooks(mapping.script_hooks),
          };
        case "MultipleTap":
          return {
            ...mapping,
            id,
            random_offset_x: withDefaultRandomOffset(mapping.random_offset_x),
            random_offset_y: withDefaultRandomOffset(mapping.random_offset_y),
            script_hooks: withDefaultScriptHooks(mapping.script_hooks),
          };
        case "Swipe":
          return {
            ...mapping,
            id,
            script_hooks: withDefaultScriptHooks(mapping.script_hooks),
          };
        case "Fire":
          return {
            ...mapping,
            id,
            random_offset_x: withDefaultRandomOffset(mapping.random_offset_x),
            random_offset_y: withDefaultRandomOffset(mapping.random_offset_y),
            script_hooks: withDefaultScriptHooks(mapping.script_hooks),
          };
        case "Observation":
          return {
            ...mapping,
            id,
            random_offset_x: withDefaultRandomOffset(mapping.random_offset_x),
            random_offset_y: withDefaultRandomOffset(mapping.random_offset_y),
            script_hooks: withDefaultScriptHooks(mapping.script_hooks),
          };
        case "MouseCastSpell":
          return {
            ...mapping,
            id,
            initial_duration: mapping.initial_duration ?? 0,
            random_offset_x: withDefaultRandomOffset(mapping.random_offset_x),
            random_offset_y: withDefaultRandomOffset(mapping.random_offset_y),
            script_hooks: withDefaultScriptHooks(mapping.script_hooks),
          };
        case "DirectionPad":
          return {
            ...mapping,
            id,
            enable_randomization: mapping.enable_randomization ?? false,
            random_offset_x: withDefaultRandomOffset(mapping.random_offset_x),
            random_offset_y: withDefaultRandomOffset(mapping.random_offset_y),
            random_distance_min_scale:
              mapping.random_distance_min_scale ??
              default_random_distance_min_scale,
            random_distance_max_scale:
              mapping.random_distance_max_scale ??
              default_random_distance_max_scale,
            jitter_offset_x: withDefaultJitterOffset(mapping.jitter_offset_x),
            jitter_offset_y: withDefaultJitterOffset(mapping.jitter_offset_y),
            up_boost_key: mapping.up_boost_key ?? null,
            up_boost_scale: mapping.up_boost_scale ?? 1.0,
            script_hooks: withDefaultScriptHooks(mapping.script_hooks),
          };
        case "PadCastSpell":
          return {
            ...mapping,
            id,
            enable_randomization: mapping.enable_randomization ?? false,
            random_offset_x: withDefaultRandomOffset(mapping.random_offset_x),
            random_offset_y: withDefaultRandomOffset(mapping.random_offset_y),
            script_hooks: withDefaultScriptHooks(mapping.script_hooks),
          };
        case "CancelCast":
          return {
            ...mapping,
            id,
            script_hooks: withDefaultScriptHooks(mapping.script_hooks),
          };
        case "Fps":
          {
            const normalized = { ...mapping } as FpsConfig & {
              script_hooks?: unknown;
            };
            delete normalized.script_hooks;
            return {
              ...normalized,
              id,
              max_offset_x: normalized.max_offset_x ?? -1,
              max_offset_y: normalized.max_offset_y ?? -1,
              touch_mode: normalizeFpsTouchMode(normalized.touch_mode),
            };
          }
        case "RawInput":
          {
            const normalized = { ...mapping } as RawInputConfig & {
              script_hooks?: unknown;
            };
            delete normalized.script_hooks;
            return {
              ...normalized,
              id,
            };
          }
        default:
          return {
            ...mapping,
            id,
          };
      }
    }),
  };
}

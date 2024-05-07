interface Key {
  type:
    | "SteeringWheel"
    | "DirectionalSkill"
    | "DirectionlessSkill"
    | "CancelSkill"
    | "Tap"
    | "TriggerWhenPressedSkill"
    | "TriggerWhenDoublePressedSkill"
    | "Observation"
    | "Macro";
  note: string;
  posX: number;
  posY: number;
  pointerId: number;
}

interface KeySteeringWheel extends Key {
  key: {
    left: string;
    right: string;
    up: string;
    down: string;
  };
  offset: number;
}

interface KeyDirectionalSkill extends Key {
  key: string;
  range: number;
}

interface KeyDirectionlessSkill extends Key {
  key: string;
}

interface KeyCancelSkill extends Key {
  key: string;
}

interface KeyTriggerWhenPressedSkill extends Key {
  key: string;
  directional: boolean;
  rangeOrTime: number;
}

interface KeyTriggerWhenDoublePressedSkill extends Key {
  key: string;
  range: number;
}

interface KeyObservation extends Key {
  key: string;
  scale: number;
}

interface KeyTap extends Key {
  key: string;
  time: number;
}

type KeyMacroType = "touch" | "sleep" | "swipe" | "input-text";
type KeyMacroArgs = any[];

type KeyMacroList = Array<{
  type: KeyMacroType;
  args: KeyMacroArgs;
}> | null;
interface KeyMacro extends Key {
  key: string;
  macro: {
    down: KeyMacroList;
    loop: KeyMacroList;
    up: KeyMacroList;
  };
}

type KeyMapping =
  | KeySteeringWheel
  | KeyDirectionalSkill
  | KeyDirectionlessSkill
  | KeyTriggerWhenPressedSkill
  | KeyTriggerWhenDoublePressedSkill
  | KeyObservation
  | KeyMacro
  | KeyCancelSkill
  | KeyTap;

interface KeyMappingConfig {
  relativeSize: { w: number; h: number };
  title: string;
  list: KeyMapping[];
}

export type {
  KeyMacroList,
  KeySteeringWheel,
  KeyDirectionalSkill,
  KeyDirectionlessSkill,
  KeyCancelSkill,
  KeyTap,
  KeyTriggerWhenPressedSkill,
  KeyTriggerWhenDoublePressedSkill,
  KeyObservation,
  KeyMacro,
  KeyMapping,
  KeyMappingConfig,
};

interface KeyBase {
  note: string;
  // pos relative to the mask
  posX: number;
  posY: number;
}

export interface KeySteeringWheel extends KeyBase {
  type: "SteeringWheel";
  pointerId: number;
  key: {
    left: string;
    right: string;
    up: string;
    down: string;
  };
  offset: number;
}

export interface KeyDirectionalSkill extends KeyBase {
  type: "DirectionalSkill";
  pointerId: number;
  key: string;
  range: number;
}

export interface KeyDirectionlessSkill extends KeyBase {
  type: "DirectionlessSkill";
  pointerId: number;
  key: string;
}

export interface KeyCancelSkill extends KeyBase {
  type: "CancelSkill";
  pointerId: number;
  key: string;
}

export interface KeyTriggerWhenPressedSkill extends KeyBase {
  type: "TriggerWhenPressedSkill";
  pointerId: number;
  key: string;
  directional: boolean;
  rangeOrTime: number;
}

export interface KeyTriggerWhenDoublePressedSkill extends KeyBase {
  type: "TriggerWhenDoublePressedSkill";
  pointerId: number;
  key: string;
  range: number;
}

export interface KeyObservation extends KeyBase {
  type: "Observation";
  pointerId: number;
  key: string;
  scale: number;
}

export interface KeyTap extends KeyBase {
  type: "Tap";
  pointerId: number;
  key: string;
  time: number;
}

export interface KeySwipe extends KeyBase {
  type: "Swipe";
  pointerId: number;
  key: string;
  pos: { x: number; y: number }[];
  intervalBetweenPos: number;
}

export type KeyMacroList = Array<{
  type: "touch" | "sleep" | "swipe" | "key-input-mode";
  args: any[];
}> | null;

export interface KeyMacro extends KeyBase {
  type: "Macro";
  key: string;
  macro: {
    down: KeyMacroList;
    loop: KeyMacroList;
    up: KeyMacroList;
  };
}

export interface KeySight extends KeyBase {
  type: "Sight";
  key: string;
  pointerId: number;
  scaleX: number;
  scaleY: number;
}

export interface KeyFire extends KeyBase {
  type: "Fire";
  drag: boolean;
  pointerId: number;
  scaleX: number;
  scaleY: number;
}

export type KeyMapping =
  | KeySteeringWheel
  | KeyDirectionalSkill
  | KeyDirectionlessSkill
  | KeyTriggerWhenPressedSkill
  | KeyTriggerWhenDoublePressedSkill
  | KeyObservation
  | KeyMacro
  | KeyCancelSkill
  | KeyTap
  | KeySwipe
  | KeySight
  | KeyFire;

export type KeyCommon = KeyMacro | KeyCancelSkill | KeyTap;

export type KeySkill =
  | KeyDirectionalSkill
  | KeyDirectionlessSkill
  | KeyTriggerWhenPressedSkill
  | KeyTriggerWhenDoublePressedSkill;

export interface KeyMappingConfig {
  relativeSize: { w: number; h: number };
  title: string;
  list: KeyMapping[];
}

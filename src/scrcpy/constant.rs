use bitflags::bitflags;
use serde::Deserialize;

bitflags! {
    #[derive(Debug, Clone, Deserialize)]
    pub struct MetaState: u32 {
        const NONE               = 0x000000;
        const SHIFT_ON           = 0x000001;
        const ALT_ON             = 0x000002;
        const SYM_ON             = 0x000004;
        const FUNCTION_ON        = 0x000008;

        const ALT_LEFT_ON        = 0x000010;
        const ALT_RIGHT_ON       = 0x000020;

        const SHIFT_LEFT_ON      = 0x000040;
        const SHIFT_RIGHT_ON     = 0x000080;

        const CTRL_ON            = 0x001000;
        const CTRL_LEFT_ON       = 0x002000;
        const CTRL_RIGHT_ON      = 0x004000;

        const META_ON            = 0x010000;
        const META_LEFT_ON       = 0x020000;
        const META_RIGHT_ON      = 0x040000;

        const CAPS_LOCK_ON       = 0x100000;
        const NUM_LOCK_ON        = 0x200000;
        const SCROLL_LOCK_ON     = 0x400000;
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Deserialize)]
pub enum KeyEventAction {
    /** The key has been pressed down. */
    Down = 0,

    /** The key has been released. */
    Up = 1,

    /**
     * Multiple duplicate key events have occurred in a row, or a
     * complex string is being delivered.
     */
    Multiple = 2,
}

#[repr(u32)]
#[derive(Debug, Clone, Deserialize)]
pub enum Keycode {
    Unknown = 0,
    SoftLeft = 1,
    SoftRight = 2,
    Home = 3,
    Back = 4,
    Call = 5,
    Endcall = 6,
    Keycode0 = 7,
    Keycode1 = 8,
    Keycode2 = 9,
    Keycode3 = 10,
    Keycode4 = 11,
    Keycode5 = 12,
    Keycode6 = 13,
    Keycode7 = 14,
    Keycode8 = 15,
    Keycode9 = 16,
    Star = 17,
    Pound = 18,
    DpadUp = 19,
    DpadDown = 20,
    DpadLeft = 21,
    DpadRight = 22,
    DpadCenter = 23,
    VolumeUp = 24,
    VolumeDown = 25,
    Power = 26,
    Camera = 27,
    Clear = 28,
    A = 29,
    B = 30,
    C = 31,
    D = 32,
    E = 33,
    F = 34,
    G = 35,
    H = 36,
    I = 37,
    J = 38,
    K = 39,
    L = 40,
    M = 41,
    N = 42,
    O = 43,
    P = 44,
    Q = 45,
    R = 46,
    S = 47,
    T = 48,
    U = 49,
    V = 50,
    W = 51,
    X = 52,
    Y = 53,
    Z = 54,
    Comma = 55,
    Period = 56,
    AltLeft = 57,
    AltRight = 58,
    ShiftLeft = 59,
    ShiftRight = 60,
    Tab = 61,
    Space = 62,
    Sym = 63,
    Explorer = 64,
    Envelope = 65,
    Enter = 66,
    Del = 67,
    Grave = 68,
    Minus = 69,
    Equals = 70,
    LeftBracket = 71,
    RightBracket = 72,
    Backslash = 73,
    Semicolon = 74,
    Apostrophe = 75,
    Slash = 76,
    At = 77,
    Num = 78,
    Headsethook = 79,
    Focus = 80,
    Plus = 81,
    Menu = 82,
    Notification = 83,
    Search = 84,
    MediaPlayPause = 85,
    MediaStop = 86,
    MediaNext = 87,
    MediaPrevious = 88,
    MediaRewind = 89,
    MediaFastForward = 90,
    Mute = 91,
    PageUp = 92,
    PageDown = 93,
    Pictsymbols = 94,
    SwitchCharset = 95,
    ButtonA = 96,
    ButtonB = 97,
    ButtonC = 98,
    ButtonX = 99,
    ButtonY = 100,
    ButtonZ = 101,
    ButtonL1 = 102,
    ButtonR1 = 103,
    ButtonL2 = 104,
    ButtonR2 = 105,
    ButtonThumbl = 106,
    ButtonThumbr = 107,
    ButtonStart = 108,
    ButtonSelect = 109,
    ButtonMode = 110,
    Escape = 111,
    ForwardDel = 112,
    CtrlLeft = 113,
    CtrlRight = 114,
    CapsLock = 115,
    ScrollLock = 116,
    MetaLeft = 117,
    MetaRight = 118,
    Function = 119,
    Sysrq = 120,
    Break = 121,
    MoveHome = 122,
    MoveEnd = 123,
    Insert = 124,
    Forward = 125,
    MediaPlay = 126,
    MediaPause = 127,
    MediaClose = 128,
    MediaEject = 129,
    MediaRecord = 130,
    F1 = 131,
    F2 = 132,
    F3 = 133,
    F4 = 134,
    F5 = 135,
    F6 = 136,
    F7 = 137,
    F8 = 138,
    F9 = 139,
    F10 = 140,
    F11 = 141,
    F12 = 142,
    NumLock = 143,
    Numpad0 = 144,
    Numpad1 = 145,
    Numpad2 = 146,
    Numpad3 = 147,
    Numpad4 = 148,
    Numpad5 = 149,
    Numpad6 = 150,
    Numpad7 = 151,
    Numpad8 = 152,
    Numpad9 = 153,
    NumpadDivide = 154,
    NumpadMultiply = 155,
    NumpadSubtract = 156,
    NumpadAdd = 157,
    NumpadDot = 158,
    NumpadComma = 159,
    NumpadEnter = 160,
    NumpadEquals = 161,
    NumpadLeftParen = 162,
    NumpadRightParen = 163,
    VolumeMute = 164,
    Info = 165,
    ChannelUp = 166,
    ChannelDown = 167,
    ZoomIn = 168,
    ZoomOut = 169,
    Tv = 170,
    Window = 171,
    Guide = 172,
    Dvr = 173,
    Bookmark = 174,
    Captions = 175,
    Settings = 176,
    TvPower = 177,
    TvInput = 178,
    StbPower = 179,
    StbInput = 180,
    AvrPower = 181,
    AvrInput = 182,
    ProgRed = 183,
    ProgGreen = 184,
    ProgYellow = 185,
    ProgBlue = 186,
    AppSwitch = 187,
    Button1 = 188,
    Button2 = 189,
    Button3 = 190,
    Button4 = 191,
    Button5 = 192,
    Button6 = 193,
    Button7 = 194,
    Button8 = 195,
    Button9 = 196,
    Button10 = 197,
    Button11 = 198,
    Button12 = 199,
    Button13 = 200,
    Button14 = 201,
    Button15 = 202,
    Button16 = 203,
    LanguageSwitch = 204,
    MannerMode = 205,
    Keycode3dMode = 206,
    Contacts = 207,
    Calendar = 208,
    Music = 209,
    Calculator = 210,
    ZenkakuHankaku = 211,
    Eisu = 212,
    Muhenkan = 213,
    Henkan = 214,
    KatakanaHiragana = 215,
    Yen = 216,
    Ro = 217,
    Kana = 218,
    Assist = 219,
    BrightnessDown = 220,
    BrightnessUp = 221,
    MediaAudioTrack = 222,
    Sleep = 223,
    Wakeup = 224,
    Pairing = 225,
    MediaTopMenu = 226,
    Keycode11 = 227,
    Keycode12 = 228,
    LastChannel = 229,
    TvDataService = 230,
    VoiceAssist = 231,
    TvRadioService = 232,
    TvTeletext = 233,
    TvNumberEntry = 234,
    TvTerrestrialAnalog = 235,
    TvTerrestrialDigital = 236,
    TvSatellite = 237,
    TvSatelliteBs = 238,
    TvSatelliteCs = 239,
    TvSatelliteService = 240,
    TvNetwork = 241,
    TvAntennaCable = 242,
    TvInputHdmi1 = 243,
    TvInputHdmi2 = 244,
    TvInputHdmi3 = 245,
    TvInputHdmi4 = 246,
    TvInputComposite1 = 247,
    TvInputComposite2 = 248,
    TvInputComponent1 = 249,
    TvInputComponent2 = 250,
    TvInputVga1 = 251,
    TvAudioDescription = 252,
    TvAudioDescriptionMixUp = 253,
    TvAudioDescriptionMixDown = 254,
    TvZoomMode = 255,
    TvContentsMenu = 256,
    TvMediaContextMenu = 257,
    TvTimerProgramming = 258,
    Help = 259,
    NavigatePrevious = 260,
    NavigateNext = 261,
    NavigateIn = 262,
    NavigateOut = 263,
    StemPrimary = 264,
    Stem1 = 265,
    Stem2 = 266,
    Stem3 = 267,
    DpadUpLeft = 268,
    DpadDownLeft = 269,
    DpadUpRight = 270,
    DpadDownRight = 271,
    MediaSkipForward = 272,
    MediaSkipBackward = 273,
    MediaStepForward = 274,
    MediaStepBackward = 275,
    SoftSleep = 276,
    Cut = 277,
    Copy = 278,
    Paste = 279,
    SystemNavigationUp = 280,
    SystemNavigationDown = 281,
    SystemNavigationLeft = 282,
    SystemNavigationRight = 283,
    AllApps = 284,
    Refresh = 285,
    ThumbsUp = 286,
    ThumbsDown = 287,
    ProfileSwitch = 288,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum MotionEventAction {
    Down = 0,
    Up = 1,
    Move = 2,
    // ...
}

bitflags! {
    #[derive(Debug, Clone, Deserialize)]
    pub struct MotionEventButtons: u32 {
        /// Primary button (e.g. left mouse)
        const PRIMARY = 1 << 0;
        /// Secondary button (e.g. right mouse)
        const SECONDARY = 1 << 1;
        /// Tertiary button (e.g. middle mouse)
        const TERTIARY = 1 << 2;
        /// Back button (mouse back)
        const BACK = 1 << 3;
        /// Forward button (mouse forward)
        const FORWARD = 1 << 4;
        /// Stylus primary button
        const STYLUS_PRIMARY = 1 << 5;
        /// Stylus secondary button
        const STYLUS_SECONDARY = 1 << 6;
    }
}

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum CopyKey {
    None = 0,
    Copy = 1,
    Cut = 2,
}

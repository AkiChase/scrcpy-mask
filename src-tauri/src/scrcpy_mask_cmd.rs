pub async fn exec_pfc(cmd_type: ScrcpyMaskCmdType) {
    match cmd_type {
        _ => {}
    }
    println!("收到纯前端命令:{:?}", cmd_type);
}

#[derive(Debug)]
pub enum ScrcpyMaskCmdType {
    PasteText,
}

impl ScrcpyMaskCmdType {
    pub fn from_i64(value: i64) -> Option<Self> {
        match value {
            15 => Some(Self::PasteText),
            _ => None,
        }
    }
}

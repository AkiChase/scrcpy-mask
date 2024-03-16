use std::path::PathBuf;

use scrcpy_mask::client::ScrcpyClient;

#[test]
fn test() {
    let dir = PathBuf::from("C:/MyProject/github/scrcpy-mask/src-tauri/resource");
    let id = "emulator-5554";
    match ScrcpyClient::get_screen_size(&dir, id) {
        Ok(size) => println!("{:?}", size),
        Err(e) => println!("{}", e),
    };
}

# Scrcpy-mask

A Scrcpy client in Rust & Tarui aimed at providing mouse and key mapping to control Android device.

Due to the delay and blurred image quality of the mirror screen. This project found another way, directly abandoned the mirror screen, and instead used a transparent mask to display the screen content behind the window, which fundamentally put an end to the delay in casting the screen.

## Features

- [x] Wired and wireless connections to Android devices
- [x] Start scrcpy-server and connect to it
- [x] Implement scrcpy client control protocol
- [x] Mouse and keyboard key mapping
- [x] Visually setting the mapping
- [x] Key mapping config import and export
- [x] Update check
- [ ] Switch between key mapping and raw input
- [ ] Better macro support
- [ ] Provide external interface through websocket

## Screenshot

![Device](https://pic.superbed.cc/item/6637190cf989f2fb975b6162.png)
![Key mapping](https://pic.superbed.cc/item/66371911f989f2fb975b62a3.png)
![Mask](https://pic.superbed.cc/item/66371a03f989f2fb975c07f3.png)

## Contribution.

If you are interested in this project, you are welcome to submit pull request or issue. But my time and energy is limited, so I may not be able to deal with it all.

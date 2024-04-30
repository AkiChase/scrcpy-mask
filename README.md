# Scrcpy-mask

A Scrcpy (control) client in Rust & Tarui aimed at providing mouse and key mapping.

Due to the delay and blurred image quality of the mirror screen. This project found another way, directly abandoned the mirror screen, and instead used a transparent mask to display the screen content behind the window, which fundamentally put an end to the delay in casting the screen.

## Features

- [ ] Wired and wireless connections to Android devices
- [x] Start scrcpy-server and connect to it
- [x] Implement scrcpy client control protocol
- [x] Mouse and keyboard mapping
- [x] Visually setting the mapping
- [ ] Provide external interface through websocket

## contribution.

If you are interested in this project, you are welcome to submit pull request or issue. But my time and energy is limited, so I may not be able to deal with it all.

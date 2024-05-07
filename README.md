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
- [x] Switch between key mapping and input-text box
- [ ] Internationalization (i18n)
- [ ] Gamepad key mapping
- [ ] Better macro support
- [ ] Provide external interface through websocket
- [ ] Help document

## Demonstration video

- [M 系列 Mac 电脑玩王者，暃排位实录，使用 Android Stuido 模拟器和开源 Scrcpy Mask 按键映射工具-哔哩哔哩](https://b23.tv/q6iDW1w)
- [自制跨平台开源项目 Scrcpy Mask ，像模拟器一样用键鼠控制任意安卓设备！以 M 系列芯片 MacBook 打王者为例-哔哩哔哩](https://b23.tv/gqmriXr)
- [如何用 PC 控制安卓手机打王者？只要思想不滑坡，办法总比困难多！-哔哩哔哩](https://b23.tv/dmUOpff)
- [M 芯片 Mac 怎么用 Android Studio 模拟器打王者？这是 Up 耗时数个月给出的答案-哔哩哔哩](https://b23.tv/ckJgyK5)

## Screenshot

- Device control

![](https://pic.superbed.cc/item/6637190cf989f2fb975b6162.png)

- Key mapping setting

![](https://pic.superbed.cc/item/66371911f989f2fb975b62a3.png)

- Mask above game

![](https://pic.superbed.cc/item/66373c8cf989f2fb97679dfd.png)

## Basic using

1. Install software suitable for your system platform from [releases](https://github.com/AkiChase/scrcpy-mask/releases)
2. Identify your Android device type
	1. For physical devices like phones or tablets
		1. You need to solve the problem of screen casting on your own. Recommend using the official screen mirror method of your device brand to achieve the minimum delay
		2. Enable ADB debugging on your device via USB or wirelessly, then connect it to your computer.
	2. For emulator, you don't need screen mirror, and emulator generally default to enabling ADB wired debugging. So this is the best way for game, I think.
3. Launch the software and navigate to the Device page.
	1. Find your device among the available devices (if not found, please search for how to enable ADB debugging for your device).
	2. Right-click on your device and choose "Get Screen Size". Use the obtained screen size as a reference and enter the device's width and height correctly. Note: If the width or height is incorrect (for example, they are reversed in portrait and landscape modes), all touch operations will be ignored, but no error message will appear.
	3. Right-click on your device again and choose "Control this device".
4. Navigate to the Settings page -> Mask Settings, and set the width and height of the mask to a certain multiple of the device's size to ensure the mask size is appropriate.
5. Navigate to the Mask page where you can see a transparent mask. Next, adjust and move your emulator window or screen mirroring window to align the displayed content area with the transparent mask area.
6. Navigate to the Key mapping page and switch or edit the key mapping configs.
7. Return to the Mask page and start enjoying.

## Contribution.

If you are interested in this project, you are welcome to submit pull request or issue. But my time and energy is limited, so I may not be able to deal with it all.

[![Star History Chart](https://api.star-history.com/svg?repos=AkiChase/scrcpy-mask&type=Date)](https://star-history.com/#AkiChase/scrcpy-mask&Date)
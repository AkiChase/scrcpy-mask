# Scrcpy Mask

[中文介绍](./README-zh.md)

To achieve computer control of Android devices, I developed a cross-platform desktop client using Tarui + Vue 3 + Rust. This client provides visual mouse and keyboard mapping configuration, enabling multi-touch operations similar to Android emulators through key mapping, with millisecond-level response time. This tool can be widely used for controlling Android devices from computers to play mobile games, providing a smooth touch experience.

I express my deep respect and gratitude to the developers of the Scrcpy project. Scrcpy is a powerful and efficient open-source tool that greatly facilitates control over Android devices. This project is built upon the excellent architecture of Scrcpy, with optimizations and adjustments for mouse and keyboard control.

**This project does not provide Scrcpy's screen mirroring feature! It only implements Scrcpy's control protocol.**

Because screen mirroring may involve latency and blurriness issues, this project takes a different approach by directly abandoning screen mirroring and instead using a transparent mask to display the content behind the window (which can be AVD, low-latency screen mirroring provided by your phone manufacturers, etc.), Completely eliminates the problem of poor screen casting experience inherent in Scrcpy.

**If you really need screen mirroring and don't mind the latency and performance issues,** you can use the Android app [ScreenStream](https://github.com/dkrivoruchko/ScreenStream) for LAN screen mirroring. Scrcpy MAsk has been adapted to work with ScreenStream since version `v0.5.0`. While its performance may leave something to be desired, it is ready to use out of the box.

Furthermore, to better support interaction between Scrcpy Mask and Android devices, I have made some modifications to the scrcpy-server, leading to the creation of a separate branch project called [scrcpy-mask-server](https://github.com/AkiChase/scrcpy-mask-server).

## Features

- [x] Wired and wireless connections to Android devices
- [x] Start scrcpy-server and connect to it
- [x] Implement scrcpy client control protocol
- [x] Mouse and keyboard key mapping
- [x] Visually setting the mapping
- [x] Key mapping config import and export
- [x] Update check
- [x] Toggle between key mapping and key input
- [x] Internationalization (i18n)
- [x] ScreenStream screen mirror
- [ ] Gamepad key mapping
- [ ] Better macro support
- [x] Provide external control through websocket, see [external control](https://github.com/AkiChase/scrcpy-mask-external-control)
- [ ] Help document

## Demonstration video

- [别再说你不会用电脑控制手机玩手游了，Scrcpy Mask 纯小白教程+常见问题解答](https://www.bilibili.com/video/BV1Sm42157md/?share_source=copy_web&vd_source=36923115230d8a46ae8b587fc5348e6e)
- [DNF 手游触屏操作反人类？但又不能在模拟器上玩 DNF 手游？不好意思，Scrcpy Mask “模拟器”的机制遥遥领先](https://www.bilibili.com/video/BV17U411Z7cN/?share_source=copy_web&vd_source=36923115230d8a46ae8b587fc5348e6e)
- [如何用电脑玩 FPS 手游？这样的“安卓模拟器”，也不是不可以-哔哩哔哩](https://www.bilibili.com/video/BV1EU411Z7TC/?share_source=copy_web&vd_source=36923115230d8a46ae8b587fc5348e6e)
- [M 系列 Mac 电脑玩王者，暃排位实录，使用 Android Stuido 模拟器和开源 Scrcpy Mask 按键映射工具-哔哩哔哩](https://b23.tv/q6iDW1w)
- [自制跨平台开源项目 Scrcpy Mask ，像模拟器一样用键鼠控制任意安卓设备！以 M 系列芯片 MacBook 打王者为例-哔哩哔哩](https://b23.tv/gqmriXr)
- [如何用 PC 控制安卓手机打王者？只要思想不滑坡，办法总比困难多！-哔哩哔哩](https://b23.tv/dmUOpff)
- [M 芯片 Mac 怎么用 Android Studio 模拟器打王者？这是 Up 耗时数个月给出的答案-哔哩哔哩](https://b23.tv/ckJgyK5)

## Implementation principle

- [Scrcpy Mask 实现原理剖析，如何像模拟器一样用键鼠控制你的安卓设备？架构、通信篇 - 掘金](https://juejin.cn/post/7366799820734939199)
- [Scrcpy Mask 实现原理剖析，如何像模拟器一样用键鼠控制你的安卓设备？前端可视化、按键映射篇 - 掘金](https://juejin.cn/post/7367620233140748299)
- [Scrcpy Mask 实现原理剖析，如何在前端实现王者荣耀中技能的准确释放？ - 掘金](https://juejin.cn/post/7367568884198047807)

## Screenshot

- Device control

![](https://pic.superbed.cc/item/6637190cf989f2fb975b6162.png)

- Key mapping setting

![](https://pic.superbed.cc/item/66371911f989f2fb975b62a3.png)

- Mask above game

![](https://pic.superbed.cc/item/66373c8cf989f2fb97679dfd.png)

![](https://pic.superbed.cc/item/6649cf0cfcada11d37c05b5e.jpg)

## Basic using

1. Install software suitable for your system platform from [releases](https://github.com/AkiChase/scrcpy-mask/releases)
2. Identify your Android device type
   1. For physical devices like phones or tablets
      1. You need to solve the problem of screen casting on your own. Recommend using the official screen mirror method of your device brand to achieve the minimum delay. Since `v0.5.0` version, it can be used with [ScreenStream](https://github.com/dkrivoruchko/ScreenStream) to cast screen under the same LAN.
      2. Enable ADB debugging on your device via USB or wirelessly, then connect it to your computer.
   2. For emulator, you don't need screen mirror, and emulator generally default to enabling ADB wired debugging. So this is the best way for game, I think.
3. Launch the software and navigate to the Device page.
   1. Find your device among the available devices (if not found, please search for how to enable ADB debugging for your device).
   2. Right-click on your device again and choose "Control this device".
4. Navigate to the Settings page -> Mask Settings, set the width and height of the mask to the same ratio of the device screen size and ensure that the mask size is appropriate.
5. Navigate to the Mask page where you can see a transparent mask. Next, adjust and move your emulator window or screen mirroring window to align the displayed content area with the transparent mask area.
6. Navigate to the Key mapping page and switch or edit the key mapping configs.
7. Return to the Mask page and start enjoying.

## About Macros

Currently, the structure of macros is simply a JSON object with limited functionality, serving as a transitional solution. **Please refrain from investing too much time in writing macros, as the specifications for macro creation may change with version updates.**

An example of macros can be found in the `async function execMacro` function in [hotkey.ts](https://github.com/AkiChase/scrcpy-mask/blob/master/src/hotkey.ts) file.

For instance, the `key-input-mode` macro can switch from key mapping mode to key input mode, commonly used for text input. An example is as follows:

```json
[{ "args": [], "type": "key-input-mode" }]
```

## Error Report

When asking a question, please provide as much detailed information as possible regarding the issue, including the operating system and software version. Specifically, if there is an error output, please be sure to include the relevant logs.

There are two sources of logs that might help in identifying and solving the error. Generally, the error output can be found in the Web logs.

1. Web Logs: Open Developer Tools by pressing `Ctrl+Shift+I` or `Cmd+Opt+I`, click on the console tab, and check the information output in the console.
2. Rust Logs:
   1. On macOS or Linux, navigate to the installation directory, use the **terminal** to run `scrcpy-mask`, and you can see the program's output in real-time in the terminal.
   2. On Windows, you need to clone the project and run it yourself to view the Rust output.

## Contribution.

If you are interested in this project, you are welcome to submit pull request or issue. But my time and energy is limited, so I may not be able to deal with it all.

[![Star History Chart](https://api.star-history.com/svg?repos=AkiChase/scrcpy-mask&type=Date)](https://star-history.com/#AkiChase/scrcpy-mask&Date)

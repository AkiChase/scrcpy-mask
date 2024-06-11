# Scrcpy Mask

为了实现电脑控制安卓设备，本人使用 Tarui + Vue 3 + Rust 开发了一款跨平台桌面客户端。该客户端能够提供可视化的鼠标和键盘按键映射配置。通过按键映射实现了实现类似安卓模拟器的多点触控操作，具有毫秒级响应速度。该工具可广泛用于电脑控制安卓设备玩手游等等，提供流畅的触控体验。

本人对 Scrcpy 项目的开发者表示深深的敬意和感谢。Scrcpy 是一个强大而高效的开源工具，极大地方便了对 Android 设备的控制。本项目的实现基于 Scrcpy 的优秀架构，进行了鼠标键盘控制的优化和调整。

**本项目不提供 Scrcpy 的投屏功能！本项目仅实现了 Scrcpy 的控制协议。**

原因是投屏会存在延迟和模糊问题，本项目另辟蹊径，直接放弃投屏，而使用透明的蒙版显示窗口背后的内容（可以使用电脑安卓模拟器 、手机厂商提供的低延迟投屏等），从根本上杜绝了 Scrcpy 的投屏体验差的问题。

**如果您确实需要一个开箱即用的投屏功能，并且不在意延迟和性能问题**，可以使用安卓应用 [ScreenStream](https://github.com/dkrivoruchko/ScreenStream) 在局域网内投屏。本项目已适配 ScreenStream 投屏（自 `v0.5.0` 版本起），虽然它的性能可能不太理想，但开箱即用。

除此之外，为了更好的支持 Scrcpy Mask 与安卓设备交互，本人对 scrcpy-server 进行了一些修改，在此扩展出了一个分支项目 [scrcpy-mask-server](https://github.com/AkiChase/scrcpy-mask-server)

## 特性

- [x] 有线、无线连接安卓设备
- [x] 启动并连接 Scrcpy 服务端
- [x] 实现 Scrcpy 控制协议
- [x] 鼠标和键盘按键映射
- [x] 可视化编辑按键映射配置
- [x] 按键映射配置的导入与导出
- [x] 更新检查
- [x] 在按键映射和按键输入之间切换
- [x] 国际化
- [x] ScreenStream 投屏
- [ ] 手柄按键映射
- [ ] 更好的宏
- [x] 通过 WebSocket 提供外部控制，见[外部控制](https://github.com/AkiChase/scrcpy-mask-external-control)
- [ ] 帮助文档

## 视频演示

- [别再说你不会用电脑控制手机玩手游了，Scrcpy Mask 纯小白教程+常见问题解答](https://www.bilibili.com/video/BV1Sm42157md/?share_source=copy_web&vd_source=36923115230d8a46ae8b587fc5348e6e)
- [DNF 手游触屏操作反人类？但又不能在模拟器上玩 DNF 手游？不好意思，Scrcpy Mask “模拟器”的机制遥遥领先](https://www.bilibili.com/video/BV17U411Z7cN/?share_source=copy_web&vd_source=36923115230d8a46ae8b587fc5348e6e)
- [如何用电脑玩 FPS 手游？这样的“安卓模拟器”，也不是不可以-哔哩哔哩](https://www.bilibili.com/video/BV1EU411Z7TC/?share_source=copy_web&vd_source=36923115230d8a46ae8b587fc5348e6e)
- [M 系列 Mac 电脑玩王者，暃排位实录，使用 Android Stuido 模拟器和开源 Scrcpy Mask 按键映射工具-哔哩哔哩](https://b23.tv/q6iDW1w)
- [自制跨平台开源项目 Scrcpy Mask ，像模拟器一样用键鼠控制任意安卓设备！以 M 系列芯片 MacBook 打王者为例-哔哩哔哩](https://b23.tv/gqmriXr)
- [如何用 PC 控制安卓手机打王者？只要思想不滑坡，办法总比困难多！-哔哩哔哩](https://b23.tv/dmUOpff)
- [M 芯片 Mac 怎么用 Android Studio 模拟器打王者？这是 Up 耗时数个月给出的答案-哔哩哔哩](https://b23.tv/ckJgyK5)

## 实现原理

- [Scrcpy Mask 实现原理剖析，如何像模拟器一样用键鼠控制你的安卓设备？架构、通信篇 - 掘金](https://juejin.cn/post/7366799820734939199)
- [Scrcpy Mask 实现原理剖析，如何像模拟器一样用键鼠控制你的安卓设备？前端可视化、按键映射篇 - 掘金](https://juejin.cn/post/7367620233140748299)
- [Scrcpy Mask 实现原理剖析，如何在前端实现王者荣耀中技能的准确释放？ - 掘金](https://juejin.cn/post/7367568884198047807)

## 截图

- 设备控制

![](https://pic.superbed.cc/item/6637190cf989f2fb975b6162.png)

- 可视化编辑按键映射配置

![](https://pic.superbed.cc/item/66371911f989f2fb975b62a3.png)

- 游戏控制

![](https://pic.superbed.cc/item/66373c8cf989f2fb97679dfd.png)

![](https://pic.superbed.cc/item/6649cf0cfcada11d37c05b5e.jpg)

## 基本使用

1. 从 [releases](https://github.com/AkiChase/scrcpy-mask/releases) 中安装适合你系统平台的软件包
2. 确认你的安卓设备类型
   1. 对于手机或平板电脑等物理设备
      1. 你需要自己解决投屏的问题。推荐使用设备品牌的官方投屏方式，这样一般延迟最小。自 `v0.5.0` 版本起，可以配合[ScreenStream](https://github.com/dkrivoruchko/ScreenStream)在同一局域网下投屏。
      2. 通过 USB 或无线方式在设备上启用 ADB 调试，然后将其连接到电脑。
   2. 对于模拟器，不仅不需要投屏，而且模拟器通常默认已经启用了 ADB 有线调试。所以几乎不用操作就能获得最好的体验。
3. 启动软件并导航到设备页面。
   1. 在可用的设备中查找你的设备(如果未找到，请自行搜索如何为安装设备启用 ADB 调试)。
   2. 右击设备并选择“控制此设备”。
4. 导航到设置页面->蒙版设置，将蒙版的宽度和高度设置为设备屏幕尺寸相同的比例，确保蒙版大小合适。
5. 导航到蒙版页面，你可以在其中看到一个完全透明的蒙版区域。接下来，调整并移动模拟器窗口或投屏窗口，让其内容区域与透明蒙版区域完全对齐。
6. 导航到键映射页面，切换或编辑键映射配置。
7. 返回到蒙版界面，开始使用吧！

## 关于宏

目前宏的结构仅仅是一个 JSON 对象，功能有限，仅仅是作为过渡使用的。请勿投入太多时间来编写宏，因为**宏的编写规范随时可能因版本更新而变动**。

宏的示例可见 [hotkey.ts](https://github.com/AkiChase/scrcpy-mask/blob/master/src/hotkey.ts) 的 `async function execMacro` 函数注释。

比如 `key-input-mode` 宏，可以从按键映射模式切换到按键输入模式，常用于文本输入。示例如下：

```json
[{ "args": [], "type": "key-input-mode" }]
```

## 错误报告

提问时请尽可能全面而清晰地提供问题相关的信息，包括操作系统和软件版本。特别是如果有错误输出，请务必附带相关日志。

日志有两个来源，可能对定位并解决错误有所帮助。一般来说，Web 日志中就可以找到错误输出。

1. Web 日志：通过 `Ctrl+Shift+I` 或 `Cmd+Opt+I` 打开开发者工具，点击控制台 (console)，查看控制台内输出的信息。
2. Rust 日志：
   1. 在 macOS 或 Linux 系统下，可以进入安装位置，使用**终端**运行 `scrcpy-mask`，可在终端中实时看到程序的输出信息。
   2. 在 Windows 系统下，目前只能克隆项目后自行运行，查看 Rust 输出信息。

## 贡献

如果你对这个项目感兴趣，欢迎提 PR 或 Issue。但我的时间和精力有限，所以可能无法全部及时处理。

[![Star History Chart](https://api.star-history.com/svg?repos=AkiChase/scrcpy-mask&type=Date)](https://star-history.com/#AkiChase/scrcpy-mask&Date)

# Scrcpy-mask

为了实现电脑控制安卓设备，本人使用 Tarui + Vue 3 + Rust 开发了一款跨平台桌面客户端。该客户端能够提供可视化的鼠标和键盘按键映射配置。通过按键映射实现了实现类似安卓模拟器的多点触控操作，具有毫秒级响应速度。该工具可广泛用于电脑控制安卓设备玩手游等等，提供流畅的触控体验。

本人对 Scrcpy 项目的开发者表示深深的敬意和感谢。Scrcpy 是一个强大而高效的开源工具，极大地方便了对 Android 设备的控制。本项目的实现基于 Scrcpy 的优秀架构，进行了鼠标键盘控制的优化和调整。

**本项目不提供投屏功能，不提供投屏功能，不提供投屏功能！**本项目仅实现了 Scrcpy 的控制协议。

原因是投屏会存在延迟和模糊问题，本项目另辟蹊径，直接放弃投屏，而使用透明的蒙版显示窗口背后的内容（可以使用 AVD 、手机厂商提供的低延迟投屏等），从根本上杜绝了 Scrcpy 的投屏体验差的问题。

除此之外，为了更好的支持 Scrcpy Mask 与安卓设备交互，本人对 scrcpy-server 进行了一些修改，在此扩展出了一个分支项目 [scrcpy-mask-server](https://github.com/AkiChase/scrcpy-mask-server)

## 特性

- [x] 有线、无线连接安卓设备
- [x] 启动并连接 Scrcpy 服务端
- [x] 实现 Scrcpy 控制协议
- [x] 鼠标和键盘按键映射
- [x] 可视化编辑按键映射配置
- [x] 按键映射配置的导入与导出
- [x] 更新检查
- [x] 在按键映射和插入文本之间切换
- [x] 国际化
- [ ] 手柄按键映射
- [ ] 更好的宏
- [x] 通过 WebSocket 提供外部控制，见[外部控制](https://github.com/AkiChase/scrcpy-mask-external-control)
- [ ] 帮助文档

## 视频演示

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
      1. 你需要自己解决投屏的问题。推荐使用设备品牌的官方投屏方式，这样一般延迟最小。
      2. 通过 USB 或无线方式在设备上启用 ADB 调试，然后将其连接到电脑。
   2. 对于模拟器，不仅不需要投屏，而且模拟器通常默认启用 ADB 有线调试。所以几乎不用操作就能获得最好的体验。
3. 启动软件并导航到设备页面。
   1. 在可用的设备中查找你的设备(如果未找到，请自行搜索如何为安装设备启用 ADB 调试)。
   2. 右击设备并选择“控制此设备”。
4. 导航到设置页面->蒙版设置，将蒙版的宽度和高度设置为设备屏幕尺寸相同的比例，确保蒙版大小合适。
5. 导航到蒙版页面，你可以在其中看到一个完全透明的蒙版区域。接下来，调整并移动模拟器窗口或投屏窗口，让其内容区域与透明蒙版区域完全对齐。
6. 导航到键映射页面，切换或编辑键映射配置。
7. 返回到蒙版界面，开始使用吧！

## 贡献

如果你对这个项目感兴趣，欢迎提 PR 或 Issue。但我的时间和精力有限，所以可能无法全部及时处理。

[![Star History Chart](https://api.star-history.com/svg?repos=AkiChase/scrcpy-mask&type=Date)](https://star-history.com/#AkiChase/scrcpy-mask&Date)

# Scrcpy Mask

[English](./README.md)

**Scrcpy Mask** 是一款基于 **Rust + Bevy + React** 构建的跨平台桌面客户端，用于高效控制安卓设备。
它提供了可视化的鼠标与键盘按键映射配置，实现了类似安卓模拟器的多点触控操作，具备毫秒级响应速度，带来流畅自然的交互体验。该工具可广泛应用于在电脑上操控安卓设备、玩手游等场景。

> 从 **v0.7.0** 开始，项目已完全迁移至 **Bevy 游戏引擎**，带来了更强大、更稳定的功能，包括 **scrcpy 投屏**、**组合按键映射**、以及更灵活的 **内置脚本系统与外部控制机制**。
>
> ⚠️ 旧版基于 **Rust + Tauri + Vue** 的分支已停止维护。

特别感谢 **Scrcpy** 项目的开发者。Scrcpy 是一个功能强大、性能卓越的开源工具，为 Android 设备的远程控制提供了坚实基础。
Scrcpy Mask 基于其优秀架构，针对鼠标与键盘控制进行了进一步的增强与扩展。

为了更好地支持 **Scrcpy Mask** 与安卓设备的交互，对原 **scrcpy-server** 进行了功能扩展与优化，并基于此创建了分支项目 [**scrcpy-mask-server**](https://github.com/AkiChase/scrcpy-mask-server)。

## 特性

- [x] 国际化
- [x] 有线、无线连接安卓设备
- [x] Scrcpy 控制协议（映射）
- [x] Scrcpy 视频协议（投屏）
- [x] 按键映射配置可视化
- [x] 内置脚本，见[脚本语法规则简介](./scripts-help-zh.md)
- [x] 鼠标、键盘按键映射
- [ ] 手柄按键映射（等待赞助支持❤️）
- [ ] 外部程序控制，见[外部控制](https://github.com/AkiChase/scrcpy-mask-external-control)（等待更新）
- [ ] 帮助文档

## 截图

- 设备

![](https://pic1.imgdb.cn/item/68e79a25c5157e1a885fb7e9.png)

- 映射

![](https://pic1.imgdb.cn/item/68e79a27c5157e1a885fb7ec.png)

- 投屏

![](https://pic1.imgdb.cn/item/68e79a27c5157e1a885fb7ed.png)

- 设置

![](https://pic1.imgdb.cn/item/68e79a25c5157e1a885fb7e8.png)

## 使用方法，如何调整蒙版（windows）
1.解压到文件夹，双击打开scrcpy-mask.exe，会自行启动并行打开webui网页，在网页中选择设备进行连接,勾选【视频】选项，那么就会投屏到蒙版中。
![](https://pic1.imgdb.cn/item/692f90a17313ea6c25100920.png)
2.点击【映射】，点击左上角【管理】，点击【新建】，这里调整设置和自己手机一样分辨率的宽高（注意横屏竖屏）
![](https://pic1.imgdb.cn/item/692f921a7313ea6c25100940.png)
之后就会得到和自己手机一样分辨率大小的背景区域，我们这里可以调整一下大小，他会自动进行等比例缩放。

3.然后我们点击【设置】，去调整我们蒙版的大小和在屏幕上的相对位置。我这里使用的微信截图工具，来获取我左上角的在屏幕上的起始坐标。记住这个坐标，然后调整蒙版的【宽度】和背景完全贴合。
![](https://pic1.imgdb.cn/item/692f988c7313ea6c25100a05.jpg)
![](https://pic1.imgdb.cn/item/692f97bd7e390957debc2ced.png)
4.最终如下。这个时候我们可以开始根据自己的游戏调整自己的键位了，这个我还没摸索出来，没有预设弄起来比较麻烦，先写到这里。
![](https://pic1.imgdb.cn/item/692f98007313ea6c25100a01.png)


## 贡献

[build-help](./build-help.md) 简要说明了如何运行和编译项目。

如果你对本项目感兴趣，欢迎提交 PR 或 Issue。
由于个人时间和精力有限，可能无法及时处理所有反馈，敬请谅解。

[![Star History Chart](https://api.star-history.com/svg?repos=AkiChase/scrcpy-mask&type=Date)](https://star-history.com/#AkiChase/scrcpy-mask&Date)

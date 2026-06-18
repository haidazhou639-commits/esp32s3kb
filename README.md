# ESP32-S3 RMK 蓝牙机械键盘固件项目

本项目使用 [RMK (Rust Mechanical Keyboard)](https://github.com/HaoboGu/rmk) 固件库，为 ESP32-S3 提供了具备 **蓝牙/有线双模** 且支持 **Vial 实时在线改键** 软件的可配置键盘固件。

## 🚀 快速上手流程

由于在本地搭建 ESP32-S3 (Xtensa) 的 Rust 编译环境较为复杂，本项目已内置 **GitHub Actions 云编译**。

1. **推送代码**：将本项目文件夹推送至您的 GitHub 个人仓库。
2. **下载固件**：在 GitHub 仓库的 **Actions** 面板中等待编译结束，下载生成的 `esp32s3-keyboard-firmware.zip` 并解压得到 `keyboard.bin`。
3. **网页烧录**：使用 Chrome/Edge 浏览器打开 [ESP 网页端烧录器](https://esp.github.io/esptool-js/)，连接 ESP32-S3，在 `0x0` 地址烧录 `keyboard.bin`。
4. **蓝牙配对**：配对连接 `ESP32S3 Keyboard`。
5. **在线改键**：访问 [Vial Web App (vial.rocks)](https://vial.rocks/) 在线调整按键映射与宏定义。

## 📁 文件结构

- **`keyboard.toml`**：最核心的配置文件。定义了物理键盘矩阵的行/列引脚、键盘图层（Layer）、蓝牙功能开启、断电保存存储配置。
- **`vial.json`**：KLE (Keyboard Layout Editor) 生成的布局文件，向 Vial 软件提供按键位置信息。
- **`src/main.rs`**：通过 `#[rmk_keyboard]` 过程宏自动加载 `keyboard.toml` 并驱动整个键盘运行。
- **`Cargo.toml` & `.cargo/config.toml`**：管理项目依赖及编译时的 Xtensa 目标平台参数。
- **`.github/workflows/build.yml`**：自动触发云编译的工作流。

## 🔧 自定义引脚说明

编辑 `keyboard.toml` 中的 `[matrix]` 区域，修改引脚（如 `"GPIO1"`，`"GPIO2"` 等）：
```toml
[matrix]
input_pins = ["GPIO1", "GPIO2", "GPIO3", "GPIO4"]   # 修改为您的行引脚 (Rows)
output_pins = ["GPIO5", "GPIO6", "GPIO7", "GPIO8"] # 修改为您的列引脚 (Cols)
```
修改后直接推送，GitHub 就会自动为您编译最新的固件。

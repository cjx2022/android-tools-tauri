# 安卓工具集 v1.5.0

一款功能强大的Android设备管理工具，基于 Rust + Tauri 技术栈开发。

## 功能特性

### 📱 设备管理
- 自动检测已连接的安卓设备
- 网络连接设备（支持IP:端口方式）
- 批量导入IP地址连接
- 批量重启、断开设备

### 📦 APK 管理
- 批量安装 APK 到多个设备
- 批量卸载指定应用

### 🖥️ 显示控制（安卓10及以下）
- 隐藏/显示状态栏
- 隐藏/显示导航栏

### 📁 文件管理
- 上传文件到设备
- 从设备下载文件
- 设备文件浏览器

### 🗑️ 批量删除
- 批量删除设备上的文件或文件夹

### 📋 APP 配置备份与恢复
- 备份应用配置数据
- 批量恢复到多台设备

### 🖼️ 屏幕镜像
- 远程查看设备屏幕（基于scrcpy）

### 💻 ADB 命令行
- 执行任意ADB命令
- 终端风格输出

## 系统要求

- Windows 7/8/10/11
- 安卓设备需开启USB调试或网络调试

## 使用方法

### 启动程序
双击 `安卓工具集.exe` 即可启动，无需安装任何依赖。

程序首次运行时会自动在同目录创建 `.tools_cache` 文件夹，释放内置的ADB和scrcpy工具。

### 连接设备

**方式一：USB连接**
- 使用USB线连接设备到电脑
- 设备需开启USB调试模式
- 程序会自动识别设备

**方式二：网络连接**
- 在顶部"网络连接"区域输入设备IP地址
- 点击"连接"按钮
- 支持批量导入IP（每行一个）

### 屏幕镜像
1. 在设备列表中选择一台在线设备
2. 点击"屏幕镜像"按钮
3. 弹出窗口显示设备屏幕

### ADB命令行
1. 在设备列表中选择一台设备
2. 在底部"ADB命令行"模块输入命令
3. 点击"执行"或按回车

**示例命令：**
```bash
shell pm list packages    # 列出已安装应用
shell ls /sdcard/         # 查看sdcard目录
shell ps                  # 查看进程列表
```

## 技术栈

- **后端**: Rust + Tauri
- **前端**: HTML/CSS/JavaScript
- **ADB**: Android Debug Bridge（内嵌）
- **屏幕镜像**: scrcpy（内嵌）

## 编译要求

- Rust 1.70+ : https://rustup.rs/
- Tauri CLI: `cargo install tauri-cli`
- Windows 10/11

## 编译步骤

```bash
cd android-tools
cargo tauri build
```

编译后的文件在 `android-tools/target/release/` 目录

## 文件结构

```
tauri_version/
├── android-tools/
│   ├── Cargo.toml          # Rust项目配置
│   ├── tauri.conf.json     # Tauri配置
│   ├── build.rs            # 构建脚本
│   ├── scrcpy_files/       # 内嵌工具文件
│   └── src/
│       ├── main.rs         # Rust后端代码
│       └── lib.rs          # 库代码
├── src/
│   └── index.html          # 前端界面
├── 发布版本v1.5.0/         # 预编译发布包
└── README.md               # 项目说明
```

## 注意事项

1. 删除操作不可恢复，请谨慎操作
2. 显示控制功能仅适用于安卓10及以下版本
3. 部分功能需要设备有root权限

## 许可证

MIT License

## 作者

BillChen

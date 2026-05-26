@echo off
chcp 65001 >nul
echo ==========================================
echo      安卓工具集 Tauri版 - 构建脚本
echo ==========================================
echo.

cd /d "%~dp0"

echo [1/3] 检查Rust环境...
rustc --version >nul 2>&1
if errorlevel 1 (
    echo 错误: 未找到Rust，请先安装Rust
    echo 下载地址: https://rustup.rs/
    pause
    exit /b 1
)

echo [2/3] 检查Tauri CLI...
cargo tauri --version >nul 2>&1
if errorlevel 1 (
    echo 安装Tauri CLI...
    cargo install tauri-cli
    if errorlevel 1 (
        echo 错误: 安装Tauri CLI失败
        pause
        exit /b 1
    )
)

echo [3/3] 构建应用程序...
cd android-tools
cargo tauri build
if errorlevel 1 (
    echo 错误: 构建失败
    pause
    exit /b 1
)

cd ..

echo.
echo ==========================================
echo      构建完成！
echo ==========================================
echo 输出文件: android-tools\src-tauri\target\release\android-tools.exe
echo.
echo 提示: 将exe文件复制到adb.exe所在目录即可使用
echo.
pause

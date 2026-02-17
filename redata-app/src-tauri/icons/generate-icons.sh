#!/bin/bash

# 使用 sips (macOS 内置工具) 创建简单的占位图标

# 创建一个基础的 1024x1024 PNG（纯色背景）
cat > icon.png.base64 << 'BASE64'
iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==
BASE64

base64 -d icon.png.base64 > temp.png

# 使用 sips 调整大小
sips -z 32 32 temp.png --out 32x32.png
sips -z 128 128 temp.png --out 128x128.png
sips -z 256 256 temp.png --out 128x128@2x.png

# 创建 .icns (macOS)
mkdir icon.iconset
sips -z 16 16 temp.png --out icon.iconset/icon_16x16.png
sips -z 32 32 temp.png --out icon.iconset/icon_16x16@2x.png
sips -z 32 32 temp.png --out icon.iconset/icon_32x32.png
sips -z 64 64 temp.png --out icon.iconset/icon_32x32@2x.png
sips -z 128 128 temp.png --out icon.iconset/icon_128x128.png
sips -z 256 256 temp.png --out icon.iconset/icon_128x128@2x.png
sips -z 256 256 temp.png --out icon.iconset/icon_256x256.png
sips -z 512 512 temp.png --out icon.iconset/icon_256x256@2x.png
sips -z 512 512 temp.png --out icon.iconset/icon_512x512.png
sips -z 1024 1024 temp.png --out icon.iconset/icon_512x512@2x.png

iconutil -c icns icon.iconset -o icon.icns

# 创建 .ico (Windows) - 使用 sips 创建多个尺寸
sips -z 256 256 temp.png --out icon.ico

# 清理
rm -rf icon.iconset temp.png icon.png.base64

echo "✅ 图标文件已生成"

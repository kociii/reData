#!/bin/bash

echo "🔍 检查开发环境..."
echo ""

# 检查 Node.js
if command -v node &> /dev/null; then
    echo "✅ Node.js: $(node --version)"
else
    echo "❌ Node.js 未安装"
    exit 1
fi

# 检查 npm
if command -v npm &> /dev/null; then
    echo "✅ npm: $(npm --version)"
else
    echo "❌ npm 未安装"
    exit 1
fi

# 检查 Python
if command -v python3 &> /dev/null; then
    echo "✅ Python: $(python3 --version)"
else
    echo "❌ Python 3 未安装"
    exit 1
fi

# 检查 uv
if command -v uv &> /dev/null; then
    echo "✅ uv: $(uv --version)"
else
    echo "❌ uv 未安装"
    echo "   安装命令: curl -LsSf https://astral.sh/uv/install.sh | sh"
    exit 1
fi

# 检查 Rust
if command -v rustc &> /dev/null; then
    echo "✅ Rust: $(rustc --version)"
else
    echo "❌ Rust 未安装"
    echo "   安装命令: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# 检查 Cargo
if command -v cargo &> /dev/null; then
    echo "✅ Cargo: $(cargo --version)"
else
    echo "❌ Cargo 未安装"
    exit 1
fi

echo ""
echo "🎉 所有依赖已安装！"
echo ""
echo "📦 检查项目依赖..."

# 检查 node_modules
if [ -d "node_modules" ]; then
    echo "✅ 前端依赖已安装"
else
    echo "⚠️  前端依赖未安装"
    echo "   运行: npm install"
fi

# 检查后端依赖
if [ -d "backend/.venv" ]; then
    echo "✅ 后端依赖已安装"
else
    echo "⚠️  后端依赖未安装"
    echo "   运行: cd backend && uv sync"
fi

echo ""
echo "🚀 准备启动应用..."
echo "   运行: npm run tauri:dev"

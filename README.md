# reData - 智能数据处理平台

<div align="center">

**基于 Tauri 构建的智能数据处理桌面应用**

[![Tauri](https://img.shields.io/badge/Tauri-2.x-blue)](https://tauri.app/)
[![Nuxt](https://img.shields.io/badge/Nuxt-4.x-00DC82)](https://nuxt.com/)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/Version-0.1.0-green)]()

</div>

## 项目简介

reData 是一个多项目管理系统，允许用户创建不同的项目，每个项目可以自定义需要提取的字段。系统使用 AI 模型自动识别表头，并从非标准化的 Excel 文件中提取结构化数据。

### 核心特性

- 🎯 **多项目管理** - 独立的项目空间，灵活的字段定义
- 📝 **类 Excel 字段编辑器** - 直观的表格界面定义字段
- 🤖 **AI 列映射分析** - 每 Sheet 仅 1 次 AI 调用，节省 99.9% Token
- ✅ **本地验证导入** - 格式规则验证，智能数据清理
- 🔄 **可配置去重** - 灵活的去重策略
- 📡 **Tauri 事件系统** - 实时进度推送，零延迟通信
- 🔍 **数据搜索** - 全文搜索，防抖优化
- 💾 **本地 SQLite 存储** - JSON 统一存储方案

## 技术架构

### 前端
- **Nuxt 4.x** - 全栈 Vue 框架
- **Nuxt UI 4.x** - 基于 Tailwind CSS 的 UI 组件库
- **TypeScript** - 完整类型安全
- **Pinia** - 状态管理

### 桌面框架
- **Tauri 2.x** - 轻量级桌面应用框架
- **Tauri Commands** - 零网络开销的前后端通信

### 后端（Rust）
- **SeaORM 1.0** - 异步 ORM
- **async-openai 0.24** - OpenAI API 客户端
- **calamine** - Excel 解析
- **36 个 Tauri Commands** - 完整功能覆盖

### 数据库
- **SQLite 3.40+** - 本地数据库
- **JSON 统一存储** - 以 field_id 为 key 存储动态字段

## 快速开始

### 环境要求

- **Node.js** 18+
- **Rust** 1.75+
- **Cargo** (Rust 包管理器)

### 安装与运行

```bash
# 克隆仓库
git clone <repository-url>
cd reData/redata-app

# 安装依赖
npm install

# 启动开发模式
npm run tauri:dev

# 生产构建
npm run tauri:build
```

## 性能优势

| 指标 | Tauri Commands | 传统 HTTP API |
|------|----------------|---------------|
| 通信延迟 | 0ms（直接调用） | ~1-5ms |
| 内存占用 | ~10 MB | ~15-20 MB |
| 启动时间 | ~1 秒 | ~2-3 秒 |
| 架构复杂度 | 简单 | 复杂 |

## 功能模块

### 项目管理
- 创建、编辑、删除项目
- 项目级别数据隔离

### 字段定义
- 支持 text、phone、email、number、date、id_card 等类型
- 正则验证规则配置
- AI 辅助生成字段元数据

### 数据处理
- AI 列映射分析（智能识别表头）
- 本地验证导入（格式验证 + 数据清理）
- 智能去重检查
- 任务暂停/恢复/取消

### 数据展示
- 分页列表展示
- 全文搜索
- 动态字段显示

## 数据清理机制

根据字段类型自动清理 Excel 中的非标准格式数据：

| 字段类型 | 清理规则 |
|---------|---------|
| phone | 仅保留数字和 + 号 |
| email | 去除空格、换行，转小写 |
| number/id_card | 仅保留数字和字母 |
| date | 仅保留数字和日期分隔符 |
| 其他 | 压缩连续空白为单个空格 |

## 数据库

数据库文件位置：`redata-app/src-tauri/data/app.db`

首次运行时自动创建所有表结构。

## 文档

- [CLAUDE.md](CLAUDE.md) - Claude Code 工作指南
- [prd/v0.1.0/RELEASE_NOTES.md](prd/v0.1.0/RELEASE_NOTES.md) - v0.1.0 发布说明
- [prd/v0.1.0/](prd/v0.1.0/) - 完整产品文档

## 安全

- API 密钥使用 AES-256-GCM 加密存储
- 数据库文件保持本地，不上传云端
- 参数化查询防止 SQL 注入

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

[MIT License](LICENSE)

## 致谢

- [Tauri](https://tauri.app/) - 桌面应用框架
- [Nuxt](https://nuxt.com/) - Vue 全栈框架
- [SeaORM](https://www.sea-ql.org/SeaORM/) - Rust ORM
- [Nuxt UI](https://ui.nuxt.com/) - UI 组件库

---

**版本**: v0.1.0
**发布日期**: 2026-02-18

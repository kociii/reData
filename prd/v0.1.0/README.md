# 智能数据处理平台 - 文档索引 (v0.1.0)

## 版本信息

**当前版本**: v0.1.0
**发布日期**: 2026-02-18
**状态**: 正式发布
**技术架构**: Tauri Commands（Rust 后端）

## 文档列表

| 文档 | 说明 |
|------|------|
| [RELEASE_NOTES.md](./RELEASE_NOTES.md) | v0.1.0 发布说明（新增） |
| [prd.md](./prd.md) | 产品需求文档 |
| [design.md](./design.md) | 界面设计文档 |
| [plan.md](./plan.md) | 开发实施计划 |
| [dev.md](./dev.md) | 技术文档 |

## v0.1.0 核心特性

### 已实现功能
- ✅ 多项目管理
- ✅ 灵活的字段定义（类 Excel 表格编辑器）
- ✅ AI 列映射分析（每 Sheet 仅 1 次 AI 调用）
- ✅ 本地验证导入（格式验证 + 数据清理）
- ✅ 智能去重
- ✅ 数据搜索
- ✅ 实时进度推送（Tauri 事件系统）
- ✅ 任务暂停/恢复/取消

### 技术架构
- **前端**: Nuxt 4.x + Nuxt UI 4.x + TypeScript + Pinia
- **桌面框架**: Tauri 2.x
- **后端**: Rust + Tauri Commands（36 个命令）
- **数据库**: SQLite（JSON 统一存储方案）

### 性能优势
| 指标 | Tauri Commands |
|------|----------------|
| 通信延迟 | 0ms（直接调用） |
| 内存占用 | ~10 MB |
| 启动时间 | ~1 秒 |

## 快速开始

```bash
cd redata-app
npm install
npm run tauri:dev
```

## 历史版本

| 版本 | 日期 | 说明 |
|------|------|------|
| v0.1.0 | 2026-02-18 | 首个正式版本，Tauri Commands 架构 |

---

**维护团队**: reData 开发团队

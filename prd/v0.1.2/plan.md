# reData v0.1.2 实现计划

## 文档索引

| 文档 | 说明 |
|------|------|
| [design.md](./design.md) | 界面设计：ASCII 界面图、交互流程 |
| [dev.md](./dev.md) | 开发实现：数据结构、API 接口、代码实现 |
| [plan.md](./plan.md) | 本文档：阶段划分、风险评估 |

---

## 1. 阶段划分

| 阶段 | 内容 | 状态 | 预估 |
|------|------|------|------|
| 阶段1 | 导入撤回后端 API + 数据库变更 | ✅ 已完成 | 1天 |
| 阶段2 | 导入撤回前端 UI + 批次详情展示 | ✅ 已完成 | 1天 |
| 阶段3 | 项目分组 UI + 后端 API | ✅ 已完成 | 2天 |
| 阶段4 | 筛选条件 UI + 后端查询 | ✅ 已完成 | 2天 |
| 阶段5 | xlsx 导出 + 性能优化 + Bug 修复 | ✅ 已完成 | 1天 |
| 阶段6 | 筛选预设（延后） | 🔲 待开发 | - |

---

## 2. 依赖关系

```
阶段1 ─→ 阶段2
    ↘
     阶段3 ─→ 阶段4 ─→ 阶段5
                        ↓
                     阶段6（延后）
```

- 阶段1 和 阶段3 可以并行开发
- 阶段2 依赖阶段1
- 阶段4 依赖阶段3（筛选需要考虑分组维度）
- 阶段5 依赖阶段4
- 阶段6（筛选预设）延后至 v0.1.3

---

## 3. 阶段详情

### 阶段1：导入撤回后端 API ✅

**任务**：
- [x] 添加 rollback_batch/file/sheet 命令
- [x] 添加 get_batch_details 命令
- [x] 添加 get_project_batches_with_stats 命令
- [x] 确保数据库索引

**产出**：`src-tauri/src/commands/tasks.rs`

### 阶段2：导入撤回前端 UI ✅

**任务**：
- [x] 结果页面数据来源面板
- [x] 批次/文件/Sheet 层级展示
- [x] 撤回按钮和确认对话框

**产出**：`app/pages/project/[id]/results.vue`、`app/utils/api.ts`

### 阶段3：项目分组管理 ✅

**任务**：
- [x] 创建 project_groups 表
- [x] 修改 projects 表添加 group_id
- [x] 实现分组 CRUD 命令
- [x] 实现项目移动命令
- [x] 前端左侧分组列表
- [x] 前端项目卡片拖拽归类

**产出**：
- `src-tauri/src/commands/project_groups.rs`
- `src-tauri/src/backend/infrastructure/persistence/models/project_group.rs`
- `app/pages/index.vue`

### 阶段4：筛选条件增强 ✅

**任务**：
- [x] 后端：query_records_advanced 命令
- [x] 后端：get_field_distinct_values 命令
- [x] 后端：get_source_files 命令
- [x] 前端：筛选条件编辑器组件
- [x] 前端：字段运算符 UI
- [x] 前端：组合条件支持

**产出**：`src-tauri/src/commands/records.rs`、`app/pages/project/[id]/results.vue`

### 阶段5：xlsx 导出与优化 ✅

**任务**：
- [x] 后端：export_records_xlsx 命令（rust_xlsxwriter）
- [x] 前端：导出对话框 UI（范围选择、字段选择）
- [x] 前端：@tauri-apps/plugin-dialog save() 原生保存对话框
- [x] 性能优化：N+1 查询优化（get_project_batches_with_stats）
- [x] 性能优化：批量插入优化（事务 + 分批）
- [x] 性能优化：批量更新优化（单条 UPDATE）
- [x] 安全增强：加密密钥校验（生产环境）
- [x] 安全增强：SQL 注入防护（字段 ID 验证）
- [x] Bug 修复：分组筛选失效
- [x] Bug 修复：分组筛选导航后重置

**产出**：`src-tauri/src/commands/records.rs`、`app/pages/project/[id]/results.vue`

### 阶段6：筛选预设 🔲（延后至 v0.1.3）

**任务**：
- [ ] 后端：save_filter_preset 命令
- [ ] 后端：get_filter_presets 命令
- [ ] 前端：筛选预设 UI

---

## 4. 风险评估

| 风险 | 影响 | 缓解措施 | 状态 |
|------|------|---------|------|
| 撤回数据量大时性能 | 删除大量记录耗时 | 使用事务批量删除，显示进度 | ✅ |
| 撤回后数据一致性 | 去重记录关联处理 | 明确只删除当前批次记录 | ✅ |
| 筛选性能 | JSON 字段查询慢 | 添加索引，考虑缓存 | ✅ |
| 拖拽交互体验 | 拖拽与滚动冲突 | 使用成熟拖拽库，明确视觉反馈 | ✅ |
| N+1 查询性能 | 批次列表加载慢 | 单次聚合查询替代循环查询 | ✅ 已修复 |
| 批量插入性能 | 大文件导入慢 | 事务批量插入（500条/批） | ✅ 已修复 |
| SQL 注入风险 | 字段 ID 恶意输入 | 仅允许数字格式的字段 ID | ✅ 已修复 |
| 加密密钥泄露 | 开发环境密钥误用 | 生产环境强制要求设置 ENCRYPTION_KEY | ✅ 已修复 |

---

## 5. 测试要点

### 5.1 撤回功能
- 撤回整个批次 / 单个文件 / 单个 Sheet
- 撤回确认对话框显示正确记录数
- 撤回不影响其他批次数据

### 5.2 筛选功能
- 全文搜索响应 < 500ms（1万条内）
- 字段筛选支持所有运算符
- AND/OR 组合条件正常

### 5.3 分组功能
- 分组树形结构正确显示
- 拖拽归类正常工作
- 删除分组后项目不丢失
- 分组筛选导航后保持状态（URL 参数持久化）

### 5.4 xlsx 导出功能
- 三种导出范围正常工作
- 字段选择正确
- 导入时间、来源文件列正确
- 原生保存对话框正常弹出

---

**文档版本**: v0.1.2-plan
**创建日期**: 2026-02-19
**作者**: Claude Code

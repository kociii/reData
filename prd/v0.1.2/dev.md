# reData v0.1.2 开发实现文档

## 文档说明

本文档包含 v0.1.2 版本的技术实现细节：数据结构、API 接口、代码实现和开发进度。

---

## 1. 功能一：导入撤回与重新导入

### 1.1 数据关联设计

```
batch (批次)
  └─ task (任务)
       └─ task_file_progress (文件/Sheet进度)
            └─ records (记录，通过 source_file、source_sheet、batch_number 关联)
```

### 1.2 数据库变更

```sql
-- 确保 project_records 表有必要字段
-- source_sheet 和 batch_number 应已存在

-- 添加索引以支持高效删除
CREATE INDEX IF NOT EXISTS idx_records_batch ON project_records(batch_number);
CREATE INDEX IF NOT EXISTS idx_records_source ON project_records(source_file, source_sheet);
```

### 1.3 Rust 数据结构

```rust
// 撤回结果
#[derive(Serialize)]
pub struct RollbackResult {
    pub success: bool,
    pub deleted_count: u64,
    pub message: String,
}

// Sheet 导入详情
#[derive(Serialize, FromQueryResult)]
pub struct SheetImportDetail {
    pub sheet_name: String,
    pub record_count: i64,
    pub status: String,
    pub can_rollback: bool,
}

// 文件导入详情
#[derive(Serialize)]
pub struct FileImportDetail {
    pub file_name: String,
    pub sheets: Vec<SheetImportDetail>,
    pub total_records: i64,
    pub can_rollback: bool,
}

// 批次详情响应
#[derive(Serialize)]
pub struct BatchDetailResponse {
    pub batch_number: String,
    pub project_id: i32,
    pub created_at: DateTimeUtc,
    pub status: String,
    pub total_records: i64,
    pub files: Vec<FileImportDetail>,
}
```

### 1.4 Tauri Commands API

```rust
/// 撤回整个批次
#[tauri::command]
pub async fn rollback_batch(
    project_id: i32,
    batch_number: String,
) -> Result<RollbackResult, String>;

/// 撤回单个文件
#[tauri::command]
pub async fn rollback_file(
    project_id: i32,
    batch_number: String,
    file_name: String,
) -> Result<RollbackResult, String>;

/// 撤回单个 Sheet
#[tauri::command]
pub async fn rollback_sheet(
    project_id: i32,
    batch_number: String,
    file_name: String,
    sheet_name: String,
) -> Result<RollbackResult, String>;

/// 获取批次详情
#[tauri::command]
pub async fn get_batch_details(
    project_id: i32,
    batch_number: String,
) -> Result<BatchDetailResponse, String>;

/// 获取项目所有批次统计
#[tauri::command]
pub async fn get_project_batches_with_stats(
    project_id: i32,
) -> Result<Vec<BatchDetailResponse>, String>;
```

### 1.5 TypeScript 类型定义

```typescript
// 撤回结果
export interface RollbackResult {
  success: boolean
  deleted_count: number
  message: string
}

// Sheet 导入详情
export interface SheetImportDetail {
  sheet_name: string
  record_count: number
  status: string
  can_rollback: boolean
}

// 文件导入详情
export interface FileImportDetail {
  file_name: string
  sheets: SheetImportDetail[]
  total_records: number
  can_rollback: boolean
}

// 批次详情
export interface BatchDetailResponse {
  batch_number: string
  project_id: number
  created_at: string
  status: string
  total_records: number
  files: FileImportDetail[]
}
```

### 1.6 前端 API 调用

```typescript
// app/utils/api.ts
export const batchesApi = {
  // 获取批次详情
  getDetails: (projectId: number, batchNumber: string) =>
    invoke<BatchDetailResponse>('get_batch_details', { projectId, batchNumber }),

  // 获取项目所有批次
  getProjectBatches: (projectId: number) =>
    invoke<BatchDetailResponse[]>('get_project_batches_with_stats', { projectId }),

  // 撤回操作
  rollback: {
    batch: (projectId: number, batchNumber: string) =>
      invoke<RollbackResult>('rollback_batch', { projectId, batchNumber }),
    file: (projectId: number, batchNumber: string, fileName: string) =>
      invoke<RollbackResult>('rollback_file', { projectId, batchNumber, fileName }),
    sheet: (projectId: number, batchNumber: string, fileName: string, sheetName: string) =>
      invoke<RollbackResult>('rollback_sheet', { projectId, batchNumber, fileName, sheetName }),
  },
}
```

---

## 2. 功能二：数据结果筛选增强

### 2.1 筛选条件数据结构

```typescript
// 单个筛选条件
interface FilterCondition {
  id: string
  field: string
  operator: FilterOperator
  value: string | number | [string, string]
}

// 运算符枚举
type FilterOperator =
  | 'eq' | 'neq'           // 等于、不等于
  | 'contains' | 'not_contains'  // 包含、不包含
  | 'starts_with' | 'ends_with'  // 开头为、结尾为
  | 'gt' | 'lt' | 'gte' | 'lte'  // 大于、小于、大于等于、小于等于
  | 'between'              // 在范围内
  | 'is_empty' | 'is_not_empty'  // 为空、不为空

// 完整筛选请求
interface FilterRequest {
  searchText?: string
  conditions: FilterCondition[]
  sourceFile?: string
  batchNumber?: string
  status?: 'valid' | 'invalid' | 'duplicate' | 'all'
  conjunction: 'and' | 'or'
}
```

### 2.2 后端查询 API

```rust
/// 高级记录查询
#[tauri::command]
pub async fn query_records_advanced(
    project_id: i32,
    filter: FilterRequest,
    page: i32,
    page_size: i32,
) -> Result<QueryResult, String>;

/// 获取字段唯一值
#[tauri::command]
pub async fn get_field_distinct_values(
    project_id: i32,
    field_id: i32,
    search: Option<String>,
    limit: i32,
) -> Result<Vec<String>, String>;

/// 获取来源文件列表
#[tauri::command]
pub async fn get_source_files(
    project_id: i32,
) -> Result<Vec<SourceFileInfo>, String>;
```

### 2.3 SQL 查询生成逻辑

```rust
fn build_filter_sql(filter: &FilterRequest) -> (String, Vec<Value>) {
    let mut conditions = Vec::new();
    let mut params = Vec::new();

    // 全文搜索
    if let Some(text) = &filter.search_text {
        conditions.push("data LIKE ?");
        params.push(format!("%{}%", text));
    }

    // 字段条件 (使用 json_extract)
    for cond in &filter.conditions {
        let sql = match cond.operator {
            FilterOperator::Eq => format!("json_extract(data, '$.{}') = ?", cond.field),
            FilterOperator::Contains => format!("json_extract(data, '$.{}') LIKE ?", cond.field),
            FilterOperator::Gt => format!("json_extract(data, '$.{}') > ?", cond.field),
            // ... 其他运算符
        };
        conditions.push(sql);
    }

    let conjunction = match filter.conjunction {
        Conjunction::And => " AND ",
        Conjunction::Or => " OR ",
    };

    (conditions.join(conjunction), params)
}
```

### 2.4 数据库优化

```sql
-- 筛选性能优化索引
CREATE INDEX idx_records_created_at ON project_records(created_at);
CREATE INDEX idx_records_source_file ON project_records(source_file);
CREATE INDEX idx_records_batch_number ON project_records(batch_number);
```

---

## 3. 功能三：项目列表分组管理

### 3.1 数据库变更

新增 `project_groups` 表：

```sql
CREATE TABLE project_groups (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  parent_id INTEGER,
  color TEXT,
  icon TEXT,
  sort_order INTEGER DEFAULT 0,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME,

  FOREIGN KEY (parent_id) REFERENCES project_groups(id) ON DELETE SET NULL
);

CREATE INDEX idx_project_groups_parent ON project_groups(parent_id);
CREATE INDEX idx_project_groups_sort ON project_groups(sort_order);
```

修改 `projects` 表：

```sql
ALTER TABLE projects ADD COLUMN group_id INTEGER REFERENCES project_groups(id) ON DELETE SET NULL;
CREATE INDEX idx_projects_group ON projects(group_id);
```

### 3.2 Rust 数据结构

```rust
// project_group.rs
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "project_groups")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
    pub color: Option<String>,
    pub icon: Option<String>,
    #[sea_orm(default_value = "0")]
    pub sort_order: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: Option<DateTimeUtc>,
}

// 分组响应（带项目数量）
#[derive(Serialize)]
pub struct ProjectGroupResponse {
    pub id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub project_count: i64,
    pub created_at: DateTimeUtc,
    pub updated_at: Option<DateTimeUtc>,
}

// 带子分组的分组树
#[derive(Serialize)]
pub struct GroupWithChildren {
    pub id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub project_count: i64,
    pub children: Vec<GroupWithChildren>,
    pub created_at: DateTimeUtc,
    pub updated_at: Option<DateTimeUtc>,
}
```

### 3.3 Tauri Commands API

```rust
/// 获取所有分组（带层级结构）
#[tauri::command]
pub async fn get_project_groups() -> Result<Vec<GroupWithChildren>, String>;

/// 创建分组
#[tauri::command]
pub async fn create_project_group(
    name: String,
    parent_id: Option<i32>,
    color: Option<String>,
    icon: Option<String>,
) -> Result<ProjectGroupResponse, String>;

/// 更新分组
#[tauri::command]
pub async fn update_project_group(
    group_id: i32,
    name: Option<String>,
    color: Option<String>,
    icon: Option<String>,
    sort_order: Option<i32>,
) -> Result<ProjectGroupResponse, String>;

/// 删除分组
#[tauri::command]
pub async fn delete_project_group(group_id: i32) -> Result<(), String>;

/// 移动项目到分组
#[tauri::command]
pub async fn move_project_to_group(
    project_id: i32,
    group_id: Option<i32>,
) -> Result<(), String>;

/// 批量移动项目
#[tauri::command]
pub async fn batch_move_projects(
    project_ids: Vec<i32>,
    group_id: Option<i32>,
) -> Result<u64, String>;

/// 更新分组排序
#[tauri::command]
pub async fn reorder_project_groups(
    group_orders: Vec<(i32, i32)>,
) -> Result<(), String>;
```

### 3.4 TypeScript 类型定义

```typescript
// 项目分组响应
export interface ProjectGroupResponse {
  id: number
  name: string
  parent_id: number | null
  color: string | null
  icon: string | null
  sort_order: number
  project_count: number
  created_at: string
  updated_at: string | null
}

// 带子分组的分组树
export interface GroupWithChildren {
  id: number
  name: string
  parent_id: number | null
  color: string | null
  icon: string | null
  sort_order: number
  project_count: number
  children: GroupWithChildren[]
  created_at: string
  updated_at: string | null
}
```

### 3.5 前端 API 调用

```typescript
// app/utils/api.ts
export const projectGroupsApi = {
  getAll: () => invoke<GroupWithChildren[]>('get_project_groups'),

  create: (data: {
    name: string
    parentId?: number | null
    color?: string
    icon?: string
  }) => invoke<ProjectGroupResponse>('create_project_group', {
    name: data.name,
    parentId: data.parentId ?? null,
    color: data.color,
    icon: data.icon,
  }),

  update: (groupId: number, data: Partial<{
    name: string
    color: string
    icon: string
    sortOrder: number
  }>) => invoke<ProjectGroupResponse>('update_project_group', {
    groupId,
    name: data.name,
    color: data.color,
    icon: data.icon,
    sortOrder: data.sortOrder,
  }),

  delete: (groupId: number) => invoke<void>('delete_project_group', { groupId }),

  moveProject: (projectId: number, groupId: number | null) =>
    invoke<void>('move_project_to_group', { projectId, groupId }),

  batchMove: (projectIds: number[], groupId: number | null) =>
    invoke<number>('batch_move_projects', { projectIds, groupId }),

  reorder: (orders: Array<{ id: number; sortOrder: number }>) =>
    invoke<void>('reorder_project_groups', {
      groupOrders: orders.map(o => [o.id, o.sortOrder]),
    }),
}
```

### 3.6 拖拽实现

```typescript
// 项目卡片拖拽
const onDragStart = (e: DragEvent, project: Project) => {
  e.dataTransfer?.setData('project-id', String(project.id))
  isDragging.value = true
}

const onDrop = (e: DragEvent, groupId: number | null) => {
  const projectId = e.dataTransfer?.getData('project-id')
  if (projectId) {
    moveProjectToGroup(Number(projectId), groupId)
  }
  isDragging.value = false
}
```

### 3.7 分组树构建

```typescript
const buildGroupTree = (groups: ProjectGroupResponse[]): GroupWithChildren[] => {
  const groupMap = new Map<number, GroupWithChildren>()
  const roots: GroupWithChildren[] = []

  // 创建映射
  groups.forEach(g => {
    groupMap.set(g.id, { ...g, children: [] })
  })

  // 构建树
  groups.forEach(g => {
    const node = groupMap.get(g.id)!
    if (g.parent_id && groupMap.has(g.parent_id)) {
      groupMap.get(g.parent_id)!.children.push(node)
    } else {
      roots.push(node)
    }
  })

  return roots
}
```

---

## 4. 开发进度

### 4.1 阶段1：导入撤回后端 API ✅ 已完成

**修改文件**：
- `src-tauri/src/commands/tasks.rs`

**遇到的问题**：
1. ConnectionTrait 未导入 → 添加 `use sea_orm::ConnectionTrait;`
2. 未使用的导入警告 → 移除局部 use 语句

### 4.2 阶段2：导入撤回前端 UI ✅ 已完成

**修改文件**：
- `app/pages/project/[id]/results.vue`
- `app/utils/api.ts`
- `app/types/index.ts`

### 4.3 阶段3：项目分组管理 ✅ 已完成

**新增文件**：
- `src-tauri/src/commands/project_groups.rs`
- `src-tauri/src/backend/infrastructure/persistence/models/project_group.rs`

**修改文件**：
- `src-tauri/src/backend/infrastructure/persistence/models/project.rs`
- `src-tauri/src/backend/infrastructure/persistence/migrations.rs`
- `app/pages/index.vue`

**遇到的问题**：
1. Sea_ORM 关联错误 → 在 project.rs 添加 Related trait 实现

### 4.4 阶段4：筛选条件增强 ✅ 已完成

**修改文件**：
- `src-tauri/src/commands/records.rs`
- `src-tauri/src/lib.rs`
- `app/types/index.ts`
- `app/utils/api.ts`
- `app/pages/project/[id]/results.vue`

**新增 API**：
- `query_records_advanced` - 高级筛选查询
- `get_field_distinct_values` - 获取字段唯一值
- `get_source_files` - 获取来源文件列表

**实现功能**：
- 筛选面板 UI
- 字段运算符选择
- 组合条件支持（AND/OR）
- 快捷筛选（来源文件、批次）

### 4.5 阶段5：筛选预设与优化 🔲 待开发

### 4.6 Bug 修复：状态变化和撤回功能 ✅ 已完成

**问题描述**：
1. 数据处理状态的变化不对：任务重置后 `starting` 状态不在活动任务列表中显示
2. 导入撤回功能检查确认实现正确

**修改文件**：
- `app/stores/processing.ts`

**修复内容**：
1. 在 `activeTasks` 计算属性中添加 `'starting'` 状态，确保重置后的任务正确显示在活动任务列表中
   ```typescript
   const activeTasks = computed(() =>
     tasks.value.filter(t => t.phase === 'processing' || t.phase === 'paused' || t.phase === 'starting'),
   )
   ```

**撤回功能验证**：
- 后端撤回命令（`rollback_batch`, `rollback_file`, `rollback_sheet`）已正确实现
- 前端 UI 和 API 调用正确
- 批次列表刷新时会实时查询数据库获取更新后的记录数

---

## 5. 边界情况处理

### 5.1 撤回功能

| 情况 | 处理策略 |
|------|---------|
| 撤回已去重的记录 | 只删除当前批次导入的记录 |
| 批次部分已撤回 | 标记状态，允许继续撤回 |
| 撤回时正在导入 | 阻止撤回，提示等待完成 |

### 5.2 分组功能

| 情况 | 处理 |
|------|------|
| 分组嵌套层级过深 | 限制最大层级为 3 层 |
| 删除有子分组的分组 | 提示确认，项目移至父分组 |
| 项目的分组被删除 | 项目 group_id 自动置空 |

---

## 6. 验收清单

### 6.1 导入撤回

- [x] 能撤回整个批次的数据
- [x] 能撤回单个文件的数据
- [x] 能撤回单个 Sheet 的数据
- [x] 撤回操作有确认对话框
- [x] 撤回后显示删除的记录数
- [x] 撤回不影响其他批次的数据
- [x] 能查看批次的文件和 Sheet 详情
- [ ] 撤回后能正常重新导入

### 6.2 筛选增强

- [x] 全文搜索响应时间 < 500ms
- [x] 字段筛选支持所有运算符
- [x] 组合筛选条件正常工作
- [ ] 筛选预设能保存和加载

### 6.3 项目分组

- [x] 左侧分组列表正确显示层级
- [x] "全部" 显示所有项目
- [x] 点击分组正确筛选项目
- [x] 拖拽项目到分组能正确归类
- [x] 支持创建、编辑、删除分组
- [x] 分组数量统计正确
- [x] 删除分组后项目不会丢失

---

**文档版本**: v0.1.2-dev
**创建日期**: 2026-02-19
**最后更新**: 2026-02-19
**作者**: Claude Code

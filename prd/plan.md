# 智能表格数据提取系统 - 实施计划

## Context（项目背景）

本项目旨在解决大规模本地表格数据的结构化提取问题。用户拥有数百万条存储在 Excel 文件中的非结构化数据，这些表格格式不统一（表头位置不固定、有无表头不确定、多 Sheet 等），需要通过 AI 大模型智能识别表头并提取关键字段（姓名、手机号、公司、地区）。

系统将构建为本地桌面应用，使用 Tauri 框架，提供可视化的处理界面和结果查询界面，支持并行处理多个文件，实时展示处理进度，并将提取结果存储在本地 SQLite 数据库中。

## 核心需求总结

### 1. 系统架构
- **桌面应用框架**：Tauri 2.x
- **前端技术栈**：Vue 3 + TypeScript + Vite + Pinia
- **UI 组件库**：Element Plus
- **数据库**：SQLite
- **AI 集成**：OpenAI SDK（支持兼容接口）

### 2. 两个主要界面

#### 结果页面
- 展示所有提取的数据（姓名、手机号、公司、地区）
- 支持编辑功能（直接修改字段值）
- 支持导出功能（Excel/CSV）
- 支持筛选和搜索（按来源文件、日期、地区等）
- 分页展示

#### 处理界面
- **布局**：左右分栏
  - **左侧**：处理中的文件列表
    - 显示文件名、进度、状态
    - 支持选择查看详情
  - **右侧**：选中文件的详细处理过程
    - **左侧区域**：整个 Sheet 的预览（表格形式）
    - **右侧区域**：当前正在处理的行的提取结果
- **功能**：
  - 选择文件夹或文件进行处理
  - 支持并行处理多个文件
  - 暂停/恢复处理
  - 取消处理任务
- **进度显示**：
  - 行数进度（当前行/总行数）
  - 百分比进度条
  - 成功/失败统计
  - 处理速度（行/分钟）

### 3. 核心处理流程

1. **文件导入**：
   - 用户选择文件夹或文件
   - 系统复制文件到 `history/batch_XXX/` 目录（批次号递增）
   - 原文件保持不变

2. **表头识别**（完全自动）：
   - 读取每个 Sheet 的前 5 行
   - 提交给 AI 模型识别表头所在行号
   - 缓存表头信息用于后续提取

3. **数据提取**：
   - 逐行读取数据
   - 组装「表头:值」格式
   - 提交给 AI 模型提取目标字段
   - 连续 10 个空行则跳过当前 Sheet

4. **数据存储**：
   - 按手机号实时自动去重
   - 存储到 SQLite 数据库
   - 记录原始内容、来源文件、来源 Sheet

5. **失败处理**：
   - 记录错误信息
   - 继续处理下一行
   - 最后展示失败统计

### 4. AI 配置管理
- 支持多个 AI 配置预设
- 可配置：API URL、Model 名称、API Key、温度、最大 Token
- 设置默认配置
- 处理时自动使用默认配置

## 技术架构设计

### 项目结构

```
reData/
├── src/                          # 前端源码
│   ├── main.ts                   # 入口文件
│   ├── App.vue                   # 根组件
│   ├── router/                   # 路由配置
│   │   └── index.ts
│   ├── stores/                   # Pinia 状态管理
│   │   ├── processing.ts         # 处理任务状态
│   │   ├── config.ts             # AI 配置状态
│   │   └── result.ts             # 结果数据状态
│   ├── views/                    # 页面组件
│   │   ├── ProcessingView.vue    # 处理界面
│   │   └── ResultView.vue        # 结果页面
│   ├── components/               # 通用组件
│   │   ├── FileList.vue          # 文件列表组件
│   │   ├── SheetPreview.vue      # Sheet 预览组件
│   │   ├── ExtractionResult.vue  # 提取结果组件
│   │   ├── ProgressBar.vue       # 进度条组件
│   │   └── ConfigDialog.vue      # 配置对话框
│   ├── types/                    # TypeScript 类型定义
│   │   ├── processing.ts
│   │   ├── config.ts
│   │   └── result.ts
│   └── utils/                    # 工具函数
│       └── format.ts
├── src-tauri/                    # Tauri Rust 后端
│   ├── src/
│   │   ├── main.rs               # 主入口
│   │   ├── commands/             # Tauri 命令
│   │   │   ├── mod.rs
│   │   │   ├── file.rs           # 文件操作命令
│   │   │   ├── processing.rs     # 处理命令
│   │   │   ├── config.rs         # 配置命令
│   │   │   └── result.rs         # 结果查询命令
│   │   ├── services/             # 业务逻辑
│   │   │   ├── mod.rs
│   │   │   ├── excel_parser.rs   # Excel 解析服务
│   │   │   ├── ai_client.rs      # AI 客户端服务
│   │   │   ├── extractor.rs      # 数据提取服务
│   │   │   └── storage.rs        # 数据存储服务
│   │   ├── models/               # 数据模型
│   │   │   ├── mod.rs
│   │   │   ├── task.rs           # 任务模型
│   │   │   ├── config.rs         # 配置模型
│   │   │   └── record.rs         # 记录模型
│   │   └── db/                   # 数据库
│   │       ├── mod.rs
│   │       └── schema.rs         # 数据库表结构
│   ├── Cargo.toml
│   └── tauri.conf.json
├── history/                      # 历史文件目录
│   ├── batch_001/
│   ├── batch_002/
│   └── ...
├── data/                         # 数据库文件
│   └── app.db
├── prd/                          # 需求文档
│   └── prd.md
├── package.json
├── vite.config.ts
└── tsconfig.json
```

### 核心模块设计

#### 1. 前端模块

**路由设计**：
- `/processing` - 处理界面
- `/result` - 结果页面
- `/settings` - 设置页面（AI 配置管理）

**状态管理（Pinia Stores）**：

1. **processing.ts** - 处理任务状态
   - 当前处理的文件列表
   - 每个文件的处理进度
   - 选中的文件
   - 暂停/恢复/取消控制

2. **config.ts** - AI 配置状态
   - 配置列表
   - 默认配置
   - 当前使用的配置

3. **result.ts** - 结果数据状态
   - 数据列表
   - 分页信息
   - 筛选条件

#### 2. 后端模块（Rust）

**Tauri Commands**（前端调用的 API）：

1. **file.rs**
   - `select_files()` - 选择文件
   - `select_folder()` - 选择文件夹
   - `copy_to_history(files: Vec<String>)` - 复制文件到历史目录
   - `get_next_batch_number()` - 获取下一个批次号

2. **processing.rs**
   - `start_processing(files: Vec<String>, config_id: i32)` - 开始处理
   - `pause_processing(task_id: String)` - 暂停处理
   - `resume_processing(task_id: String)` - 恢复处理
   - `cancel_processing(task_id: String)` - 取消处理
   - `get_processing_status(task_id: String)` - 获取处理状态

3. **config.rs**
   - `get_configs()` - 获取所有配置
   - `get_default_config()` - 获取默认配置
   - `save_config(config: AiConfig)` - 保存配置
   - `set_default_config(config_id: i32)` - 设置默认配置
   - `delete_config(config_id: i32)` - 删除配置

4. **result.rs**
   - `query_results(filter: ResultFilter, page: i32, page_size: i32)` - 查询结果
   - `update_record(id: i32, record: Record)` - 更新记录
   - `delete_record(id: i32)` - 删除记录
   - `export_results(filter: ResultFilter, format: String)` - 导出结果

**Services**（业务逻辑）：

1. **excel_parser.rs** - Excel 解析
   - 读取 Excel 文件
   - 遍历所有 Sheet
   - 读取指定行数据
   - 检测空行

2. **ai_client.rs** - AI 客户端
   - 调用 OpenAI 兼容 API
   - 表头识别 Prompt
   - 数据提取 Prompt
   - 错误重试机制

3. **extractor.rs** - 数据提取
   - 协调整个提取流程
   - 表头识别
   - 逐行提取
   - 进度回调
   - 错误处理

4. **storage.rs** - 数据存储
   - SQLite 操作
   - 数据插入（带去重）
   - 数据查询
   - 数据更新/删除

## 数据库设计

### 表结构

```sql
-- 提取的数据记录表
CREATE TABLE extracted_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT,                          -- 姓名
    phone TEXT UNIQUE,                  -- 手机号（唯一，用于去重）
    company TEXT,                       -- 公司
    region TEXT,                        -- 地区
    raw_content TEXT,                   -- 原始内容
    source_file TEXT,                   -- 来源文件路径
    source_sheet TEXT,                  -- 来源 Sheet 名称
    row_number INTEGER,                 -- 行号
    batch_number TEXT,                  -- 批次号
    status TEXT DEFAULT 'success',      -- 状态：success/failed
    error_message TEXT,                 -- 错误信息（如果失败）
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 处理任务表
CREATE TABLE processing_tasks (
    id TEXT PRIMARY KEY,                -- 任务 ID（UUID）
    batch_number TEXT,                  -- 批次号
    file_path TEXT,                     -- 文件路径
    total_sheets INTEGER,               -- 总 Sheet 数
    processed_sheets INTEGER DEFAULT 0, -- 已处理 Sheet 数
    total_rows INTEGER,                 -- 总行数
    processed_rows INTEGER DEFAULT 0,   -- 已处理行数
    success_count INTEGER DEFAULT 0,    -- 成功数量
    failed_count INTEGER DEFAULT 0,     -- 失败数量
    status TEXT DEFAULT 'pending',      -- 状态：pending/processing/paused/completed/cancelled
    config_id INTEGER,                  -- 使用的 AI 配置 ID
    started_at DATETIME,
    completed_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- AI 配置表
CREATE TABLE ai_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,                 -- 配置名称
    api_url TEXT NOT NULL,              -- API URL
    model_name TEXT NOT NULL,           -- 模型名称
    api_key TEXT NOT NULL,              -- API Key
    temperature REAL DEFAULT 0.7,       -- 温度
    max_tokens INTEGER DEFAULT 1000,    -- 最大 Token
    is_default INTEGER DEFAULT 0,       -- 是否默认配置
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 批次表
CREATE TABLE batches (
    batch_number TEXT PRIMARY KEY,      -- 批次号（batch_001）
    file_count INTEGER DEFAULT 0,       -- 文件数量
    total_records INTEGER DEFAULT 0,    -- 总记录数
    success_count INTEGER DEFAULT 0,    -- 成功数量
    failed_count INTEGER DEFAULT 0,     -- 失败数量
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX idx_phone ON extracted_records(phone);
CREATE INDEX idx_batch ON extracted_records(batch_number);
CREATE INDEX idx_task_status ON processing_tasks(status);
CREATE INDEX idx_config_default ON ai_configs(is_default);
```

## 核心处理流程详细设计

### 1. 文件导入流程

```
用户选择文件/文件夹
    ↓
获取下一个批次号（batch_XXX）
    ↓
创建 history/batch_XXX/ 目录
    ↓
复制文件到历史目录
    ↓
返回文件列表和批次号
```

### 2. 处理流程

```
开始处理
    ↓
获取默认 AI 配置
    ↓
遍历每个文件
    ↓
    遍历每个 Sheet
        ↓
        读取前 5 行 → AI 识别表头
        ↓
        判断是否有表头
        ↓
    ┌───┴────┐
    │        │
有表头    无表头
    │        │
    ↓        ↓
从表头下一行  从第 1 行
开始处理     开始处理
    │        │
    └───┬────┘
        ↓
    逐行处理
        ↓
        检查空行计数器
        ↓
    ┌───┴────┐
    │        │
有表头    无表头
    │        │
    ↓        ↓
组装「表头:值」 直接提交
格式         原始数据
    │        │
    └───┬────┘
        ↓
        调用 AI 提取字段
        ↓
        解析 AI 返回结果
        ↓
        检查手机号是否重复
        ↓
        插入数据库（不重复则插入）
        ↓
        更新进度（通过事件发送到前端）
        ↓
        如果失败：记录错误，继续下一行
    ↓
    如果连续 10 空行：跳过当前 Sheet
    ↓
完成处理
```

### 3. 实时进度更新

使用 Tauri 的事件系统（Event）实现实时进度推送：

**后端发送事件**：
```rust
app.emit_all("processing-progress", ProgressPayload {
    task_id: task_id.clone(),
    current_row: current_row,
    total_rows: total_rows,
    success_count: success_count,
    failed_count: failed_count,
    current_sheet: sheet_name.clone(),
    processing_speed: speed,
});
```

**前端监听事件**：
```typescript
import { listen } from '@tauri-apps/api/event'

listen('processing-progress', (event) => {
  // 更新 Pinia store
  processingStore.updateProgress(event.payload)
})
```

## AI Prompt 设计

### 1. 表头识别 Prompt

```
你是一个表格分析专家。以下是一个 Excel 表格的前 5 行数据：

[第 1 行] {row_1_data}
[第 2 行] {row_2_data}
[第 3 行] {row_3_data}
[第 4 行] {row_4_data}
[第 5 行] {row_5_data}

请分析并判断：
1. 第几行是表头？（返回行号 1-5，如果没有表头则返回 0）
2. 表头包含哪些字段？（返回字段列表，如果没有表头则返回空数组）

请以 JSON 格式返回：
{
  "header_row": 1,  // 1-5 表示表头行号，0 表示无表头
  "headers": ["字段1", "字段2", "字段3", ...]  // 无表头时返回 []
}
```

### 2. 数据提取 Prompt

#### 2.1 有表头的表格

```
你是一个数据提取专家。请从以下数据中提取指定字段：

原始数据：
{formatted_row_data}  // 格式：表头1:值1; 表头2:值2; ...

请提取以下字段：
- 姓名（支持中文、英文、带称呼如"李先生"、"王总"）
- 手机号（11位数字）
- 公司
- 地区（优先从地址字段提取，也可从公司名称推演，如"北京XX公司"→"北京市"）
- 邮箱

请以 JSON 格式返回：
{
  "name": "提取的姓名",
  "phone": "提取的手机号",
  "company": "提取的公司",
  "region": "提取的地区",
  "email": "提取的邮箱"
}

如果某个字段无法提取，请返回空字符串。
```

#### 2.2 无表头的表格

```
你是一个数据提取专家。请从以下原始数据中提取指定字段：

原始数据：
{raw_row_data}  // 格式：值1 | 值2 | 值3 | ...

请提取以下字段：
- 姓名（支持中文、英文、带称呼如"李先生"、"王总"）
- 手机号（11位数字）
- 公司
- 地区（优先从地址字段提取，也可从公司名称推演，如"北京XX公司"→"北京市"）
- 邮箱

请以 JSON 格式返回：
{
  "name": "提取的姓名",
  "phone": "提取的手机号",
  "company": "提取的公司",
  "region": "提取的地区",
  "email": "提取的邮箱"
}

如果某个字段无法提取，请返回空字符串。
```

## 实施步骤

### Phase 1：项目初始化（第 1 天）

1. 创建 Tauri 项目
   ```bash
   npm create tauri-app@latest
   # 选择 Vue + TypeScript
   ```

2. 安装依赖
   ```bash
   # 前端依赖
   npm install vue-router pinia element-plus
   npm install @element-plus/icons-vue

   # Rust 依赖（在 src-tauri/Cargo.toml 中添加）
   # - calamine（Excel 解析）
   # - rusqlite（SQLite）
   # - reqwest（HTTP 客户端）
   # - serde, serde_json（序列化）
   # - tokio（异步运行时）
   # - uuid（生成任务 ID）
   ```

3. 配置项目结构
   - 创建目录结构
   - 配置路由
   - 配置 Pinia

### Phase 2：数据库和基础服务（第 2-3 天）

1. 实现数据库模块
   - `src-tauri/src/db/schema.rs` - 创建表结构
   - `src-tauri/src/db/mod.rs` - 数据库连接和初始化

2. 实现数据模型
   - `src-tauri/src/models/` - 定义所有数据模型

3. 实现存储服务
   - `src-tauri/src/services/storage.rs` - 数据库 CRUD 操作

### Phase 3：AI 集成和 Excel 解析（第 4-5 天）

1. 实现 AI 客户端
   - `src-tauri/src/services/ai_client.rs`
   - OpenAI SDK 集成
   - Prompt 模板
   - 错误重试

2. 实现 Excel 解析
   - `src-tauri/src/services/excel_parser.rs`
   - 使用 calamine 库
   - 读取 Sheet
   - 读取行数据

3. 实现数据提取服务
   - `src-tauri/src/services/extractor.rs`
   - 协调表头识别和数据提取
   - 进度回调

### Phase 4：Tauri Commands（第 6 天）

实现所有 Tauri 命令：
- `src-tauri/src/commands/file.rs`
- `src-tauri/src/commands/processing.rs`
- `src-tauri/src/commands/config.rs`
- `src-tauri/src/commands/result.rs`

### Phase 5：前端 - 配置管理（第 7 天）

1. 实现配置页面
   - AI 配置列表
   - 添加/编辑/删除配置
   - 设置默认配置

2. 实现配置 Store
   - `src/stores/config.ts`

### Phase 6：前端 - 处理界面（第 8-10 天）

1. 实现处理界面布局
   - `src/views/ProcessingView.vue`
   - 左侧文件列表
   - 右侧详情区域

2. 实现文件列表组件
   - `src/components/FileList.vue`
   - 显示文件名、进度、状态
   - 选择文件查看详情

3. 实现 Sheet 预览组件
   - `src/components/SheetPreview.vue`
   - 表格形式展示 Sheet 数据
   - 高亮当前处理行

4. 实现提取结果组件
   - `src/components/ExtractionResult.vue`
   - 显示当前行的提取结果

5. 实现进度条组件
   - `src/components/ProgressBar.vue`
   - 显示行数进度、百分比、成功/失败统计、处理速度

6. 实现处理控制
   - 暂停/恢复/取消按钮
   - 文件选择对话框

7. 实现处理 Store
   - `src/stores/processing.ts`
   - 监听处理进度事件

### Phase 7：前端 - 结果页面（第 11-12 天）

1. 实现结果页面
   - `src/views/ResultView.vue`
   - 数据表格
   - 分页
   - 筛选和搜索
   - 编辑功能
   - 导出功能

2. 实现结果 Store
   - `src/stores/result.ts`

### Phase 8：测试和优化（第 13-14 天）

1. 功能测试
   - 测试各种表格格式
   - 测试并行处理
   - 测试暂停/恢复/取消
   - 测试去重功能

2. 性能优化
   - 优化 AI 调用频率
   - 优化数据库批量插入
   - 优化前端渲染性能

3. 错误处理
   - 完善错误提示
   - 完善日志记录

### Phase 9：打包和部署（第 15 天）

1. 配置打包
   - 配置 Tauri 打包选项
   - 配置应用图标

2. 打包应用
   ```bash
   npm run tauri build
   ```

3. 测试安装包

## 关键技术点

### 1. 并行处理实现

使用 Rust 的 `tokio` 异步运行时实现并行处理：

```rust
use tokio::task;

async fn process_files(files: Vec<String>) {
    let mut handles = vec![];

    for file in files {
        let handle = task::spawn(async move {
            process_single_file(file).await
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}
```

### 2. 手机号去重实现

在数据库层面使用 UNIQUE 约束：

```sql
CREATE TABLE extracted_records (
    ...
    phone TEXT UNIQUE,
    ...
);
```

插入时使用 `INSERT OR IGNORE` 或 `INSERT OR REPLACE`：

```rust
conn.execute(
    "INSERT OR IGNORE INTO extracted_records (...) VALUES (...)",
    params![...],
)?;
```

### 3. 实时进度更新

使用 Tauri 的事件系统：

```rust
// 后端
app.emit_all("processing-progress", payload)?;

// 前端
listen('processing-progress', (event) => {
    // 更新 UI
});
```

### 4. 暂停/恢复实现

使用 `Arc<Mutex<bool>>` 共享暂停状态：

```rust
use std::sync::{Arc, Mutex};

let paused = Arc::new(Mutex::new(false));

// 在处理循环中检查
if *paused.lock().unwrap() {
    tokio::time::sleep(Duration::from_millis(100)).await;
    continue;
}
```

## 验证方法

### 1. 功能验证

1. **文件导入验证**
   - 选择文件夹，检查是否正确复制到 `history/batch_XXX/`
   - 检查批次号是否递增

2. **表头识别验证**
   - 准备不同格式的测试表格（表头在第 1、2、3 行）
   - 检查 AI 是否正确识别表头位置

3. **数据提取验证**
   - 检查提取的字段是否准确
   - 检查原始内容是否完整保存

4. **去重验证**
   - 导入包含重复手机号的数据
   - 检查数据库中是否只保留一条记录

5. **并行处理验证**
   - 同时导入多个文件
   - 检查是否并行处理
   - 检查进度显示是否正确

6. **暂停/恢复验证**
   - 处理过程中点击暂停
   - 检查是否停止处理
   - 点击恢复，检查是否继续处理

7. **结果页面验证**
   - 检查数据是否正确显示
   - 测试编辑功能
   - 测试导出功能
   - 测试筛选和搜索

### 2. 性能验证

1. 测试处理 10 万条数据的时间
2. 测试并行处理 10 个文件的性能
3. 测试数据库查询性能（分页、筛选）

### 3. 错误处理验证

1. 测试 AI API 调用失败的情况
2. 测试无效的 Excel 文件
3. 测试网络断开的情况
4. 测试磁盘空间不足的情况

## 关键文件路径

### 前端关键文件
- `src/views/ProcessingView.vue` - 处理界面
- `src/views/ResultView.vue` - 结果页面
- `src/stores/processing.ts` - 处理状态管理
- `src/stores/result.ts` - 结果状态管理
- `src/stores/config.ts` - 配置状态管理

### 后端关键文件
- `src-tauri/src/services/extractor.rs` - 核心提取逻辑
- `src-tauri/src/services/ai_client.rs` - AI 客户端
- `src-tauri/src/services/excel_parser.rs` - Excel 解析
- `src-tauri/src/services/storage.rs` - 数据存储
- `src-tauri/src/commands/processing.rs` - 处理命令
- `src-tauri/src/db/schema.rs` - 数据库表结构

## 风险和注意事项

1. **AI 调用成本**
   - 百万级数据调用成本较高
   - 建议：提供本地模型选项或批量处理优化

2. **AI 提取准确率**
   - 复杂格式可能识别不准确
   - 建议：提供手动校验和修正功能（已包含在结果页面）

3. **性能瓶颈**
   - 大文件处理可能卡顿
   - 建议：使用流式处理、分批加载

4. **错误处理**
   - 需要完善的错误日志和重试机制
   - 建议：记录详细的错误信息，方便排查

5. **数据安全**
   - API Key 需要加密存储
   - 建议：使用 Tauri 的安全存储 API

## 总结

本计划详细描述了智能表格数据提取系统的完整实施方案，包括技术架构、数据库设计、核心流程、实施步骤和验证方法。整个项目预计需要 15 天完成，分为 9 个阶段逐步实施。

核心技术栈：
- 前端：Vue 3 + TypeScript + Vite + Pinia + Element Plus
- 桌面框架：Tauri 2.x
- 后端：Rust + SQLite + OpenAI SDK
- 关键库：calamine（Excel）、rusqlite（数据库）、reqwest（HTTP）、tokio（异步）

关键特性：
- 并行处理多个文件
- 实时进度更新
- 暂停/恢复/取消控制
- 按手机号自动去重
- 可视化处理界面
- 结果编辑和导出

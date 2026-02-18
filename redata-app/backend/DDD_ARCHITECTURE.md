# reData Rust 后端 DDD 架构设计

## 一、DDD 架构概述

本项目采用领域驱动设计（Domain-Driven Design, DDD）架构，将系统分为四个主要层次：

```
┌─────────────────────────────────────────────────────────┐
│              Presentation Layer (表现层)                 │
│  - API Controllers (HTTP/WebSocket)                     │
│  - Request/Response DTOs                                │
│  - Middleware (CORS, Logging, Auth)                     │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│             Application Layer (应用层)                   │
│  - Use Cases (业务用例)                                  │
│  - Application Services                                 │
│  - Commands & Queries (CQRS)                            │
│  - DTOs (数据传输对象)                                   │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│               Domain Layer (领域层)                      │
│  - Entities (实体)                                       │
│  - Value Objects (值对象)                                │
│  - Domain Services (领域服务)                            │
│  - Repository Interfaces (仓储接口)                      │
│  - Domain Events (领域事件)                              │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│           Infrastructure Layer (基础设施层)              │
│  - Repository Implementations (仓储实现)                 │
│  - Database (SeaORM)                                    │
│  - External Services (AI Client, Excel Parser)          │
│  - Configuration                                        │
└─────────────────────────────────────────────────────────┘
```

## 二、目录结构

```
src-tauri/src/backend/
├── mod.rs                          # 后端模块入口
│
├── domain/                         # 领域层（核心业务逻辑）
│   ├── mod.rs
│   ├── entities/                   # 实体
│   │   ├── mod.rs
│   │   ├── project.rs              # 项目实体
│   │   ├── field.rs                # 字段实体
│   │   ├── ai_config.rs            # AI 配置实体
│   │   ├── processing_task.rs      # 处理任务实体
│   │   └── record.rs               # 数据记录实体
│   ├── value_objects/              # 值对象
│   │   ├── mod.rs
│   │   ├── field_type.rs           # 字段类型
│   │   ├── task_status.rs          # 任务状态
│   │   ├── dedup_strategy.rs       # 去重策略
│   │   └── validation_rule.rs      # 验证规则
│   ├── repositories/               # 仓储接口（trait）
│   │   ├── mod.rs
│   │   ├── project_repository.rs
│   │   ├── field_repository.rs
│   │   ├── ai_config_repository.rs
│   │   ├── task_repository.rs
│   │   └── record_repository.rs
│   ├── services/                   # 领域服务
│   │   ├── mod.rs
│   │   ├── field_validator.rs      # 字段验证服务
│   │   ├── dedup_service.rs        # 去重服务
│   │   └── column_mapper.rs        # 列映射服务
│   └── events/                     # 领域事件
│       ├── mod.rs
│       ├── project_created.rs
│       ├── field_added.rs
│       └── task_completed.rs
│
├── application/                    # 应用层（用例编排）
│   ├── mod.rs
│   ├── use_cases/                  # 用例
│   │   ├── mod.rs
│   │   ├── project/
│   │   │   ├── mod.rs
│   │   │   ├── create_project.rs
│   │   │   ├── update_project.rs
│   │   │   ├── delete_project.rs
│   │   │   └── list_projects.rs
│   │   ├── field/
│   │   │   ├── mod.rs
│   │   │   ├── add_field.rs
│   │   │   ├── update_field.rs
│   │   │   ├── delete_field.rs
│   │   │   └── generate_field_metadata.rs
│   │   ├── processing/
│   │   │   ├── mod.rs
│   │   │   ├── start_processing.rs
│   │   │   ├── pause_task.rs
│   │   │   ├── resume_task.rs
│   │   │   └── cancel_task.rs
│   │   └── result/
│   │       ├── mod.rs
│   │       ├── query_results.rs
│   │       ├── update_record.rs
│   │       └── export_results.rs
│   ├── dtos/                       # 数据传输对象
│   │   ├── mod.rs
│   │   ├── project_dto.rs
│   │   ├── field_dto.rs
│   │   ├── task_dto.rs
│   │   └── record_dto.rs
│   ├── commands/                   # 命令（写操作）
│   │   ├── mod.rs
│   │   ├── create_project_command.rs
│   │   ├── add_field_command.rs
│   │   └── start_processing_command.rs
│   └── queries/                    # 查询（读操作）
│       ├── mod.rs
│       ├── get_project_query.rs
│       ├── list_fields_query.rs
│       └── get_results_query.rs
│
├── infrastructure/                 # 基础设施层（技术实现）
│   ├── mod.rs
│   ├── persistence/                # 持久化
│   │   ├── mod.rs
│   │   ├── database.rs             # 数据库连接
│   │   ├── migrations.rs           # 数据库迁移
│   │   ├── repositories/           # 仓储实现
│   │   │   ├── mod.rs
│   │   │   ├── project_repo_impl.rs
│   │   │   ├── field_repo_impl.rs
│   │   │   ├── ai_config_repo_impl.rs
│   │   │   ├── task_repo_impl.rs
│   │   │   └── record_repo_impl.rs
│   │   └── models/                 # ORM 模型
│   │       ├── mod.rs
│   │       ├── project_model.rs
│   │       ├── field_model.rs
│   │       └── task_model.rs
│   ├── external_services/          # 外部服务
│   │   ├── mod.rs
│   │   ├── ai_client.rs            # AI 客户端（async-openai）
│   │   ├── excel_parser.rs         # Excel 解析器（calamine）
│   │   └── file_storage.rs         # 文件存储
│   └── config/                     # 配置
│       ├── mod.rs
│       ├── app_config.rs
│       └── database_config.rs
│
└── presentation/                   # 表现层（API 接口）
    ├── mod.rs
    ├── api/                        # API 路由
    │   ├── mod.rs
    │   ├── projects.rs             # 项目 API
    │   ├── fields.rs               # 字段 API
    │   ├── ai_configs.rs           # AI 配置 API
    │   ├── files.rs                # 文件 API
    │   ├── processing.rs           # 处理任务 API
    │   ├── results.rs              # 结果 API
    │   └── health.rs               # 健康检查
    └── middleware/                 # 中间件
        ├── mod.rs
        ├── cors.rs
        ├── logging.rs
        └── error_handler.rs
```

## 三、各层职责详解

### 3.1 Domain Layer（领域层）

**职责**：包含核心业务逻辑和业务规则，独立于技术实现。

#### Entities（实体）
- 具有唯一标识的业务对象
- 包含业务逻辑和验证规则
- 示例：`Project`, `Field`, `ProcessingTask`

```rust
// domain/entities/project.rs
pub struct Project {
    id: ProjectId,
    name: String,
    description: Option<String>,
    dedup_enabled: bool,
    dedup_strategy: DedupStrategy,
    created_at: DateTime<Utc>,
}

impl Project {
    pub fn new(name: String, description: Option<String>) -> Result<Self, DomainError> {
        // 业务规则验证
        if name.is_empty() {
            return Err(DomainError::InvalidProjectName);
        }

        Ok(Self {
            id: ProjectId::new(),
            name,
            description,
            dedup_enabled: false,
            dedup_strategy: DedupStrategy::Skip,
            created_at: Utc::now(),
        })
    }

    pub fn enable_dedup(&mut self, strategy: DedupStrategy) {
        self.dedup_enabled = true;
        self.dedup_strategy = strategy;
    }
}
```

#### Value Objects（值对象）
- 没有唯一标识，通过属性值来区分
- 不可变对象
- 示例：`FieldType`, `TaskStatus`, `DedupStrategy`

```rust
// domain/value_objects/field_type.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    Text,
    Number,
    Phone,
    Email,
    Url,
    Date,
}

impl FieldType {
    pub fn validation_pattern(&self) -> Option<&str> {
        match self {
            FieldType::Phone => Some(r"^1[3-9]\d{9}$"),
            FieldType::Email => Some(r"^[\w\.-]+@[\w\.-]+\.\w+$"),
            FieldType::Url => Some(r"^https?://"),
            _ => None,
        }
    }
}
```

#### Repository Interfaces（仓储接口）
- 定义数据访问的抽象接口
- 不包含具体实现

```rust
// domain/repositories/project_repository.rs
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError>;
    async fn find_all(&self) -> Result<Vec<Project>, RepositoryError>;
    async fn save(&self, project: &Project) -> Result<(), RepositoryError>;
    async fn delete(&self, id: &ProjectId) -> Result<(), RepositoryError>;
}
```

#### Domain Services（领域服务）
- 不属于任何实体的业务逻辑
- 协调多个实体的操作

```rust
// domain/services/dedup_service.rs
pub struct DedupService;

impl DedupService {
    pub fn check_duplicate(
        &self,
        record: &Record,
        existing_records: &[Record],
        dedup_keys: &[String],
    ) -> Option<RecordId> {
        // 去重逻辑
    }
}
```

### 3.2 Application Layer（应用层）

**职责**：编排用例流程，协调领域对象和基础设施服务。

#### Use Cases（用例）
- 实现具体的业务用例
- 调用领域服务和仓储
- 处理事务边界

```rust
// application/use_cases/project/create_project.rs
pub struct CreateProjectUseCase<R: ProjectRepository> {
    project_repo: Arc<R>,
}

impl<R: ProjectRepository> CreateProjectUseCase<R> {
    pub async fn execute(&self, command: CreateProjectCommand) -> Result<ProjectDto, AppError> {
        // 1. 创建领域实体
        let project = Project::new(command.name, command.description)?;

        // 2. 保存到仓储
        self.project_repo.save(&project).await?;

        // 3. 返回 DTO
        Ok(ProjectDto::from(project))
    }
}
```

#### Commands & Queries（命令和查询）
- CQRS 模式
- Commands：修改状态的操作
- Queries：只读查询操作

```rust
// application/commands/create_project_command.rs
pub struct CreateProjectCommand {
    pub name: String,
    pub description: Option<String>,
}

// application/queries/get_project_query.rs
pub struct GetProjectQuery {
    pub project_id: ProjectId,
}
```

### 3.3 Infrastructure Layer（基础设施层）

**职责**：提供技术实现，如数据库访问、外部服务调用等。

#### Repository Implementations（仓储实现）
- 实现领域层定义的仓储接口
- 使用 SeaORM 进行数据库操作

```rust
// infrastructure/persistence/repositories/project_repo_impl.rs
pub struct ProjectRepositoryImpl {
    db: DatabaseConnection,
}

#[async_trait]
impl ProjectRepository for ProjectRepositoryImpl {
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError> {
        let model = project::Entity::find_by_id(id.value())
            .one(&self.db)
            .await?;

        Ok(model.map(|m| m.into_domain()))
    }

    async fn save(&self, project: &Project) -> Result<(), RepositoryError> {
        let model = ProjectModel::from_domain(project);
        model.insert(&self.db).await?;
        Ok(())
    }
}
```

#### External Services（外部服务）
- AI 客户端（async-openai）
- Excel 解析器（calamine）
- 文件存储

```rust
// infrastructure/external_services/ai_client.rs
pub struct AiClient {
    client: async_openai::Client,
}

impl AiClient {
    pub async fn analyze_column_mapping(
        &self,
        sample_rows: Vec<Vec<String>>,
        fields: Vec<Field>,
    ) -> Result<ColumnMapping, AiError> {
        // 调用 OpenAI API
    }
}
```

### 3.4 Presentation Layer（表现层）

**职责**：处理 HTTP 请求，调用应用层用例，返回响应。

#### API Controllers（API 控制器）
- 定义 HTTP 路由
- 请求验证
- 调用用例
- 响应序列化

```rust
// presentation/api/projects.rs
pub fn create_routes() -> Router {
    Router::new()
        .route("/", get(list_projects).post(create_project))
        .route("/:id", get(get_project).put(update_project).delete(delete_project))
}

async fn create_project(
    State(use_case): State<Arc<CreateProjectUseCase<ProjectRepositoryImpl>>>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<ProjectResponse>, AppError> {
    let command = CreateProjectCommand {
        name: req.name,
        description: req.description,
    };

    let dto = use_case.execute(command).await?;
    Ok(Json(ProjectResponse::from(dto)))
}
```

## 四、依赖关系

```
Presentation → Application → Domain
                ↓
         Infrastructure
```

- **Presentation** 依赖 **Application**
- **Application** 依赖 **Domain**
- **Infrastructure** 实现 **Domain** 的接口
- **Domain** 不依赖任何其他层（纯业务逻辑）

## 五、关键设计模式

### 5.1 Repository Pattern（仓储模式）
- 领域层定义接口
- 基础设施层实现接口
- 解耦业务逻辑和数据访问

### 5.2 CQRS（命令查询职责分离）
- Commands：修改状态
- Queries：只读查询
- 分离读写关注点

### 5.3 Dependency Injection（依赖注入）
- 使用 `Arc<dyn Trait>` 注入依赖
- 便于测试和替换实现

### 5.4 Domain Events（领域事件）
- 解耦领域对象之间的依赖
- 实现最终一致性

## 六、数据流示例

### 创建项目的完整流程

```
1. HTTP Request
   ↓
2. Presentation Layer (API Controller)
   - 接收请求
   - 验证请求格式
   ↓
3. Application Layer (Use Case)
   - 创建 Command
   - 调用领域服务
   ↓
4. Domain Layer (Entity)
   - 创建 Project 实体
   - 执行业务规则验证
   ↓
5. Infrastructure Layer (Repository)
   - 将实体转换为 ORM 模型
   - 保存到数据库
   ↓
6. Application Layer (Use Case)
   - 将实体转换为 DTO
   - 返回结果
   ↓
7. Presentation Layer (API Controller)
   - 将 DTO 转换为 Response
   - 返回 HTTP 响应
```

## 七、优势

1. **关注点分离**：每一层都有明确的职责
2. **可测试性**：领域逻辑独立，易于单元测试
3. **可维护性**：业务逻辑集中在领域层
4. **可扩展性**：易于添加新功能和替换实现
5. **技术无关性**：领域层不依赖具体技术

## 八、实施步骤

1. **Phase 1**: 搭建基础架构和目录结构
2. **Phase 2**: 实现领域层（实体、值对象、仓储接口）
3. **Phase 3**: 实现基础设施层（仓储实现、数据库）
4. **Phase 4**: 实现应用层（用例、Commands/Queries）
5. **Phase 5**: 实现表现层（API 控制器）
6. **Phase 6**: 集成和测试

---

**文档版本**: v1.0
**创建日期**: 2026-02-18
**架构模式**: Domain-Driven Design (DDD)

// ORM 模型模块

pub mod ai_config;
pub mod batch;
pub mod field;
pub mod project;
pub mod project_group;
pub mod record;
pub mod task;
pub mod task_file_progress;

// 导出实体
pub use ai_config::Entity as AiConfig;
pub use batch::Entity as Batch;
pub use field::Entity as ProjectField;
pub use project::Entity as Project;
pub use project_group::Entity as ProjectGroup;
pub use record::Entity as ProjectRecord;
pub use task::Entity as ProcessingTask;
pub use task_file_progress::Entity as TaskFileProgress;

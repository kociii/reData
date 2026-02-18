// ORM 模型模块

pub mod ai_config;
pub mod batch;
pub mod field;
pub mod project;
pub mod task;

// 导出实体
pub use ai_config::Entity as AiConfig;
pub use batch::Entity as Batch;
pub use field::Entity as ProjectField;
pub use project::Entity as Project;
pub use task::Entity as ProcessingTask;

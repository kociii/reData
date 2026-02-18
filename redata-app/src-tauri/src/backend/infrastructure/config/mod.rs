// config module

pub mod error;
pub mod logging;

// 导出常用类型
pub use error::{AppError, Result};
pub use logging::init_logging;

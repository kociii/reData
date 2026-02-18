// CORS 中间件配置

use tower_http::cors::{Any, CorsLayer};

/// 创建 CORS 层
pub fn create_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

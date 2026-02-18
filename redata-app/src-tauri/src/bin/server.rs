// Rust 后端服务器启动程序
// 用于独立测试 Rust 后端

use redata_lib::backend;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 启动服务器在端口 8001
    backend::run_server(8001).await
}

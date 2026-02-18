from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from .db.base import init_db
from .api import projects, fields, ai_configs, files, processing, results

app = FastAPI(
    title="reData Backend API",
    description="智能数据处理平台后端服务",
    version="0.2.0"
)

# CORS 配置（允许 Tauri 前端访问）
app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://localhost:3000",
        "http://127.0.0.1:3000",
        "tauri://localhost",
        "https://tauri.localhost",
        "http://tauri.localhost",
        "https://localhost",  # Tauri 2.x dev mode
    ],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# 初始化数据库
@app.on_event("startup")
async def startup_event():
    init_db()

# 注册路由
app.include_router(projects.router, prefix="/api/projects", tags=["projects"])
app.include_router(fields.router, prefix="/api/fields", tags=["fields"])
app.include_router(ai_configs.router, prefix="/api/ai-configs", tags=["ai-configs"])
app.include_router(files.router, prefix="/api/files", tags=["files"])
app.include_router(processing.router, prefix="/api/processing", tags=["processing"])
app.include_router(results.router, prefix="/api/results", tags=["results"])

@app.get("/")
async def root():
    return {
        "message": "reData Backend API",
        "version": "0.2.0",
        "docs": "/docs",
        "features": [
            "项目管理",
            "字段定义",
            "AI 配置",
            "文件上传",
            "数据处理",
            "结果查询"
        ]
    }

@app.get("/health")
async def health():
    return {"status": "ok"}

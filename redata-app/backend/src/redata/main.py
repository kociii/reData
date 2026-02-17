from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from .db.base import init_db
from .api import projects, fields, ai_configs

app = FastAPI(
    title="reData Backend API",
    description="智能数据处理平台后端服务",
    version="0.1.0"
)

# CORS 配置（允许 Tauri 前端访问）
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:3000", "tauri://localhost"],
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

@app.get("/")
async def root():
    return {"message": "reData Backend API", "version": "0.1.0"}

@app.get("/health")
async def health():
    return {"status": "ok"}

"""
处理任务 API 路由
处理任务启动、暂停、恢复、取消和状态查询
"""

import asyncio
import json
from typing import List, Optional
from fastapi import APIRouter, HTTPException, Depends, WebSocket, WebSocketDisconnect
from pydantic import BaseModel
from sqlalchemy.orm import Session

from ..db.base import get_db
from ..models.project import Project, ProcessingTask, AiConfig
from ..services.extractor import (
    Extractor, ProcessingProgress, ProcessingResult,
    get_extractor, register_extractor, unregister_extractor
)


router = APIRouter()


# ========== Schemas ==========

class StartProcessingRequest(BaseModel):
    """启动处理请求"""
    project_id: int
    file_paths: List[str]
    ai_config_id: Optional[int] = None


class TaskStatusResponse(BaseModel):
    """任务状态响应"""
    task_id: str
    project_id: int
    status: str
    total_files: int
    processed_files: int
    total_rows: int
    processed_rows: int
    success_count: int
    error_count: int
    batch_number: Optional[str]
    message: Optional[str] = None


class TaskListResponse(BaseModel):
    """任务列表响应"""
    tasks: List[TaskStatusResponse]
    total: int


# ========== WebSocket 连接管理 ==========

class ConnectionManager:
    """WebSocket 连接管理器"""

    def __init__(self):
        self.active_connections: dict[str, List[WebSocket]] = {}

    async def connect(self, websocket: WebSocket, task_id: str):
        """建立连接"""
        await websocket.accept()
        if task_id not in self.active_connections:
            self.active_connections[task_id] = []
        self.active_connections[task_id].append(websocket)

    def disconnect(self, websocket: WebSocket, task_id: str):
        """断开连接"""
        if task_id in self.active_connections:
            if websocket in self.active_connections[task_id]:
                self.active_connections[task_id].remove(websocket)
            if not self.active_connections[task_id]:
                del self.active_connections[task_id]

    async def broadcast(self, task_id: str, message: dict):
        """广播消息到指定任务的所有连接"""
        if task_id in self.active_connections:
            for connection in self.active_connections[task_id]:
                try:
                    await connection.send_json(message)
                except Exception:
                    pass


manager = ConnectionManager()


# ========== Routes ==========

@router.post("/start", response_model=TaskStatusResponse)
async def start_processing(
    request: StartProcessingRequest,
    db: Session = Depends(get_db)
):
    """
    启动处理任务

    Args:
        request: 启动请求
        db: 数据库 Session

    Returns:
        TaskStatusResponse
    """
    # 检查项目是否存在
    project = db.query(Project).filter(Project.id == request.project_id).first()
    if not project:
        raise HTTPException(status_code=404, detail="项目不存在")

    # 获取 AI 配置
    if request.ai_config_id:
        ai_config = db.query(AiConfig).filter(AiConfig.id == request.ai_config_id).first()
    else:
        # 使用默认配置
        ai_config = db.query(AiConfig).filter(AiConfig.is_default == True).first()

    if not ai_config:
        raise HTTPException(status_code=400, detail="未找到 AI 配置")

    # 创建提取器
    extractor = Extractor(
        db=db,
        project=project,
        ai_config=ai_config,
        progress_callback=lambda p: asyncio.create_task(
            broadcast_progress(p)
        )
    )

    # 注册提取器
    register_extractor(extractor.task_id, extractor)

    # 启动后台处理任务
    asyncio.create_task(
        run_processing(extractor, request.file_paths)
    )

    return TaskStatusResponse(
        task_id=extractor.task_id,
        project_id=project.id,
        status="processing",
        total_files=len(request.file_paths),
        processed_files=0,
        total_rows=0,
        processed_rows=0,
        success_count=0,
        error_count=0,
        batch_number=extractor.batch_number
    )


async def run_processing(extractor: Extractor, file_paths: List[str]):
    """
    运行处理任务

    Args:
        extractor: 提取器实例
        file_paths: 文件路径列表
    """
    try:
        result = await extractor.process_files(file_paths)
        # 发送完成通知
        await manager.broadcast(extractor.task_id, {
            "event": "completed",
            "task_id": result.task_id,
            "success": result.success,
            "total_rows": result.total_rows,
            "success_count": result.success_count,
            "error_count": result.error_count
        })
    except Exception as e:
        await manager.broadcast(extractor.task_id, {
            "event": "error",
            "task_id": extractor.task_id,
            "message": str(e)
        })
    finally:
        unregister_extractor(extractor.task_id)


async def broadcast_progress(progress: ProcessingProgress):
    """
    广播进度更新

    Args:
        progress: 进度数据
    """
    await manager.broadcast(progress.task_id, {
        "event": progress.event,
        "task_id": progress.task_id,
        "current_file": progress.current_file,
        "current_sheet": progress.current_sheet,
        "current_row": progress.current_row,
        "total_rows": progress.total_rows,
        "processed_rows": progress.processed_rows,
        "success_count": progress.success_count,
        "error_count": progress.error_count,
        "speed": progress.speed,
        "message": progress.message
    })


@router.post("/pause/{task_id}", response_model=TaskStatusResponse)
async def pause_processing(task_id: str, db: Session = Depends(get_db)):
    """
    暂停处理任务

    Args:
        task_id: 任务 ID
        db: 数据库 Session

    Returns:
        TaskStatusResponse
    """
    extractor = get_extractor(task_id)
    if not extractor:
        raise HTTPException(status_code=404, detail="任务不存在或已完成")

    extractor.pause()

    # 更新数据库状态
    task = db.query(ProcessingTask).filter(ProcessingTask.id == task_id).first()
    if task:
        task.status = "paused"
        db.commit()

    return get_task_status(task_id, db)


@router.post("/resume/{task_id}", response_model=TaskStatusResponse)
async def resume_processing(task_id: str, db: Session = Depends(get_db)):
    """
    恢复处理任务

    Args:
        task_id: 任务 ID
        db: 数据库 Session

    Returns:
        TaskStatusResponse
    """
    extractor = get_extractor(task_id)
    if not extractor:
        raise HTTPException(status_code=404, detail="任务不存在或已完成")

    extractor.resume()

    # 更新数据库状态
    task = db.query(ProcessingTask).filter(ProcessingTask.id == task_id).first()
    if task:
        task.status = "processing"
        db.commit()

    return get_task_status(task_id, db)


@router.post("/cancel/{task_id}", response_model=TaskStatusResponse)
async def cancel_processing(task_id: str, db: Session = Depends(get_db)):
    """
    取消处理任务

    Args:
        task_id: 任务 ID
        db: 数据库 Session

    Returns:
        TaskStatusResponse
    """
    extractor = get_extractor(task_id)
    if extractor:
        extractor.cancel()
        unregister_extractor(task_id)

    # 更新数据库状态
    task = db.query(ProcessingTask).filter(ProcessingTask.id == task_id).first()
    if task:
        task.status = "cancelled"
        db.commit()

    return get_task_status(task_id, db)


@router.get("/status/{task_id}", response_model=TaskStatusResponse)
async def get_status(task_id: str, db: Session = Depends(get_db)):
    """
    获取任务状态

    Args:
        task_id: 任务 ID
        db: 数据库 Session

    Returns:
        TaskStatusResponse
    """
    return get_task_status(task_id, db)


def get_task_status(task_id: str, db: Session) -> TaskStatusResponse:
    """
    获取任务状态（内部函数）

    Args:
        task_id: 任务 ID
        db: 数据库 Session

    Returns:
        TaskStatusResponse
    """
    task = db.query(ProcessingTask).filter(ProcessingTask.id == task_id).first()
    if not task:
        raise HTTPException(status_code=404, detail="任务不存在")

    return TaskStatusResponse(
        task_id=task.id,
        project_id=task.project_id,
        status=task.status,
        total_files=task.total_files,
        processed_files=task.processed_files,
        total_rows=task.total_rows,
        processed_rows=task.processed_rows,
        success_count=task.success_count,
        error_count=task.error_count,
        batch_number=task.batch_number
    )


@router.get("/list/{project_id}", response_model=TaskListResponse)
async def list_tasks(
    project_id: int,
    status: Optional[str] = None,
    limit: int = 20,
    offset: int = 0,
    db: Session = Depends(get_db)
):
    """
    获取项目的任务列表

    Args:
        project_id: 项目 ID
        status: 状态筛选
        limit: 限制数量
        offset: 偏移量
        db: 数据库 Session

    Returns:
        TaskListResponse
    """
    query = db.query(ProcessingTask).filter(ProcessingTask.project_id == project_id)

    if status:
        query = query.filter(ProcessingTask.status == status)

    total = query.count()
    tasks = query.order_by(ProcessingTask.created_at.desc()).offset(offset).limit(limit).all()

    return TaskListResponse(
        tasks=[
            TaskStatusResponse(
                task_id=task.id,
                project_id=task.project_id,
                status=task.status,
                total_files=task.total_files,
                processed_files=task.processed_files,
                total_rows=task.total_rows,
                processed_rows=task.processed_rows,
                success_count=task.success_count,
                error_count=task.error_count,
                batch_number=task.batch_number
            )
            for task in tasks
        ],
        total=total
    )


@router.websocket("/ws/progress/{task_id}")
async def websocket_progress(websocket: WebSocket, task_id: str):
    """
    WebSocket 进度推送

    Args:
        websocket: WebSocket 连接
        task_id: 任务 ID
    """
    await manager.connect(websocket, task_id)

    try:
        while True:
            # 保持连接，等待进度更新
            data = await websocket.receive_text()

            # 处理客户端消息（如心跳）
            try:
                message = json.loads(data)
                if message.get("type") == "ping":
                    await websocket.send_json({"type": "pong"})
            except json.JSONDecodeError:
                pass

    except WebSocketDisconnect:
        manager.disconnect(websocket, task_id)
    except Exception:
        manager.disconnect(websocket, task_id)

"""
结果 API 路由
处理结果查询、更新、删除和导出
"""

from typing import Optional, List, Dict, Any
from fastapi import APIRouter, HTTPException, Depends, Query
from fastapi.responses import Response
from pydantic import BaseModel
from sqlalchemy.orm import Session

from ..db.base import get_db
from ..models.project import Project
from ..services.storage import StorageService, StorageError


router = APIRouter()


# ========== Schemas ==========

class ResultsQuery(BaseModel):
    """结果查询参数"""
    page: int = 1
    page_size: int = 50
    batch_number: Optional[str] = None
    status: Optional[str] = None
    search: Optional[str] = None
    order_by: str = "id"
    order_desc: bool = True


class RecordUpdate(BaseModel):
    """记录更新"""
    data: Dict[str, Any]


class RecordResponse(BaseModel):
    """记录响应"""
    id: int
    data: Dict[str, Any]
    raw_content: Optional[str] = None
    source_file: Optional[str] = None
    source_sheet: Optional[str] = None
    row_number: Optional[int] = None
    batch_number: Optional[str] = None
    status: Optional[str] = None
    error_message: Optional[str] = None
    created_at: Optional[str] = None
    updated_at: Optional[str] = None


class ResultsResponse(BaseModel):
    """结果响应"""
    total: int
    page: int
    page_size: int
    records: List[Dict[str, Any]]


class ExportRequest(BaseModel):
    """导出请求"""
    format: str = "xlsx"  # xlsx 或 csv
    batch_number: Optional[str] = None


class StatisticsResponse(BaseModel):
    """统计响应"""
    total_records: int
    success_count: int
    error_count: int
    batch_count: int


# ========== Routes ==========

@router.get("/{project_id}", response_model=ResultsResponse)
async def query_results(
    project_id: int,
    page: int = Query(1, ge=1),
    page_size: int = Query(50, ge=1, le=500),
    batch_number: Optional[str] = None,
    status: Optional[str] = None,
    search: Optional[str] = None,
    order_by: str = Query("id"),
    order_desc: bool = Query(True),
    db: Session = Depends(get_db)
):
    """
    查询结果（分页）

    Args:
        project_id: 项目 ID
        page: 页码
        page_size: 每页数量
        batch_number: 批次号筛选
        status: 状态筛选
        search: 搜索关键词
        order_by: 排序字段
        order_desc: 是否降序
        db: 数据库 Session

    Returns:
        ResultsResponse
    """
    # 检查项目是否存在
    project = db.query(Project).filter(Project.id == project_id).first()
    if not project:
        raise HTTPException(status_code=404, detail="项目不存在")

    storage = StorageService(db)

    # 检查表是否存在
    if not storage.table_exists(project_id):
        return ResultsResponse(
            total=0,
            page=page,
            page_size=page_size,
            records=[]
        )

    # 查询记录
    result = storage.query_records(
        project_id=project_id,
        page=page,
        page_size=page_size,
        batch_number=batch_number,
        status=status,
        search=search,
        order_by=order_by,
        order_desc=order_desc
    )

    return ResultsResponse(
        total=result.total,
        page=result.page,
        page_size=result.page_size,
        records=result.records
    )


@router.get("/{project_id}/{record_id}", response_model=RecordResponse)
async def get_record(
    project_id: int,
    record_id: int,
    db: Session = Depends(get_db)
):
    """
    获取单条记录

    Args:
        project_id: 项目 ID
        record_id: 记录 ID
        db: 数据库 Session

    Returns:
        RecordResponse
    """
    storage = StorageService(db)
    record = storage.get_record(project_id, record_id)

    if not record:
        raise HTTPException(status_code=404, detail="记录不存在")

    # 分离数据字段和元数据
    meta_fields = {
        "id", "raw_content", "source_file", "source_sheet",
        "row_number", "batch_number", "status", "error_message",
        "created_at", "updated_at"
    }

    data = {k: v for k, v in record.items() if k not in meta_fields}

    return RecordResponse(
        id=record.get("id"),
        data=data,
        raw_content=record.get("raw_content"),
        source_file=record.get("source_file"),
        source_sheet=record.get("source_sheet"),
        row_number=record.get("row_number"),
        batch_number=record.get("batch_number"),
        status=record.get("status"),
        error_message=record.get("error_message"),
        created_at=str(record.get("created_at")) if record.get("created_at") else None,
        updated_at=str(record.get("updated_at")) if record.get("updated_at") else None
    )


@router.put("/{project_id}/{record_id}")
async def update_record(
    project_id: int,
    record_id: int,
    update: RecordUpdate,
    db: Session = Depends(get_db)
):
    """
    更新记录

    Args:
        project_id: 项目 ID
        record_id: 记录 ID
        update: 更新数据
        db: 数据库 Session

    Returns:
        成功消息
    """
    storage = StorageService(db)

    # 检查记录是否存在
    record = storage.get_record(project_id, record_id)
    if not record:
        raise HTTPException(status_code=404, detail="记录不存在")

    # 更新记录
    success = storage.update_record(project_id, record_id, update.data)

    if not success:
        raise HTTPException(status_code=500, detail="更新失败")

    return {"message": "更新成功", "record_id": record_id}


@router.delete("/{project_id}/{record_id}")
async def delete_record(
    project_id: int,
    record_id: int,
    db: Session = Depends(get_db)
):
    """
    删除记录

    Args:
        project_id: 项目 ID
        record_id: 记录 ID
        db: 数据库 Session

    Returns:
        成功消息
    """
    storage = StorageService(db)

    # 检查记录是否存在
    record = storage.get_record(project_id, record_id)
    if not record:
        raise HTTPException(status_code=404, detail="记录不存在")

    # 删除记录
    success = storage.delete_record(project_id, record_id)

    if not success:
        raise HTTPException(status_code=500, detail="删除失败")

    return {"message": "删除成功", "record_id": record_id}


@router.delete("/{project_id}/batch/{batch_number}")
async def delete_batch_records(
    project_id: int,
    batch_number: str,
    db: Session = Depends(get_db)
):
    """
    删除批次所有记录

    Args:
        project_id: 项目 ID
        batch_number: 批次号
        db: 数据库 Session

    Returns:
        成功消息
    """
    storage = StorageService(db)

    if not storage.table_exists(project_id):
        raise HTTPException(status_code=404, detail="项目数据表不存在")

    # 查询批次记录
    result = storage.query_records(
        project_id=project_id,
        page=1,
        page_size=1000000,
        batch_number=batch_number
    )

    if not result.records:
        raise HTTPException(status_code=404, detail="批次记录不存在")

    # 删除所有记录
    deleted = 0
    for record in result.records:
        if storage.delete_record(project_id, record["id"]):
            deleted += 1

    return {"message": f"已删除 {deleted} 条记录", "batch_number": batch_number}


@router.get("/export/{project_id}")
async def export_results(
    project_id: int,
    format: str = Query("xlsx", pattern="^(xlsx|csv)$"),
    batch_number: Optional[str] = None,
    db: Session = Depends(get_db)
):
    """
    导出结果

    Args:
        project_id: 项目 ID
        format: 导出格式（xlsx 或 csv）
        batch_number: 批次号筛选
        db: 数据库 Session

    Returns:
        文件下载响应
    """
    # 检查项目是否存在
    project = db.query(Project).filter(Project.id == project_id).first()
    if not project:
        raise HTTPException(status_code=404, detail="项目不存在")

    storage = StorageService(db)

    if not storage.table_exists(project_id):
        raise HTTPException(status_code=404, detail="没有可导出的数据")

    # 导出数据
    file_data = storage.export_records(
        project_id=project_id,
        format=format,
        batch_number=batch_number
    )

    if not file_data:
        raise HTTPException(status_code=404, detail="没有可导出的数据")

    # 设置响应头
    filename = f"{project.name}_results.{format}"
    content_type = (
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        if format == "xlsx"
        else "text/csv"
    )

    return Response(
        content=file_data,
        media_type=content_type,
        headers={
            "Content-Disposition": f'attachment; filename="{filename}"'
        }
    )


@router.get("/statistics/{project_id}", response_model=StatisticsResponse)
async def get_statistics(
    project_id: int,
    db: Session = Depends(get_db)
):
    """
    获取项目统计信息

    Args:
        project_id: 项目 ID
        db: 数据库 Session

    Returns:
        StatisticsResponse
    """
    # 检查项目是否存在
    project = db.query(Project).filter(Project.id == project_id).first()
    if not project:
        raise HTTPException(status_code=404, detail="项目不存在")

    storage = StorageService(db)

    total_records = storage.get_record_count(project_id)

    # 统计成功和失败数量
    success_count = 0
    error_count = 0
    batch_count = 0

    if storage.table_exists(project_id):
        # 获取所有记录统计
        success_result = storage.query_records(project_id, page=1, page_size=1, status="success")
        success_count = success_result.total

        error_result = storage.query_records(project_id, page=1, page_size=1, status="error")
        error_count = error_result.total

        # 获取批次数量（从数据库查询）
        from ..models.project import Batch
        batch_count = db.query(Batch).filter(Batch.project_id == project_id).count()

    return StatisticsResponse(
        total_records=total_records,
        success_count=success_count,
        error_count=error_count,
        batch_count=batch_count
    )

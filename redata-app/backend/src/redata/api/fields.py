from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session
from typing import List
from datetime import datetime
from ..db.base import get_db
from ..models.project import ProjectField, Project, AiConfig
from ..models.schemas import (
    ProjectFieldCreate,
    ProjectFieldUpdate,
    ProjectFieldResponse,
    GenerateFieldMetadataRequest,
    GenerateFieldMetadataResponse
)
from ..services.ai_client import AIClient
from ..services.storage import StorageService

router = APIRouter()

@router.post("/generate-metadata", response_model=GenerateFieldMetadataResponse)
async def generate_field_metadata(
    request: GenerateFieldMetadataRequest,
    db: Session = Depends(get_db)
):
    """使用 AI 生成字段元数据"""
    # 获取默认 AI 配置
    ai_config = db.query(AiConfig).filter(AiConfig.is_default == True).first()
    if not ai_config:
        raise HTTPException(status_code=404, detail="未找到默认 AI 配置")

    # 创建 AI 客户端
    ai_client = AIClient(ai_config)

    try:
        # 生成字段元数据
        metadata = await ai_client.generate_field_metadata(
            field_label=request.field_label,
            field_type=request.field_type,
            additional_requirement=request.additional_requirement
        )

        return GenerateFieldMetadataResponse(
            field_name=metadata.field_name,
            validation_rule=metadata.validation_rule,
            extraction_hint=metadata.extraction_hint
        )
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"生成字段元数据失败: {str(e)}")


def get_active_fields(project_id: int, db: Session) -> List[ProjectField]:
    """获取项目所有未删除的字段"""
    return db.query(ProjectField).filter(
        ProjectField.project_id == project_id,
        ProjectField.is_deleted == False
    ).order_by(ProjectField.display_order).all()


def sync_table_structure(project_id: int, db: Session):
    """同步项目数据表结构"""
    storage = StorageService(db)
    fields = get_active_fields(project_id, db)
    storage.migrate_table_structure(project_id, fields)


@router.post("/", response_model=ProjectFieldResponse)
def create_field(field: ProjectFieldCreate, db: Session = Depends(get_db)):
    """创建字段"""
    # 验证项目存在
    project = db.query(Project).filter(Project.id == field.project_id).first()
    if not project:
        raise HTTPException(status_code=404, detail="项目不存在")

    # 检查是否存在同名已删除的字段（可以恢复）
    existing = db.query(ProjectField).filter(
        ProjectField.project_id == field.project_id,
        ProjectField.field_name == field.field_name,
        ProjectField.is_deleted == True
    ).first()

    if existing:
        # 恢复已删除的字段
        existing.is_deleted = False
        existing.deleted_at = None
        existing.field_label = field.field_label
        existing.field_type = field.field_type
        existing.is_required = field.is_required if field.is_required is not None else False
        existing.is_dedup_key = field.is_dedup_key if field.is_dedup_key is not None else False
        existing.additional_requirement = field.additional_requirement
        existing.validation_rule = field.validation_rule
        existing.extraction_hint = field.extraction_hint
        db.commit()
        db.refresh(existing)
        # 同步表结构（添加列）
        storage = StorageService(db)
        if storage.table_exists(field.project_id):
            storage.add_column_to_table(field.project_id, existing)
        return existing

    db_field = ProjectField(**field.model_dump())
    db.add(db_field)
    db.commit()
    db.refresh(db_field)

    # 同步项目数据表结构（添加新列）
    storage = StorageService(db)
    if storage.table_exists(field.project_id):
        storage.add_column_to_table(field.project_id, db_field)

    return db_field

@router.get("/project/{project_id}", response_model=List[ProjectFieldResponse])
def list_fields(project_id: int, db: Session = Depends(get_db)):
    """获取项目的所有字段（不包括已删除的）"""
    fields = get_active_fields(project_id, db)
    return fields

@router.get("/project/{project_id}/all", response_model=List[ProjectFieldResponse])
def list_all_fields(project_id: int, db: Session = Depends(get_db)):
    """获取项目的所有字段（包括已删除的）"""
    fields = db.query(ProjectField).filter(
        ProjectField.project_id == project_id
    ).order_by(ProjectField.display_order).all()
    return fields

@router.put("/{field_id}", response_model=ProjectFieldResponse)
def update_field(field_id: int, field: ProjectFieldUpdate, db: Session = Depends(get_db)):
    """更新字段"""
    db_field = db.query(ProjectField).filter(ProjectField.id == field_id).first()
    if not db_field:
        raise HTTPException(status_code=404, detail="字段不存在")

    project_id = db_field.project_id
    old_field_name = db_field.field_name
    old_field_type = db_field.field_type

    for key, value in field.model_dump(exclude_unset=True).items():
        setattr(db_field, key, value)

    db.commit()
    db.refresh(db_field)

    # 如果字段名称或类型改变，需要同步表结构
    if field.field_name is not None or field.field_type is not None:
        sync_table_structure(project_id, db)

    return db_field

@router.delete("/{field_id}")
def delete_field(field_id: int, db: Session = Depends(get_db)):
    """删除字段（软删除）"""
    db_field = db.query(ProjectField).filter(ProjectField.id == field_id).first()
    if not db_field:
        raise HTTPException(status_code=404, detail="字段不存在")

    # 软删除：标记为已删除
    db_field.is_deleted = True
    db_field.deleted_at = datetime.now()
    db.commit()

    return {"message": "字段已删除", "field_name": db_field.field_name}

@router.post("/{field_id}/restore")
def restore_field(field_id: int, db: Session = Depends(get_db)):
    """恢复已删除的字段"""
    db_field = db.query(ProjectField).filter(ProjectField.id == field_id).first()
    if not db_field:
        raise HTTPException(status_code=404, detail="字段不存在")

    if not db_field.is_deleted:
        raise HTTPException(status_code=400, detail="字段未被删除")

    db_field.is_deleted = False
    db_field.deleted_at = None
    db.commit()

    # 同步表结构（添加列）
    storage = StorageService(db)
    if storage.table_exists(db_field.project_id):
        storage.add_column_to_table(db_field.project_id, db_field)

    return {"message": "字段已恢复", "field": db_field}

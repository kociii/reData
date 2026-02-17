from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session
from typing import List
from ..db.base import get_db
from ..models.project import ProjectField, AiConfig
from ..models.schemas import (
    ProjectFieldCreate,
    ProjectFieldUpdate,
    ProjectFieldResponse,
    GenerateFieldMetadataRequest,
    GenerateFieldMetadataResponse
)
from ..services.ai_client import AIClient

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


@router.post("/", response_model=ProjectFieldResponse)
def create_field(field: ProjectFieldCreate, db: Session = Depends(get_db)):
    """创建字段"""
    db_field = ProjectField(**field.model_dump())
    db.add(db_field)
    db.commit()
    db.refresh(db_field)
    return db_field

@router.get("/project/{project_id}", response_model=List[ProjectFieldResponse])
def list_fields(project_id: int, db: Session = Depends(get_db)):
    """获取项目的所有字段"""
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
    
    for key, value in field.model_dump(exclude_unset=True).items():
        setattr(db_field, key, value)
    
    db.commit()
    db.refresh(db_field)
    return db_field

@router.delete("/{field_id}")
def delete_field(field_id: int, db: Session = Depends(get_db)):
    """删除字段"""
    db_field = db.query(ProjectField).filter(ProjectField.id == field_id).first()
    if not db_field:
        raise HTTPException(status_code=404, detail="字段不存在")
    
    db.delete(db_field)
    db.commit()
    return {"message": "字段已删除"}

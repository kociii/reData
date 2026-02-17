from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session
from typing import List
from ..db.base import get_db
from ..models.project import AiConfig
from ..models.schemas import AiConfigCreate, AiConfigUpdate, AiConfigResponse

router = APIRouter()

@router.post("/", response_model=AiConfigResponse)
def create_ai_config(config: AiConfigCreate, db: Session = Depends(get_db)):
    """创建 AI 配置"""
    # 检查名称是否已存在
    existing = db.query(AiConfig).filter(AiConfig.name == config.name).first()
    if existing:
        raise HTTPException(status_code=400, detail="配置名称已存在")
    
    db_config = AiConfig(**config.model_dump())
    db.add(db_config)
    db.commit()
    db.refresh(db_config)
    return db_config

@router.get("/", response_model=List[AiConfigResponse])
def list_ai_configs(db: Session = Depends(get_db)):
    """获取所有 AI 配置"""
    configs = db.query(AiConfig).order_by(AiConfig.created_at.desc()).all()
    return configs

@router.get("/{config_id}", response_model=AiConfigResponse)
def get_ai_config(config_id: int, db: Session = Depends(get_db)):
    """获取单个 AI 配置"""
    config = db.query(AiConfig).filter(AiConfig.id == config_id).first()
    if not config:
        raise HTTPException(status_code=404, detail="配置不存在")
    return config

@router.put("/{config_id}", response_model=AiConfigResponse)
def update_ai_config(config_id: int, config: AiConfigUpdate, db: Session = Depends(get_db)):
    """更新 AI 配置"""
    db_config = db.query(AiConfig).filter(AiConfig.id == config_id).first()
    if not db_config:
        raise HTTPException(status_code=404, detail="配置不存在")
    
    for key, value in config.model_dump(exclude_unset=True).items():
        setattr(db_config, key, value)
    
    db.commit()
    db.refresh(db_config)
    return db_config

@router.delete("/{config_id}")
def delete_ai_config(config_id: int, db: Session = Depends(get_db)):
    """删除 AI 配置"""
    db_config = db.query(AiConfig).filter(AiConfig.id == config_id).first()
    if not db_config:
        raise HTTPException(status_code=404, detail="配置不存在")
    
    db.delete(db_config)
    db.commit()
    return {"message": "配置已删除"}

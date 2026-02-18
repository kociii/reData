from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session
from typing import List
from pydantic import BaseModel
from ..db.base import get_db
from ..models.project import AiConfig
from ..models.schemas import AiConfigCreate, AiConfigUpdate, AiConfigResponse
from ..services.ai_client import test_ai_connection


# ========== Additional Schemas ==========

class TestConnectionRequest(BaseModel):
    """测试连接请求"""
    config_id: int = None
    # 或者直接提供配置
    api_url: str = None
    api_key: str = None
    model_name: str = None


class TestConnectionResponse(BaseModel):
    """测试连接响应"""
    success: bool
    message: str
    response: str = ""


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


@router.post("/test-connection", response_model=TestConnectionResponse)
async def test_connection(request: TestConnectionRequest, db: Session = Depends(get_db)):
    """
    测试 AI 配置连接

    可以通过 config_id 测试已保存的配置，或直接提供配置参数测试
    """
    config = None

    if request.config_id:
        # 使用已保存的配置
        config = db.query(AiConfig).filter(AiConfig.id == request.config_id).first()
        if not config:
            raise HTTPException(status_code=404, detail="配置不存在")
    elif all([request.api_url, request.api_key, request.model_name]):
        # 使用临时配置
        config = AiConfig(
            name="temp",
            api_url=request.api_url,
            api_key=request.api_key,
            model_name=request.model_name
        )
    else:
        raise HTTPException(status_code=400, detail="请提供 config_id 或完整的配置参数")

    result = await test_ai_connection(config)

    return TestConnectionResponse(
        success=result["success"],
        message=result["message"],
        response=result["response"]
    )


@router.post("/{config_id}/set-default", response_model=AiConfigResponse)
def set_default_config(config_id: int, db: Session = Depends(get_db)):
    """设置默认 AI 配置"""
    db_config = db.query(AiConfig).filter(AiConfig.id == config_id).first()
    if not db_config:
        raise HTTPException(status_code=404, detail="配置不存在")

    # 清除其他默认配置
    db.query(AiConfig).filter(AiConfig.is_default == True).update({"is_default": False})

    # 设置当前配置为默认
    db_config.is_default = True
    db.commit()
    db.refresh(db_config)

    return db_config


@router.get("/default", response_model=AiConfigResponse)
def get_default_config(db: Session = Depends(get_db)):
    """获取默认 AI 配置"""
    config = db.query(AiConfig).filter(AiConfig.is_default == True).first()
    if not config:
        raise HTTPException(status_code=404, detail="未设置默认配置")
    return config

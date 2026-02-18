from pydantic import BaseModel, Field
from typing import Optional, List
from datetime import datetime

# ========== Project Schemas ==========

class ProjectBase(BaseModel):
    name: str
    description: Optional[str] = None

class ProjectCreate(ProjectBase):
    pass

class ProjectUpdate(ProjectBase):
    name: Optional[str] = None

class ProjectResponse(ProjectBase):
    id: int
    created_at: datetime
    updated_at: Optional[datetime] = None
    
    class Config:
        from_attributes = True

# ========== ProjectField Schemas ==========

class ProjectFieldBase(BaseModel):
    field_name: str
    field_label: str
    field_type: str
    is_required: bool = False
    is_dedup_key: bool = False  # 是否参与去重
    additional_requirement: Optional[str] = None
    validation_rule: Optional[str] = None
    extraction_hint: Optional[str] = None
    display_order: int = 0

class ProjectFieldCreate(ProjectFieldBase):
    project_id: int

class ProjectFieldUpdate(ProjectFieldBase):
    field_name: Optional[str] = None
    field_label: Optional[str] = None
    field_type: Optional[str] = None

class ProjectFieldResponse(ProjectFieldBase):
    id: int
    project_id: int
    is_deleted: bool = False
    deleted_at: Optional[datetime] = None
    created_at: datetime

    class Config:
        from_attributes = True

# ========== ProcessingTask Schemas ==========

class ProcessingTaskBase(BaseModel):
    project_id: int
    status: str = "pending"

class ProcessingTaskCreate(ProcessingTaskBase):
    pass

class ProcessingTaskResponse(ProcessingTaskBase):
    id: str
    total_files: int = 0
    processed_files: int = 0
    total_rows: int = 0
    processed_rows: int = 0
    success_count: int = 0
    error_count: int = 0
    batch_number: Optional[str] = None
    created_at: datetime
    updated_at: Optional[datetime] = None
    
    class Config:
        from_attributes = True

# ========== AiConfig Schemas ==========

class AiConfigBase(BaseModel):
    name: str
    api_url: str
    model_name: str
    api_key: str
    temperature: float = 0.7
    max_tokens: int = 1000
    is_default: bool = False

class AiConfigCreate(AiConfigBase):
    pass

class AiConfigUpdate(AiConfigBase):
    name: Optional[str] = None
    api_url: Optional[str] = None
    model_name: Optional[str] = None
    api_key: Optional[str] = None

class AiConfigResponse(AiConfigBase):
    id: int
    created_at: datetime
    updated_at: Optional[datetime] = None

    class Config:
        from_attributes = True

# ========== Field Metadata Generation Schemas ==========

class GenerateFieldMetadataRequest(BaseModel):
    field_label: str
    field_type: str
    additional_requirement: Optional[str] = None

class GenerateFieldMetadataResponse(BaseModel):
    field_name: str
    validation_rule: Optional[str] = None
    extraction_hint: str

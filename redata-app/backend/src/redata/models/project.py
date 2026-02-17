from sqlalchemy import Column, Integer, String, Boolean, DateTime, Text
from sqlalchemy.sql import func
from ..db.base import Base
import json

class Project(Base):
    __tablename__ = "projects"
    
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String, unique=True, nullable=False, index=True)
    description = Column(Text, nullable=True)
    dedup_enabled = Column(Boolean, default=True)
    dedup_fields = Column(Text, nullable=True)  # JSON string
    dedup_strategy = Column(String, default="skip")  # skip, update, merge
    created_at = Column(DateTime(timezone=True), server_default=func.now())
    updated_at = Column(DateTime(timezone=True), onupdate=func.now())
    
    @property
    def dedup_fields_list(self):
        """获取去重字段列表"""
        if self.dedup_fields:
            return json.loads(self.dedup_fields)
        return []
    
    @dedup_fields_list.setter
    def dedup_fields_list(self, value):
        """设置去重字段列表"""
        if value:
            self.dedup_fields = json.dumps(value)
        else:
            self.dedup_fields = None

class ProjectField(Base):
    __tablename__ = "project_fields"
    
    id = Column(Integer, primary_key=True, index=True)
    project_id = Column(Integer, nullable=False, index=True)
    field_name = Column(String, nullable=False)
    field_label = Column(String, nullable=False)
    field_type = Column(String, nullable=False)  # text, number, email, phone, date, url
    is_required = Column(Boolean, default=False)
    validation_rule = Column(Text, nullable=True)
    extraction_hint = Column(Text, nullable=True)
    display_order = Column(Integer, default=0)
    created_at = Column(DateTime(timezone=True), server_default=func.now())

class ProcessingTask(Base):
    __tablename__ = "processing_tasks"
    
    id = Column(String, primary_key=True)  # UUID
    project_id = Column(Integer, nullable=False, index=True)
    status = Column(String, nullable=False, index=True)  # pending, processing, paused, completed, cancelled
    total_files = Column(Integer, default=0)
    processed_files = Column(Integer, default=0)
    total_rows = Column(Integer, default=0)
    processed_rows = Column(Integer, default=0)
    success_count = Column(Integer, default=0)
    error_count = Column(Integer, default=0)
    batch_number = Column(String, nullable=True)
    created_at = Column(DateTime(timezone=True), server_default=func.now())
    updated_at = Column(DateTime(timezone=True), onupdate=func.now())

class AiConfig(Base):
    __tablename__ = "ai_configs"
    
    id = Column(Integer, primary_key=True, index=True)
    name = Column(String, unique=True, nullable=False)
    api_url = Column(String, nullable=False)
    model_name = Column(String, nullable=False)
    api_key = Column(String, nullable=False)  # TODO: 加密存储
    temperature = Column(Integer, default=0.7)
    max_tokens = Column(Integer, default=1000)
    is_default = Column(Boolean, default=False)
    created_at = Column(DateTime(timezone=True), server_default=func.now())
    updated_at = Column(DateTime(timezone=True), onupdate=func.now())

class Batch(Base):
    __tablename__ = "batches"
    
    id = Column(Integer, primary_key=True, index=True)
    batch_number = Column(String, unique=True, nullable=False)
    project_id = Column(Integer, nullable=False, index=True)
    file_count = Column(Integer, default=0)
    record_count = Column(Integer, default=0)
    created_at = Column(DateTime(timezone=True), server_default=func.now())

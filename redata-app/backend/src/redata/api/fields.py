from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session
from typing import List
from ..db.base import get_db
from ..models.project import ProjectField
from ..models.schemas import ProjectFieldCreate, ProjectFieldUpdate, ProjectFieldResponse

router = APIRouter()

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

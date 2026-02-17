from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session
from typing import List
from ..db.base import get_db
from ..models.project import Project
from ..models.schemas import ProjectCreate, ProjectUpdate, ProjectResponse

router = APIRouter()

@router.post("/", response_model=ProjectResponse)
def create_project(project: ProjectCreate, db: Session = Depends(get_db)):
    """创建项目"""
    # 检查项目名称是否已存在
    existing = db.query(Project).filter(Project.name == project.name).first()
    if existing:
        raise HTTPException(status_code=400, detail="项目名称已存在")
    
    db_project = Project(**project.model_dump())
    db.add(db_project)
    db.commit()
    db.refresh(db_project)
    return db_project

@router.get("/", response_model=List[ProjectResponse])
def list_projects(db: Session = Depends(get_db)):
    """获取所有项目"""
    projects = db.query(Project).order_by(Project.created_at.desc()).all()
    return projects

@router.get("/{project_id}", response_model=ProjectResponse)
def get_project(project_id: int, db: Session = Depends(get_db)):
    """获取单个项目"""
    project = db.query(Project).filter(Project.id == project_id).first()
    if not project:
        raise HTTPException(status_code=404, detail="项目不存在")
    return project

@router.put("/{project_id}", response_model=ProjectResponse)
def update_project(project_id: int, project: ProjectUpdate, db: Session = Depends(get_db)):
    """更新项目"""
    db_project = db.query(Project).filter(Project.id == project_id).first()
    if not db_project:
        raise HTTPException(status_code=404, detail="项目不存在")
    
    for key, value in project.model_dump(exclude_unset=True).items():
        setattr(db_project, key, value)
    
    db.commit()
    db.refresh(db_project)
    return db_project

@router.delete("/{project_id}")
def delete_project(project_id: int, db: Session = Depends(get_db)):
    """删除项目"""
    db_project = db.query(Project).filter(Project.id == project_id).first()
    if not db_project:
        raise HTTPException(status_code=404, detail="项目不存在")
    
    db.delete(db_project)
    db.commit()
    return {"message": "项目已删除"}

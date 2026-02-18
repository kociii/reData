"""
pytest 配置文件
提供测试夹具和配置
"""

import os
import pytest
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker
from sqlalchemy.pool import StaticPool
from fastapi.testclient import TestClient

# 设置测试环境变量
os.environ["TESTING"] = "true"

from src.redata.db.base import Base, get_db
from src.redata.main import app

# 导入所有模型以确保 Base.metadata.create_all 创建所有表
from src.redata.models.project import Project, ProjectField, AiConfig, ProcessingTask, Batch


# 使用 function 作用域的测试引擎
@pytest.fixture(scope="function")
def db_session():
    """创建测试数据库会话"""
    # 使用内存数据库，StaticPool 确保所有连接使用同一个内存数据库
    engine = create_engine(
        "sqlite:///:memory:",
        connect_args={"check_same_thread": False},
        poolclass=StaticPool,
        echo=False
    )

    # 创建所有表
    Base.metadata.create_all(bind=engine)

    # 创建会话
    TestingSessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)
    session = TestingSessionLocal()

    yield session

    # 清理
    session.close()
    Base.metadata.drop_all(bind=engine)


@pytest.fixture(scope="function")
def client(db_session):
    """创建测试客户端"""

    def override_get_db():
        try:
            yield db_session
        finally:
            pass

    app.dependency_overrides[get_db] = override_get_db

    with TestClient(app) as c:
        yield c

    app.dependency_overrides.clear()


@pytest.fixture
def sample_project_data():
    """示例项目数据"""
    return {
        "name": "测试项目",
        "description": "这是一个测试项目"
    }


@pytest.fixture
def sample_field_data():
    """示例字段数据"""
    return {
        "field_name": "name",
        "field_label": "姓名",
        "field_type": "text",
        "is_required": True,
        "is_dedup_key": True,
        "validation_rule": None,
        "extraction_hint": "提取客户的姓名"
    }


@pytest.fixture
def sample_ai_config_data():
    """示例 AI 配置数据"""
    return {
        "name": "测试配置",
        "api_url": "https://api.openai.com/v1",
        "model_name": "gpt-4",
        "api_key": "sk-test-key-12345",
        "temperature": 0.7,
        "max_tokens": 2000,
        "is_default": True
    }


@pytest.fixture
def created_project(client, sample_project_data):
    """创建一个项目并返回"""
    response = client.post("/api/projects/", json=sample_project_data)
    assert response.status_code == 200, f"创建项目失败: {response.text}"
    return response.json()


@pytest.fixture
def created_ai_config(client, sample_ai_config_data):
    """创建一个 AI 配置并返回"""
    response = client.post("/api/ai-configs/", json=sample_ai_config_data)
    assert response.status_code == 200, f"创建 AI 配置失败: {response.text}"
    return response.json()


@pytest.fixture
def created_field(client, created_project, sample_field_data):
    """创建一个字段并返回"""
    field_data = {**sample_field_data, "project_id": created_project["id"]}
    response = client.post("/api/fields/", json=field_data)
    assert response.status_code == 200, f"创建字段失败: {response.text}"
    return response.json()

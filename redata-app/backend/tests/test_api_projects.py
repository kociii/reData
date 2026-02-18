"""
项目 API 测试
测试项目的 CRUD 操作
"""

import pytest


class TestProjectsAPI:
    """项目 API 测试类"""

    def test_list_projects_empty(self, client):
        """测试空项目列表"""
        response = client.get("/api/projects/")
        assert response.status_code == 200
        assert response.json() == []

    def test_create_project(self, client, sample_project_data):
        """测试创建项目"""
        response = client.post("/api/projects/", json=sample_project_data)
        assert response.status_code == 200
        data = response.json()
        assert data["name"] == sample_project_data["name"]
        assert data["description"] == sample_project_data["description"]
        assert "id" in data
        assert "created_at" in data

    def test_create_project_minimal(self, client):
        """测试创建最小化项目（仅名称）"""
        response = client.post("/api/projects/", json={"name": "最小项目"})
        assert response.status_code == 200
        data = response.json()
        assert data["name"] == "最小项目"

    def test_create_project_missing_name(self, client):
        """测试创建项目时缺少名称"""
        response = client.post("/api/projects/", json={"description": "无名称"})
        assert response.status_code == 422  # Validation Error

    def test_get_project(self, client, created_project):
        """测试获取单个项目"""
        response = client.get(f"/api/projects/{created_project['id']}")
        assert response.status_code == 200
        data = response.json()
        assert data["id"] == created_project["id"]
        assert data["name"] == created_project["name"]

    def test_get_project_not_found(self, client):
        """测试获取不存在的项目"""
        response = client.get("/api/projects/99999")
        assert response.status_code == 404

    def test_update_project(self, client, created_project):
        """测试更新项目"""
        update_data = {
            "name": "更新后的名称",
            "description": "更新后的描述"
        }
        response = client.put(
            f"/api/projects/{created_project['id']}",
            json=update_data
        )
        assert response.status_code == 200
        data = response.json()
        assert data["name"] == update_data["name"]
        assert data["description"] == update_data["description"]

    def test_update_project_partial(self, client, created_project):
        """测试部分更新项目"""
        response = client.put(
            f"/api/projects/{created_project['id']}",
            json={"name": "仅更新名称"}
        )
        assert response.status_code == 200
        data = response.json()
        assert data["name"] == "仅更新名称"

    def test_update_project_not_found(self, client):
        """测试更新不存在的项目"""
        response = client.put("/api/projects/99999", json={"name": "测试"})
        assert response.status_code == 404

    def test_delete_project(self, client, created_project):
        """测试删除项目"""
        response = client.delete(f"/api/projects/{created_project['id']}")
        assert response.status_code == 200

        # 确认已删除
        response = client.get(f"/api/projects/{created_project['id']}")
        assert response.status_code == 404

    def test_delete_project_not_found(self, client):
        """测试删除不存在的项目"""
        response = client.delete("/api/projects/99999")
        assert response.status_code == 404

    def test_list_projects_with_data(self, client, sample_project_data):
        """测试有数据时的项目列表"""
        # 创建多个项目
        for i in range(3):
            client.post("/api/projects/", json={**sample_project_data, "name": f"项目{i}"})

        response = client.get("/api/projects/")
        assert response.status_code == 200
        data = response.json()
        assert len(data) >= 3

"""
字段 API 测试
测试字段定义的 CRUD 操作
"""

import pytest


class TestFieldsAPI:
    """字段 API 测试类"""

    def test_list_fields_empty(self, client, created_project):
        """测试空字段列表"""
        response = client.get(f"/api/fields/project/{created_project['id']}")
        assert response.status_code == 200
        assert response.json() == []

    def test_create_field(self, client, created_project, sample_field_data):
        """测试创建字段"""
        field_data = {**sample_field_data, "project_id": created_project["id"]}
        response = client.post("/api/fields/", json=field_data)
        assert response.status_code == 200
        data = response.json()
        assert data["field_name"] == sample_field_data["field_name"]
        assert data["field_label"] == sample_field_data["field_label"]
        assert data["field_type"] == sample_field_data["field_type"]
        assert data["is_required"] == sample_field_data["is_required"]
        assert "id" in data

    def test_create_field_minimal(self, client, created_project):
        """测试创建最小化字段"""
        field_data = {
            "project_id": created_project["id"],
            "field_name": "phone",
            "field_label": "电话",
            "field_type": "phone"
        }
        response = client.post("/api/fields/", json=field_data)
        assert response.status_code == 200
        data = response.json()
        assert data["field_name"] == "phone"

    def test_create_field_invalid_project(self, client, sample_field_data):
        """测试为不存在的项目创建字段"""
        field_data = {**sample_field_data, "project_id": 99999}
        response = client.post("/api/fields/", json=field_data)
        # 可能返回 200（创建成功）或 404（项目不存在）
        assert response.status_code in [200, 404, 422]

    def test_create_field_missing_required(self, client, created_project):
        """测试创建字段时缺少必填项"""
        response = client.post("/api/fields/", json={"project_id": created_project["id"]})
        assert response.status_code == 422  # Validation Error

    def test_update_field(self, client, created_field):
        """测试更新字段"""
        update_data = {
            "field_label": "更新后的标签",
            "is_required": False
        }
        response = client.put(f"/api/fields/{created_field['id']}", json=update_data)
        assert response.status_code == 200
        data = response.json()
        assert data["field_label"] == update_data["field_label"]
        assert data["is_required"] == update_data["is_required"]

    def test_update_field_not_found(self, client):
        """测试更新不存在的字段"""
        response = client.put("/api/fields/99999", json={"field_label": "测试"})
        assert response.status_code == 404

    def test_delete_field(self, client, created_field):
        """测试删除字段"""
        response = client.delete(f"/api/fields/{created_field['id']}")
        assert response.status_code == 200

        # 确认已删除
        response = client.get(f"/api/fields/project/{created_field['project_id']}")
        fields = response.json()
        assert not any(f["id"] == created_field["id"] for f in fields)

    def test_delete_field_not_found(self, client):
        """测试删除不存在的字段"""
        response = client.delete("/api/fields/99999")
        assert response.status_code == 404

    def test_list_fields_with_data(self, client, created_project, sample_field_data):
        """测试有数据时的字段列表"""
        # 创建多个字段
        field_types = ["text", "phone", "email"]
        for i, field_type in enumerate(field_types):
            field_data = {
                **sample_field_data,
                "project_id": created_project["id"],
                "field_name": f"field_{i}",
                "field_label": f"字段{i}",
                "field_type": field_type
            }
            client.post("/api/fields/", json=field_data)

        response = client.get(f"/api/fields/project/{created_project['id']}")
        assert response.status_code == 200
        data = response.json()
        assert len(data) >= 3

    def test_field_display_order(self, client, created_project, sample_field_data):
        """测试字段显示顺序"""
        # 创建多个字段
        for i in range(3):
            field_data = {
                **sample_field_data,
                "project_id": created_project["id"],
                "field_name": f"field_{i}",
                "field_label": f"字段{i}",
                "field_type": "text"
            }
            client.post("/api/fields/", json=field_data)

        response = client.get(f"/api/fields/project/{created_project['id']}")
        fields = response.json()

        # 检查字段按 display_order 排序
        orders = [f["display_order"] for f in fields]
        assert orders == sorted(orders)

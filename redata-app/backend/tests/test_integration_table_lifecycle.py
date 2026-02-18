"""
集成测试 - 数据表生命周期和软删除功能
测试项目创建/删除时的数据表管理，以及字段软删除功能
"""

import pytest
from sqlalchemy import inspect, text


class TestTableLifecycle:
    """测试数据表生命周期"""

    def test_create_project_creates_table(self, client, sample_project_data):
        """创建项目时应该自动创建数据表"""
        # 创建项目
        response = client.post("/api/projects/", json=sample_project_data)
        assert response.status_code == 200
        project = response.json()
        project_id = project["id"]

        # 检查数据表是否存在
        from src.redata.services.storage import StorageService
        from src.redata.db.base import get_db_url
        from sqlalchemy import create_engine

        engine = create_engine(get_db_url())
        inspector = inspect(engine)
        table_name = f"project_{project_id}_records"

        assert table_name in inspector.get_table_names(), f"数据表 {table_name} 应该被创建"

    def test_delete_project_drops_table(self, client, sample_project_data):
        """删除项目时应该删除数据表"""
        # 创建项目
        response = client.post("/api/projects/", json=sample_project_data)
        assert response.status_code == 200
        project = response.json()
        project_id = project["id"]

        # 删除项目
        delete_response = client.delete(f"/api/projects/{project_id}")
        assert delete_response.status_code == 200

        # 检查数据表是否被删除
        from src.redata.services.storage import StorageService
        from src.redata.db.base import get_db_url
        from sqlalchemy import create_engine

        engine = create_engine(get_db_url())
        inspector = inspect(engine)
        table_name = f"project_{project_id}_records"

        assert table_name not in inspector.get_table_names(), f"数据表 {table_name} 应该被删除"

    def test_create_field_adds_column(self, client, created_project, sample_field_data):
        """创建字段时应该添加列到数据表"""
        project_id = created_project["id"]

        # 创建字段
        field_data = {**sample_field_data, "project_id": project_id}
        response = client.post("/api/fields/", json=field_data)
        assert response.status_code == 200

        # 检查列是否存在
        from src.redata.db.base import get_db_url
        from sqlalchemy import create_engine

        engine = create_engine(get_db_url())
        inspector = inspect(engine)
        table_name = f"project_{project_id}_records"

        columns = [col["name"] for col in inspector.get_columns(table_name)]
        assert sample_field_data["field_name"] in columns, f"列 {sample_field_data['field_name']} 应该被添加"


class TestFieldSoftDelete:
    """测试字段软删除功能"""

    def test_delete_field_marks_as_deleted(self, client, created_field):
        """删除字段应该标记为已删除而不是真正删除"""
        field_id = created_field["id"]

        # 删除字段
        response = client.delete(f"/api/fields/{field_id}")
        assert response.status_code == 200

        # 获取所有字段（包括已删除的）
        project_id = created_field["project_id"]
        all_fields_response = client.get(f"/api/fields/project/{project_id}/all")
        assert all_fields_response.status_code == 200
        all_fields = all_fields_response.json()

        # 字段应该在列表中
        deleted_field = next((f for f in all_fields if f["id"] == field_id), None)
        assert deleted_field is not None, "字段应该仍然存在"
        assert deleted_field["is_deleted"] is True, "字段应该被标记为已删除"

    def test_list_fields_excludes_deleted(self, client, created_project, sample_field_data):
        """获取字段列表应该排除已删除的字段"""
        project_id = created_project["id"]

        # 创建字段
        field_data = {**sample_field_data, "project_id": project_id, "field_name": "test_field"}
        create_response = client.post("/api/fields/", json=field_data)
        assert create_response.status_code == 200
        field_id = create_response.json()["id"]

        # 获取字段列表
        list_response = client.get(f"/api/fields/project/{project_id}")
        assert list_response.status_code == 200
        fields_before_delete = list_response.json()

        # 删除字段
        client.delete(f"/api/fields/{field_id}")

        # 再次获取字段列表
        list_response = client.get(f"/api/fields/project/{project_id}")
        fields_after_delete = list_response.json()

        assert len(fields_after_delete) == len(fields_before_delete) - 1

    def test_restore_deleted_field(self, client, created_field):
        """应该能够恢复已删除的字段"""
        field_id = created_field["id"]
        project_id = created_field["project_id"]

        # 删除字段
        client.delete(f"/api/fields/{field_id}")

        # 恢复字段
        restore_response = client.post(f"/api/fields/{field_id}/restore")
        assert restore_response.status_code == 200

        # 获取字段列表
        list_response = client.get(f"/api/fields/project/{project_id}")
        fields = list_response.json()

        # 字段应该出现在列表中
        restored_field = next((f for f in fields if f["id"] == field_id), None)
        assert restored_field is not None, "恢复的字段应该出现在列表中"
        assert restored_field["is_deleted"] is False, "恢复的字段应该标记为未删除"

    def test_create_field_with_same_name_restores_deleted(self, client, created_project, sample_field_data):
        """创建与已删除字段同名的字段应该恢复该字段"""
        project_id = created_project["id"]

        # 创建字段
        field_data = {**sample_field_data, "project_id": project_id, "field_name": "unique_field"}
        create_response = client.post("/api/fields/", json=field_data)
        assert create_response.status_code == 200
        original_field_id = create_response.json()["id"]

        # 删除字段
        client.delete(f"/api/fields/{original_field_id}")

        # 创建同名字段
        new_field_data = {**sample_field_data, "project_id": project_id, "field_name": "unique_field"}
        new_create_response = client.post("/api/fields/", json=new_field_data)
        assert new_create_response.status_code == 200
        new_field = new_create_response.json()

        # 应该恢复原来的字段而不是创建新的
        assert new_field["id"] == original_field_id, "应该恢复原来的字段"
        assert new_field["is_deleted"] is False, "字段应该标记为未删除"


class TestTableStructureMigration:
    """测试表结构迁移"""

    def test_add_field_to_existing_table(self, client, created_project):
        """向已有数据的项目添加字段"""
        project_id = created_project["id"]

        # 创建第一个字段
        field1 = {
            "project_id": project_id,
            "field_name": "name",
            "field_label": "姓名",
            "field_type": "text"
        }
        response = client.post("/api/fields/", json=field1)
        assert response.status_code == 200

        # 创建第二个字段
        field2 = {
            "project_id": project_id,
            "field_name": "phone",
            "field_label": "手机号",
            "field_type": "phone"
        }
        response = client.post("/api/fields/", json=field2)
        assert response.status_code == 200

        # 检查两个列都存在
        from src.redata.db.base import get_db_url
        from sqlalchemy import create_engine

        engine = create_engine(get_db_url())
        inspector = inspect(engine)
        table_name = f"project_{project_id}_records"

        columns = [col["name"] for col in inspector.get_columns(table_name)]
        assert "name" in columns
        assert "phone" in columns

    def test_update_field_type_migrates_table(self, client, created_field):
        """更新字段类型应该迁移表结构"""
        field_id = created_field["id"]

        # 更新字段类型
        update_data = {"field_type": "email"}
        response = client.put(f"/api/fields/{field_id}", json=update_data)
        assert response.status_code == 200

"""
存储服务测试
测试 StorageService 的基础功能
"""

import pytest
from unittest.mock import Mock, patch


class TestStorageService:
    """存储服务测试类"""

    @pytest.fixture
    def storage(self, db_session):
        """创建存储服务实例"""
        from src.redata.services.storage import StorageService
        return StorageService(db_session)

    # ========== 表名称测试 ==========

    def test_get_table_name(self, storage):
        """测试获取表名称"""
        table_name = storage.get_table_name(1)
        assert table_name == "project_1_records"

        table_name = storage.get_table_name(123)
        assert table_name == "project_123_records"

    # ========== 表创建测试 ==========

    def test_create_project_table(self, storage, created_project):
        """测试创建项目数据表"""
        # 使用 Mock 模拟字段
        field = Mock()
        field.field_name = "name"
        field.field_type = "text"
        field.is_required = True

        fields = [field]
        storage.create_project_table(created_project["id"], fields)

        # 验证表存在
        assert storage.table_exists(created_project["id"])

    def test_create_project_table_empty_fields(self, storage, created_project):
        """测试创建没有字段的项目数据表"""
        storage.create_project_table(created_project["id"], [])
        assert storage.table_exists(created_project["id"])

    # ========== 表存在性测试 ==========

    def test_table_exists_false(self, storage):
        """测试不存在的表"""
        assert storage.table_exists(99999) == False

    # ========== 表删除测试 ==========

    def test_drop_project_table(self, storage, created_project):
        """测试删除项目数据表"""
        storage.create_project_table(created_project["id"], [])
        assert storage.table_exists(created_project["id"]) == True

        storage.drop_project_table(created_project["id"])
        assert storage.table_exists(created_project["id"]) == False

    # ========== 记录计数测试 ==========

    def test_get_record_count_empty(self, storage, created_project):
        """测试空表记录计数"""
        storage.create_project_table(created_project["id"], [])
        count = storage.get_record_count(created_project["id"])
        assert count == 0

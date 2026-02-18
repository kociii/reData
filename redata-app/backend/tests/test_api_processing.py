"""
处理任务 API 测试
测试数据处理任务的启动、暂停、恢复、取消等操作
"""

import pytest
from unittest.mock import patch, MagicMock
import asyncio


class TestProcessingAPI:
    """处理任务 API 测试类"""

    def test_list_tasks_empty(self, client, created_project):
        """测试空任务列表"""
        response = client.get(f"/api/processing/list/{created_project['id']}")
        assert response.status_code == 200
        data = response.json()
        assert data["tasks"] == []
        assert data["total"] == 0

    def test_start_processing_missing_ai_config(self, client, created_project):
        """测试启动处理但缺少 AI 配置"""
        request_data = {
            "project_id": created_project["id"],
            "file_paths": ["/path/to/file.xlsx"]
        }
        response = client.post("/api/processing/start", json=request_data)
        # 没有默认 AI 配置时应该返回 400
        assert response.status_code == 400

    def test_start_processing_project_not_found(self, client):
        """测试启动处理但项目不存在"""
        request_data = {
            "project_id": 99999,
            "file_paths": ["/path/to/file.xlsx"]
        }
        response = client.post("/api/processing/start", json=request_data)
        assert response.status_code == 404

    @patch("src.redata.api.processing.run_processing")
    def test_start_processing_success(self, mock_run, client, created_project, created_ai_config):
        """测试成功启动处理任务"""
        # Mock 后台任务
        mock_run.return_value = asyncio.sleep(0)

        request_data = {
            "project_id": created_project["id"],
            "file_paths": ["/path/to/file.xlsx"]
        }
        response = client.post("/api/processing/start", json=request_data)

        assert response.status_code == 200
        data = response.json()
        assert "task_id" in data
        assert data["project_id"] == created_project["id"]
        assert data["status"] == "processing"
        assert data["total_files"] == 1
        assert "batch_number" in data

    @patch("src.redata.api.processing.get_extractor")
    def test_pause_task_not_found(self, mock_get_extractor, client):
        """测试暂停不存在的任务"""
        mock_get_extractor.return_value = None
        response = client.post("/api/processing/pause/non-existent-task-id")
        assert response.status_code == 404

    @patch("src.redata.api.processing.get_extractor")
    def test_resume_task_not_found(self, mock_get_extractor, client):
        """测试恢复不存在的任务"""
        mock_get_extractor.return_value = None
        response = client.post("/api/processing/resume/non-existent-task-id")
        assert response.status_code == 404

    @patch("src.redata.api.processing.get_extractor")
    def test_cancel_task_not_found(self, mock_get_extractor, client):
        """测试取消不存在的任务"""
        mock_get_extractor.return_value = None
        response = client.post("/api/processing/cancel/non-existent-task-id")
        assert response.status_code == 404

    def test_get_status_not_found(self, client):
        """测试获取不存在任务的状态"""
        response = client.get("/api/processing/status/non-existent-task-id")
        assert response.status_code == 404

    def test_list_tasks_with_status_filter(self, client, created_project, created_ai_config, db_session):
        """测试按状态筛选任务列表"""
        import uuid
        from src.redata.models.project import ProcessingTask

        # 创建几个测试任务，使用唯一的 ID
        for i, status in enumerate(["completed", "completed", "cancelled"]):
            task = ProcessingTask(
                id=f"test-{status}-{uuid.uuid4().hex[:8]}",
                project_id=created_project["id"],
                status=status,
                total_files=1,
                processed_files=1,
                total_rows=10,
                processed_rows=10,
                success_count=10,
                error_count=0,
                batch_number=f"batch_{status}_{i}"
            )
            db_session.add(task)
        db_session.commit()

        # 筛选完成的任务
        response = client.get(f"/api/processing/list/{created_project['id']}?status=completed")
        assert response.status_code == 200
        data = response.json()
        assert all(t["status"] == "completed" for t in data["tasks"])

    def test_list_tasks_pagination(self, client, created_project, created_ai_config, db_session):
        """测试任务列表分页"""
        from src.redata.models.project import ProcessingTask

        # 创建多个任务
        for i in range(25):
            task = ProcessingTask(
                id=f"test-task-{i}",
                project_id=created_project["id"],
                status="completed",
                total_files=1,
                processed_files=1,
                total_rows=10,
                processed_rows=10,
                success_count=10,
                error_count=0,
                batch_number=f"batch_{i:03d}"
            )
            db_session.add(task)
        db_session.commit()

        # 测试默认分页
        response = client.get(f"/api/processing/list/{created_project['id']}")
        assert response.status_code == 200
        data = response.json()
        assert data["total"] == 25
        assert len(data["tasks"]) <= 20  # 默认 limit=20

        # 测试自定义分页
        response = client.get(f"/api/processing/list/{created_project['id']}?limit=10&offset=5")
        assert response.status_code == 200
        data = response.json()
        assert data["total"] == 25
        assert len(data["tasks"]) == 10

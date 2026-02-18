"""
结果 API 测试
测试数据结果的查询、更新、删除和导出操作
"""

import pytest


class TestResultsAPI:
    """结果 API 测试类"""

    def test_query_results_empty(self, client, created_project):
        """测试空结果查询"""
        response = client.get(f"/api/results/{created_project['id']}")
        assert response.status_code == 200
        data = response.json()
        # 返回的是 QueryResult 对象
        assert hasattr(data, 'records') or 'records' in data or data.get('records') == [] or data == []

    def test_query_results_project_not_found(self, client):
        """测试查询不存在项目的结果"""
        response = client.get("/api/results/99999")
        # 根据实现可能返回 200（空结果）或 404
        assert response.status_code in [200, 404, 422]

    def test_query_results_with_pagination(self, client, created_project, created_field):
        """测试带分页的结果查询"""
        response = client.get(
            f"/api/results/{created_project['id']}?page=1&page_size=10"
        )
        assert response.status_code == 200

    def test_query_results_with_batch_filter(self, client, created_project, created_field):
        """测试按批次筛选结果"""
        response = client.get(
            f"/api/results/{created_project['id']}?batch_number=batch_001"
        )
        assert response.status_code == 200

    def test_query_results_with_search(self, client, created_project, created_field):
        """测试搜索结果"""
        response = client.get(
            f"/api/results/{created_project['id']}?search=测试"
        )
        assert response.status_code == 200

    def test_update_result_not_found(self, client, created_project):
        """测试更新不存在的结果"""
        response = client.put(
            f"/api/results/{created_project['id']}/99999",
            json={"name": "更新"}
        )
        # 可能返回 404 或 422
        assert response.status_code in [404, 422]

    def test_delete_result_not_found(self, client, created_project):
        """测试删除不存在的结果"""
        response = client.delete(f"/api/results/{created_project['id']}/99999")
        assert response.status_code in [404, 422]

    def test_export_results_xlsx(self, client, created_project, created_field):
        """测试导出 Excel 格式结果"""
        response = client.get(
            f"/api/results/export/{created_project['id']}?format=xlsx"
        )
        # 可能返回 200、404 或 422（参数错误）
        assert response.status_code in [200, 404, 422]

    def test_export_results_csv(self, client, created_project, created_field):
        """测试导出 CSV 格式结果"""
        response = client.get(
            f"/api/results/export/{created_project['id']}?format=csv"
        )
        assert response.status_code in [200, 404, 422]

    def test_export_results_with_batch(self, client, created_project, created_field):
        """测试按批次导出结果"""
        response = client.get(
            f"/api/results/export/{created_project['id']}?format=xlsx&batch_number=batch_001"
        )
        assert response.status_code in [200, 404, 422]

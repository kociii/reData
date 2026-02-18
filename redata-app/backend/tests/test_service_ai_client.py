"""
AI 客户端测试
测试 AIClient 的基础功能
"""

import pytest
from unittest.mock import Mock, patch, AsyncMock


class TestAIClient:
    """AI 客户端测试类"""

    @pytest.fixture
    def mock_config(self):
        """创建模拟 AI 配置"""
        config = Mock()
        config.api_url = "https://api.openai.com/v1"
        config.model_name = "gpt-4"
        config.api_key = "sk-test-key"
        config.temperature = 0.7
        config.max_tokens = 2000
        return config

    # ========== 初始化测试 ==========

    def test_init_client(self, mock_config):
        """测试初始化 AI 客户端"""
        from src.redata.services.ai_client import AIClient
        client = AIClient(mock_config)
        assert client is not None
        assert client.config == mock_config

    def test_client_has_model_name(self, mock_config):
        """测试客户端包含模型名称"""
        from src.redata.services.ai_client import AIClient
        client = AIClient(mock_config)
        assert client.config.model_name == "gpt-4"


class TestAIClientIntegration:
    """AI 客户端集成测试（需要真实 API，默认跳过）"""

    @pytest.mark.skip(reason="需要真实 API 密钥")
    @pytest.mark.asyncio
    async def test_real_api_call(self):
        """测试真实 API 调用"""
        pass

    @pytest.mark.skip(reason="需要真实 API 密钥")
    @pytest.mark.asyncio
    async def test_analyze_column_mapping(self):
        """测试列映射分析"""
        pass

    @pytest.mark.skip(reason="需要真实 API 密钥")
    @pytest.mark.asyncio
    async def test_generate_field_metadata(self):
        """测试字段元数据生成"""
        pass

"""
AI 客户端服务
负责与 OpenAI 兼容 API 通信，提供重试机制和超时控制
"""

import json
import asyncio
from typing import Optional, Dict, Any, List
from dataclasses import dataclass
import httpx
from openai import AsyncOpenAI

from ..models.project import AiConfig, ProjectField


@dataclass
class FieldMetadata:
    """AI 生成的字段元数据"""
    field_name: str
    validation_rule: Optional[str]
    extraction_hint: str


@dataclass
class HeaderRecognitionResult:
    """表头识别结果"""
    header_row: int  # 0 表示无表头，1-5 表示表头行号
    headers: List[str]


@dataclass
class ColumnMapping:
    """列映射结果"""
    header_row: int  # 表头行号 (0=无表头, 1-10=表头位置)
    column_mappings: Dict[int, str]  # 列索引 -> 字段名 的映射，例如: {0: "name", 2: "phone"}
    confidence: float  # 匹配置信度 (0-1)
    unmatched_columns: List[int]  # 未匹配的列索引


class AIClientError(Exception):
    """AI 客户端错误"""
    pass


class AIClient:
    """AI 客户端服务"""

    # 重试配置
    MAX_RETRIES = 3
    RETRY_DELAY = 2.0  # 秒
    TIMEOUT = 120.0  # 秒（AI 分析可能较慢）

    def __init__(self, config: AiConfig):
        """
        初始化 AI 客户端

        Args:
            config: AI 配置对象
        """
        self.config = config
        self.client = AsyncOpenAI(
            api_key=config.api_key,
            base_url=config.api_url,
            timeout=self.TIMEOUT,
            max_retries=self.MAX_RETRIES,
        )

    async def call_api(self, prompt: str) -> str:
        """
        调用 AI API

        Args:
            prompt: 提示词

        Returns:
            AI 响应文本

        Raises:
            AIClientError: API 调用失败
        """
        try:
            response = await self.client.chat.completions.create(
                model=self.config.model_name,
                messages=[
                    {"role": "system", "content": "你是一个数据处理专家，请严格按照要求的格式返回结果。"},
                    {"role": "user", "content": prompt}
                ],
                temperature=self.config.temperature,
                max_tokens=self.config.max_tokens,
            )
            return response.choices[0].message.content.strip()
        except Exception as e:
            raise AIClientError(f"AI API 调用失败: {str(e)}")

    async def generate_field_metadata(
        self,
        field_label: str,
        field_type: str,
        additional_requirement: Optional[str] = None
    ) -> FieldMetadata:
        """
        生成字段元数据（只翻译英文名，验证规则本地生成）

        Args:
            field_label: 字段标签（中文）
            field_type: 字段类型
            additional_requirement: 补充提取要求（可选，暂未使用）

        Returns:
            FieldMetadata 对象
        """
        # 简化提示词，只要求翻译英文名
        prompt = f"""将以下中文字段名翻译成标准的英文字段名（snake_case 格式）。

字段名称：{field_label}

要求：
1. 使用 snake_case 命名规范（小写字母，下划线分隔）
2. 翻译要准确，符合数据库字段命名规范
3. 只返回英文字段名，不要有其他内容

示例：
- 客户姓名 -> customer_name
- 手机号码 -> phone_number
- 电子邮箱 -> email
- 公司名称 -> company_name
- 联系地址 -> address"""

        response = await self.call_api(prompt)

        # 清理响应，去除可能的空格和换行
        field_name = response.strip().lower().replace(" ", "_").replace("-", "_")

        # 本地生成验证规则（根据字段类型）
        validation_rule = self._get_validation_rule(field_type)

        # 生成本地提取提示
        extraction_hint = f"提取{field_label}字段"

        return FieldMetadata(
            field_name=field_name,
            validation_rule=validation_rule,
            extraction_hint=extraction_hint
        )

    def _get_validation_rule(self, field_type: str) -> Optional[str]:
        """
        根据字段类型获取验证规则

        Args:
            field_type: 字段类型

        Returns:
            验证规则正则表达式，如果不需要验证则返回 None
        """
        VALIDATION_RULES = {
            "phone": r"^1[3-9]\d{9}$",                    # 中国手机号
            "email": r"^[\w\.-]+@[\w\.-]+\.\w+$",         # 邮箱
            "url": r"^https?://",                          # URL
            "date": r"^\d{4}[-/]\d{1,2}[-/]\d{1,2}$",    # 日期
            "number": r"^-?\d+(\.\d+)?$",                  # 数字
            "text": None,                                  # 文本不需要验证
            "address": None,                               # 地址不需要验证
            "company": None,                               # 公司名称不需要验证
        }
        return VALIDATION_RULES.get(field_type)

    async def recognize_header(self, rows: List[List[str]]) -> HeaderRecognitionResult:
        """
        识别表头行

        Args:
            rows: 前 5 行数据，每行是一个列表

        Returns:
            HeaderRecognitionResult 对象
        """
        # 格式化行数据
        formatted_rows = []
        for i, row in enumerate(rows[:5], start=1):
            row_str = " | ".join(str(cell) for cell in row if cell)
            formatted_rows.append(f"[第 {i} 行] {row_str}")

        rows_text = "\n".join(formatted_rows)

        prompt = f"""你是一个表格分析专家。以下是一个 Excel 表格的前 5 行数据：

{rows_text}

请分析并判断：
1. 第几行是表头？（返回行号 1-5，如果没有表头则返回 0）
2. 表头包含哪些字段？（返回字段列表，如果没有表头则返回空数组）

请以 JSON 格式返回：
{{
  "header_row": 1,
  "headers": ["字段1", "字段2", "字段3"]
}}

只返回 JSON，不要有其他内容。"""

        response = await self.call_api(prompt)

        try:
            json_str = self._extract_json(response)
            data = json.loads(json_str)
            return HeaderRecognitionResult(
                header_row=data.get("header_row", 0),
                headers=data.get("headers", [])
            )
        except json.JSONDecodeError as e:
            raise AIClientError(f"解析表头识别结果失败: {str(e)}")

    async def analyze_column_mapping(
        self,
        sample_rows: List[List[str]],
        fields: List[ProjectField]
    ) -> ColumnMapping:
        """
        分析列与项目字段的匹配关系（两阶段处理 - 阶段一）

        这是新方案的核心：每个 sheet 仅调用 1 次 AI，分析列映射关系。
        后续的行数据处理完全在本地进行，无需 AI 调用。

        Args:
            sample_rows: 前 10 行样本数据
            fields: 项目字段定义列表

        Returns:
            ColumnMapping 对象，包含列映射和置信度
        """
        # 格式化行数据
        formatted_rows = []
        for i, row in enumerate(sample_rows[:10], start=1):
            row_values = []
            for cell in row:
                if cell is not None:
                    row_values.append(str(cell))
            row_str = " | ".join(row_values) if row_values else "(空行)"
            formatted_rows.append(f"[第 {i} 行] {row_str}")

        rows_text = "\n".join(formatted_rows)

        # 构建字段描述
        field_descriptions = []
        field_names = []
        for field in fields:
            required_mark = "【必填】" if field.is_required else ""
            hint = f" - 提取提示: {field.extraction_hint}" if field.extraction_hint else ""
            field_descriptions.append(
                f"- {field.field_name}（{field.field_label}，类型: {field.field_type}）{required_mark}{hint}"
            )
            field_names.append(field.field_name)

        prompt = f"""你是一个数据表格分析专家。以下是一个 Excel 表格的前 10 行数据：

{rows_text}

项目需要提取的字段：
{chr(10).join(field_descriptions)}

请分析：
1. 第几行是表头？（返回 1-10，如果没有表头返回 0）
2. 每一列对应哪个字段？（返回列索引到字段名的映射）

注意事项：
- 列索引从 0 开始
- 只映射能明确识别的字段
- 如果某一列无法匹配任何字段，放入 unmatched_columns
- confidence 表示整体匹配的置信度（0-1 之间）

请以 JSON 格式返回：
{{
  "header_row": 1,
  "column_mappings": {{
    "0": "name",
    "2": "phone",
    "5": "email"
  }},
  "confidence": 0.95,
  "unmatched_columns": [1, 3, 4]
}}

只返回 JSON，不要有其他内容。"""

        response = await self.call_api(prompt)

        try:
            json_str = self._extract_json(response)
            data = json.loads(json_str)

            # 解析 column_mappings，将字符串键转为整数
            column_mappings = {}
            for key, value in data.get("column_mappings", {}).items():
                try:
                    column_mappings[int(key)] = value
                except (ValueError, TypeError):
                    continue

            return ColumnMapping(
                header_row=data.get("header_row", 0),
                column_mappings=column_mappings,
                confidence=float(data.get("confidence", 0.5)),
                unmatched_columns=data.get("unmatched_columns", [])
            )
        except json.JSONDecodeError as e:
            raise AIClientError(f"解析列映射结果失败: {str(e)}")
        except (ValueError, TypeError) as e:
            raise AIClientError(f"列映射数据格式错误: {str(e)}")

    async def extract_data(
        self,
        row_data: str,
        fields: List[ProjectField],
        has_header: bool = True
    ) -> Dict[str, Any]:
        """
        从行数据中提取字段值

        Args:
            row_data: 格式化的行数据
            fields: 项目字段定义列表
            has_header: 是否有表头

        Returns:
            提取的字段值字典
        """
        # 构建字段描述
        field_descriptions = []
        for field in fields:
            required_mark = "【必填】" if field.is_required else ""
            hint = f"：{field.extraction_hint}" if field.extraction_hint else ""
            field_descriptions.append(
                f"- {field.field_label}（{field.field_type}）{required_mark}{hint}"
            )

        # 构建 JSON 返回格式
        json_fields = []
        for field in fields:
            json_fields.append(f'  "{field.field_name}": "提取的{field.field_label}"')

        if has_header:
            prompt = f"""你是一个数据提取专家。请从以下数据中提取指定字段：

原始数据：
{row_data}

请提取以下字段：
{chr(10).join(field_descriptions)}

请以 JSON 格式返回：
{{
{",".join(json_fields)}
}}

如果某个字段无法提取，请返回空字符串。只返回 JSON，不要有其他内容。"""
        else:
            prompt = f"""你是一个数据提取专家。请从以下原始数据中提取指定字段：

原始数据：
{row_data}

请提取以下字段：
{chr(10).join(field_descriptions)}

请以 JSON 格式返回：
{{
{",".join(json_fields)}
}}

如果某个字段无法提取，请返回空字符串。只返回 JSON，不要有其他内容。"""

        response = await self.call_api(prompt)

        try:
            json_str = self._extract_json(response)
            data = json.loads(json_str)
            return data
        except json.JSONDecodeError as e:
            raise AIClientError(f"解析数据提取结果失败: {str(e)}")

    def _extract_json(self, text: str) -> str:
        """
        从文本中提取 JSON（处理 markdown 代码块）

        Args:
            text: 可能包含 JSON 的文本

        Returns:
            纯 JSON 字符串
        """
        text = text.strip()

        # 处理 markdown 代码块
        if text.startswith("```"):
            # 移除开头的代码块标记
            first_newline = text.find("\n")
            if first_newline != -1:
                text = text[first_newline + 1:]

            # 移除结尾的代码块标记
            if text.endswith("```"):
                text = text[:-3]

        return text.strip()

    async def close(self):
        """关闭客户端连接"""
        await self.client.close()


async def test_ai_connection(config: AiConfig) -> dict:
    """
    测试 AI 配置连接是否正常

    Args:
        config: AI 配置对象

    Returns:
        包含 success, message, response 的字典
    """
    try:
        client = AIClient(config)
        # 简单的测试调用
        response = await client.call_api("请回复 OK")
        await client.close()

        success = len(response) > 0
        return {
            "success": success,
            "message": "连接成功" if success else "连接失败",
            "response": response[:200] if response else ""  # 截取前200字符
        }
    except Exception as e:
        return {
            "success": False,
            "message": f"连接失败: {str(e)}",
            "response": ""
        }

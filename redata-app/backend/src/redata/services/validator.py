"""
数据验证服务
提供字段格式验证和数据标准化功能
"""

import re
from typing import Any, Tuple, Optional, List
from dataclasses import dataclass

from ..models.project import ProjectField


@dataclass
class ValidationResult:
    """验证结果"""
    is_valid: bool
    errors: List[str]
    normalized_value: Any


class DataValidator:
    """数据格式验证器"""

    # 预定义验证规则（正则表达式）
    VALIDATORS = {
        "phone": r"^1[3-9]\d{9}$",  # 11位手机号
        "email": r"^[\w\.-]+@[\w\.-]+\.\w+$",  # 邮箱
        "url": r"^https?://",  # URL
        "date": r"^\d{4}[-/]\d{1,2}[-/]\d{1,2}$",  # 日期 (YYYY-MM-DD 或 YYYY/MM/DD)
        "number": r"^-?\d+(\.\d+)?$",  # 数字（整数或小数）
        "id_card": r"^\d{17}[\dXx]$",  # 18位身份证号
    }

    # 标准化函数映射
    NORMALIZERS = {
        "phone": "_normalize_phone",
        "email": "_normalize_email",
        "date": "_normalize_date",
        "number": "_normalize_number",
    }

    def validate(
        self,
        value: Any,
        field: ProjectField
    ) -> Tuple[bool, Optional[str]]:
        """
        验证数据

        Args:
            value: 要验证的值
            field: 字段定义（包含 field_type, validation_rule, is_required）

        Returns:
            (是否有效, 错误信息)
        """
        errors = []

        # 1. 必填检查
        if field.is_required:
            if value is None or (isinstance(value, str) and not value.strip()):
                return False, "必填字段不能为空"

        # 2. 空值跳过（非必填）
        if value is None or (isinstance(value, str) and not value.strip()):
            return True, None

        # 转换为字符串进行验证
        str_value = str(value).strip()

        # 3. 类型验证
        field_type = field.field_type or "text"
        if field_type in self.VALIDATORS:
            pattern = self.VALIDATORS[field_type]
            if not re.match(pattern, str_value):
                type_names = {
                    "phone": "手机号",
                    "email": "邮箱",
                    "url": "URL",
                    "date": "日期",
                    "number": "数字",
                    "id_card": "身份证号",
                }
                return False, f"格式不正确，期望 {type_names.get(field_type, field_type)} 格式"

        # 4. 自定义规则验证（如果有）
        if field.validation_rule:
            try:
                if not re.match(field.validation_rule, str_value):
                    return False, "不符合自定义验证规则"
            except re.error:
                # 正则表达式无效，跳过自定义验证
                pass

        return True, None

    def validate_record(
        self,
        record: dict,
        fields: List[ProjectField]
    ) -> Tuple[bool, List[str]]:
        """
        验证整条记录

        Args:
            record: 记录数据 {字段名: 值}
            fields: 字段定义列表

        Returns:
            (是否全部有效, 错误信息列表)
        """
        errors = []

        for field in fields:
            value = record.get(field.field_name)
            is_valid, error = self.validate(value, field)
            if not is_valid and error:
                errors.append(f"{field.field_label}: {error}")

        return len(errors) == 0, errors

    def normalize(
        self,
        value: Any,
        field_type: str
    ) -> Any:
        """
        标准化数据

        Args:
            value: 原始值
            field_type: 字段类型

        Returns:
            标准化后的值
        """
        if value is None:
            return None

        # 转换为字符串
        str_value = str(value).strip() if value else ""

        if not str_value:
            return ""

        # 查找对应的标准化函数
        normalizer_name = self.NORMALIZERS.get(field_type)
        if normalizer_name and hasattr(self, normalizer_name):
            return getattr(self, normalizer_name)(str_value)

        # 默认返回清理后的字符串
        return str_value

    def normalize_record(
        self,
        record: dict,
        fields: List[ProjectField]
    ) -> dict:
        """
        标准化整条记录

        Args:
            record: 记录数据 {字段名: 值}
            fields: 字段定义列表

        Returns:
            标准化后的记录
        """
        normalized = {}

        for field in fields:
            field_name = field.field_name
            field_type = field.field_type or "text"

            if field_name in record:
                normalized[field_name] = self.normalize(
                    record[field_name],
                    field_type
                )
            else:
                normalized[field_name] = None

        return normalized

    # ========== 内部标准化方法 ==========

    def _normalize_phone(self, value: str) -> str:
        """
        标准化手机号
        - 去除空格、横线
        - 提取数字部分
        """
        # 移除所有非数字字符
        digits = re.sub(r"[^\d]", "", value)

        # 如果以 86 开头且长度为 13，去掉 86
        if digits.startswith("86") and len(digits) == 13:
            digits = digits[2:]

        return digits

    def _normalize_email(self, value: str) -> str:
        """
        标准化邮箱
        - 转小写
        - 去除首尾空格
        """
        return value.lower().strip()

    def _normalize_date(self, value: str) -> str:
        """
        标准化日期
        - 统一为 YYYY-MM-DD 格式
        """
        # 替换 / 为 -
        normalized = value.replace("/", "-")

        # 尝试解析并重新格式化
        match = re.match(r"(\d{4})-(\d{1,2})-(\d{1,2})", normalized)
        if match:
            year, month, day = match.groups()
            return f"{year}-{month.zfill(2)}-{day.zfill(2)}"

        return normalized

    def _normalize_number(self, value: str) -> str:
        """
        标准化数字
        - 去除千分位分隔符
        - 统一格式
        """
        # 移除千分位分隔符
        normalized = value.replace(",", "")

        # 尝试转换为数字
        try:
            num = float(normalized)
            # 如果是整数，返回整数格式
            if num.is_integer():
                return str(int(num))
            return str(num)
        except ValueError:
            return normalized


class ColumnMappingValidator:
    """列映射验证器"""

    # 高置信度阈值
    HIGH_CONFIDENCE_THRESHOLD = 0.8

    def validate_confidence(self, confidence: float) -> Tuple[bool, str]:
        """
        验证置信度是否足够

        Args:
            confidence: 置信度 (0-1)

        Returns:
            (是否通过, 消息)
        """
        if confidence >= self.HIGH_CONFIDENCE_THRESHOLD:
            return True, "置信度较高，自动导入"
        elif confidence >= 0.5:
            return True, "置信度中等，建议检查"
        else:
            return False, "置信度较低，建议手动确认"

    def check_required_fields_mapped(
        self,
        column_mappings: dict,
        fields: List[ProjectField]
    ) -> Tuple[bool, List[str]]:
        """
        检查所有必填字段是否都有映射

        Args:
            column_mappings: 列映射 {列索引: 字段名}
            fields: 字段定义列表

        Returns:
            (是否全部映射, 未映射的必填字段列表)
        """
        mapped_fields = set(column_mappings.values())
        unmapped_required = []

        for field in fields:
            if field.is_required and field.field_name not in mapped_fields:
                unmapped_required.append(field.field_label)

        return len(unmapped_required) == 0, unmapped_required

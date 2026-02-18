"""
数据验证器测试
测试 DataValidator 服务
"""

import pytest
from unittest.mock import Mock
from src.redata.services.validator import DataValidator


def make_field(field_name="test_field", field_type="text", is_required=True, validation_rule=None):
    """创建模拟字段对象"""
    field = Mock()
    field.field_name = field_name
    field.field_type = field_type
    field.is_required = is_required
    field.validation_rule = validation_rule
    field.field_label = field_name
    return field


class TestDataValidator:
    """数据验证器测试类"""

    @pytest.fixture
    def validator(self):
        """创建验证器实例"""
        return DataValidator()

    # ========== 必填验证测试 ==========

    def test_validate_required_field_empty(self, validator):
        """测试必填字段为空"""
        field = make_field(is_required=True)
        is_valid, error = validator.validate("", field)
        assert is_valid == False
        assert "必填" in error or "不能为空" in error

    def test_validate_required_field_none(self, validator):
        """测试必填字段为 None"""
        field = make_field(is_required=True)
        is_valid, error = validator.validate(None, field)
        assert is_valid == False

    def test_validate_required_field_with_value(self, validator):
        """测试必填字段有值"""
        field = make_field(is_required=True)
        is_valid, error = validator.validate("有值", field)
        assert is_valid == True
        assert error is None

    def test_validate_optional_field_empty(self, validator):
        """测试可选字段为空"""
        field = make_field(is_required=False)
        is_valid, error = validator.validate("", field)
        assert is_valid == True  # 可选字段可以为空

    # ========== 手机号验证测试 ==========

    def test_validate_phone_valid(self, validator):
        """测试有效手机号"""
        field = make_field(field_name="phone", field_type="phone", is_required=True)
        valid_phones = ["13812345678", "15900001111", "18888888888"]
        for phone in valid_phones:
            is_valid, _ = validator.validate(phone, field)
            assert is_valid == True, f"手机号 {phone} 应该有效"

    def test_validate_phone_invalid(self, validator):
        """测试无效手机号"""
        field = make_field(field_name="phone", field_type="phone", is_required=True)
        invalid_phones = ["12345678901", "1381234567", "abcdefghijk", "138123456789"]
        for phone in invalid_phones:
            is_valid, error = validator.validate(phone, field)
            assert is_valid == False, f"手机号 {phone} 应该无效"

    # ========== 邮箱验证测试 ==========

    def test_validate_email_valid(self, validator):
        """测试有效邮箱"""
        field = make_field(field_name="email", field_type="email", is_required=True)
        valid_emails = ["test@example.com", "user.name@domain.org", "test123@test.co.jp"]
        for email in valid_emails:
            is_valid, _ = validator.validate(email, field)
            assert is_valid == True, f"邮箱 {email} 应该有效"

    def test_validate_email_invalid(self, validator):
        """测试无效邮箱"""
        field = make_field(field_name="email", field_type="email", is_required=True)
        invalid_emails = ["notanemail", "@example.com", "test@"]
        for email in invalid_emails:
            is_valid, error = validator.validate(email, field)
            assert is_valid == False, f"邮箱 {email} 应该无效"

    # ========== URL 验证测试 ==========

    def test_validate_url_valid(self, validator):
        """测试有效 URL"""
        field = make_field(field_name="url", field_type="url", is_required=True)
        valid_urls = ["https://example.com", "http://test.org/path", "https://www.google.com/search?q=test"]
        for url in valid_urls:
            is_valid, _ = validator.validate(url, field)
            assert is_valid == True, f"URL {url} 应该有效"

    def test_validate_url_invalid(self, validator):
        """测试无效 URL"""
        field = make_field(field_name="url", field_type="url", is_required=True)
        invalid_urls = ["notaurL", "ftp://file.com", "example.com"]
        for url in invalid_urls:
            is_valid, error = validator.validate(url, field)
            assert is_valid == False, f"URL {url} 应该无效"

    # ========== 日期验证测试 ==========

    def test_validate_date_valid(self, validator):
        """测试有效日期"""
        field = make_field(field_name="date", field_type="date", is_required=True)
        valid_dates = ["2024-01-15", "2024/12/31", "2024-1-1"]
        for date in valid_dates:
            is_valid, _ = validator.validate(date, field)
            assert is_valid == True, f"日期 {date} 应该有效"

    # ========== 数字验证测试 ==========

    def test_validate_number_valid(self, validator):
        """测试有效数字"""
        field = make_field(field_name="amount", field_type="number", is_required=True)
        valid_numbers = ["123", "45.67", "-10", "0"]
        for num in valid_numbers:
            is_valid, _ = validator.validate(num, field)
            assert is_valid == True, f"数字 {num} 应该有效"

    def test_validate_number_invalid(self, validator):
        """测试无效数字"""
        field = make_field(field_name="amount", field_type="number", is_required=True)
        invalid_numbers = ["abc", "12.34.56"]
        for num in invalid_numbers:
            is_valid, error = validator.validate(num, field)
            assert is_valid == False, f"数字 {num} 应该无效"

    # ========== 自定义验证规则测试 ==========

    def test_validate_custom_regex(self, validator):
        """测试自定义正则验证规则"""
        field = make_field(
            field_name="custom",
            field_type="text",
            is_required=True,
            validation_rule=r"^[A-Z]{3}\d{4}$"  # 如 ABC1234
        )
        is_valid, _ = validator.validate("ABC1234", field)
        assert is_valid == True

        is_valid, _ = validator.validate("abc1234", field)
        assert is_valid == False

    # ========== 批量验证测试 ==========

    def test_validate_record(self, validator):
        """测试整条记录验证"""
        fields = [
            make_field(field_name="name", field_type="text", is_required=True),
            make_field(field_name="phone", field_type="phone", is_required=True),
            make_field(field_name="email", field_type="email", is_required=False),
        ]
        record = {
            "name": "测试用户",
            "phone": "13812345678",
            "email": "test@example.com"
        }
        is_valid, errors = validator.validate_record(record, fields)
        assert is_valid == True
        assert len(errors) == 0

    def test_validate_record_with_errors(self, validator):
        """测试有错误的记录验证"""
        fields = [
            make_field(field_name="name", field_type="text", is_required=True),
            make_field(field_name="phone", field_type="phone", is_required=True),
        ]
        record = {
            "name": "",  # 空，必填
            "phone": "invalid"  # 格式错误
        }
        is_valid, errors = validator.validate_record(record, fields)
        assert is_valid == False
        assert len(errors) >= 1

    # ========== 标准化测试 ==========

    def test_normalize_phone(self, validator):
        """测试手机号标准化"""
        result = validator.normalize("138-1234-5678", "phone")
        assert result == "13812345678"

        result = validator.normalize("+86 138 1234 5678", "phone")
        assert result == "13812345678"

    def test_normalize_email(self, validator):
        """测试邮箱标准化"""
        result = validator.normalize("TEST@Example.COM", "email")
        assert result == "test@example.com"

    def test_normalize_date(self, validator):
        """测试日期标准化"""
        result = validator.normalize("2024/1/5", "date")
        assert result == "2024-01-05"

    def test_normalize_number(self, validator):
        """测试数字标准化"""
        result = validator.normalize("1,234.56", "number")
        assert result == "1234.56"

        result = validator.normalize("1000", "number")
        assert result == "1000"

"""
数据存储服务
负责动态表创建、管理、去重处理和记录 CRUD 操作
"""

from typing import List, Dict, Any, Optional
from datetime import datetime
from sqlalchemy import (
    Column, Integer, String, Text, DateTime,
    inspect, text, UniqueConstraint, Index
)
from sqlalchemy.orm import Session
from sqlalchemy.exc import IntegrityError
from dataclasses import dataclass

from ..db.base import get_db_url, engine as shared_engine
from ..models.project import Project, ProjectField


# 字段类型到 SQL 类型的映射
FIELD_TYPE_MAPPING = {
    "text": Text,
    "number": Integer,
    "email": String(255),
    "phone": String(20),
    "date": String(50),
    "url": String(500),
}


@dataclass
class QueryResult:
    """查询结果"""
    records: List[Dict[str, Any]]
    total: int
    page: int
    page_size: int


class StorageError(Exception):
    """存储服务错误"""
    pass


class StorageService:
    """数据存储服务"""

    def __init__(self, db: Session):
        """
        初始化存储服务

        Args:
            db: SQLAlchemy Session
        """
        self.db = db
        self.engine = shared_engine  # 使用共享 engine，避免多引擎连接问题
        self._column_cache: Dict[int, set] = {}  # 列名缓存: project_id -> set of column names

    def get_table_name(self, project_id: int) -> str:
        """
        获取项目数据表名

        Args:
            project_id: 项目 ID

        Returns:
            表名
        """
        return f"project_{project_id}_records"

    def table_exists(self, project_id: int) -> bool:
        """
        检查项目数据表是否存在

        Args:
            project_id: 项目 ID

        Returns:
            表是否存在
        """
        table_name = self.get_table_name(project_id)
        inspector = inspect(self.engine)
        return table_name in inspector.get_table_names()

    def create_project_table(
        self,
        project_id: int,
        fields: List[ProjectField]
    ) -> None:
        """
        创建项目数据表

        Args:
            project_id: 项目 ID
            fields: 字段定义列表
        """
        table_name = self.get_table_name(project_id)

        # 如果表已存在，先删除
        if self.table_exists(project_id):
            self.drop_project_table(project_id)

        # 构建 CREATE TABLE 语句
        columns_sql = [
            "id INTEGER PRIMARY KEY AUTOINCREMENT",
            # 动态字段列
        ]

        # 添加动态字段列
        for field in fields:
            # SQLite 类型映射：所有字符串类型都映射为 TEXT
            field_type = field.field_type
            if field_type in ("text", "email", "phone", "date", "url"):
                col_type_name = "TEXT"
            elif field_type == "number":
                col_type_name = "INTEGER"
            else:
                col_type_name = "TEXT"
            columns_sql.append(f'"{field.field_name}" {col_type_name}')

        # 添加元数据列
        columns_sql.extend([
            "raw_content TEXT",
            "source_file TEXT",
            "source_sheet TEXT",
            "row_number INTEGER",
            "batch_number TEXT",
            "status TEXT DEFAULT 'success'",
            "error_message TEXT",
            "created_at DATETIME DEFAULT CURRENT_TIMESTAMP",
            "updated_at DATETIME"
        ])

        create_sql = f'CREATE TABLE "{table_name}" ({", ".join(columns_sql)})'

        with self.engine.connect() as conn:
            conn.execute(text(create_sql))
            conn.commit()

        self._invalidate_column_cache(project_id)

    def drop_project_table(self, project_id: int) -> None:
        """
        删除项目数据表

        Args:
            project_id: 项目 ID
        """
        table_name = self.get_table_name(project_id)
        with self.engine.connect() as conn:
            conn.execute(text(f'DROP TABLE IF EXISTS "{table_name}"'))
            conn.commit()

    def rebuild_project_table(
        self,
        project_id: int,
        fields: List[ProjectField]
    ) -> None:
        """
        重建项目数据表（字段变更时使用）

        注意: 这会删除所有数据

        Args:
            project_id: 项目 ID
            fields: 新的字段定义列表
        """
        self.create_project_table(project_id, fields)

    def add_column_to_table(
        self,
        project_id: int,
        field: ProjectField
    ) -> bool:
        """
        向项目数据表添加列

        Args:
            project_id: 项目 ID
            field: 要添加的字段

        Returns:
            是否添加成功
        """
        table_name = self.get_table_name(project_id)

        # SQLite 类型映射：所有字符串类型都映射为 TEXT
        field_type = field.field_type
        if field_type in ("text", "email", "phone", "date", "url"):
            col_type_name = "TEXT"
        elif field_type == "number":
            col_type_name = "INTEGER"
        else:
            col_type_name = "TEXT"

        try:
            with self.engine.connect() as conn:
                conn.execute(text(f'ALTER TABLE "{table_name}" ADD COLUMN "{field.field_name}" {col_type_name}'))
                conn.commit()
            return True
        except Exception as e:
            print(f"Error adding column: {e}")
            return False

    def get_table_columns(self, project_id: int) -> List[str]:
        """
        获取项目数据表的所有列名

        Args:
            project_id: 项目 ID

        Returns:
            列名列表
        """
        table_name = self.get_table_name(project_id)
        inspector = inspect(self.engine)

        if not self.table_exists(project_id):
            return []

        columns = inspector.get_columns(table_name)
        return [col['name'] for col in columns]

    def migrate_table_structure(
        self,
        project_id: int,
        new_fields: List[ProjectField]
    ) -> None:
        """
        智能迁移表结构（尽可能保留数据）

        Args:
            project_id: 项目 ID
            new_fields: 新的字段定义列表
        """
        if not self.table_exists(project_id):
            # 表不存在，直接创建
            self.create_project_table(project_id, new_fields)
            return

        # 获取当前表结构
        current_columns = self.get_table_columns(project_id)
        meta_columns = {'id', 'raw_content', 'source_file', 'source_sheet',
                        'row_number', 'batch_number', 'status', 'error_message',
                        'created_at', 'updated_at'}
        current_field_columns = [col for col in current_columns if col not in meta_columns]

        # 计算新字段名
        new_field_names = {field.field_name for field in new_fields}

        # 检查是否只需要添加列（最简单的情况）
        columns_to_add = new_field_names - set(current_field_columns)
        columns_to_remove = set(current_field_columns) - new_field_names

        if not columns_to_remove and columns_to_add:
            # 只需要添加列，不需要删除
            field_map = {field.field_name: field for field in new_fields}
            for col_name in columns_to_add:
                self.add_column_to_table(project_id, field_map[col_name])
            self._invalidate_column_cache(project_id)
        elif columns_to_remove:
            # 需要删除列或重命名，必须重建表
            # 但我们要保留数据
            self._rebuild_table_with_data(project_id, new_fields)
            self._invalidate_column_cache(project_id)

    def _rebuild_table_with_data(
        self,
        project_id: int,
        new_fields: List[ProjectField]
    ) -> None:
        """
        重建表并尽可能保留数据

        Args:
            project_id: 项目 ID
            new_fields: 新的字段定义列表
        """
        table_name = self.get_table_name(project_id)
        temp_table_name = f"{table_name}_temp_{project_id}"

        # 获取当前表的所有数据
        with self.engine.connect() as conn:
            result = conn.execute(text(f'SELECT * FROM "{table_name}"'))
            rows = [dict(row._mapping) for row in result.fetchall()]

        # 重命名旧表
        with self.engine.connect() as conn:
            conn.execute(text(f'ALTER TABLE "{table_name}" RENAME TO "{temp_table_name}"'))
            conn.commit()

        # 创建新表
        self.create_project_table(project_id, new_fields)

        if not rows:
            # 没有数据，直接删除临时表
            with self.engine.connect() as conn:
                conn.execute(text(f'DROP TABLE IF EXISTS "{temp_table_name}"'))
                conn.commit()
            return

        # 获取新表的列名
        new_columns = self.get_table_columns(project_id)

        # 迁移数据（只保留共有的列）
        for row in rows:
            new_row = {}
            for col in new_columns:
                if col in row and row[col] is not None:
                    new_row[col] = row[col]

            if new_row:
                # 构建插入语句
                columns = [f'"{k}"' for k in new_row.keys()]
                placeholders = [f":{k}" for k in new_row.keys()]
                insert_sql = f'''
                    INSERT INTO "{table_name}" ({", ".join(columns)})
                    VALUES ({", ".join(placeholders)})
                '''
                try:
                    with self.engine.connect() as conn:
                        conn.execute(text(insert_sql), new_row)
                        conn.commit()
                except Exception:
                    pass  # 忽略单条记录的插入错误

        # 删除临时表
        with self.engine.connect() as conn:
            conn.execute(text(f'DROP TABLE IF EXISTS "{temp_table_name}"'))
            conn.commit()

    def _get_cached_columns(self, project_id: int) -> set:
        """获取缓存的列名集合"""
        if project_id not in self._column_cache:
            self._column_cache[project_id] = set(self.get_table_columns(project_id))
        return self._column_cache[project_id]

    def _invalidate_column_cache(self, project_id: int) -> None:
        """清除列名缓存"""
        self._column_cache.pop(project_id, None)

    def insert_record(
        self,
        project_id: int,
        data: Dict[str, Any],
        meta: Dict[str, Any]
    ) -> Optional[int]:
        """
        插入记录

        Args:
            project_id: 项目 ID
            data: 字段数据
            meta: 元数据（source_file, source_sheet, row_number, batch_number, status, error_message）

        Returns:
            新记录的 ID，如果因去重被跳过则返回 None
        """
        table_name = self.get_table_name(project_id)

        # 合并数据和元数据
        record = {**data}
        record["raw_content"] = meta.get("raw_content", "")
        record["source_file"] = meta.get("source_file", "")
        record["source_sheet"] = meta.get("source_sheet", "")
        record["row_number"] = meta.get("row_number", 0)
        record["batch_number"] = meta.get("batch_number", "")
        record["status"] = meta.get("status", "success")
        record["error_message"] = meta.get("error_message", "")
        record["created_at"] = datetime.now().isoformat()

        # 过滤掉表中不存在的列
        valid_columns = self._get_cached_columns(project_id)
        if valid_columns:
            record = {k: v for k, v in record.items() if k in valid_columns}

        # 构建插入语句
        columns = [f'"{k}"' for k in record.keys()]
        placeholders = [f":{k}" for k in record.keys()]

        insert_sql = f'''
            INSERT INTO "{table_name}" ({", ".join(columns)})
            VALUES ({", ".join(placeholders)})
        '''

        try:
            with self.engine.connect() as conn:
                result = conn.execute(text(insert_sql), record)
                conn.commit()
                return result.lastrowid
        except IntegrityError:
            # 唯一约束冲突（去重）
            return None

    def update_record(
        self,
        project_id: int,
        record_id: int,
        data: Dict[str, Any]
    ) -> bool:
        """
        更新记录

        Args:
            project_id: 项目 ID
            record_id: 记录 ID
            data: 要更新的字段数据

        Returns:
            是否更新成功
        """
        table_name = self.get_table_name(project_id)

        data["updated_at"] = datetime.now().isoformat()

        set_clauses = [f'"{k}" = :{k}' for k in data.keys()]
        update_sql = f'''
            UPDATE "{table_name}"
            SET {", ".join(set_clauses)}
            WHERE id = :record_id
        '''

        params = {**data, "record_id": record_id}

        with self.engine.connect() as conn:
            result = conn.execute(text(update_sql), params)
            conn.commit()
            return result.rowcount > 0

    def delete_record(self, project_id: int, record_id: int) -> bool:
        """
        删除记录

        Args:
            project_id: 项目 ID
            record_id: 记录 ID

        Returns:
            是否删除成功
        """
        table_name = self.get_table_name(project_id)

        delete_sql = f'DELETE FROM "{table_name}" WHERE id = :record_id'

        with self.engine.connect() as conn:
            result = conn.execute(text(delete_sql), {"record_id": record_id})
            conn.commit()
            return result.rowcount > 0

    def get_record(self, project_id: int, record_id: int) -> Optional[Dict[str, Any]]:
        """
        获取单条记录

        Args:
            project_id: 项目 ID
            record_id: 记录 ID

        Returns:
            记录字典，不存在返回 None
        """
        table_name = self.get_table_name(project_id)

        select_sql = f'SELECT * FROM "{table_name}" WHERE id = :record_id'

        with self.engine.connect() as conn:
            result = conn.execute(text(select_sql), {"record_id": record_id})
            row = result.fetchone()
            if row:
                return dict(row._mapping)
            return None

    def query_records(
        self,
        project_id: int,
        page: int = 1,
        page_size: int = 50,
        batch_number: Optional[str] = None,
        status: Optional[str] = None,
        search: Optional[str] = None,
        order_by: str = "id",
        order_desc: bool = True
    ) -> QueryResult:
        """
        查询记录（分页）

        Args:
            project_id: 项目 ID
            page: 页码
            page_size: 每页数量
            batch_number: 批次号筛选
            status: 状态筛选
            search: 搜索关键词
            order_by: 排序字段
            order_desc: 是否降序

        Returns:
            QueryResult 对象
        """
        table_name = self.get_table_name(project_id)

        # 构建 WHERE 条件
        conditions = []
        params = {}

        if batch_number:
            conditions.append("batch_number = :batch_number")
            params["batch_number"] = batch_number

        if status:
            conditions.append("status = :status")
            params["status"] = status

        if search:
            conditions.append("raw_content LIKE :search")
            params["search"] = f"%{search}%"

        where_clause = f"WHERE {' AND '.join(conditions)}" if conditions else ""

        # 排序
        order_direction = "DESC" if order_desc else "ASC"

        # 查询总数
        count_sql = f'SELECT COUNT(*) as total FROM "{table_name}" {where_clause}'
        with self.engine.connect() as conn:
            result = conn.execute(text(count_sql), params)
            total = result.fetchone()[0]

        # 分页查询
        offset = (page - 1) * page_size
        params["limit"] = page_size
        params["offset"] = offset

        select_sql = f'''
            SELECT * FROM "{table_name}"
            {where_clause}
            ORDER BY "{order_by}" {order_direction}
            LIMIT :limit OFFSET :offset
        '''

        with self.engine.connect() as conn:
            result = conn.execute(text(select_sql), params)
            records = [dict(row._mapping) for row in result.fetchall()]

        return QueryResult(
            records=records,
            total=total,
            page=page,
            page_size=page_size
        )

    def export_records(
        self,
        project_id: int,
        format: str = "xlsx",
        batch_number: Optional[str] = None
    ) -> bytes:
        """
        导出记录

        Args:
            project_id: 项目 ID
            format: 导出格式（xlsx 或 csv）
            batch_number: 批次号筛选

        Returns:
            导出文件的字节数据
        """
        import pandas as pd
        from io import BytesIO

        # 查询所有记录
        result = self.query_records(
            project_id,
            page=1,
            page_size=1000000,  # 获取所有记录
            batch_number=batch_number
        )

        if not result.records:
            return b""

        # 转换为 DataFrame
        df = pd.DataFrame(result.records)

        output = BytesIO()

        if format == "csv":
            df.to_csv(output, index=False, encoding="utf-8-sig")
        else:
            # 默认 xlsx
            df.to_excel(output, index=False, engine="openpyxl")

        return output.getvalue()

    def get_record_count(self, project_id: int) -> int:
        """
        获取记录总数

        Args:
            project_id: 项目 ID

        Returns:
            记录总数
        """
        table_name = self.get_table_name(project_id)

        if not self.table_exists(project_id):
            return 0

        count_sql = f'SELECT COUNT(*) as total FROM "{table_name}"'

        with self.engine.connect() as conn:
            result = conn.execute(text(count_sql))
            return result.fetchone()[0]

    def handle_dedup(
        self,
        project: Project,
        data: Dict[str, Any],
        storage: 'StorageService'
    ) -> Optional[int]:
        """
        处理去重逻辑

        Args:
            project: 项目对象
            data: 要插入的数据
            storage: 存储服务实例

        Returns:
            记录 ID，如果被跳过则返回 None
        """
        if not project.dedup_enabled:
            # 不去重，直接插入
            return None  # 由调用者执行插入

        dedup_fields = project.dedup_fields_list
        if not dedup_fields:
            return None

        # 检查是否存在重复
        table_name = self.get_table_name(project.id)
        conditions = []
        params = {}

        for field in dedup_fields:
            if field in data:
                conditions.append(f'"{field}" = :{field}')
                params[field] = data[field]

        if not conditions:
            return None

        check_sql = f'''
            SELECT id FROM "{table_name}"
            WHERE {" AND ".join(conditions)}
            LIMIT 1
        '''

        with self.engine.connect() as conn:
            result = conn.execute(text(check_sql), params)
            existing = result.fetchone()

        if existing:
            # 存在重复记录
            existing_id = existing[0]

            if project.dedup_strategy == "skip":
                # 跳过
                return None
            elif project.dedup_strategy == "update":
                # 更新
                return existing_id  # 返回 ID 让调用者更新
            elif project.dedup_strategy == "merge":
                # 合并（这里简化为更新非空值）
                return existing_id

        return None  # 无重复，由调用者执行插入

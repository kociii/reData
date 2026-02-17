"""
数据提取协调器
协调 AI 客户端、Excel 解析器和存储服务，完成数据提取流程
"""

import asyncio
import json
import uuid
from typing import List, Dict, Any, Optional, Callable
from datetime import datetime
from dataclasses import dataclass, field
from pathlib import Path
import shutil

from sqlalchemy.orm import Session

from .ai_client import AIClient, AIClientError, HeaderRecognitionResult, ColumnMapping
from .excel_parser import ExcelParser, ExcelParserError
from .storage import StorageService
from .validator import DataValidator, ColumnMappingValidator
from ..models.project import Project, ProjectField, ProcessingTask, AiConfig, Batch


@dataclass
class ProcessingProgress:
    """处理进度"""
    task_id: str
    event: str
    current_file: str = ""
    current_sheet: str = ""
    current_row: int = 0
    total_rows: int = 0
    processed_rows: int = 0
    success_count: int = 0
    error_count: int = 0
    speed: float = 0.0  # 行/秒
    message: str = ""


@dataclass
class ProcessingResult:
    """处理结果"""
    task_id: str
    success: bool
    total_files: int
    processed_files: int
    total_rows: int
    success_count: int
    error_count: int
    batch_number: str
    error_message: Optional[str] = None


class ExtractorError(Exception):
    """提取器错误"""
    pass


class Extractor:
    """数据提取协调器"""

    def __init__(
        self,
        db: Session,
        project: Project,
        ai_config: AiConfig,
        progress_callback: Optional[Callable[[ProcessingProgress], None]] = None
    ):
        """
        初始化提取器

        Args:
            db: 数据库 Session
            project: 项目对象
            ai_config: AI 配置对象
            progress_callback: 进度回调函数
        """
        self.db = db
        self.project = project
        self.ai_config = ai_config
        self.progress_callback = progress_callback

        # 状态控制
        self.paused = False
        self.cancelled = False

        # 服务实例
        self.ai_client = AIClient(ai_config)
        self.storage = StorageService(db)
        self.validator = DataValidator()
        self.mapping_validator = ColumnMappingValidator()

        # 字段定义（缓存）
        self._fields: Optional[List[ProjectField]] = None

        # 进度跟踪
        self.task_id: str = ""
        self.batch_number: str = ""
        self.start_time: float = 0
        self.processed_count: int = 0

    @property
    def fields(self) -> List[ProjectField]:
        """获取项目字段定义（带缓存）"""
        if self._fields is None:
            self._fields = self.db.query(ProjectField).filter(
                ProjectField.project_id == self.project.id
            ).order_by(ProjectField.display_order).all()
        return self._fields

    def send_progress(self, event: str, **kwargs) -> None:
        """
        发送进度更新

        Args:
            event: 事件类型
            **kwargs: 进度数据
        """
        if self.progress_callback:
            progress = ProcessingProgress(
                task_id=self.task_id,
                event=event,
                **kwargs
            )
            self.progress_callback(progress)

    async def process_files(
        self,
        file_paths: List[str],
        task_id: Optional[str] = None
    ) -> ProcessingResult:
        """
        处理多个文件

        Args:
            file_paths: 文件路径列表
            task_id: 任务 ID（可选，不提供则自动生成）

        Returns:
            ProcessingResult 对象
        """
        # 初始化任务
        self.task_id = task_id or str(uuid.uuid4())
        self.batch_number = self._generate_batch_number()
        self.start_time = asyncio.get_event_loop().time()
        self.processed_count = 0

        # 创建任务记录
        task = ProcessingTask(
            id=self.task_id,
            project_id=self.project.id,
            status="processing",
            total_files=len(file_paths),
            batch_number=self.batch_number
        )
        self.db.add(task)
        self.db.commit()

        # 创建批次目录
        batch_dir = self._create_batch_directory()
        copied_files = []

        # 复制文件到批次目录
        for file_path in file_paths:
            try:
                dest_path = Path(batch_dir) / Path(file_path).name
                shutil.copy2(file_path, dest_path)
                copied_files.append(str(dest_path))
            except Exception as e:
                self.send_progress("error", message=f"复制文件失败: {e}")

        # 创建批次记录
        batch = Batch(
            batch_number=self.batch_number,
            project_id=self.project.id,
            file_count=len(copied_files)
        )
        self.db.add(batch)
        self.db.commit()

        # 确保项目数据表存在
        if not self.storage.table_exists(self.project.id):
            self.storage.create_project_table(self.project.id, self.fields)

        total_rows = 0
        success_count = 0
        error_count = 0
        processed_files = 0

        try:
            for file_path in copied_files:
                if self.cancelled:
                    break

                while self.paused:
                    await asyncio.sleep(0.1)

                try:
                    result = await self.process_file(file_path)
                    total_rows += result.get("total_rows", 0)
                    success_count += result.get("success_count", 0)
                    error_count += result.get("error_count", 0)
                    processed_files += 1

                    # 更新任务进度
                    task.processed_files = processed_files
                    task.total_rows = total_rows
                    task.processed_rows = success_count + error_count
                    task.success_count = success_count
                    task.error_count = error_count
                    self.db.commit()

                except Exception as e:
                    self.send_progress("error", message=f"处理文件失败: {e}")
                    error_count += 1

            # 更新最终状态
            final_status = "cancelled" if self.cancelled else "completed"
            task.status = final_status
            self.db.commit()

            # 更新批次记录数
            batch.record_count = success_count
            self.db.commit()

        except Exception as e:
            task.status = "error"
            self.db.commit()
            return ProcessingResult(
                task_id=self.task_id,
                success=False,
                total_files=len(file_paths),
                processed_files=processed_files,
                total_rows=total_rows,
                success_count=success_count,
                error_count=error_count,
                batch_number=self.batch_number,
                error_message=str(e)
            )

        await self.ai_client.close()

        return ProcessingResult(
            task_id=self.task_id,
            success=not self.cancelled,
            total_files=len(file_paths),
            processed_files=processed_files,
            total_rows=total_rows,
            success_count=success_count,
            error_count=error_count,
            batch_number=self.batch_number
        )

    async def process_file(self, file_path: str) -> Dict[str, Any]:
        """
        处理单个文件

        Args:
            file_path: 文件路径

        Returns:
            处理结果统计
        """
        self.send_progress("file_start", current_file=file_path)

        total_rows = 0
        success_count = 0
        error_count = 0

        with ExcelParser(file_path) as parser:
            sheets = parser.get_sheets()

            for sheet_name in sheets:
                if self.cancelled:
                    break

                while self.paused:
                    await asyncio.sleep(0.1)

                sheet_result = await self.process_sheet(
                    parser,
                    sheet_name,
                    file_path
                )

                total_rows += sheet_result.get("total_rows", 0)
                success_count += sheet_result.get("success_count", 0)
                error_count += sheet_result.get("error_count", 0)

        self.send_progress(
            "file_complete",
            current_file=file_path,
            message=f"文件处理完成: {success_count} 成功, {error_count} 失败"
        )

        return {
            "total_rows": total_rows,
            "success_count": success_count,
            "error_count": error_count
        }

    async def process_sheet(
        self,
        parser: ExcelParser,
        sheet_name: str,
        file_path: str
    ) -> Dict[str, Any]:
        """
        处理单个 Sheet（两阶段处理）

        阶段一：AI 分析列映射（每 sheet 仅 1 次 AI 调用）
        阶段二：本地验证导入（无 AI 调用）

        Args:
            parser: Excel 解析器
            sheet_name: Sheet 名称
            file_path: 文件路径

        Returns:
            处理结果统计
        """
        self.send_progress("sheet_start", current_sheet=sheet_name)

        sheet = parser.get_sheet(sheet_name)

        # ========== 阶段一：AI 分析列映射（每 sheet 仅 1 次）==========

        # 1. 读取前 10 行样本
        sample_rows = parser.read_rows(sheet, start_row=1, count=10)

        # 2. AI 分析列映射
        try:
            mapping = await self.ai_client.analyze_column_mapping(
                sample_rows=sample_rows,
                fields=self.fields
            )
        except AIClientError as e:
            self.send_progress("error", message=f"列映射分析失败: {e}")
            # 返回空结果
            return {
                "total_rows": 0,
                "success_count": 0,
                "error_count": 0,
                "error_message": f"列映射分析失败: {e}"
            }

        # 3. 记录映射结果
        self.send_progress(
            "column_mapping",
            current_sheet=sheet_name,
            header_row=mapping.header_row,
            mappings={str(k): v for k, v in mapping.column_mappings.items()},
            confidence=mapping.confidence,
            unmatched_columns=mapping.unmatched_columns
        )

        # 4. 检查必填字段是否都有映射
        is_valid, unmapped_required = self.mapping_validator.check_required_fields_mapped(
            mapping.column_mappings,
            self.fields
        )
        if not is_valid:
            self.send_progress(
                "warning",
                message=f"以下必填字段未匹配: {', '.join(unmapped_required)}"
            )

        # 5. 确定数据起始行
        start_row = mapping.header_row + 1 if mapping.header_row > 0 else 1

        # ========== 阶段二：本地验证导入（无 AI 调用）==========

        total_rows = parser.get_total_rows(sheet, start_row)
        success_count = 0
        error_count = 0
        processed = 0

        # 获取需要读取的列索引
        column_indices = list(mapping.column_mappings.keys())

        # 如果没有映射，跳过这个 sheet
        if not column_indices:
            self.send_progress(
                "sheet_complete",
                current_sheet=sheet_name,
                message="没有可用的列映射，跳过此 Sheet"
            )
            return {
                "total_rows": 0,
                "success_count": 0,
                "error_count": 0
            }

        # 迭代处理行
        for row_num, row_data in parser.iterate_rows(sheet, start_row):
            if self.cancelled:
                break

            while self.paused:
                await asyncio.sleep(0.1)

            processed += 1

            try:
                # 根据映射提取字段值
                record = {}
                for col_idx, field_name in mapping.column_mappings.items():
                    if col_idx < len(row_data):
                        record[field_name] = row_data[col_idx]
                    else:
                        record[field_name] = None

                # 标准化数据
                normalized_record = self.validator.normalize_record(record, self.fields)

                # 格式验证
                is_valid_record, errors = self.validator.validate_record(
                    normalized_record,
                    self.fields
                )

                if is_valid_record:
                    # 去重处理
                    existing_id = None
                    if self.project.dedup_enabled:
                        existing_id = self.storage.handle_dedup(
                            self.project,
                            normalized_record,
                            self.storage
                        )

                    # 准备元数据
                    meta = {
                        "raw_content": self._format_row_for_storage(row_data, mapping),
                        "source_file": Path(file_path).name,
                        "source_sheet": sheet_name,
                        "row_number": row_num,
                        "batch_number": self.batch_number,
                        "status": "success",
                        "column_mapping_confidence": mapping.confidence
                    }

                    if existing_id:
                        # 更新现有记录
                        if self.project.dedup_strategy in ["update", "merge"]:
                            self.storage.update_record(
                                self.project.id,
                                existing_id,
                                normalized_record
                            )
                    else:
                        # 插入新记录
                        self.storage.insert_record(self.project.id, normalized_record, meta)

                    success_count += 1
                else:
                    # 验证失败，保存错误记录
                    self._save_error_record(
                        raw_content=self._format_row_for_storage(row_data, mapping),
                        source_file=Path(file_path).name,
                        source_sheet=sheet_name,
                        row_number=row_num,
                        error_message="; ".join(errors)
                    )
                    error_count += 1

            except Exception as e:
                error_count += 1
                # 记录错误但继续处理
                self._save_error_record(
                    raw_content=str(row_data),
                    source_file=Path(file_path).name,
                    source_sheet=sheet_name,
                    row_number=row_num,
                    error_message=str(e)
                )

            # 计算速度
            elapsed = asyncio.get_event_loop().time() - self.start_time
            speed = processed / elapsed if elapsed > 0 else 0

            # 发送进度
            self.send_progress(
                "row_processed",
                current_sheet=sheet_name,
                current_row=row_num,
                total_rows=total_rows,
                processed_rows=processed,
                success_count=success_count,
                error_count=error_count,
                speed=speed
            )

        self.send_progress(
            "sheet_complete",
            current_sheet=sheet_name,
            message=f"Sheet 处理完成: {success_count} 成功, {error_count} 失败"
        )

        return {
            "total_rows": total_rows,
            "success_count": success_count,
            "error_count": error_count
        }

    def _format_row_for_storage(
        self,
        row_data: List[Any],
        mapping: ColumnMapping
    ) -> str:
        """
        格式化行数据用于存储

        Args:
            row_data: 行数据
            mapping: 列映射

        Returns:
            格式化的字符串
        """
        pairs = []
        for col_idx, field_name in mapping.column_mappings.items():
            if col_idx < len(row_data):
                value = row_data[col_idx]
                if value is not None:
                    pairs.append(f"{field_name}:{value}")

        return "; ".join(pairs) if pairs else "(空行)"

    def _save_error_record(
        self,
        raw_content: str,
        source_file: str,
        source_sheet: str,
        row_number: int,
        error_message: str
    ) -> None:
        """
        保存错误记录

        Args:
            raw_content: 原始内容
            source_file: 源文件名
            source_sheet: 源 Sheet 名
            row_number: 行号
            error_message: 错误信息
        """
        meta = {
            "raw_content": raw_content,
            "source_file": source_file,
            "source_sheet": source_sheet,
            "row_number": row_number,
            "batch_number": self.batch_number,
            "status": "error",
            "error_message": error_message
        }
        self.storage.insert_record(self.project.id, {}, meta)

    def _generate_batch_number(self) -> str:
        """
        生成批次号

        Returns:
            批次号（格式: batch_YYYYMMDD_XXXX）
        """
        date_str = datetime.now().strftime("%Y%m%d")

        # 查询今天的最大批次号
        from sqlalchemy import func
        max_batch = self.db.query(Batch.batch_number).filter(
            Batch.batch_number.like(f"batch_{date_str}_%")
        ).order_by(Batch.batch_number.desc()).first()

        if max_batch:
            last_num = int(max_batch[0].split("_")[-1])
            new_num = last_num + 1
        else:
            new_num = 1

        return f"batch_{date_str}_{new_num:04d}"

    def _create_batch_directory(self) -> str:
        """
        创建批次目录

        Returns:
            批次目录路径
        """
        # 获取项目根目录下的 history 目录
        backend_dir = Path(__file__).parent.parent.parent.parent
        history_dir = backend_dir / "history" / self.batch_number
        history_dir.mkdir(parents=True, exist_ok=True)
        return str(history_dir)

    def pause(self) -> None:
        """暂停处理"""
        self.paused = True
        self._update_task_status("paused")

    def resume(self) -> None:
        """恢复处理"""
        self.paused = False
        self._update_task_status("processing")

    def cancel(self) -> None:
        """取消处理"""
        self.cancelled = True
        self._update_task_status("cancelled")

    def _update_task_status(self, status: str) -> None:
        """
        更新任务状态

        Args:
            status: 新状态
        """
        task = self.db.query(ProcessingTask).filter(
            ProcessingTask.id == self.task_id
        ).first()
        if task:
            task.status = status
            self.db.commit()


# 活动任务管理
_active_extractors: Dict[str, Extractor] = {}


def get_extractor(task_id: str) -> Optional[Extractor]:
    """
    获取活动的提取器

    Args:
        task_id: 任务 ID

    Returns:
        Extractor 对象，不存在返回 None
    """
    return _active_extractors.get(task_id)


def register_extractor(task_id: str, extractor: Extractor) -> None:
    """
    注册提取器

    Args:
        task_id: 任务 ID
        extractor: Extractor 对象
    """
    _active_extractors[task_id] = extractor


def unregister_extractor(task_id: str) -> None:
    """
    注销提取器

    Args:
        task_id: 任务 ID
    """
    if task_id in _active_extractors:
        del _active_extractors[task_id]

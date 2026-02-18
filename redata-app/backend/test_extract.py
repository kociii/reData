"""测试数据提取脚本"""
import asyncio
import sys
sys.path.insert(0, 'src')

from redata.db.base import SessionLocal
from redata.models.project import Project, AiConfig
from redata.services.extractor import Extractor
from redata.services.ai_client import AIClient


async def test_extraction():
    db = SessionLocal()

    try:
        # 获取项目
        project = db.query(Project).filter(Project.id == 3).first()
        if not project:
            print("项目不存在")
            return

        print(f"项目: {project.name}")
        print(f"去重启用: {project.dedup_enabled}")

        # 获取 AI 配置
        ai_config = db.query(AiConfig).filter(AiConfig.is_default == True).first()
        if not ai_config:
            print("没有默认 AI 配置")
            return

        print(f"AI 配置: {ai_config.model_name}")

        # 创建提取器，添加更详细的调试回调
        def debug_progress(p):
            msg = f"进度: {p.event}"
            if p.event == "row_processed":
                msg += f" - 行: {p.current_row}/{p.total_rows}, 成功: {p.success_count}, 错误: {p.error_count}"
            elif p.event == "column_mapping":
                msg += f" - mappings: {p.mappings}"
            elif p.event == "error":
                msg += f" - {p.message}"
            else:
                msg += f" - {p.message}"
            print(msg)

        extractor = Extractor(
            db=db,
            project=project,
            ai_config=ai_config,
            progress_callback=debug_progress
        )

        # 设置 task_id 和 batch_number
        extractor.task_id = "test-task-001"
        extractor.batch_number = "batch_test_001"

        # 测试文件路径
        file_path = "/Users/ziyi/Desktop/data/reData/redata-app/backend/history/batch_20260218_0009/安徽卖家550.xlsx"

        # 打印字段信息
        fields = extractor.fields
        print(f"\n字段列表 ({len(fields)} 个):")
        for f in fields:
            print(f"  - {f.field_name} ({f.field_label}), 类型: {f.field_type}, 必填: {f.is_required}")

        # 检查表是否存在
        from redata.services.storage import StorageService
        storage = StorageService(db)
        table_exists = storage.table_exists(project.id)
        print(f"\n表是否存在: {table_exists}")

        if table_exists:
            columns = storage.get_table_columns(project.id)
            print(f"表列: {columns}")

            # 检查记录数
            count = storage.get_record_count(project.id)
            print(f"当前记录数: {count}")

        # 尝试处理文件
        print(f"\n开始处理文件: {file_path}")

        result = await extractor.process_files([file_path])

        print(f"\n处理结果:")
        print(f"  成功: {result.success}")
        print(f"  总文件: {result.total_files}")
        print(f"  已处理文件: {result.processed_files}")
        print(f"  总行数: {result.total_rows}")
        print(f"  成功数: {result.success_count}")
        print(f"  错误数: {result.error_count}")

        if result.error_message:
            print(f"  错误信息: {result.error_message}")

        # 检查最终记录数
        count = storage.get_record_count(project.id)
        print(f"\n最终记录数: {count}")

        # 检查错误记录
        print("\n错误记录示例:")
        errors = storage.query_records(project.id, status="error", page_size=5)
        for e in errors.records:
            print(f"  - {e.get('error_message', 'N/A')}")

    except Exception as e:
        import traceback
        print(f"错误: {e}")
        traceback.print_exc()
    finally:
        db.close()


if __name__ == "__main__":
    asyncio.run(test_extraction())

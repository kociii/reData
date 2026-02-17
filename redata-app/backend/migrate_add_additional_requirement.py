"""
添加 additional_requirement 字段到 project_fields 表
"""
import sqlite3
import os

# 数据库文件路径
DB_PATH = os.path.join(os.path.dirname(__file__), 'data', 'app.db')

def migrate():
    """执行迁移"""
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()

    try:
        # 检查字段是否已存在
        cursor.execute("PRAGMA table_info(project_fields)")
        columns = [row[1] for row in cursor.fetchall()]

        if 'additional_requirement' not in columns:
            print("添加 additional_requirement 字段...")
            cursor.execute("""
                ALTER TABLE project_fields
                ADD COLUMN additional_requirement TEXT
            """)
            conn.commit()
            print("✓ 迁移成功")
        else:
            print("✓ 字段已存在，无需迁移")

    except Exception as e:
        print(f"✗ 迁移失败: {e}")
        conn.rollback()
    finally:
        conn.close()

if __name__ == '__main__':
    migrate()

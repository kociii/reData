"""
数据库迁移脚本：为 project_fields 表添加 is_dedup_key 字段
"""
import sqlite3
import os

def migrate():
    db_path = os.path.join(os.path.dirname(__file__), 'data', 'app.db')

    if not os.path.exists(db_path):
        print("数据库文件不存在，将在首次运行时自动创建")
        return

    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()

    # 检查列是否已存在
    cursor.execute("PRAGMA table_info(project_fields)")
    columns = [col[1] for col in cursor.fetchall()]

    if 'is_dedup_key' not in columns:
        print("添加 is_dedup_key 列...")
        cursor.execute("ALTER TABLE project_fields ADD COLUMN is_dedup_key BOOLEAN DEFAULT 0")
        conn.commit()
        print("迁移完成!")
    else:
        print("is_dedup_key 列已存在，跳过迁移")

    conn.close()

if __name__ == "__main__":
    migrate()

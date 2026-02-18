"""
数据库迁移脚本
为 project_fields 表添加 is_deleted 和 deleted_at 列
"""

import os
import sqlite3

DB_PATH = os.path.join(os.path.dirname(os.path.dirname(__file__)), "data", "app.db")

def migrate():
    """执行迁移"""
    if not os.path.exists(DB_PATH):
        print(f"数据库文件不存在: {DB_PATH}")
        return

    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()

    # 检查列是否存在
    cursor.execute("PRAGMA table_info(project_fields)")
    columns = [row[1] for row in cursor.fetchall()]

    print(f"当前表结构: {columns}")

    # 添加 is_deleted 列
    if 'is_deleted' not in columns:
        cursor.execute("ALTER TABLE project_fields ADD COLUMN is_deleted BOOLEAN DEFAULT 0")
        print("添加列: is_deleted")

    # 添加 deleted_at 列
    if 'deleted_at' not in columns:
        cursor.execute("ALTER TABLE project_fields ADD COLUMN deleted_at TIMESTAMP")
        print("添加列: deleted_at")

    conn.commit()
    conn.close()
    print("迁移完成!")

if __name__ == "__main__":
    migrate()

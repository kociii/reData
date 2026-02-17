from sqlalchemy import create_engine
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker
import os

# 数据库文件路径
DB_PATH = os.path.join(os.getcwd(), "data", "app.db")
os.makedirs(os.path.dirname(DB_PATH), exist_ok=True)

# 数据库连接
SQLALCHEMY_DATABASE_URL = f"sqlite:///{DB_PATH}"


def get_db_url() -> str:
    """获取数据库 URL"""
    return SQLALCHEMY_DATABASE_URL

engine = create_engine(
    SQLALCHEMY_DATABASE_URL,
    connect_args={"check_same_thread": False}
)

SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)

Base = declarative_base()

def get_db():
    """获取数据库会话"""
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()

def init_db():
    """初始化数据库"""
    Base.metadata.create_all(bind=engine)

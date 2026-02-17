"""
文件 API 路由
处理文件上传、预览和批次管理
"""

import os
import uuid
import shutil
from typing import List, Optional
from pathlib import Path
from fastapi import APIRouter, UploadFile, File, HTTPException, Form
from fastapi.responses import FileResponse
from pydantic import BaseModel

from ..services.excel_parser import ExcelParser, ExcelParserError, get_excel_preview


router = APIRouter()


# ========== Schemas ==========

class FileInfo(BaseModel):
    """文件信息"""
    file_id: str
    file_name: str
    file_path: str
    file_size: int
    sheets: List[str] = []


class UploadResponse(BaseModel):
    """上传响应"""
    file_id: str
    file_name: str
    file_path: str
    file_size: int


class PreviewResponse(BaseModel):
    """预览响应"""
    file_id: str
    sheet_name: str
    sheets: List[dict]
    rows: List[List]


class BatchFilesResponse(BaseModel):
    """批次文件响应"""
    batch_number: str
    files: List[FileInfo]


# ========== 临时文件存储 ==========

# 临时文件目录
TEMP_DIR = Path(__file__).parent.parent.parent / "temp" / "uploads"
TEMP_DIR.mkdir(parents=True, exist_ok=True)

# 批次目录
HISTORY_DIR = Path(__file__).parent.parent.parent / "history"


# ========== Routes ==========

@router.post("/upload", response_model=UploadResponse)
async def upload_file(file: UploadFile = File(...)):
    """
    上传文件

    Args:
        file: 上传的文件

    Returns:
        UploadResponse
    """
    # 检查文件类型
    if not file.filename:
        raise HTTPException(status_code=400, detail="文件名不能为空")

    suffix = Path(file.filename).suffix.lower()
    if suffix not in [".xlsx", ".xls"]:
        raise HTTPException(status_code=400, detail="只支持 Excel 文件 (.xlsx, .xls)")

    # 生成文件 ID
    file_id = str(uuid.uuid4())

    # 保存文件
    file_path = TEMP_DIR / f"{file_id}{suffix}"

    try:
        with open(file_path, "wb") as f:
            shutil.copyfileobj(file.file, f)
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"保存文件失败: {str(e)}")

    # 获取文件大小
    file_size = os.path.getsize(file_path)

    return UploadResponse(
        file_id=file_id,
        file_name=file.filename,
        file_path=str(file_path),
        file_size=file_size
    )


@router.post("/upload-multiple", response_model=List[UploadResponse])
async def upload_multiple_files(files: List[UploadFile] = File(...)):
    """
    批量上传文件

    Args:
        files: 上传的文件列表

    Returns:
        UploadResponse 列表
    """
    results = []
    for file in files:
        result = await upload_file(file)
        results.append(result)
    return results


@router.get("/preview/{file_id}", response_model=PreviewResponse)
async def preview_file(file_id: str, sheet_name: Optional[str] = None):
    """
    预览文件

    Args:
        file_id: 文件 ID
        sheet_name: 指定 Sheet 名称（可选）

    Returns:
        PreviewResponse
    """
    # 查找文件
    file_path = None
    for f in TEMP_DIR.iterdir():
        if f.stem == file_id:
            file_path = f
            break

    if not file_path or not file_path.exists():
        raise HTTPException(status_code=404, detail="文件不存在")

    try:
        preview = get_excel_preview(str(file_path), sheet_name)
    except ExcelParserError as e:
        raise HTTPException(status_code=400, detail=str(e))

    return PreviewResponse(
        file_id=file_id,
        sheet_name=preview.sheet_name,
        sheets=[{"name": s.name, "rows": s.row_count, "columns": s.column_count} for s in preview.sheets],
        rows=preview.rows
    )


@router.get("/info/{file_id}", response_model=FileInfo)
async def get_file_info(file_id: str):
    """
    获取文件信息

    Args:
        file_id: 文件 ID

    Returns:
        FileInfo
    """
    # 查找文件
    file_path = None
    for f in TEMP_DIR.iterdir():
        if f.stem == file_id:
            file_path = f
            break

    if not file_path or not file_path.exists():
        raise HTTPException(status_code=404, detail="文件不存在")

    # 获取文件大小
    file_size = os.path.getsize(file_path)

    # 获取 Sheet 信息
    sheets = []
    try:
        with ExcelParser(str(file_path)) as parser:
            sheets = parser.get_sheets()
    except ExcelParserError:
        pass

    return FileInfo(
        file_id=file_id,
        file_name=file_path.name,
        file_path=str(file_path),
        file_size=file_size,
        sheets=sheets
    )


@router.delete("/{file_id}")
async def delete_file(file_id: str):
    """
    删除临时文件

    Args:
        file_id: 文件 ID

    Returns:
        成功消息
    """
    # 查找文件
    file_path = None
    for f in TEMP_DIR.iterdir():
        if f.stem == file_id:
            file_path = f
            break

    if not file_path or not file_path.exists():
        raise HTTPException(status_code=404, detail="文件不存在")

    try:
        os.remove(file_path)
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"删除文件失败: {str(e)}")

    return {"message": "文件已删除"}


@router.get("/batch/{batch_number}", response_model=BatchFilesResponse)
async def get_batch_files(batch_number: str):
    """
    获取批次文件列表

    Args:
        batch_number: 批次号

    Returns:
        BatchFilesResponse
    """
    batch_dir = HISTORY_DIR / batch_number

    if not batch_dir.exists():
        raise HTTPException(status_code=404, detail="批次不存在")

    files = []
    for f in batch_dir.iterdir():
        if f.is_file() and f.suffix.lower() in [".xlsx", ".xls"]:
            # 获取 Sheet 信息
            sheets = []
            try:
                with ExcelParser(str(f)) as parser:
                    sheets = parser.get_sheets()
            except ExcelParserError:
                pass

            files.append(FileInfo(
                file_id=f.stem,
                file_name=f.name,
                file_path=str(f),
                file_size=os.path.getsize(f),
                sheets=sheets
            ))

    return BatchFilesResponse(
        batch_number=batch_number,
        files=files
    )


@router.get("/download/{file_id}")
async def download_file(file_id: str):
    """
    下载文件

    Args:
        file_id: 文件 ID

    Returns:
        FileResponse
    """
    # 查找文件
    file_path = None
    for f in TEMP_DIR.iterdir():
        if f.stem == file_id:
            file_path = f
            break

    if not file_path or not file_path.exists():
        raise HTTPException(status_code=404, detail="文件不存在")

    return FileResponse(
        path=file_path,
        filename=file_path.name,
        media_type="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    )


@router.post("/cleanup")
async def cleanup_temp_files():
    """
    清理临时文件

    Returns:
        清理结果
    """
    cleaned = 0
    for f in TEMP_DIR.iterdir():
        try:
            if f.is_file():
                f.unlink()
                cleaned += 1
        except Exception:
            pass

    return {"message": f"已清理 {cleaned} 个临时文件"}

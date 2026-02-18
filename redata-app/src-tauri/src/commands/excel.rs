// Excel 解析 Tauri Commands
//
// 使用 calamine 读取 .xlsx/.xls 文件

use calamine::{open_workbook_auto, Reader, Data};
use serde::Serialize;

// ============ 响应结构 ============

#[derive(Debug, Serialize)]
pub struct SheetInfoResponse {
    pub name: String,
    pub row_count: u32,
    pub column_count: u32,
}

#[derive(Debug, Serialize)]
pub struct ExcelPreviewResponse {
    pub sheets: Vec<SheetInfoResponse>,
    pub rows: Vec<Vec<String>>,
    pub sheet_name: String,
}

// ============ 辅助函数 ============

fn data_to_string(data: &Data) -> String {
    match data {
        Data::Int(i) => i.to_string(),
        Data::Float(f) => {
            if *f == (*f as i64) as f64 {
                (*f as i64).to_string()
            } else {
                f.to_string()
            }
        }
        Data::String(s) => s.clone(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => dt.to_string(),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("#ERR:{:?}", e),
        Data::Empty => String::new(),
    }
}

// ============ Tauri Commands ============

/// 获取 Excel 文件的所有 Sheet 信息
#[tauri::command]
pub async fn get_excel_sheets(
    file_path: String,
) -> Result<Vec<SheetInfoResponse>, String> {
    let path = file_path.clone();
    tokio::task::spawn_blocking(move || {
        let mut workbook = open_workbook_auto(&path)
            .map_err(|e| format!("无法打开文件 {}: {}", path, e))?;

        let sheet_names = workbook.sheet_names().to_vec();
        let mut sheets = Vec::new();

        for name in &sheet_names {
            if let Ok(range) = workbook.worksheet_range(name) {
                let (rows, cols) = range.get_size();
                sheets.push(SheetInfoResponse {
                    name: name.clone(),
                    row_count: rows as u32,
                    column_count: cols as u32,
                });
            } else {
                sheets.push(SheetInfoResponse {
                    name: name.clone(),
                    row_count: 0,
                    column_count: 0,
                });
            }
        }

        Ok::<_, String>(sheets)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

/// 预览 Excel 文件内容
#[tauri::command]
pub async fn preview_excel(
    file_path: String,
    sheet_name: Option<String>,
    max_rows: Option<u32>,
) -> Result<ExcelPreviewResponse, String> {
    let path = file_path.clone();
    let max = max_rows.unwrap_or(10) as usize;

    tokio::task::spawn_blocking(move || {
        let mut workbook = open_workbook_auto(&path)
            .map_err(|e| format!("无法打开文件 {}: {}", path, e))?;

        let sheet_names = workbook.sheet_names().to_vec();
        if sheet_names.is_empty() {
            return Err("文件中没有 Sheet".to_string());
        }

        let target_sheet = sheet_name.unwrap_or_else(|| sheet_names[0].clone());

        // 收集所有 sheet 信息
        let mut sheets = Vec::new();
        for name in &sheet_names {
            if let Ok(range) = workbook.worksheet_range(name) {
                let (rows, cols) = range.get_size();
                sheets.push(SheetInfoResponse {
                    name: name.clone(),
                    row_count: rows as u32,
                    column_count: cols as u32,
                });
            }
        }

        // 读取目标 sheet 的数据
        let range = workbook
            .worksheet_range(&target_sheet)
            .map_err(|e| format!("无法读取 Sheet '{}': {}", target_sheet, e))?;

        let rows: Vec<Vec<String>> = range
            .rows()
            .take(max)
            .map(|row| row.iter().map(data_to_string).collect())
            .collect();

        Ok::<_, String>(ExcelPreviewResponse {
            sheets,
            rows,
            sheet_name: target_sheet,
        })
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

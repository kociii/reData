// AI 调用公共工具函数
//
// 从 ai_service.rs 提取，供 ai_service.rs 和 processing.rs 共用

use futures_util::StreamExt;

/// 调用 AI API（OpenAI 兼容接口，支持阿里云结构化输出）
///
/// # 参数
/// - `json_mode`: 是否启用 JSON 结构化输出模式（response_format: {"type": "json_object"}）
///   - 启用后确保 AI 返回标准 JSON 格式
///   - 注意：prompt 中必须包含 "JSON" 关键词（阿里云要求）
pub async fn call_ai(
    api_url: &str,
    api_key: &str,
    model_name: &str,
    system_prompt: &str,
    user_prompt: &str,
    temperature: f32,
    max_tokens: i32,
    json_mode: bool,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", api_url.trim_end_matches('/'));

    // 构建请求体
    let mut body = serde_json::json!({
        "model": model_name,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt}
        ],
        "temperature": temperature,
        "max_tokens": max_tokens,
    });

    // 添加 JSON 结构化输出支持（阿里云/OpenAI 兼容）
    if json_mode {
        body["response_format"] = serde_json::json!({"type": "json_object"});
    }

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| format!("AI API 请求失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("AI API 返回错误 {}: {}", status, body));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析 AI 响应失败: {}", e))?;

    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("AI 响应格式错误")?
        .to_string();

    Ok(content)
}

/// 流式调用 AI API，支持回调处理每个 chunk
///
/// # 参数
/// - `on_chunk`: 每个 chunk 的回调函数，接收 chunk 内容
/// - `json_mode`: 是否启用 JSON 结构化输出模式
pub async fn call_ai_stream<F>(
    api_url: &str,
    api_key: &str,
    model_name: &str,
    system_prompt: &str,
    user_prompt: &str,
    temperature: f32,
    max_tokens: i32,
    json_mode: bool,
    mut on_chunk: F,
) -> Result<String, String>
where
    F: FnMut(&str) + Send,
{
    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", api_url.trim_end_matches('/'));

    // 构建请求体
    let mut body = serde_json::json!({
        "model": model_name,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt}
        ],
        "temperature": temperature,
        "max_tokens": max_tokens,
        "stream": true,
    });

    // 添加 JSON 结构化输出支持（阿里云/OpenAI 兼容）
    if json_mode {
        body["response_format"] = serde_json::json!({"type": "json_object"});
    }

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| format!("AI API 请求失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("AI API 返回错误 {}: {}", status, body));
    }

    let mut full_content = String::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("读取流失败: {}", e))?;
        let chunk_str = String::from_utf8_lossy(&chunk);

        // 解析 SSE 格式: data: {...}
        for line in chunk_str.lines() {
            if let Some(json_str) = line.strip_prefix("data: ") {
                if json_str == "[DONE]" {
                    continue;
                }

                if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let Some(delta) = json["choices"][0]["delta"]["content"].as_str() {
                        full_content.push_str(delta);
                        on_chunk(delta);
                    }
                }
            }
        }
    }

    Ok(full_content)
}

/// 从 AI 响应中提取 JSON 字符串
pub fn extract_json(response: &str) -> Result<String, String> {
    let trimmed = response.trim();

    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Ok(trimmed.to_string());
    }

    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            if end > start {
                return Ok(trimmed[start..=end].to_string());
            }
        }
    }

    Err("无法从响应中提取 JSON".to_string())
}

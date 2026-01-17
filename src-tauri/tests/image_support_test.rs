// 图片支持功能测试

#[cfg(test)]
mod tests {
    use serde_json::json;
    
    // 模拟 extract_images_from_content 函数（从 converter.rs 复制）
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct KiroImage {
        format: String,
        source: KiroImageSource,
    }
    
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct KiroImageSource {
        bytes: String,
    }
    
    fn extract_images_from_content(content: &Option<serde_json::Value>) -> (Vec<KiroImage>, usize) {
        let content = match content {
            Some(c) => c,
            None => return (Vec::new(), 0),
        };
        
        let content_array = match content.as_array() {
            Some(arr) => arr,
            None => return (Vec::new(), 0),
        };
        
        let mut images = Vec::new();
        
        for item in content_array {
            let obj = match item.as_object() {
                Some(o) => o,
                None => continue,
            };
            
            let item_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
            
            // OpenAI 格式
            if item_type == "image_url" {
                let url = if let Some(url_str) = obj.get("image_url").and_then(|v| v.as_str()) {
                    url_str
                } else if let Some(url_obj) = obj.get("image_url").and_then(|v| v.as_object()) {
                    url_obj.get("url").and_then(|v| v.as_str()).unwrap_or("")
              
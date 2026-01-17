// Thinking Parser - Extended Thinking 支持
// 解析 Kiro API 返回的 <thinking>...</thinking> 标签
// 转换为 Anthropic 官方 Extended Thinking 格式

#[derive(Debug, Clone, PartialEq)]
pub enum SegmentType {
    Thinking,
    Text,
}

#[derive(Debug, Clone)]
pub struct TextSegment {
    pub segment_type: SegmentType,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
enum ParseState {
    Initial,        // 初始状态，等待检测是否以 <thinking> 开头
    InThinking,     // 在 thinking 块内
    AfterThinking,  // thinking 块结束后，处理普通文本
    Passthrough,    // 直通模式（响应不以 <thinking> 开头）
}

pub struct ThinkingParser {
    buffer: String,
    state: ParseState,
    thinking_extracted: bool,
}

impl ThinkingParser {
    const OPEN_TAG: &'static str = "<thinking>";
    const CLOSE_TAG: &'static str = "</thinking>";
    const QUOTE_CHARS: &'static [char] = &['`', '"', '\'', '"', '"', '\'', '\'', '「', '」', '『', '』'];

    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            state: ParseState::Initial,
            thinking_extracted: false,
        }
    }

    /// 增量解析输入文本
    pub fn push_and_parse(&mut self, incoming: &str) -> Vec<TextSegment> {
        if incoming.is_empty() {
            return Vec::new();
        }

        self.buffer.push_str(incoming);
        let mut segments = Vec::new();

        loop {
            match self.state {
                ParseState::Initial => {
                    if let Some(should_continue) = self.handle_initial_state() {
                        if !should_continue {
                            break;
                        }
                        continue;
                    }
                    break;
                }
                ParseState::InThinking => {
                    if let Some(segment) = self.handle_in_thinking_state() {
                        if !segment.content.is_empty() {
                            segments.push(segment);
                        }
                        continue;
                    }
                    break;
                }
                ParseState::AfterThinking => {
                    if !self.buffer.is_empty() {
                        segments.push(TextSegment {
                            segment_type: SegmentType::Text,
                            content: self.buffer.clone(),
                        });
                        self.buffer.clear();
                    }
                    break;
                }
                ParseState::Passthrough => {
                    if !self.buffer.is_empty() {
                        segments.push(TextSegment {
                            segment_type: SegmentType::Text,
                            content: self.buffer.clone(),
                        });
                        self.buffer.clear();
                    }
                    break;
                }
            }
        }

        segments
    }

    /// 流结束时刷新缓冲区
    pub fn flush(&mut self) -> Vec<TextSegment> {
        let mut segments = Vec::new();

        match self.state {
            ParseState::Initial => {
                if !self.buffer.is_empty() {
                    segments.push(TextSegment {
                        segment_type: SegmentType::Text,
                        content: self.buffer.clone(),
                    });
                    self.buffer.clear();
                }
            }
            ParseState::InThinking => {
                if !self.buffer.is_empty() {
                    log::warn!("[ThinkingParser] Thinking block not properly closed, flushing {} chars as thinking", self.buffer.len());
                    segments.push(TextSegment {
                        segment_type: SegmentType::Thinking,
                        content: self.buffer.clone(),
                    });
                    self.buffer.clear();
                }
            }
            ParseState::AfterThinking | ParseState::Passthrough => {
                if !self.buffer.is_empty() {
                    segments.push(TextSegment {
                        segment_type: SegmentType::Text,
                        content: self.buffer.clone(),
                    });
                    self.buffer.clear();
                }
            }
        }

        segments
    }

    #[allow(dead_code)]
    pub fn is_thinking_mode(&self) -> bool {
        matches!(self.state, ParseState::InThinking | ParseState::AfterThinking)
    }

    #[allow(dead_code)]
    pub fn has_extracted_thinking(&self) -> bool {
        self.thinking_extracted
    }

    fn handle_initial_state(&mut self) -> Option<bool> {
        let stripped = self.buffer.trim_start();
        let _whitespace_len = self.buffer.len() - stripped.len();

        if stripped.len() < Self::OPEN_TAG.len() {
            if !stripped.is_empty() && Self::OPEN_TAG.starts_with(stripped) {
                return None; // 可能是 <thinking>，等待更多数据
            } else if !stripped.is_empty() {
                self.state = ParseState::Passthrough;
                return Some(true);
            } else {
                return None; // 只有空白，等待更多数据
            }
        }

        if stripped.starts_with(Self::OPEN_TAG) {
            self.buffer = stripped[Self::OPEN_TAG.len()..].to_string();
            self.state = ParseState::InThinking;
            log::debug!("[ThinkingParser] Detected <thinking> tag at start, entering thinking mode");
            return Some(true);
        } else {
            self.state = ParseState::Passthrough;
            return Some(true);
        }
    }

    fn handle_in_thinking_state(&mut self) -> Option<TextSegment> {
        let close_pos = self.find_real_close_tag();

        if close_pos.is_none() {
            let safe_len = self.buffer.len().saturating_sub(Self::CLOSE_TAG.len() - 1);
            if safe_len > 0 {
                let thinking_content = self.buffer[..safe_len].to_string();
                self.buffer = self.buffer[safe_len..].to_string();
                return Some(TextSegment {
                    segment_type: SegmentType::Thinking,
                    content: thinking_content,
                });
            }
            return None;
        }

        let close_pos = close_pos.unwrap();
        let thinking_content = self.buffer[..close_pos].to_string();
        let after_tag = &self.buffer[close_pos + Self::CLOSE_TAG.len()..];
        let after_tag = after_tag.trim_start_matches('\n');

        self.buffer = after_tag.to_string();
        self.state = ParseState::AfterThinking;
        self.thinking_extracted = true;

        log::debug!("[ThinkingParser] Extracted thinking block: {} chars", thinking_content.len());
        Some(TextSegment {
            segment_type: SegmentType::Thinking,
            content: thinking_content,
        })
    }

    fn find_real_close_tag(&self) -> Option<usize> {
        let mut search_start = 0;

        loop {
            let pos = self.buffer[search_start..].find(Self::CLOSE_TAG)?;
            let pos = search_start + pos;

            if self.is_quoted_tag(pos) {
                search_start = pos + 1;
                continue;
            }

            let after_pos = pos + Self::CLOSE_TAG.len();
            if after_pos < self.buffer.len() {
                let next_char = self.buffer.chars().nth(after_pos);
                if matches!(next_char, Some('\n') | Some('\r')) {
                    return Some(pos);
                }
                if self.buffer.len() - after_pos > 10 {
                    search_start = pos + 1;
                    continue;
                }
                return Some(pos);
            } else {
                return Some(pos);
            }
        }
    }

    fn is_quoted_tag(&self, tag_pos: usize) -> bool {
        if tag_pos == 0 {
            return false;
        }

        if let Some(prev_char) = self.buffer.chars().nth(tag_pos - 1) {
            if Self::QUOTE_CHARS.contains(&prev_char) {
                return true;
            }
        }

        let before_text = &self.buffer[..tag_pos];
        let backtick_count = before_text.matches('`').count();
        if backtick_count % 2 == 1 {
            return true;
        }

        false
    }
}

impl Default for ThinkingParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_thinking() {
        let mut parser = ThinkingParser::new();
        
        let segments = parser.push_and_parse("<thinking>Let me think...</thinking>\nHere is the answer.");
        
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].segment_type, SegmentType::Thinking);
        assert_eq!(segments[0].content, "Let me think...");
        assert_eq!(segments[1].segment_type, SegmentType::Text);
        assert_eq!(segments[1].content, "Here is the answer.");
    }

    #[test]
    fn test_no_thinking() {
        let mut parser = ThinkingParser::new();
        
        let segments = parser.push_and_parse("Just a normal response.");
        
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].segment_type, SegmentType::Text);
        assert_eq!(segments[0].content, "Just a normal response.");
    }

    #[test]
    fn test_incremental_parsing() {
        let mut parser = ThinkingParser::new();
        
        let seg1 = parser.push_and_parse("<think");
        assert_eq!(seg1.len(), 0);
        
        let seg2 = parser.push_and_parse("ing>Part 1");
        assert_eq!(seg2.len(), 0);
        
        let seg3 = parser.push_and_parse(" Part 2</thinking>\nText");
        assert!(seg3.len() >= 1);
        assert_eq!(seg3[0].segment_type, SegmentType::Thinking);
    }
}

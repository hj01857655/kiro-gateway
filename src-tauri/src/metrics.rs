// kiro-gateway 统计模块

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// 延迟直方图桶边界（秒）
const LATENCY_BUCKETS: [f64; 11] = [
    0.1,
    0.25,
    0.5,
    1.0,
    2.5,
    5.0,
    10.0,
    30.0,
    60.0,
    120.0,
    f64::INFINITY,
];
#[allow(dead_code)]
const MAX_RECENT_REQUESTS: usize = 50;
#[allow(dead_code)]
const MAX_RESPONSE_TIMES: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentRequest {
    pub timestamp: String,
    pub endpoint: String,
    pub status_code: u16,
    pub model: String,
    pub response_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyStats {
    pub hour: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LatencyHistogram {
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricsData {
    pub total_requests: u64,
    pub streaming_requests: u64,
    pub non_streaming_requests: u64,
    pub requests_by_endpoint: HashMap<String, u64>,
    pub requests_by_status: HashMap<String, u64>,
    pub requests_by_model: HashMap<String, u64>,
    pub api_type_usage: HashMap<String, u64>,
    pub response_times: Vec<f64>,
    pub latency_histogram: LatencyHistogram,
    pub recent_requests: Vec<RecentRequest>,
    pub hourly_stats: Vec<HourlyStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MetricsInner {
    // 请求计数：{endpoint:status:model -> count}
    request_total: HashMap<String, u64>,
    // 流式/非流式计数
    stream_requests: u64,
    non_stream_requests: u64,
    // API 类型使用量
    api_type_usage: HashMap<String, u64>,
    // 响应时间（毫秒）
    response_times: Vec<f64>,
    // 最近请求
    recent_requests: Vec<RecentRequest>,
    // 小时请求统计
    hourly_requests: HashMap<u64, u64>,
    // 延迟直方图
    latency_buckets: Vec<u64>,
    #[allow(dead_code)]
    latency_sum: f64,
    latency_count: u64,
    // 启动时间戳（保留用于未来功能）
    #[allow(dead_code)]
    start_timestamp: u64,
}

pub struct Metrics {
    inner: RwLock<MetricsInner>,
}

impl Metrics {
    pub fn new() -> Self {
        let metrics = Self {
            inner: RwLock::new(MetricsInner {
                request_total: HashMap::new(),
                stream_requests: 0,
                non_stream_requests: 0,
                api_type_usage: HashMap::new(),
                response_times: Vec::new(),
                recent_requests: Vec::new(),
                hourly_requests: HashMap::new(),
                latency_buckets: vec![0; LATENCY_BUCKETS.len()],
                latency_sum: 0.0,
                latency_count: 0,
                start_timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::ZERO)
                    .as_millis() as u64,
            }),
        };

        // 尝试加载历史数据
        if let Err(e) = metrics.load_from_file("data/metrics.json") {
            tracing::debug!("未加载历史 metrics 数据: {}", e);
        } else {
            tracing::info!("已加载历史 metrics 数据");
        }

        metrics
    }

    /// 记录请求
    pub fn record_request(
        &self,
        endpoint: &str,
        status_code: u16,
        duration_ms: f64,
        model: &str,
        is_stream: bool,
        api_type: &str,
    ) {
        let mut inner = match self.inner.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::error!("Metrics RwLock 中毒，恢复数据");
                poisoned.into_inner()
            }
        };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis() as u64;

        // 请求计数
        let key = format!("{}:{}:{}", endpoint, status_code, model);
        *inner.request_total.entry(key).or_insert(0) += 1;

        // 流式/非流式计数
        if is_stream {
            inner.stream_requests += 1;
        } else {
            inner.non_stream_requests += 1;
        }

        // API 类型统计
        *inner
            .api_type_usage
            .entry(api_type.to_string())
            .or_insert(0) += 1;

        // 响应时间
        inner.response_times.push(duration_ms);
        if inner.response_times.len() > MAX_RESPONSE_TIMES {
            inner.response_times.remove(0);
        }

        // 延迟直方图
        let latency_sec = duration_ms / 1000.0;
        for (i, &bucket) in LATENCY_BUCKETS.iter().enumerate() {
            if latency_sec <= bucket {
                inner.latency_buckets[i] += 1;
                break;
            }
        }
        inner.latency_sum += latency_sec;
        inner.latency_count += 1;

        // 最近请求
        inner.recent_requests.push(RecentRequest {
            timestamp: now.to_string(),
            endpoint: endpoint.to_string(),
            status_code,
            model: model.to_string(),
            response_time_ms: duration_ms,
        });
        if inner.recent_requests.len() > MAX_RECENT_REQUESTS {
            inner.recent_requests.remove(0);
        }

        // 小时统计
        let hour_ts = (now / 3600000) * 3600000;
        *inner.hourly_requests.entry(hour_ts).or_insert(0) += 1;

        // 清理 24 小时前的数据
        let cutoff = hour_ts.saturating_sub(24 * 3600000);
        inner.hourly_requests.retain(|&k, _| k >= cutoff);
    }

    /// 获取统计数据
    pub fn get_metrics(&self) -> MetricsData {
        let inner = match self.inner.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::error!("Metrics RwLock 中毒，使用受损数据");
                poisoned.into_inner()
            }
        };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis() as u64;

        // 按端点、状态码、模型分组统计
        let mut requests_by_endpoint: HashMap<String, u64> = HashMap::new();
        let mut requests_by_status: HashMap<String, u64> = HashMap::new();
        let mut requests_by_model: HashMap<String, u64> = HashMap::new();

        for (key, &count) in &inner.request_total {
            let parts: Vec<&str> = key.splitn(3, ':').collect();
            if parts.len() == 3 {
                let endpoint = parts[0];
                let status = parts[1];
                let model = parts[2];

                *requests_by_endpoint
                    .entry(endpoint.to_string())
                    .or_insert(0) += count;
                *requests_by_status.entry(status.to_string()).or_insert(0) += count;
                if model != "unknown" {
                    *requests_by_model.entry(model.to_string()).or_insert(0) += count;
                }
            }
        }

        // 计算总请求数
        let total = requests_by_endpoint.values().sum();

        // 计算延迟百分位
        let (p50, p95, p99) =
            self.calculate_percentiles(&inner.latency_buckets, inner.latency_count);

        // 构建 24 小时数据
        let current_hour = (now / 3600000) * 3600000;
        let hourly_stats: Vec<HourlyStats> = (0..24)
            .map(|i| {
                let hour_ts = current_hour - (23 - i) * 3600000;
                HourlyStats {
                    hour: hour_ts.to_string(),
                    count: *inner.hourly_requests.get(&hour_ts).unwrap_or(&0),
                }
            })
            .collect();

        // 直接使用已有的最近请求数据
        let recent_requests = inner.recent_requests.clone();

        MetricsData {
            total_requests: total,
            streaming_requests: inner.stream_requests,
            non_streaming_requests: inner.non_stream_requests,
            requests_by_endpoint,
            requests_by_status,
            requests_by_model,
            api_type_usage: inner.api_type_usage.clone(),
            response_times: inner.response_times.clone(),
            latency_histogram: LatencyHistogram { p50, p95, p99 },
            recent_requests,
            hourly_stats,
        }
    }

    fn calculate_percentiles(&self, buckets: &[u64], total: u64) -> (f64, f64, f64) {
        if total == 0 {
            return (0.0, 0.0, 0.0);
        }

        let p50 = self.percentile_from_buckets(buckets, total, 0.50);
        let p95 = self.percentile_from_buckets(buckets, total, 0.95);
        let p99 = self.percentile_from_buckets(buckets, total, 0.99);
        (p50, p95, p99)
    }

    fn percentile_from_buckets(&self, buckets: &[u64], total: u64, percentile: f64) -> f64 {
        let target = (total as f64 * percentile) as u64;
        let mut cumulative = 0u64;

        for (i, &count) in buckets.iter().enumerate() {
            cumulative += count;
            if cumulative >= target {
                let bucket_val = LATENCY_BUCKETS[i];
                return if bucket_val.is_infinite() {
                    120.0
                } else {
                    bucket_val
                };
            }
        }
        LATENCY_BUCKETS[LATENCY_BUCKETS.len() - 2]
    }

    /// 保存到文件
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        // 确保父目录存在
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
        }

        let inner = match self.inner.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::warn!("Metrics RwLock 中毒，尝试保存受损数据");
                poisoned.into_inner()
            }
        };
        let json =
            serde_json::to_string_pretty(&*inner).map_err(|e| format!("序列化失败: {}", e))?;
        std::fs::write(path, json).map_err(|e| format!("写入文件失败: {}", e))?;
        Ok(())
    }

    /// 从文件加载
    pub fn load_from_file(&self, path: &str) -> Result<(), String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("读取文件失败: {}", e))?;
        let loaded: MetricsInner =
            serde_json::from_str(&content).map_err(|e| format!("反序列化失败: {}", e))?;

        let mut inner = match self.inner.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::error!("Metrics RwLock 中毒，强制恢复");
                poisoned.into_inner()
            }
        };
        *inner = loaded;
        Ok(())
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

// 全局 Metrics 实例
use once_cell::sync::Lazy;
pub static METRICS: Lazy<Metrics> = Lazy::new(Metrics::new);

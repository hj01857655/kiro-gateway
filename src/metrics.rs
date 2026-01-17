// KiroGate 统计模块

use serde::Serialize;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// 延迟直方图桶边界（秒）
const LATENCY_BUCKETS: [f64; 11] = [0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0, 120.0, f64::INFINITY];
const MAX_RECENT_REQUESTS: usize = 50;
const MAX_RESPONSE_TIMES: usize = 100;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentRequest {
  pub timestamp: u64,
  pub api_type: String,
  pub path: String,
  pub status: u16,
  pub duration: f64,
  pub model: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HourlyData {
  pub hour: u64,
  pub count: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricsData {
  pub total_requests: u64,
  pub success_requests: u64,
  pub failed_requests: u64,
  pub avg_response_time: f64,
  pub response_times: Vec<f64>,
  pub stream_requests: u64,
  pub non_stream_requests: u64,
  pub model_usage: HashMap<String, u64>,
  pub api_type_usage: HashMap<String, u64>,
  pub recent_requests: Vec<RecentRequest>,
  pub start_time: u64,
  pub hourly_requests: Vec<HourlyData>,
  pub latency_p50: f64,
  pub latency_p95: f64,
  pub latency_p99: f64,
}

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
  latency_sum: f64,
  latency_count: u64,
  // 启动时间戳
  start_timestamp: u64,
}

pub struct Metrics {
  inner: RwLock<MetricsInner>,
}

impl Metrics {
  pub fn new() -> Self {
    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap_or(Duration::ZERO)
      .as_millis() as u64;

    Self {
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
        start_timestamp: now,
      }),
    }
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
    let mut inner = self.inner.write().unwrap();
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
    *inner.api_type_usage.entry(api_type.to_string()).or_insert(0) += 1;

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
      timestamp: now,
      api_type: api_type.to_string(),
      path: endpoint.to_string(),
      status: status_code,
      duration: duration_ms,
      model: model.to_string(),
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
    let inner = self.inner.read().unwrap();
    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap_or(Duration::ZERO)
      .as_millis() as u64;

    // 计算总请求数和成功/失败数
    let mut total = 0u64;
    let mut success = 0u64;
    let mut model_usage: HashMap<String, u64> = HashMap::new();

    for (key, &count) in &inner.request_total {
      total += count;
      let parts: Vec<&str> = key.rsplitn(3, ':').collect();
      if parts.len() >= 2 {
        if let Ok(status) = parts[1].parse::<u16>() {
          if (200..400).contains(&status) {
            success += count;
          }
        }
        let model = parts[0];
        if model != "unknown" {
          *model_usage.entry(model.to_string()).or_insert(0) += count;
        }
      }
    }

    // 平均响应时间
    let avg_response_time = if inner.response_times.is_empty() {
      0.0
    } else {
      inner.response_times.iter().sum::<f64>() / inner.response_times.len() as f64
    };

    // 计算延迟百分位
    let (p50, p95, p99) = self.calculate_percentiles(&inner.latency_buckets, inner.latency_count);

    // 构建 24 小时数据
    let current_hour = (now / 3600000) * 3600000;
    let hourly_requests: Vec<HourlyData> = (0..24)
      .map(|i| {
        let hour_ts = current_hour - (23 - i) * 3600000;
        HourlyData {
          hour: hour_ts,
          count: *inner.hourly_requests.get(&hour_ts).unwrap_or(&0),
        }
      })
      .collect();

    MetricsData {
      total_requests: total,
      success_requests: success,
      failed_requests: total - success,
      avg_response_time,
      response_times: inner.response_times.clone(),
      stream_requests: inner.stream_requests,
      non_stream_requests: inner.non_stream_requests,
      model_usage,
      api_type_usage: inner.api_type_usage.clone(),
      recent_requests: inner.recent_requests.clone(),
      start_time: inner.start_timestamp,
      hourly_requests,
      latency_p50: p50,
      latency_p95: p95,
      latency_p99: p99,
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
        return if bucket_val.is_infinite() { 120.0 } else { bucket_val };
      }
    }
    LATENCY_BUCKETS[LATENCY_BUCKETS.len() - 2]
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

use chrono::Utc;
use parking_lot::RwLock;
use std::collections::HashMap;
use tracing::{info, warn};

use crate::account::Account;
use crate::error::AppError;

/// Token 使用统计
#[derive(Debug, Clone)]
pub struct TokenStats {
    pub success_count: u32,
    pub fail_count: u32,
    pub last_used: Option<i64>,
}

impl TokenStats {
    fn new() -> Self {
        Self {
            success_count: 0,
            fail_count: 0,
            last_used: None,
        }
    }

    /// 计算成功率
    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.fail_count;
        if total == 0 {
            1.0 // 新 Token 给予高分
        } else {
            self.success_count as f64 / total as f64
        }
    }
}

/// 智能 Token 分配器
/// 
/// 基于成功率、新鲜度和负载均衡的智能分配算法
pub struct SmartTokenAllocator {
    stats: RwLock<HashMap<String, TokenStats>>,
    min_success_rate: f64,
}

impl SmartTokenAllocator {
    pub fn new() -> Self {
        Self {
            stats: RwLock::new(HashMap::new()),
            min_success_rate: 0.7, // 最低成功率阈值
        }
    }

    /// 计算 Token 评分 (0-100)
    /// 
    /// 评分基于：
    /// - 成功率 (权重 60%)
    /// - 新鲜度 (权重 20%)
    /// - 负载均衡 (权重 20%)
    fn calculate_score(&self, account: &Account, stats: &TokenStats) -> f64 {
        let now = Utc::now().timestamp_millis();
        let total = stats.success_count + stats.fail_count;
        let success_rate = stats.success_rate();

        // 基础分: 成功率 (权重60%)
        let base_score = if success_rate < self.min_success_rate && total > 10 {
            // 如果成功率低于阈值，大幅降分
            success_rate * 30.0
        } else {
            success_rate * 60.0
        };

        // 新鲜度: 最近使用时间 (权重20%)
        let freshness = if let Some(last_used) = stats.last_used {
            let hours_since_use = (now - last_used) as f64 / 3600000.0;
            if hours_since_use < 1.0 {
                20.0
            } else if hours_since_use < 24.0 {
                15.0
            } else {
                (20.0 - hours_since_use / 24.0).max(5.0)
            }
        } else {
            20.0 // 从未使用，视为新鲜
        };

        // 负载均衡: 使用频率 (权重20%)
        // 使用次数少的 Token 优先，避免单个 Token 过载
        let usage_score = (20.0 - (total as f64 / 100.0)).max(0.0);

        // 如果账号被限流，大幅降分
        let throttle_penalty = if account.is_throttled() { -50.0 } else { 0.0 };

        base_score + freshness + usage_score + throttle_penalty
    }

    /// 获取最优 Token
    pub fn get_best_account(&self, accounts: &[Account]) -> Result<Account, AppError> {
        if accounts.is_empty() {
            return Err(AppError::NoToken);
        }

        // 过滤可用账号
        let available: Vec<_> = accounts.iter()
            .filter(|a| a.is_available())
            .collect();

        if available.is_empty() {
            return Err(AppError::NoToken);
        }

        let stats = self.stats.read();

        // 计算每个账号的评分
        let mut scored: Vec<_> = available.iter()
            .map(|account| {
                let account_stats = stats.get(&account.id)
                    .cloned()
                    .unwrap_or_else(TokenStats::new);
                let score = self.calculate_score(account, &account_stats);
                (account, score, account_stats)
            })
            .collect();

        // 按评分排序（降序）
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // 过滤掉低成功率的 Token
        let good_tokens: Vec<_> = scored.iter()
            .filter(|(_, _, stats)| {
                let total = stats.success_count + stats.fail_count;
                stats.success_rate() >= self.min_success_rate || total < 10
            })
            .collect();

        let best = if !good_tokens.is_empty() {
            good_tokens[0].0
        } else {
            // 如果没有好的 Token，使用评分最高的
            scored[0].0
        };

        info!(
            "选择账号 {} (评分: {:.2}, 成功率: {:.2}%, 使用次数: {})",
            best.id,
            scored[0].1,
            scored[0].2.success_rate() * 100.0,
            scored[0].2.success_count + scored[0].2.fail_count
        );

        Ok((*best).clone())
    }

    /// 记录 Token 使用结果
    pub fn record_usage(&self, account_id: &str, success: bool) {
        let mut stats = self.stats.write();
        let entry = stats.entry(account_id.to_string())
            .or_insert_with(TokenStats::new);

        if success {
            entry.success_count += 1;
        } else {
            entry.fail_count += 1;
        }
        entry.last_used = Some(Utc::now().timestamp_millis());

        let success_rate = entry.success_rate();
        let total = entry.success_count + entry.fail_count;

        if success {
            info!(
                "账号 {} 使用成功 (成功率: {:.2}%, 总次数: {})",
                account_id,
                success_rate * 100.0,
                total
            );
        } else {
            warn!(
                "账号 {} 使用失败 (成功率: {:.2}%, 总次数: {})",
                account_id,
                success_rate * 100.0,
                total
            );
        }

        // 如果成功率过低，发出警告
        if total > 10 && success_rate < self.min_success_rate {
            warn!(
                "账号 {} 成功率过低 ({:.2}%)，建议检查",
                account_id,
                success_rate * 100.0
            );
        }
    }

    /// 获取所有 Token 统计
    pub fn get_all_stats(&self) -> HashMap<String, TokenStats> {
        self.stats.read().clone()
    }

    /// 清除指定 Token 的统计
    pub fn clear_stats(&self, account_id: &str) {
        let mut stats = self.stats.write();
        stats.remove(account_id);
        info!("已清除账号 {} 的统计数据", account_id);
    }

    /// 重置所有统计
    pub fn reset_all_stats(&self) {
        let mut stats = self.stats.write();
        stats.clear();
        info!("已重置所有账号统计数据");
    }
}

impl Default for SmartTokenAllocator {
    fn default() -> Self {
        Self::new()
    }
}

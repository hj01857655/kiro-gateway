// kiro-gateway 账号健康检查器
// 后台任务，定期检查所有账号的有效性

use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error, debug};
use crate::account::{AccountManager, AccountStatus};

pub struct HealthChecker {
    accounts: Arc<AccountManager>,
    check_interval: Duration,
}

impl HealthChecker {
    pub fn new(accounts: Arc<AccountManager>, check_interval_secs: u64) -> Self {
        Self {
            accounts,
            check_interval: Duration::from_secs(check_interval_secs),
        }
    }

    /// 启动健康检查后台任务
    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            self.run_loop().await;
        });
    }

    /// 主循环
    async fn run_loop(&self) {
        let mut ticker = interval(self.check_interval);
        info!("账号健康检查器已启动，检查间隔: {:?}", self.check_interval);

        loop {
            ticker.tick().await;
            
            match self.check_all_accounts().await {
                Ok(summary) => {
                    info!(
                        "健康检查完成: 检查 {} 个账号，有效 {} 个，无效 {} 个",
                        summary.checked, summary.valid, summary.invalid
                    );
                }
                Err(e) => {
                    error!("健康检查失败: {}", e);
                }
            }
        }
    }

    /// 检查所有账号
    pub async fn check_all_accounts(&self) -> Result<HealthCheckSummary, String> {
        let accounts = self.accounts.list_accounts();
        
        if accounts.is_empty() {
            debug!("没有账号需要检查");
            return Ok(HealthCheckSummary {
                checked: 0,
                valid: 0,
                invalid: 0,
            });
        }

        let total_count = accounts.len();
        info!("开始检查 {} 个账号", total_count);

        let mut valid_count = 0;
        let mut invalid_count = 0;

        for account in accounts {
            // 只检查启用的账号
            if !account.enabled {
                continue;
            }

            match self.check_account(&account.id).await {
                Ok(is_valid) => {
                    if is_valid {
                        valid_count += 1;
                        // 如果账号之前是 expired 状态，恢复为 active
                        if account.status == AccountStatus::Expired {
                            self.accounts.mark_status(&account.id, AccountStatus::Active);
                            info!("账号 {} 已恢复为 active 状态", account.id);
                        }
                    } else {
                        invalid_count += 1;
                        // 标记为 expired
                        self.accounts.mark_status(&account.id, AccountStatus::Expired);
                        warn!("账号 {} 标记为 expired", account.id);
                    }
                }
                Err(e) => {
                    error!("检查账号 {} 失败: {}", account.id, e);
                    invalid_count += 1;
                    self.accounts.mark_status(&account.id, AccountStatus::Error);
                }
            }

            // 延迟避免限流
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        Ok(HealthCheckSummary {
            checked: total_count,
            valid: valid_count,
            invalid: invalid_count,
        })
    }

    /// 检查单个账号
    async fn check_account(&self, account_id: &str) -> Result<bool, String> {
        debug!("检查账号: {}", account_id);

        // 尝试刷新 Token
        match self.accounts.refresh_account(account_id).await {
            Ok(_) => {
                debug!("账号 {} 健康检查通过", account_id);
                Ok(true)
            }
            Err(e) => {
                warn!("账号 {} 健康检查失败: {}", account_id, e);
                Ok(false)
            }
        }
    }
}

#[derive(Debug)]
pub struct HealthCheckSummary {
    pub checked: usize,
    pub valid: usize,
    pub invalid: usize,
}

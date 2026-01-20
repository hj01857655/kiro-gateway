// kiro-gateway 账号健康检查器
// 后台任务，定期检查所有账号的有效性

use crate::account::{AccountManager, AccountStatus};
use crate::kiro_client::KiroClient;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

pub struct HealthChecker {
    accounts: Arc<AccountManager>,
    client: Arc<KiroClient>,
    check_interval: Duration,
}

impl HealthChecker {
    pub fn new(accounts: Arc<AccountManager>, client: Arc<KiroClient>, check_interval_secs: u64) -> Self {
        Self {
            accounts,
            client,
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
                        // 但不要覆盖 Exhausted 状态（配额用尽需要手动处理）
                        if account.status == AccountStatus::Expired {
                            self.accounts
                                .mark_status(&account.id, AccountStatus::Active);
                            info!("账号 {} 已恢复为 active 状态", account.id);
                        }
                    } else {
                        invalid_count += 1;
                        // 标记为 expired（但不要覆盖 Exhausted 状态）
                        if account.status != AccountStatus::Exhausted {
                            self.accounts
                                .mark_status(&account.id, AccountStatus::Expired);
                            warn!("账号 {} 标记为 expired", account.id);
                        }
                    }
                }
                Err(e) => {
                    error!("检查账号 {} 失败: {}", account.id, e);
                    invalid_count += 1;
                    // 标记为 error（但不要覆盖 Exhausted 状态）
                    if account.status != AccountStatus::Exhausted {
                        self.accounts.mark_status(&account.id, AccountStatus::Error);
                    }
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

        // 获取账号信息
        let accounts = self.accounts.list_accounts();
        let account = accounts.iter().find(|a| a.id == account_id);

        if let Some(acc) = account {
            // 先检查 Token 是否过期
            if acc.is_expired() {
                debug!("账号 {} Token 已过期，尝试刷新", account_id);
                match self.accounts.refresh_account(account_id).await {
                    Ok(refreshed_acc) => {
                        debug!("账号 {} Token 刷新成功，进行健康检查", account_id);
                        // 刷新成功后，使用 dryRun 验证账号是否真的可用
                        let is_healthy = self.client.health_check(&refreshed_acc).await;
                        if is_healthy {
                            debug!("账号 {} 健康检查通过", account_id);
                            Ok(true)
                        } else {
                            warn!("账号 {} Token 刷新成功但健康检查失败", account_id);
                            Ok(false)
                        }
                    }
                    Err(e) => {
                        warn!("账号 {} Token 刷新失败: {}", account_id, e);
                        Ok(false)
                    }
                }
            } else {
                // Token 未过期，使用 dryRun 验证账号健康
                debug!("账号 {} Token 有效，进行健康检查", account_id);
                let is_healthy = self.client.health_check(acc).await;
                if is_healthy {
                    debug!("账号 {} 健康检查通过", account_id);
                    Ok(true)
                } else {
                    warn!("账号 {} 健康检查失败", account_id);
                    Ok(false)
                }
            }
        } else {
            Err(format!("账号 {} 不存在", account_id))
        }
    }
}

#[derive(Debug)]
pub struct HealthCheckSummary {
    pub checked: usize,
    pub valid: usize,
    pub invalid: usize,
}

# Kiro Account Manager 账号格式转换脚本
# 将 Kiro Account Manager 导出的账号转换为 kiro-gateway 格式

param(
    [Parameter(Mandatory=$true)]
    [string]$InputFile,
    
    [Parameter(Mandatory=$false)]
    [string]$OutputFile = "accounts-converted.json"
)

Write-Host "正在读取文件: $InputFile" -ForegroundColor Cyan

# 读取 JSON 文件
$content = Get-Content $InputFile -Raw -Encoding UTF8
$accounts = $content | ConvertFrom-Json

Write-Host "找到 $($accounts.Count) 个账号" -ForegroundColor Green

# 转换每个账号
$convertedAccounts = @()

foreach ($account in $accounts) {
    Write-Host "转换账号: $($account.email)" -ForegroundColor Yellow
    
    # 创建新的账号对象
    $newAccount = @{
        id = $account.id
        name = if ($account.label) { $account.label } else { $account.email }
        email = $account.email
        provider = $account.provider
        refreshToken = $account.refreshToken
        accessToken = $account.accessToken
        region = if ($account.region) { $account.region } else { "us-east-1" }
        enabled = if ($account.status -eq "active") { $true } else { $false }
    }
    
    # 转换 expiresAt 格式
    if ($account.expiresAt) {
        try {
            # 解析 "2026/01/20 16:00:40" 格式
            $dt = [DateTime]::ParseExact($account.expiresAt, "yyyy/MM/dd HH:mm:ss", $null)
            # 转换为毫秒时间戳
            $epoch = [DateTime]::new(1970, 1, 1, 0, 0, 0, [DateTimeKind]::Utc)
            $newAccount.expiresAt = [long](($dt.ToUniversalTime() - $epoch).TotalMilliseconds)
            Write-Host "  ✓ 转换 expiresAt: $($account.expiresAt) -> $($newAccount.expiresAt)" -ForegroundColor Gray
        } catch {
            Write-Host "  ✗ 无法解析 expiresAt: $($account.expiresAt)" -ForegroundColor Red
        }
    }
    
    # 判断账号类型并设置 authMethod
    if ($account.clientId) {
        # IDC 账号
        $newAccount.authMethod = "IdC"
        $newAccount.clientId = $account.clientId
        $newAccount.clientSecret = $account.clientSecret
        $newAccount.profileArn = if ($account.profileArn) { $account.profileArn } else { "" }
        Write-Host "  ✓ 账号类型: IDC (Builder ID)" -ForegroundColor Gray
    } else {
        # Social 账号
        $newAccount.authMethod = "social"
        $newAccount.profileArn = ""
        Write-Host "  ✓ 账号类型: Social" -ForegroundColor Gray
    }
    
    $convertedAccounts += $newAccount
}

# 保存转换后的文件
$json = $convertedAccounts | ConvertTo-Json -Depth 10
$json | Out-File -FilePath $OutputFile -Encoding UTF8

Write-Host "`n转换完成！" -ForegroundColor Green
Write-Host "输出文件: $OutputFile" -ForegroundColor Cyan
Write-Host "共转换 $($convertedAccounts.Count) 个账号" -ForegroundColor Green
Write-Host "`n你可以直接导入这个文件到 kiro-gateway" -ForegroundColor Yellow

# 直接测试 Kiro API 是否支持 history 字段
# 需要先从 accounts.json 读取 accessToken

Write-Host "=== 直接测试 Kiro API history 字段 ===" -ForegroundColor Cyan
Write-Host ""

# 读取账号配置
$appDataDir = "$env:APPDATA\com.kiro.gateway"
$accountsFile = "$appDataDir\accounts.json"

if (-not (Test-Path $accountsFile)) {
    Write-Host "❌ 找不到账号配置文件: $accountsFile" -ForegroundColor Red
    Write-Host "请先在 kiro-gateway 中添加账号" -ForegroundColor Yellow
    exit 1
}

$accountsData = Get-Content $accountsFile -Raw | ConvertFrom-Json
if ($accountsData.accounts.Count -eq 0) {
    Write-Host "❌ 没有可用的账号" -ForegroundColor Red
    exit 1
}

$account = $accountsData.accounts[0]
$accessToken = $account.access_token
$profileArn = $account.profile_arn

Write-Host "使用账号: $($account.email)" -ForegroundColor Gray
Write-Host ""

# Kiro API 端点
$kiroApiUrl = "https://codewhisperer.us-east-1.amazonaws.com/generateAssistantResponse"

# 测试 1: 不带 history 字段
Write-Host "测试 1: 不带 history 字段" -ForegroundColor Yellow
$payload1 = @{
    conversationState = @{
        agentContinuationId = [guid]::NewGuid().ToString()
        agentTaskType = "vibe"
        chatTriggerType = "MANUAL"
        conversationId = [guid]::NewGuid().ToString()
        currentMessage = @{
            userInputMessage = @{
                content = "你好，请回复'测试成功'"
                modelId = "qdev::claude-sonnet-4.5"
                userIntent = "CODE_GENERATION"
                origin = "AI_EDITOR"
            }
        }
    }
    profileArn = $profileArn
} | ConvertTo-Json -Depth 10 -Compress

try {
    $response1 = Invoke-RestMethod -Uri $kiroApiUrl `
        -Method Post `
        -Headers @{
            "Authorization" = "Bearer $accessToken"
            "Content-Type" = "application/json"
            "x-amzn-kiro-agent-mode" = "vibe"
        } `
        -Body $payload1
    
    Write-Host "✅ 不带 history 的请求成功" -ForegroundColor Green
} catch {
    Write-Host "❌ 不带 history 的请求失败" -ForegroundColor Red
    Write-Host "状态码: $($_.Exception.Response.StatusCode.value__)" -ForegroundColor Red
    Write-Host "错误: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host ""
Write-Host "---" -ForegroundColor Gray
Write-Host ""

# 测试 2: 带 history 字段（空数组）
Write-Host "测试 2: 带 history 字段（空数组）" -ForegroundColor Yellow
$payload2 = @{
    conversationState = @{
        agentContinuationId = [guid]::NewGuid().ToString()
        agentTaskType = "vibe"
        chatTriggerType = "MANUAL"
        conversationId = [guid]::NewGuid().ToString()
        currentMessage = @{
            userInputMessage = @{
                content = "你好，请回复'测试成功'"
                modelId = "qdev::claude-sonnet-4.5"
                userIntent = "CODE_GENERATION"
                origin = "AI_EDITOR"
            }
        }
        history = @()  # 空的 history 数组
    }
    profileArn = $profileArn
} | ConvertTo-Json -Depth 10 -Compress

try {
    $response2 = Invoke-RestMethod -Uri $kiroApiUrl `
        -Method Post `
        -Headers @{
            "Authorization" = "Bearer $accessToken"
            "Content-Type" = "application/json"
            "x-amzn-kiro-agent-mode" = "vibe"
        } `
        -Body $payload2
    
    Write-Host "✅ 带空 history 的请求成功" -ForegroundColor Green
} catch {
    Write-Host "❌ 带空 history 的请求失败" -ForegroundColor Red
    Write-Host "状态码: $($_.Exception.Response.StatusCode.value__)" -ForegroundColor Red
    Write-Host "错误: $($_.Exception.Message)" -ForegroundColor Red
    
    if ($_.Exception.Response.StatusCode.value__ -eq 400) {
        Write-Host "⚠️ 返回 400 错误，可能是 Kiro API 不支持 history 字段！" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "---" -ForegroundColor Gray
Write-Host ""

# 测试 3: 带 history 字段（有内容）
Write-Host "测试 3: 带 history 字段（有内容）" -ForegroundColor Yellow
$payload3 = @{
    conversationState = @{
        agentContinuationId = [guid]::NewGuid().ToString()
        agentTaskType = "vibe"
        chatTriggerType = "MANUAL"
        conversationId = [guid]::NewGuid().ToString()
        currentMessage = @{
            userInputMessage = @{
                content = "我刚才说我叫什么名字？"
                modelId = "qdev::claude-sonnet-4.5"
                userIntent = "CODE_GENERATION"
                origin = "AI_EDITOR"
            }
        }
        history = @(
            @{
                userInputMessage = @{
                    content = "我的名字是张三"
                    modelId = "qdev::claude-sonnet-4.5"
                    userIntent = "CODE_GENERATION"
                    origin = "AI_EDITOR"
                }
            },
            @{
                assistantResponseMessage = @{
                    content = "你好张三，很高兴认识你！"
                }
            }
        )
    }
    profileArn = $profileArn
} | ConvertTo-Json -Depth 10 -Compress

try {
    $response3 = Invoke-RestMethod -Uri $kiroApiUrl `
        -Method Post `
        -Headers @{
            "Authorization" = "Bearer $accessToken"
            "Content-Type" = "application/json"
            "x-amzn-kiro-agent-mode" = "vibe"
        } `
        -Body $payload3
    
    Write-Host "✅ 带 history 内容的请求成功" -ForegroundColor Green
} catch {
    Write-Host "❌ 带 history 内容的请求失败" -ForegroundColor Red
    Write-Host "状态码: $($_.Exception.Response.StatusCode.value__)" -ForegroundColor Red
    Write-Host "错误: $($_.Exception.Message)" -ForegroundColor Red
    
    if ($_.Exception.Response.StatusCode.value__ -eq 400) {
        Write-Host "⚠️ 返回 400 错误，确认 Kiro API 不支持 history 字段！" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "=== 测试完成 ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "结论:" -ForegroundColor Cyan
Write-Host "- 如果测试 1 成功，测试 2 或 3 失败 → Kiro API 不支持 history 字段"
Write-Host "- 如果所有测试都成功 → Kiro API 支持 history 字段"
Write-Host "- 如果测试 2 成功，测试 3 失败 → Kiro API 支持空 history，但不支持有内容的 history"

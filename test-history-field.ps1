# 测试 Kiro API 是否支持 history 字段
# 使用 kiro-gateway 的 API 进行测试

$baseUrl = "http://127.0.0.1:8080"
$apiKey = "your-api-key-here"  # 替换为实际的 API Key

Write-Host "=== 测试 Kiro API history 字段支持 ===" -ForegroundColor Cyan
Write-Host ""

# 测试 1: 简单请求（无 history）
Write-Host "测试 1: 简单请求（无 history）" -ForegroundColor Yellow
$simpleRequest = @{
    model = "claude-sonnet-4.5"
    messages = @(
        @{
            role = "user"
            content = "你好，请回复'测试成功'"
        }
    )
    stream = $false
} | ConvertTo-Json -Depth 10

try {
    $response1 = Invoke-RestMethod -Uri "$baseUrl/v1/chat/completions" `
        -Method Post `
        -Headers @{
            "Authorization" = "Bearer $apiKey"
            "Content-Type" = "application/json"
        } `
        -Body $simpleRequest
    
    Write-Host "✅ 简单请求成功" -ForegroundColor Green
    Write-Host "响应: $($response1.choices[0].message.content)" -ForegroundColor Gray
} catch {
    Write-Host "❌ 简单请求失败: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host ""
Write-Host "---" -ForegroundColor Gray
Write-Host ""

# 测试 2: 带历史消息的请求（会生成 history 字段）
Write-Host "测试 2: 带历史消息的请求（会生成 history 字段）" -ForegroundColor Yellow
$historyRequest = @{
    model = "claude-sonnet-4.5"
    messages = @(
        @{
            role = "user"
            content = "我的名字是张三"
        },
        @{
            role = "assistant"
            content = "你好张三，很高兴认识你！"
        },
        @{
            role = "user"
            content = "我刚才说我叫什么名字？"
        }
    )
    stream = $false
} | ConvertTo-Json -Depth 10

try {
    $response2 = Invoke-RestMethod -Uri "$baseUrl/v1/chat/completions" `
        -Method Post `
        -Headers @{
            "Authorization" = "Bearer $apiKey"
            "Content-Type" = "application/json"
        } `
        -Body $historyRequest
    
    Write-Host "✅ 带历史消息的请求成功" -ForegroundColor Green
    Write-Host "响应: $($response2.choices[0].message.content)" -ForegroundColor Gray
    
    # 检查是否正确记住了名字
    if ($response2.choices[0].message.content -match "张三") {
        Write-Host "✅ AI 正确记住了历史信息" -ForegroundColor Green
    } else {
        Write-Host "⚠️ AI 可能没有正确使用历史信息" -ForegroundColor Yellow
    }
} catch {
    Write-Host "❌ 带历史消息的请求失败: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "错误详情: $($_.ErrorDetails.Message)" -ForegroundColor Red
    
    # 如果是 400 错误，可能是 history 字段不支持
    if ($_.Exception.Response.StatusCode -eq 400) {
        Write-Host "⚠️ 这可能是因为 Kiro API 不支持 history 字段！" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "---" -ForegroundColor Gray
Write-Host ""

# 测试 3: 工具调用后的 tool_result（会生成 toolResults 字段）
Write-Host "测试 3: 工具调用结果（会生成 toolResults 字段）" -ForegroundColor Yellow
$toolRequest = @{
    model = "claude-sonnet-4.5"
    messages = @(
        @{
            role = "user"
            content = "请使用 get_weather 工具查询北京的天气"
        },
        @{
            role = "assistant"
            content = $null
            tool_calls = @(
                @{
                    id = "call_123"
                    type = "function"
                    function = @{
                        name = "get_weather"
                        arguments = '{"location":"北京"}'
                    }
                }
            )
        },
        @{
            role = "tool"
            tool_call_id = "call_123"
            content = "北京今天晴天，温度 20°C"
        },
        @{
            role = "user"
            content = "天气怎么样？"
        }
    )
    tools = @(
        @{
            type = "function"
            function = @{
                name = "get_weather"
                description = "获取天气信息"
                parameters = @{
                    type = "object"
                    properties = @{
                        location = @{
                            type = "string"
                            description = "城市名称"
                        }
                    }
                    required = @("location")
                }
            }
        }
    )
    stream = $false
} | ConvertTo-Json -Depth 10

try {
    $response3 = Invoke-RestMethod -Uri "$baseUrl/v1/chat/completions" `
        -Method Post `
        -Headers @{
            "Authorization" = "Bearer $apiKey"
            "Content-Type" = "application/json"
        } `
        -Body $toolRequest
    
    Write-Host "✅ 工具调用结果请求成功" -ForegroundColor Green
    Write-Host "响应: $($response3.choices[0].message.content)" -ForegroundColor Gray
} catch {
    Write-Host "❌ 工具调用结果请求失败: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "错误详情: $($_.ErrorDetails.Message)" -ForegroundColor Red
    
    if ($_.Exception.Response.StatusCode -eq 400) {
        Write-Host "⚠️ 这可能是因为 Kiro API 不支持 toolResults 字段！" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "=== 测试完成 ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "总结:" -ForegroundColor Cyan
Write-Host "- 如果测试 2 或测试 3 失败（400 错误），说明 Kiro API 不支持 history/toolResults 字段"
Write-Host "- 需要应用 chaogei v1.4.0 的修复：将 history 和 toolResults 转换为文本嵌入到 content 中"

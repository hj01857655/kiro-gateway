# 测试所有模型
$models = @(
    "auto",
    "claude-haiku-4.5",
    "claude-sonnet-4",
    "claude-sonnet-4.5",
    "claude-opus-4.5"
)

$url = "http://127.0.0.1:8080/v1/chat/completions"

foreach ($model in $models) {
    Write-Host "`n========== 测试模型: $model ==========" -ForegroundColor Cyan
    
    $body = @{
        model = $model
        messages = @(
            @{
                role = "user"
                content = "hi"
            }
        )
        stream = $false
    } | ConvertTo-Json -Depth 10
    
    try {
        $response = Invoke-RestMethod -Uri $url -Method Post -Body $body -ContentType "application/json" -ErrorAction Stop
        Write-Host "✅ 成功! 响应: $($response.choices[0].message.content.Substring(0, [Math]::Min(50, $response.choices[0].message.content.Length)))..." -ForegroundColor Green
    } catch {
        $errorBody = $_.ErrorDetails.Message
        if ($errorBody -match "INVALID_MODEL_ID") {
            Write-Host "❌ 模型不可用 (INVALID_MODEL_ID)" -ForegroundColor Red
        } else {
            Write-Host "❌ 错误: $errorBody" -ForegroundColor Red
        }
    }
}

Write-Host "`n========== 测试完成 ==========" -ForegroundColor Cyan

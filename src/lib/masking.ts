/**
 * Masks a token or sensitive string by showing only the first and last few characters.
 * Example: "pk-abc123456789" -> "pk-ab...789"
 */
export function maskToken(token: string | null | undefined, visibleStart = 5, visibleEnd = 4): string {
    if (!token) return '暂无数据'
    if (token.length <= visibleStart + visibleEnd) return token
    return `${token.slice(0, visibleStart)}...${token.slice(-visibleEnd)}`
}

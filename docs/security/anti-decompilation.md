# 防反编译措施

## 概述

kiro-gateway 是 Tauri 2.0 桌面应用，包含 Rust 后端和 React 前端。本文档说明如何保护源码免受反编译和逆向工程。

**技术栈**：
- 后端：Rust + Axum（编译为原生二进制）
- 前端：React 19 + TypeScript + Vite（打包为 JavaScript）
- 桌面框架：Tauri 2.0

---

## 威胁模型

### 攻击面

1. **Rust 二进制文件**
   - 可以被反汇编（IDA Pro、Ghidra）
   - 包含调试符号和字符串信息
   - 可以被动态调试（gdb、lldb）

2. **前端 JavaScript 代码**
   - 打包在 `dist/` 目录
   - 可以被轻易读取和分析
   - Source maps 暴露原始代码结构

3. **敏感信息泄露**
   - API 端点 URL
   - 文件路径
   - 算法逻辑
   - 账号管理逻辑

---

## Rust 后端保护措施

### 1. 编译优化（基础防护）✅

**Cargo.toml 配置**：

```toml
[profile.release]
# 优化级别：z = 优化二进制大小（同时增加混淆）
opt-level = "z"

# 链接时优化（LTO）：增加编译时间，但提高优化和混淆
lto = true

# 代码生成单元：1 = 最大优化（但编译慢）
codegen-units = 1

# 移除符号信息：减小体积，增加逆向难度
strip = "symbols"  # 或 "debuginfo"

# Panic 处理：abort = 更小的二进制，移除 unwinding 代码
panic = "abort"
```

**效果**：
- ✅ 移除调试符号和函数名
- ✅ 减小二进制体积 40-60%
- ✅ 增加反汇编难度
- ✅ 移除 panic unwinding 代码

**限制**：
- ❌ 仍可被反汇编
- ❌ 字符串仍然可见
- ❌ 控制流仍然清晰

### 2. UPX 压缩（中级防护）⚠️

**UPX** (Ultimate Packer for eXecutables) 是可执行文件压缩工具。

**安装**：
```bash
# Windows (Scoop)
scoop install upx

# macOS (Homebrew)
brew install upx

# Linux (apt)
sudo apt install upx-ucl
```

**使用**：
```bash
# 压缩 Windows 可执行文件
upx --best --lzma kiro-gateway.exe

# 压缩 Linux 可执行文件
upx --best --lzma kiro-gateway

# 压缩 macOS 可执行文件（需要特殊处理）
upx --best --lzma kiro-gateway.app/Contents/MacOS/kiro-gateway
```

**效果**：
- ✅ 减小体积 50-70%
- ✅ 增加静态分析难度
- ✅ 运行时自动解压

**限制**：
- ❌ 可以被 UPX 解压（`upx -d`）
- ❌ 部分杀毒软件误报
- ⚠️ macOS 签名可能失效

**注意事项**：
- UPX 压缩后的文件可以被轻易解压
- 主要用于减小体积，不是真正的保护
- 不推荐作为主要防护手段

### 3. OLLVM 混淆（高级防护）🔒

**OLLVM** (Obfuscator-LLVM) 是 LLVM 的混淆版本，可以在编译时混淆代码。

**混淆技术**：
- **控制流平坦化** (Control Flow Flattening)：打乱函数控制流
- **虚假控制流** (Bogus Control Flow)：插入永不执行的代码
- **指令替换** (Instruction Substitution)：用复杂指令替换简单指令
- **字符串加密** (String Encryption)：加密字符串常量

**实现步骤**：

1. **编译 OLLVM**（需要 30GB 磁盘空间和大量时间）

```bash
# 使用 Docker 镜像（推荐）
git clone https://github.com/joaovarelas/Obfuscator-LLVM-16.0
cd Obfuscator-LLVM-16.0
docker build -t rustc-ollvm .
```

2. **构建 Rust 工具链**

```bash
# 克隆 Rust 源码
git clone https://github.com/rust-lang/rust.git
cd rust

# 配置 config.toml
cp config.example.toml config.toml

# 编辑 config.toml
[rust]
debug = false
channel = "nightly"

[target.x86_64-unknown-linux-gnu]
llvm-config = "/path/to/ollvm/bin/llvm-config"

# 构建 Rust 编译器（需要数小时）
./x.py build
./x.py build cargo

# 添加自定义工具链
rustup toolchain link ollvm-rust build/x86_64-unknown-linux-gnu/stage2
```

3. **使用 OLLVM 编译**

```bash
# 启用所有混淆
RUSTFLAGS="-Cllvm-args=-enable-allobf" cargo +ollvm-rust build --release

# 或指定特定混淆
RUSTFLAGS="-Cllvm-args=-fla -sub -bcf -sobf" cargo +ollvm-rust build --release
```

**混淆标志**：
- `-fla`：控制流平坦化
- `-sub`：指令替换
- `-bcf`：虚假控制流
- `-sobf`：字符串混淆
- `-enable-allobf`：启用所有混淆

**效果**：
- ✅ 极大增加逆向难度
- ✅ 控制流变得难以理解
- ✅ 字符串被加密
- ✅ 反汇编代码难以阅读

**限制**：
- ❌ 编译时间大幅增加（10-100 倍）
- ❌ 二进制体积增加 2-5 倍
- ❌ 运行性能下降 10-30%
- ❌ 构建工具链复杂

**推荐配置**：
- 仅对关键模块启用混淆（如 `account.rs`、`auth.rs`）
- 使用中等强度混淆（避免性能损失过大）
- 在 CI/CD 中使用 Docker 镜像

---

## 前端保护措施

### 1. Vite 生产构建（基础防护）✅

**vite.config.ts 配置**：

```typescript
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  build: {
    // 最小化代码
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: true,  // 移除 console.log
        drop_debugger: true, // 移除 debugger
        pure_funcs: ['console.log', 'console.info'], // 移除特定函数
      },
      mangle: {
        toplevel: true, // 混淆顶层作用域
      },
    },
    // 禁用 source map
    sourcemap: false,
    // 代码分割
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
        },
      },
    },
  },
})
```

**效果**：
- ✅ 代码压缩和混淆
- ✅ 移除 console 和 debugger
- ✅ 变量名混淆
- ✅ 禁用 source maps

**限制**：
- ❌ 仍可被美化（Prettier）
- ❌ 逻辑结构仍然清晰
- ❌ 字符串仍然可见

### 2. JavaScript 混淆（高级防护）🔒

**使用 vite-plugin-bundle-obfuscator**：

```bash
npm install -D vite-plugin-bundle-obfuscator
```

**vite.config.ts 配置**：

```typescript
import vitePluginBundleObfuscator from 'vite-plugin-bundle-obfuscator'

export default defineConfig({
  plugins: [
    react(),
    vitePluginBundleObfuscator({
      // 自动排除 node_modules
      autoExcludeNodeModules: true,
      // 启用多线程
      threadPool: { enable: true, size: 4 },
      // 混淆配置
      options: {
        compact: true,
        controlFlowFlattening: true,
        controlFlowFlatteningThreshold: 0.75,
        deadCodeInjection: true,
        deadCodeInjectionThreshold: 0.4,
        debugProtection: false, // 生产环境可启用
        disableConsoleOutput: true,
        identifierNamesGenerator: 'hexadecimal',
        renameGlobals: false,
        selfDefending: true,
        stringArray: true,
        stringArrayEncoding: ['base64'],
        stringArrayThreshold: 0.75,
        unicodeEscapeSequence: false,
      },
    }),
  ],
})
```

**混淆选项说明**：
- `controlFlowFlattening`：控制流平坦化
- `deadCodeInjection`：注入死代码
- `stringArray`：字符串数组化
- `stringArrayEncoding`：字符串编码（base64/rc4）
- `selfDefending`：自我防护（检测格式化）
- `debugProtection`：反调试保护

**效果**：
- ✅ 代码难以阅读
- ✅ 字符串被加密
- ✅ 控制流被打乱
- ✅ 反格式化保护

**限制**：
- ❌ 增加包体积 30-50%
- ❌ 性能下降 10-20%
- ❌ 仍可被动态调试

**推荐配置**：
- 仅对关键页面启用混淆（如 Accounts、Settings）
- 使用中等强度混淆（避免性能损失）
- 排除第三方库（`autoExcludeNodeModules: true`）

---

## 综合防护策略

### 推荐方案（平衡性能和安全）

**后端（Rust）**：
1. ✅ 启用 Release 优化（`opt-level = "z"`, `lto = true`, `strip = "symbols"`）
2. ⚠️ 可选：UPX 压缩（仅用于减小体积）
3. ❌ 不推荐：OLLVM（构建复杂，性能损失大）

**前端（React）**：
1. ✅ 启用 Vite 生产构建（minify + terser）
2. ✅ 禁用 source maps
3. ⚠️ 可选：JavaScript 混淆（仅关键页面）

**配置示例**：

**Cargo.toml**：
```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = "symbols"
panic = "abort"
```

**vite.config.ts**：
```typescript
export default defineConfig({
  plugins: [react()],
  build: {
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: true,
        drop_debugger: true,
      },
    },
    sourcemap: false,
  },
})
```

### 高级方案（最大安全）

**仅在以下情况使用**：
- 商业软件需要强保护
- 包含专利算法
- 防止竞争对手抄袭

**后端（Rust）**：
1. ✅ OLLVM 混淆（关键模块）
2. ✅ Release 优化
3. ⚠️ UPX 压缩（可选）

**前端（React）**：
1. ✅ JavaScript 混淆（全部代码）
2. ✅ 字符串加密
3. ✅ 反调试保护

**注意事项**：
- ⚠️ 构建时间增加 10-100 倍
- ⚠️ 二进制体积增加 2-5 倍
- ⚠️ 运行性能下降 10-30%
- ⚠️ 需要专门的构建环境

---

## 其他安全措施

### 1. 敏感信息保护

**不要硬编码**：
- ❌ API 密钥
- ❌ 数据库密码
- ❌ 加密密钥
- ❌ 私有 URL

**正确做法**：
- ✅ 使用环境变量
- ✅ 使用配置文件（加密存储）
- ✅ 运行时从安全存储读取

### 2. 字符串混淆

**Rust 字符串混淆**（使用 `obfstr` crate）：

```rust
use obfstr::obfstr;

// 编译时加密字符串
let api_url = obfstr!("https://api.example.com");
```

**效果**：
- ✅ 字符串在二进制中被加密
- ✅ 运行时自动解密
- ✅ 静态分析看不到明文

### 3. 反调试保护

**检测调试器**（Rust）：

```rust
#[cfg(target_os = "windows")]
fn is_debugger_present() -> bool {
    use winapi::um::debugapi::IsDebuggerPresent;
    unsafe { IsDebuggerPresent() != 0 }
}

#[cfg(target_os = "linux")]
fn is_debugger_present() -> bool {
    use std::fs;
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        status.contains("TracerPid:\t0")
    } else {
        false
    }
}

// 在关键函数中检查
if is_debugger_present() {
    std::process::exit(1);
}
```

### 4. 完整性检查

**校验二进制文件**：

```rust
use sha2::{Sha256, Digest};

fn verify_integrity() -> bool {
    let exe_path = std::env::current_exe().unwrap();
    let exe_bytes = std::fs::read(exe_path).unwrap();
    let hash = Sha256::digest(&exe_bytes);
    
    // 与预期哈希比较
    let expected_hash = hex::decode("abc123...").unwrap();
    hash.as_slice() == expected_hash.as_slice()
}
```

---

## 实施建议

### 开发阶段
- ✅ 使用默认配置（快速编译）
- ✅ 启用 source maps（方便调试）
- ❌ 不启用混淆（影响开发效率）

### 测试阶段
- ✅ 使用 Release 优化
- ✅ 禁用 source maps
- ⚠️ 可选：轻度混淆（测试性能影响）

### 生产发布
- ✅ 启用所有优化
- ✅ 禁用 source maps
- ⚠️ 根据需求启用混淆

### CI/CD 集成

**GitHub Actions 示例**：

```yaml
- name: Build with optimizations
  run: |
    # Rust 优化构建
    cargo build --release
    
    # 可选：UPX 压缩
    upx --best --lzma target/release/kiro-gateway
    
    # 前端构建
    npm run build
```

---

## 性能影响对比

| 方案 | 编译时间 | 二进制大小 | 运行性能 | 逆向难度 |
|------|---------|-----------|---------|---------|
| 默认 | 1x | 100% | 100% | ⭐ |
| Release 优化 | 1.5x | 40% | 105% | ⭐⭐ |
| Release + UPX | 1.5x | 20% | 100% | ⭐⭐⭐ |
| OLLVM 轻度 | 5x | 150% | 90% | ⭐⭐⭐⭐ |
| OLLVM 重度 | 20x | 300% | 70% | ⭐⭐⭐⭐⭐ |

---

## 总结

**推荐配置（kiro-gateway）**：

1. **后端**：Release 优化 + strip symbols
2. **前端**：Vite minify + 禁用 source maps
3. **可选**：关键模块 JavaScript 混淆

**不推荐**：
- ❌ OLLVM（构建复杂，性能损失大）
- ❌ UPX（容易被解压，误报率高）

**记住**：
- 没有绝对的保护，只能增加逆向难度
- 平衡安全性和性能
- 重点保护关键算法和敏感信息
- 定期更新保护措施

---

## 参考资料

- [Rust Release Optimization](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [UPX Official Site](https://upx.github.io/)
- [OLLVM GitHub](https://github.com/obfuscator-llvm/obfuscator)
- [Obfuscating Rust Binaries using OLLVM](https://vrls.ws/posts/2023/06/obfuscating-rust-binaries-using-llvm-obfuscator-ollvm/)
- [vite-plugin-bundle-obfuscator](https://github.com/z0ffy/vite-plugin-bundle-obfuscator)
- [Minimize Rust Binary Size](https://github.com/johnthagen/min-sized-rust)

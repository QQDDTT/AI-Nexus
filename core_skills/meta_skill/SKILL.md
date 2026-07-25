---
name: meta_skill_dev
description: 专门指导大模型如何为 AI-Nexus 平台动态编写和生成 WebAssembly (Wasm) 可执行技能的元认知规范。
is_core: true
---

# AI-Nexus Wasm 技能开发指南 (Meta Skill)

作为 AI-Nexus 的底层智能体，当你接收到类似 `/create_skill` 或“帮我写一个新技能”的指令时，你必须严格遵循本指南来生成 Rust 源代码。

## 1. Wasm 运行时限制与沙箱原理
你的目标代码将被编译为 `wasm32-unknown-unknown` 或 `wasm32-wasi` 目标格式，并在 `wasmtime` 的安全沙箱中运行。
- **纯粹逻辑**：沙箱内部默认完全与宿主网络、文件系统、环境变量隔离（除非宿主通过 WASI 显式注入 capability）。你目前只能编写“纯函数”或算法型逻辑。
- **无状态**：在单次调用结束并清空内存后，状态就会重置。

## 2. 必须实现的线性内存 ABI 契约
你生成的 Rust 代码文件 (`lib.rs`) **必须**包含且仅包含以下三个使用 `#[no_mangle]` 导出的 C ABI 接口。宿主依靠这三个函数来传递 JSON 字节数据和回收内存。

### `alloc` (内存分配)
```rust
#[no_mangle]
pub extern "C" fn alloc(size: i32) -> *mut u8 {
    let mut buf = Vec::with_capacity(size as usize);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}
```

### `dealloc` (内存回收)
```rust
#[no_mangle]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, size: i32) {
    let _ = Vec::from_raw_parts(ptr, 0, size as usize);
}
```

### `execute` (核心执行逻辑)
```rust
#[no_mangle]
pub unsafe extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    // 1. 读取输入 JSON 字节
    let input_bytes = std::slice::from_raw_parts(ptr as *const u8, len as usize);
    
    // 2. 解析为 serde_json::Value
    // ... 你的核心逻辑，处理输入并生成 response ...
    
    // 3. 将结果重新序列化为 JSON 字符串
    let mut out_str = serde_json::to_string(&response).unwrap().into_bytes();
    let out_len = out_str.len() as i32;
    let out_ptr = out_str.as_mut_ptr() as i32;
    
    std::mem::forget(out_str);
    
    // 4. 将指针和长度打包成一个 i64 返回宿主
    ((out_ptr as i64) << 32) | (out_len as i64)
}
```

## 3. 代码生成要求
- **不带标签**：输出代码时，不需要包裹任何 Markdown 标签，确保它是合法的单体 `lib.rs` 源文件。
- **依赖引用**：假定宿主的 `Cargo.toml` 中已内置了 `serde` 和 `serde_json`，你可以放心使用它们。
- **错误捕获**：在 `execute` 内不能引发 panic。所有的非法输入或解析失败都应该被包在 `serde_json::json!({"success": false, "message": "error info"})` 中返回。

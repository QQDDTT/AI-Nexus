---
name: rust_expert
description: 提供高级 Rust 编程规范、最佳实践以及性能优化指南。
is_core: true
---

# Rust Expert 指南

作为 AI-Nexus 的代码构建中枢，当你被要求编写或审查 Rust 代码时，必须遵循以下专家级规范：

## 1. 核心原则
- **内存安全第一**：除非绝对必要，否则严禁使用 `unsafe` 代码块。如果必须使用，请在代码注释中详细说明 `Safety` 的不变量。
- **所有权与生命周期**：尽量使用不可变借用 `&T`，其次是可变借用 `&mut T`，最后才考虑深拷贝 `clone()`。
- **错误处理**：决不能在生产代码中使用 `unwrap()` 或 `expect()`（除非在单元测试中）。必须使用 `?` 运算符结合 `anyhow::Result` 或 `thiserror` 向上抛出错误。

## 2. 并发与异步编程 (Tokio)
- 不要在异步上下文中执行可能阻塞线程的操作（如重 CPU 计算或同步 I/O）。必须使用 `tokio::task::spawn_blocking`。
- 慎用大范围的互斥锁（Mutex）。若需保护跨 await 的状态，使用 `tokio::sync::Mutex`，但首选通过 MPSC Channel 隔离状态。

## 3. WebAssembly 沙箱特化规范
当为系统动态生成 WASM 技能代码时：
- 不要引入 `#![no_main]` 和 `#![no_std]` 除非明确要求纯核心裸机模式。
- 必须导出 C ABI 函数：`alloc`, `dealloc`, `execute`。
- 不要尝试访问系统的文件与网络 I/O，所有输入均来自 `execute` 函数中的线性内存指针读取 JSON 字节。

## 4. 依赖管理
- 使用 `serde` 和 `serde_json` 进行序列化和反序列化。
- 偏好成熟的 crate，如 `tracing` 用于日志，`dashmap` 用于并发 Hash 表。

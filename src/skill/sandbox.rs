use crate::utils::errors::AiNexusError;
use serde_json::Value;
use std::time::Instant;
use wasmtime::{Config, Engine, Module, Store, Linker};
use wasmtime_wasi::WasiCtxBuilder;
use wasmtime_wasi::p1::WasiP1Ctx;

/// 包含 WasiCtx 的执行状态
pub struct SandboxState {
    pub wasi: WasiP1Ctx,
}

/// 基于 wasmtime 的极速沙箱
pub struct WasmSandbox {
    engine: Engine,
}

impl WasmSandbox {
    /// 初始化并配置 Fuel 限制
    pub fn new() -> Result<Self, AiNexusError> {
        let mut config = Config::new();
        // 开启指令燃料消耗机制，防止死循环
        config.consume_fuel(true);
        
        let engine = Engine::new(&config).map_err(|e| {
            AiNexusError::General(format!("Failed to init Wasmtime engine: {}", e))
        })?;

        Ok(Self { engine })
    }

    /// 执行技能，注入极其严格的 Capabilities
    /// 此处实现了完整的 Wasm 内存交互 ABI (alloc, execute, dealloc)
    pub async fn execute_wasm(&self, wasm_bytes: &[u8], params: Value, fuel_limit: u64) -> Result<Value, AiNexusError> {
        // 1. 编译模块
        let module = Module::new(&self.engine, wasm_bytes).map_err(|e| {
            AiNexusError::General(format!("Failed to compile WASM module: {}", e))
        })?;

        // 2. 配置 WASI 捕获标准输出和标准错误
        let mut wasi_builder = WasiCtxBuilder::new();
        wasi_builder.inherit_stdin();
        // 此处暂未重定向管道，未来可接内存 Pipe
        wasi_builder.inherit_stdout().inherit_stderr();
        let wasi_ctx = wasi_builder.build_p1();

        // 3. 创建 Store 并设置 Fuel
        let mut store = Store::new(&self.engine, SandboxState { wasi: wasi_ctx });
        store.set_fuel(fuel_limit).map_err(|e| {
            AiNexusError::General(format!("Failed to set fuel: {}", e))
        })?;

        // 4. 初始化 Linker 并链接 WASI
        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::p1::add_to_linker_sync(&mut linker, |state: &mut SandboxState| &mut state.wasi)
            .map_err(|e| AiNexusError::General(format!("Failed to link WASI: {}", e)))?;

        // 5. 实例化
        let instance = linker.instantiate(&mut store, &module).map_err(|e| {
            AiNexusError::SkillExecutionFailed { 
                skill_name: "anonymous".to_string(), 
                reason: format!("Instantiation failed: {}", e) 
            }
        })?;

        // 获取内存模块
        let memory = instance.get_memory(&mut store, "memory").ok_or_else(|| {
            AiNexusError::General("WASM module does not export 'memory'".to_string())
        })?;

        // 获取 ABI 函数
        let alloc_func = instance.get_typed_func::<i32, i32>(&mut store, "alloc").map_err(|e| {
            AiNexusError::General(format!("Failed to find exported function 'alloc': {}", e))
        })?;
        
        let dealloc_func = instance.get_typed_func::<(i32, i32), ()>(&mut store, "dealloc").map_err(|e| {
            AiNexusError::General(format!("Failed to find exported function 'dealloc': {}", e))
        })?;

        let execute_func = instance.get_typed_func::<(i32, i32), i64>(&mut store, "execute").map_err(|e| {
            AiNexusError::General(format!("Failed to find exported function 'execute': {}", e))
        })?;

        // 4. 将参数序列化为 JSON 字符串，并写入 Wasm 内存
        let params_str = serde_json::to_string(&params).unwrap();
        let params_bytes = params_str.as_bytes();
        let params_len = params_bytes.len() as i32;

        // 在 Wasm 中分配内存
        let params_ptr = alloc_func.call(&mut store, params_len).map_err(|e| {
            AiNexusError::General(format!("Failed to call alloc: {}", e))
        })?;

        // 写入内存
        memory.write(&mut store, params_ptr as usize, params_bytes).map_err(|e| {
            AiNexusError::General(format!("Failed to write memory: {}", e))
        })?;

        // 5. 调用执行函数并计算耗时
        let start_time = Instant::now();
        let execute_result = execute_func.call(&mut store, (params_ptr, params_len));
        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        let result_packed = match execute_result {
            Ok(val) => val,
            Err(e) => {
                return Ok(serde_json::json!({
                    "exit_code": 1,
                    "stdout": "",
                    "stderr": "",
                    "execution_time_ms": execution_time_ms,
                    "result": null,
                    "trap_error": format!("Trap encountered: {}", e)
                }));
            }
        };

        // 解析返回值的指针和长度 (高 32 位是指针，低 32 位是长度)
        let res_ptr = (result_packed >> 32) as i32;
        let res_len = (result_packed & 0xFFFFFFFF) as i32;

        // 6. 读取结果内存
        let mut res_buf = vec![0u8; res_len as usize];
        memory.read(&mut store, res_ptr as usize, &mut res_buf).map_err(|e| {
            AiNexusError::General(format!("Failed to read result memory: {}", e))
        })?;

        let res_str = String::from_utf8(res_buf).map_err(|e| {
            AiNexusError::General(format!("WASM returned invalid UTF-8: {}", e))
        })?;

        // 7. 通知 Wasm 释放参数和返回值的内存
        let _ = dealloc_func.call(&mut store, (params_ptr, params_len));
        let _ = dealloc_func.call(&mut store, (res_ptr, res_len));

        // 8. 解析为 Value
        let result_val: Value = serde_json::from_str(&res_str).map_err(|e| {
            AiNexusError::General(format!("Failed to parse WASM result JSON: {}", e))
        })?;

        Ok(serde_json::json!({
            "exit_code": 0,
            "stdout": "",
            "stderr": "",
            "execution_time_ms": execution_time_ms,
            "result": result_val,
            "trap_error": null
        }))
    }
}

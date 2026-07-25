use serde::{Deserialize, Serialize};
use serde_json::Value;

// WASM 内存导出接口：分配内存
#[no_mangle]
pub extern "C" fn alloc(size: i32) -> *mut u8 {
    let mut buf = Vec::with_capacity(size as usize);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf); // 避免被释放
    ptr
}

// WASM 内存导出接口：释放内存
#[no_mangle]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, size: i32) {
    let _ = Vec::from_raw_parts(ptr, 0, size as usize);
}

// 技能的具体执行入口
#[no_mangle]
pub unsafe extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    // 1. 读取输入 JSON
    let input_bytes = std::slice::from_raw_parts(ptr as *const u8, len as usize);
    
    // 解析输入
    let input_val: Result<Value, _> = serde_json::from_slice(input_bytes);
    
    let mut response = serde_json::json!({
        "success": false,
        "message": "Initialization failed"
    });

    if let Ok(val) = input_val {
        // 执行简单的回显和修改
        response = serde_json::json!({
            "success": true,
            "received": val,
            "message": "Hello from Wasm Sandbox!",
            "status": 200
        });
    }

    // 2. 序列化输出 JSON
    let mut out_str = serde_json::to_string(&response).unwrap().into_bytes();
    let out_len = out_str.len() as i32;
    let out_ptr = out_str.as_mut_ptr() as i32;
    
    std::mem::forget(out_str);

    // 将指针和长度打包成 i64 返回
    ((out_ptr as i64) << 32) | (out_len as i64)
}

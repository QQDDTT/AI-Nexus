use serde_json::Value;

#[no_mangle]
pub extern "C" fn alloc(size: i32) -> *mut u8 {
    let mut buf = Vec::with_capacity(size as usize);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, size: i32) {
    let _ = Vec::from_raw_parts(ptr, 0, size as usize);
}

#[no_mangle]
pub unsafe extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    let input_bytes = std::slice::from_raw_parts(ptr as *const u8, len as usize);
    let mut response = serde_json::json!({
        "success": false,
        "message": "Invalid JSON input"
    });

    if let Ok(val) = serde_json::from_slice::<Value>(input_bytes) {
        // Beautify the JSON payload. If the input is just a string, it will be wrapped.
        // Usually, the skill expects the `params` object containing `json_string`.
        let mut target_val = val.clone();
        if let Some(obj) = val.as_object() {
            if let Some(json_str) = obj.get("json_string") {
                if let Some(s) = json_str.as_str() {
                    if let Ok(parsed) = serde_json::from_str::<Value>(s) {
                        target_val = parsed;
                    }
                }
            }
        }
        
        let pretty = serde_json::to_string_pretty(&target_val).unwrap_or_default();
        response = serde_json::json!({
            "success": true,
            "formatted_json": pretty,
        });
    }

    let mut out_str = serde_json::to_string(&response).unwrap().into_bytes();
    let out_len = out_str.len() as i32;
    let out_ptr = out_str.as_mut_ptr() as i32;
    
    std::mem::forget(out_str);
    ((out_ptr as i64) << 32) | (out_len as i64)
}

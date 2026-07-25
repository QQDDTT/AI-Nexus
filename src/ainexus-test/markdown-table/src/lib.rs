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
        "message": "Invalid input format"
    });

    if let Ok(val) = serde_json::from_slice::<Value>(input_bytes) {
        if let Some(obj) = val.as_object() {
            if let Some(csv_data) = obj.get("csv_data").and_then(|v| v.as_str()) {
                let mut lines = csv_data.lines();
                if let Some(header) = lines.next() {
                    let cols: Vec<&str> = header.split(',').map(|s| s.trim()).collect();
                    let mut md_table = format!("| {} |\n", cols.join(" | "));
                    md_table.push_str(&format!("|{}|\n", vec!["---"; cols.len()].join("|")));
                    
                    for line in lines {
                        let row_cols: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                        md_table.push_str(&format!("| {} |\n", row_cols.join(" | ")));
                    }
                    
                    response = serde_json::json!({
                        "success": true,
                        "markdown": md_table,
                    });
                } else {
                    response = serde_json::json!({
                        "success": false,
                        "message": "CSV data is empty"
                    });
                }
            }
        }
    }

    let mut out_str = serde_json::to_string(&response).unwrap().into_bytes();
    let out_len = out_str.len() as i32;
    let out_ptr = out_str.as_mut_ptr() as i32;
    
    std::mem::forget(out_str);
    ((out_ptr as i64) << 32) | (out_len as i64)
}

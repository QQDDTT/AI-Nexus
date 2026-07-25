
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
            let _input = std::slice::from_raw_parts(ptr as *const u8, len as usize);
            let response = serde_json::json!({"meta": "success"});
            let out_str = serde_json::to_string(&response).unwrap().into_bytes();
            let out_len = out_str.len() as i32;
            let out_ptr = out_str.as_ptr() as i32;
            std::mem::forget(out_str);
            ((out_ptr as i64) << 32) | (out_len as i64)
        }
    
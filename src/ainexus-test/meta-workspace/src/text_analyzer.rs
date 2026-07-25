use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct AnalyzeInput {
    text: String,
}

#[derive(Serialize)]
struct AnalyzeOutput {
    words: usize,
    chars: usize,
}

#[no_mangle]
pub extern "C" fn alloc(size: i32) -> *mut u8 {
    let mut buf = Vec::with_capacity(size as usize);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, size: i32) {
    let _buf = Vec::from_raw_parts(ptr, 0, size as usize);
}

#[no_mangle]
pub unsafe extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    let input_bytes = std::slice::from_raw_parts(ptr as *const u8, len as usize);
    let input: AnalyzeInput = serde_json::from_slice(input_bytes).unwrap_or(AnalyzeInput {
        text: "".to_string(),
    });

    let words = input.text.split_whitespace().count();
    let chars = input.text.chars().count();

    let output = AnalyzeOutput { words, chars };
    let mut output_bytes = serde_json::to_vec(&output).unwrap();
    
    let out_len = output_bytes.len() as i32;
    let out_ptr = output_bytes.as_mut_ptr() as i32;
    std::mem::forget(output_bytes);
    
    (out_ptr as i64) << 32 | (out_len as i64)
}

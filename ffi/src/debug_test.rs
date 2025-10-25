use std::ffi::{CStr, CString};

// Import the FFI functions from the library
use genai_keyfinder_ffi::*;

fn main() {
    unsafe {
        let temp_dir = std::env::temp_dir();
        let home_path = temp_dir.to_str().unwrap();
        println!("Testing with home directory: {}", home_path);
        
        let home = CString::new(home_path).unwrap();
        let options = CString::new(r#"{"include_full_values": false}"#).unwrap();
        
        let result = keyfinder_scan(home.as_ptr(), options.as_ptr());
        
        if result.is_null() {
            let error = keyfinder_last_error();
            if !error.is_null() {
                let error_str = CStr::from_ptr(error).to_str().unwrap();
                println!("Scan failed with error: {}", error_str);
            } else {
                println!("Scan failed with no error message");
            }
        } else {
            let result_str = CStr::from_ptr(result).to_str().unwrap();
            println!("Scan result: {}", result_str);
            keyfinder_free(result);
        }
    }
}
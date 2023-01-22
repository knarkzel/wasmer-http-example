extern {
    fn http_get(url: u32, url_len: u32) -> u32;
}

fn get(url: &str) -> String {
    unsafe {
        let offset = http_get(url.as_ptr() as u32, url.len() as u32);
        let ptr = *((offset + 0) as *const u32);
        let cap = *((offset + 4) as *const u32);
        let len = *((offset + 8) as *const u32);
        String::from_raw_parts(ptr as *mut u8, cap as usize, len as usize)        
    }
}

#[no_mangle]
fn main() {
    let response = get("https://postman-echo.com/bytes/5/mb?type=json");
    println!("Response size: {}", response.len());
}

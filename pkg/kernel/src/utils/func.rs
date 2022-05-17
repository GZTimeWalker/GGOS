pub fn test() -> ! {
    let mut count = 0;
    let id;
    if let Some(id_env) = crate::process::env("id") {
        id = id_env
    } else {
        id = "unknown".into()
    }
    loop {
        count += 1;
        if count == 100 {
            count = 0;
            print_serial!("\r{:-6} => Hello, world!", id);
        }
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

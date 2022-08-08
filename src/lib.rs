use lazy_static::lazy_static;
use libloading::{Library, Symbol};
use std::ffi::CString;
use std::arch::asm;

#[macro_export]
macro_rules! wrap_dll {
    ($dll_name:expr, $($function_name:ident)*, $(($to_hook_function_name:ident, $hook_function_name:ident))*) => {
        lazy_static! {
            static ref WRAPPED_LIBRARY: Library = unsafe {
                let library = Library::new($dll_name).expect("Couldn't load DLL");
                library
            };

            static ref WRAPPED_FUNCTIONS: Vec<(CString, unsafe extern "system" fn() -> u32)> = unsafe {
                let mut vec = Vec::new();

                $(
                    let s = CString::new(stringify!($function_name)).unwrap();
                    let function: Symbol<unsafe extern "system" fn() -> u32> = WRAPPED_LIBRARY.get(stringify!($function_name).as_bytes())
                        .expect("Unable to load function.");

                    vec.push((s, *function));
                )*

                $(
                    let s = CString::new(stringify!($to_hook_function_name)).unwrap();
                    let function: Symbol<unsafe extern "system" fn() -> u32> = WRAPPED_LIBRARY.get(stringify!($to_hook_function_name).as_bytes())
                        .expect("Unable to load function.");

                    vec.push((s, *function));
                )*

                vec
            };
        }

        $(
            create_wrapper_function!($function_name);
        )*
        $(
            create_hook_function!($to_hook_function_name, $hook_function_name);
        )*

        pub unsafe fn get_jump_address(function_name: *const u8) -> *const usize {
            let name = std::ffi::CStr::from_ptr(function_name as *const i8);
            for f in &*WRAPPED_FUNCTIONS {
                if f.0.as_c_str() == name {
                    return f.1 as *const usize;
                }
            }
            panic!("Couldn't find function: {:?}", name);
        }
    }
}

#[macro_export]
macro_rules! create_wrapper_function {
    ($function_name:ident) => {
        #[no_mangle]
        pub unsafe extern "system" fn $function_name() -> u32 {
            asm!(
                "push rcx",
                "push rdx",
                "push r8",
                "push r9",
                "push r10",
                "push r11",
                options(nostack),
            );
            asm!(
                "sub rsp, 28h",
                "call rax",
                "add rsp, 28h",
                in("rax") get_jump_address,
                in("rcx") std::concat!(stringify!($function_name), "\0").as_ptr() as usize,
                options(nostack),
            );
            asm!(
                "pop r11",
                "pop r10",
                "pop r9",
                "pop r8",
                "pop rdx",
                "pop rcx",
                "jmp rax",
                options(nostack)
            );
            1
        }
    }
}

#[macro_export]
macro_rules! create_hook_function {
    ($function_name:ident, $hook:ident) => {
        #[no_mangle]
        pub unsafe extern "system" fn $function_name() -> u32 {
            asm!(
                "push rcx",
                "push rdx",
                "push r8",
                "push r9",
                "push r10",
                "push r11",
                options(nostack),
            );
            asm!(
                "sub rsp, 28h",
                "call rax",
                "add rsp, 28h",
                in("rax") $hook,
                options(nostack),
            );
            asm!(
                "sub rsp, 28h",
                "call rax",
                "add rsp, 28h",
                in("rax") get_jump_address,
                in("rcx") std::concat!(stringify!($function_name), "\0").as_ptr() as usize,
                options(nostack),
            );
            asm!(
                "pop r11",
                "pop r10",
                "pop r9",
                "pop r8",
                "pop rdx",
                "pop rcx",
                "jmp rax",
                options(nostack)
            );
            1
        }
    }
}
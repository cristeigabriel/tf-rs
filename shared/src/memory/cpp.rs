//! C++ ABI functions/macros module

/// Get virtual function from `class` at `index`
#[macro_export]
macro_rules! get_virtual_function {
    ($class:expr, $index:expr) => {
        unsafe {
            // Get vftable, first thing in structure
            // Function pointers are 4 bytes, so we store it as a usize pointer
            // For offset to take index
            let table = *($class as *const *const usize);
            (*table.offset($index as isize)) as *const usize
        }
    };
}

/// Get virtual function from `class` at `index` and cast it to
/// `call_type` and call it with first argument always being `class`
/// then `args` expansion
#[macro_export]
macro_rules! call_virtual_function {
    ($class:expr, $index:expr, $call_type:ty, $($arg:tt)*) => {
        unsafe {
            std::mem::transmute::<*const usize, $call_type>(
                $crate::get_virtual_function!($class, $index),
            )($class as _, $($arg)*)
        }
    };
    ($class:expr, $index:expr, $call_type:ty) => {
        call_virtual_function!($class, $index, $call_type,)
    };
}

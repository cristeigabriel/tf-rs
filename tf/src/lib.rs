#![feature(abi_thiscall)]
#![feature(let_chains)]

#[cfg(not(target_os = "windows"))]
compile_error!("Windows support only");

pub mod ctx;
pub mod error;
pub mod hooks;
use shared::{console::*, entry_point, GenericErrOr, *};
use winapi::um::winnt::DLL_PROCESS_ATTACH;

/// Handles initializing console (global context), sets title to what's given
fn initialize_console(title: &str) -> ConsoleErrOr<()> {
    // In case there's already a console open
    free_console()?;

    // We want a console for IO
    alloc_console()?;

    // Set console title
    set_title(title)?;

    // Notify user
    println!("Initialized IO");

    Ok(())
}

/// Handles initialization of context
fn initialize_context(module: HMODULE) -> GenericErrOr<()> {
    // Sets up context handle
    ctx::install(module)?;

    // Notify user
    println!("Initialized Context");

    Ok(())
}

/// Handles initialization of hooks
fn initialize_hooks() -> GenericErrOr<()> {
    // Get context
    if let Ok(ctx) = try_acquire_ctx_locked!() {
        // Install hooks
        hooks::install(&*ctx)?;
    }

    // Notify user
    println!("Initialized Hooks");

    Ok(())
}

entry_point!(|module, reason_for_call| {
    if reason_for_call != DLL_PROCESS_ATTACH {
        return false;
    }

    initialize_console("tf\0").expect("Failed initializing console");
    initialize_context(module).expect("Failed initializing context");
    initialize_hooks().expect("Failed initializing hooks");

    if let Ok(ctx) = try_acquire_ctx_locked!() {
        ctx.get_cvar().console_print(c_str!("Hello, world!\0"));
    }

    true
});

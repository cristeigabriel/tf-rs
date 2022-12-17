//! Context module

use crate::{error::Error, HMODULE};
use game::types::{chl_client::CHLClient, convar::Cvar, mat_system_surface::MatSystemSurface};
use shared::{memory::module::*, GenericErrOr};
use std::sync::Mutex;

/// Type for context errors
pub(super) type ContextErrOr<T> = GenericErrOr<T>;

pub(super) struct Context {
    module: HMODULE,
    client: Module,
    engine: Module,
    vguimatsurface: Module,
    cvar: Cvar,
    chl_client: CHLClient,
    mat_system_surface: MatSystemSurface,
}

/// Singleton and synchronizer for cheat operations
///
/// Don't use alone, that's why it's not public
static mut CTX: Option<Mutex<Context>> = None;

impl Context {
    fn new(module: HMODULE) -> ContextErrOr<Self> {
        // Get modules
        let client = Module::new("client.dll\0")?;
        let engine = Module::new("engine.dll\0")?;
        let vguimatsurface = Module::new("vguimatsurface.dll\0")?;

        // Get CCvar
        let cvar = Cvar::from(engine.get_exports()["cvar"].deref(1)?.get_ptr());

        // Get CHLClient
        let chl_client = CHLClient::from(
            engine
                .find_pattern("8B 0D ? ? ? ? 8B 15 ? ? ? ? 8B")?
                .offset(2)?
                .deref(2)?
                .get_ptr(),
        );

        // Get MatSystemSurface
        let mat_system_surface = MatSystemSurface::from(
            vguimatsurface
                .find_pattern("A3 ? ? ? ? 83 3D ? ? ? ? ? 75 14 8B 04 B7 6A 00 68")?
                .offset(1)?
                .deref(2)?
                .get_ptr(),
        );

        Ok(Self {
            module,
            client,
            engine,
            vguimatsurface,
            cvar,
            chl_client,
            mat_system_surface,
        })
    }

    // Getters

    pub(super) fn get_module(&self) -> HMODULE {
        self.module
    }

    pub(super) fn get_client(&self) -> &Module {
        &self.client
    }

    pub(super) fn get_engine(&self) -> &Module {
        &self.engine
    }

    pub(super) fn get_vguimatsurface(&self) -> &Module {
        &self.vguimatsurface
    }

    // Interface getters

    pub(super) fn get_cvar(&self) -> &Cvar {
        &self.cvar
    }

    pub(super) fn get_cvar_mut(&mut self) -> &mut Cvar {
        &mut self.cvar
    }

    pub(super) fn get_chl_client(&self) -> &CHLClient {
        &self.chl_client
    }

    pub(super) fn get_chl_client_mut(&mut self) -> &mut CHLClient {
        &mut self.chl_client
    }

    pub(super) fn get_mat_system_surface(&self) -> &MatSystemSurface {
        &self.mat_system_surface
    }

    pub(super) fn get_mat_system_surface_mut(&mut self) -> &mut MatSystemSurface {
        &mut self.mat_system_surface
    }
}

// Singleton public utilities

/// Install singleton
pub(super) fn install(module: HMODULE) -> ContextErrOr<()> {
    unsafe {
        if CTX.is_some() {
            return Err(Error::AlreadyInitialized.into());
        } else {
            CTX = Some(Mutex::from(Context::new(module)?));
        };
    }

    Ok(())
}

/// Get reference to promised (unchecked) context handle
pub(super) fn get_ctx() -> &'static Mutex<Context> {
    unsafe { CTX.as_ref().unwrap() }
}

/// Syntactic sugar macro for attempting to acquire lock on context handle
#[macro_export]
macro_rules! try_acquire_ctx_locked {
    () => {
        $crate::ctx::get_ctx().try_lock()
    };
}

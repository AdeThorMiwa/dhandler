use di::ServiceProvider;
use loco_rs::prelude::*;
use std::any::type_name;
use std::sync::Arc;

/// # Errors
/// Returns an error if the type `T` is not found in the shared store.
pub fn get<T: 'static>(ctx: &AppContext) -> Result<Arc<T>> {
    let provider = ctx.shared_store.get::<ServiceProvider>().ok_or_else(|| {
        tracing::error!("Type {} not found in shared store", type_name::<T>());
        Error::InternalServerError // Or a more specific error
    })?;

    Ok(provider.get_required::<T>())
}

pub fn get_pid(auth: &auth::JWT) -> Result<Uuid> {
    let pid: Uuid = auth.claims.pid.parse().map_err(|e| {
        tracing::error!("Failed to parse pid: {}", e);
        Error::InternalServerError
    })?;

    Ok(pid)
}

//! Runtime abstraction for cross-platform async support

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::future::Future;

    pub fn spawn<F>(future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        smol::spawn(future).detach();
    }

    pub fn unblock<F, T>(f: F) -> impl Future<Output = T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        smol::unblock(f)
    }

    pub async fn sleep(duration: std::time::Duration) {
        smol::Timer::after(duration).await;
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::future::Future;
    use wasm_bindgen_futures::spawn_local;

    pub fn spawn<F>(future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        spawn_local(future);
    }

    pub async fn unblock<F, T>(f: F) -> T
    where
        F: FnOnce() -> T + 'static,
        T: 'static,
    {
        // WASM: run on main thread (no threads available)
        f()
    }

    pub async fn sleep(duration: std::time::Duration) {
        let millis = duration.as_millis() as i32;
        gloo_timers::future::sleep(std::time::Duration::from_millis(millis as u64)).await;
    }
}

use std::future::Future;

// Wait for rust async closures to be stable, then return a promise and provide a sender from poll_promise
pub fn spawn_thread<F>(work: F)
where
    F: Future<Output = ()> + 'static + Send,
{
    #[cfg(target_arch = "wasm32")]
    {
        // For WebAssembly target
        wasm_bindgen_futures::spawn_local(async move {
            work.await;
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // For native target
        tokio::spawn(async move {
            work.await;
        });
    }
}

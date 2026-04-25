#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn info(message: &str) {
    eprintln!("[calendar] {message}");
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn error(message: &str) {
    eprintln!("[calendar] ERROR {message}");
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn info(message: &str) {
    worker::console_log!("[calendar] {}", message);
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn error(message: &str) {
    worker::console_error!("[calendar] ERROR {}", message);
}

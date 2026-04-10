// Скрывает консоль на Windows при сборке релизной версии (не отладочной)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Entrypoint для desktop-обертки Tauri.

fn main() {
    #[cfg(all(debug_assertions, target_os = "linux"))]
    {
        // SAFETY: `set_var` is unsafe (env var mutations can be UB if other threads
        // concurrently access the environment on some platforms). Здесь вызов
        // выполняется в начале `main`, до запуска потоков со стороны Tauri.
        unsafe {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }

    // Вызов функции run() из lib.rs.
    // Замените `app_lib` на имя вашей библиотеки из src-tauri/Cargo.toml (обычно совпадает с именем пакета)
    app_lib::run();
}

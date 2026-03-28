/* File: src/ui/about_logs.rs */
use yew::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[function_component(AboutLogsTab)]
pub fn about_logs_tab() -> Html {
    // Вспомогательное состояние-счетчик для принудительного обновления компонента
    let tick = use_state(|| 0);

    // Таймер живет только пока существует этот компонент
    use_effect_with((), {
        let tick = tick.clone();
        move |_| {
            let interval = gloo_timers::callback::Interval::new(500, move || {
                tick.set(*tick + 1);
            });
            || drop(interval)
        }
    });

    let open_github = Callback::from(|_| {
        wasm_bindgen_futures::spawn_local(async move {
            #[derive(serde::Serialize)]
            struct OpenArgs { path: String }
            if let Ok(args) = serde_wasm_bindgen::to_value(&OpenArgs { path: "https://github.com/".to_string() }) {
                let _ = invoke("plugin:shell|open", args).await;
            }
        });
    });

    // Берем логи напрямую из глобального логгера (без использования StateContext)
    let logs = if let Ok(buffer) = crate::logger::LOG_BUFFER.lock() {
        buffer.clone()
    } else {
        Vec::new()
    };

    html! {
        <div class="tab-content fade-in">
            <div class="about-grid">
                <div class="card about-card">
                    <h3>{"О приложении"}</h3>
                    <p><strong>{"IAPWS-IF97 Calculator Pro"}</strong></p>
                    <p>{"Версия: 1.0.0 (Tauri + WASM Edition)"}</p>
                    <p>{"Высокопроизводительное ядро расчета теплофизических свойств воды и водяного пара."}</p>
                </div>
                <div class="card author-card">
                    <h3>{"Ссылки"}</h3>
                    <div class="actions flex-col">
                        <button onclick={open_github} class="btn btn-outline w-full text-left" title="Открыть GitHub">{"🌐 GitHub Репозиторий"}</button>
                        <button class="btn btn-outline w-full text-left">{"☕ Поддержать проект"}</button>
                    </div>
                </div>
            </div>

            <div class="card logs-card flex-grow">
                <div class="logs-header">
                    <h3>{"Системный монитор"}</h3>
                    <button class="btn btn-outline btn-sm">{"💾 Экспорт (.log)"}</button>
                </div>
                <div class="terminal-view">
                    { for logs.iter().map(|(level, msg)| {
                        let class_name = format!("log-line {}", level);
                        html! { <div class={class_name}>{ msg }</div> }
                    }) }
                </div>
            </div>
        </div>
    }
}
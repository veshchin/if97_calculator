use yew::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[function_component(AboutLogsTab)]
pub fn about_logs_tab() -> Html {
    let open_github = Callback::from(|_| {
        wasm_bindgen_futures::spawn_local(async move {
            #[derive(serde::Serialize)]
            struct OpenArgs {
                path: String,
            }
            if let Ok(args) = serde_wasm_bindgen::to_value(&OpenArgs {
                path: "https://github.com/".to_string(), // Замени на свою ссылку
            }) {
                let _ = invoke("plugin:shell|open", args).await;
            }
        });
    });

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
                    <div class="log-line info">{"[INFO] if97_core initialized successfully in WASM environment."}</div>
                    <div class="log-line debug">{"[DEBUG] (rho, t) solvers loaded for Region 3."}</div>
                    <div class="log-line info">{"[INFO] IPC Bridge connected to Tauri Backend."}</div>
                </div>
            </div>
        </div>
    }
}
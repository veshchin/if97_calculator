//! Вкладка "О программе" и системный журнал.

use crate::logger;
use crate::tauri_api;
use if97_app_api::LogEntryDto;
use yew::prelude::*;

#[function_component(AboutLogsTab)]
/// Вкладка с информацией о приложении и выводом журнала.
pub fn about_logs_tab() -> Html {
    let logs = use_state(Vec::<LogEntryDto>::new);

    use_effect_with((), {
        let logs = logs.clone();
        move |_| {
            let sync_logs = {
                let logs = logs.clone();
                move || {
                    let logs = logs.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let mut merged = tauri_api::read_logs().await.unwrap_or_default();
                        merged.extend(logger::snapshot_logs());
                        logs.set(merged);
                    });
                }
            };

            {
                let logs = logs.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = tauri_api::clear_logs().await;
                    logger::clear_logs();
                    let mut merged = tauri_api::read_logs().await.unwrap_or_default();
                    merged.extend(logger::snapshot_logs());
                    logs.set(merged);
                });
            }
            let interval = gloo_timers::callback::Interval::new(1000, move || {
                sync_logs();
            });

            || drop(interval)
        }
    });

    let open_repo = Callback::from(|_| {
        wasm_bindgen_futures::spawn_local(async move {
            let _ = tauri_api::open_external(
                "https://github.com/veshchin/if97_calculator".to_string(),
            )
            .await;
        });
    });

    let export_logs = {
        let logs = logs.clone();
        Callback::from(move |_| {
            let content = logs
                .iter()
                .map(|entry| entry.message.as_str())
                .collect::<Vec<_>>()
                .join("\n");

            wasm_bindgen_futures::spawn_local(async move {
                let _ = tauri_api::save_file_dialog(tauri_api::SaveFileArgs {
                    content,
                    default_name: Some("if97.log".to_string()),
                    filter_name: Some("Файл журнала".to_string()),
                    filter_ext: Some("log".to_string()),
                })
                .await;
            });
        })
    };

    html! {
        <div class="tab-content fade-in">
            <div class="about-grid">
                <div class="card about-card">
                    <h3>{"О приложении"}</h3>
                    <p><strong>{"Калькулятор IF97"}</strong></p>
                    <p>{"Версия: 0.1.4 (редакция Tauri)"}</p>
                    <p>{"Кроссплатформенный настольный калькулятор свойств воды и пара по IF97."}</p>
                </div>
                <div class="card author-card">
                    <h3>{"Ссылки"}</h3>
                    <div class="actions flex-col">
                        <button onclick={open_repo} class="btn btn-outline w-full text-left" title="Открыть репозиторий GitHub">{"Репозиторий"}</button>
                    </div>
                </div>
            </div>

            <div class="card logs-card flex-grow">
                <div class="logs-header">
                    <h3>{"Системный журнал"}</h3>
                    <button class="btn btn-outline btn-sm" onclick={export_logs}>{"Экспорт .log"}</button>
                </div>
                <div class="terminal-view">
                    { for logs.iter().map(|entry| {
                        let class_name = format!("log-line {}", entry.level);
                        html! { <div class={class_name}>{ &entry.message }</div> }
                    }) }
                </div>
            </div>
        </div>
    }
}

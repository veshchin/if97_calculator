use yew::prelude::*;
use web_sys::HtmlInputElement;
use crate::app::App;
use crate::types::{Msg, PlotType};

pub fn view_tabs(app: &App, ctx: &Context<App>) -> Html {
    html! {
        <div class="container mt-4">
            <h2 class="mb-4">{ "IF97 Water Properties Calculator" }</h2>
            <ul class="nav nav-tabs mb-4">
                <li class="nav-item">
                    <a class={if app.active_tab == 0 { "nav-link active" } else { "nav-link" }}
                       href="#"
                       onclick={ctx.link().callback(|e: MouseEvent| { e.prevent_default(); Msg::SwitchTab(0) })}>
                        { "Одиночный расчет" }
                    </a>
                </li>
                <li class="nav-item">
                    <a class={if app.active_tab == 1 { "nav-link active" } else { "nav-link" }}
                       href="#"
                       onclick={ctx.link().callback(|e: MouseEvent| { e.prevent_default(); Msg::SwitchTab(1) })}>
                        { "Табличный расчет" }
                    </a>
                </li>
                <li class="nav-item">
                    <a class={if app.active_tab == 2 { "nav-link active" } else { "nav-link" }}
                       href="#"
                       onclick={ctx.link().callback(|e: MouseEvent| { e.prevent_default(); Msg::SwitchTab(2) })}>
                        { "Графики" }
                    </a>
                </li>
            </ul>

            <div class="tab-content">
                if app.active_tab == 0 {
                    { view_single(app, ctx) }
                } else if app.active_tab == 1 {
                    { view_batch(app, ctx) }
                } else if app.active_tab == 2 {
                    { view_plot(app, ctx) }
                }
            </div>
        </div>
    }
}

fn view_single(app: &App, ctx: &Context<App>) -> Html {
    html! {
        <div class="row">
            <div class="col-md-5">
                <div class="mb-3">
                    <label class="form-label">{ "Режим расчета:" }</label>
                    <select class="form-select" onchange={ctx.link().callback(|e: Event| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Msg::SetSingleMode(input.value().parse().unwrap_or(0))
                    })}>
                        <option value="0" selected={app.single_mode == 0}>{ "p, T" }</option>
                        <option value="1" selected={app.single_mode == 1}>{ "rho, T" }</option>
                        <option value="2" selected={app.single_mode == 2}>{ "p, h" }</option>
                        <option value="3" selected={app.single_mode == 3}>{ "p, s" }</option>
                        <option value="4" selected={app.single_mode == 4}>{ "p, x" }</option>
                    </select>
                </div>
                <div class="mb-3">
                    <label class="form-label">{ "Параметр 1:" }</label>
                    <input type="text" class="form-control" value={app.input_a.clone()} oninput={ctx.link().callback(|e: InputEvent| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Msg::UpdateInputA(input.value())
                    })} />
                </div>
                <div class="mb-3">
                    <label class="form-label">{ "Параметр 2:" }</label>
                    <input type="text" class="form-control" value={app.input_b.clone()} oninput={ctx.link().callback(|e: InputEvent| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Msg::UpdateInputB(input.value())
                    })} />
                </div>

                // Настройка точности
                <div class="mb-3">
                    <label class="form-label">{ "Точность (знаков после запятой):" }</label>
                    <input type="number" class="form-control" min="0" max="10" value={app.precision.to_string()} oninput={ctx.link().callback(|e: InputEvent| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Msg::UpdatePrecision(input.value().parse().unwrap_or(4))
                    })} />
                </div>

                <button class="btn btn-primary me-2" onclick={ctx.link().callback(|_| Msg::CalculateSingle)}>{ "Рассчитать" }</button>
                <button class="btn btn-success" onclick={ctx.link().callback(|_| Msg::SaveSinglePoint)}>{ "Сохранить точку" }</button>
            </div>
            <div class="col-md-7">
                <div class="card">
                    <div class="card-header">{ "Результат" }</div>
                    <div class="card-body">
                        <pre style="font-family: monospace; white-space: pre-wrap;">{ &app.single_result_text }</pre>
                    </div>
                </div>
            </div>
        </div>
    }
}

fn view_batch(app: &App, ctx: &Context<App>) -> Html {
    html! {
        <div class="row">
            <div class="col-md-4">
                <div class="mb-3">
                    <label class="form-label">{ "Режим расчета:" }</label>
                    <select class="form-select" onchange={ctx.link().callback(|e: Event| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Msg::SetBatchMode(input.value().parse().unwrap_or(0))
                    })}>
                        <option value="0" selected={app.batch_mode == 0}>{ "p, T" }</option>
                        <option value="1" selected={app.batch_mode == 1}>{ "rho, T" }</option>
                        <option value="2" selected={app.batch_mode == 2}>{ "p, h" }</option>
                        <option value="3" selected={app.batch_mode == 3}>{ "p, s" }</option>
                        <option value="4" selected={app.batch_mode == 4}>{ "p, x" }</option>
                    </select>
                </div>

                // Настройка точности
                <div class="mb-3">
                    <label class="form-label">{ "Точность вывода (знаков):" }</label>
                    <input type="number" class="form-control" min="0" max="10" value={app.precision.to_string()} oninput={ctx.link().callback(|e: InputEvent| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Msg::UpdatePrecision(input.value().parse().unwrap_or(4))
                    })} />
                </div>

                <div class="mb-3">
                    <label class="form-label">{ "Данные (по 2 числа на строку):" }</label>
                    <textarea class="form-control" rows="10" value={app.batch_input.clone()} oninput={ctx.link().callback(|e: InputEvent| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Msg::UpdateBatchInput(input.value())
                    })}></textarea>
                </div>
                <button class="btn btn-primary me-2" onclick={ctx.link().callback(|_| Msg::CalculateBatch)}>{ "Рассчитать" }</button>
                <button class="btn btn-success" onclick={ctx.link().callback(|_| Msg::SaveBatchTable)}>{ "Сохранить набор" }</button>
            </div>
            <div class="col-md-8">
                <div class="card">
                    <div class="card-header">{ "Результаты" }</div>
                    <div class="card-body">
                        // Используем monospace, чтобы таблица отображалась ровно
                        <pre style="font-family: monospace; overflow-x: auto; white-space: pre;">
                            { app.batch_result_table.join("\n") }
                        </pre>
                    </div>
                </div>
            </div>
        </div>
    }
}

fn view_plot(app: &App, ctx: &Context<App>) -> Html {
    html! {
        <div class="row">
            <div class="col-md-3">
                <div class="mb-3">
                    <label class="form-label">{ "Тип диаграммы:" }</label>
                    <select class="form-select" onchange={ctx.link().callback(|e: Event| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        let pt = match input.value().as_str() {
                            "PT" => PlotType::PT,
                            "PV" => PlotType::PV,
                            "TS" => PlotType::TS,
                            "HS" => PlotType::HS,
                            "PH" => PlotType::PH,
                            "RhoT" => PlotType::RhoT,
                            "VT" => PlotType::VT,
                            _ => PlotType::PT,
                        };
                        Msg::SetPlotType(pt)
                    })}>
                        <option value="PT" selected={matches!(app.plot_type, PlotType::PT)}>{ "P-T" }</option>
                        <option value="PV" selected={matches!(app.plot_type, PlotType::PV)}>{ "P-v" }</option>
                        <option value="TS" selected={matches!(app.plot_type, PlotType::TS)}>{ "T-s" }</option>
                        <option value="HS" selected={matches!(app.plot_type, PlotType::HS)}>{ "h-s" }</option>
                        <option value="PH" selected={matches!(app.plot_type, PlotType::PH)}>{ "P-h" }</option>
                    </select>
                </div>
                <div class="form-check mb-2">
                    <input class="form-check-input" type="checkbox" checked={app.show_dome} onchange={ctx.link().callback(|e: Event| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Msg::ToggleDome(input.checked())
                    })} />
                    <label class="form-check-label">{ "Показать купол" }</label>
                </div>
                <div class="form-check mb-2">
                    <input class="form-check-input" type="checkbox" checked={app.swap_axes} onchange={ctx.link().callback(|e: Event| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Msg::ToggleSwapAxes(input.checked())
                    })} />
                    <label class="form-check-label">{ "Поменять оси" }</label>
                </div>
                <div class="form-check mb-3">
                    <input class="form-check-input" type="checkbox" checked={app.autoscale} onchange={ctx.link().callback(|e: Event| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Msg::ToggleAutoscale(input.checked())
                    })} />
                    <label class="form-check-label">{ "Автомасштаб" }</label>
                </div>

                if !app.autoscale {
                    <div class="mb-2"><small>{ "Мин Ось 1:" }</small>
                        <input class="form-control form-control-sm" value={app.val_min.to_string()} oninput={ctx.link().callback(|e: InputEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into(); Msg::UpdateLimit(0, input.value())
                        })} />
                    </div>
                    <div class="mb-2"><small>{ "Макс Ось 1:" }</small>
                        <input class="form-control form-control-sm" value={app.val_max.to_string()} oninput={ctx.link().callback(|e: InputEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into(); Msg::UpdateLimit(1, input.value())
                        })} />
                    </div>
                    <div class="mb-2"><small>{ "Мин Ось 2:" }</small>
                        <input class="form-control form-control-sm" value={app.t_min.to_string()} oninput={ctx.link().callback(|e: InputEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into(); Msg::UpdateLimit(2, input.value())
                        })} />
                    </div>
                    <div class="mb-3"><small>{ "Макс Ось 2:" }</small>
                        <input class="form-control form-control-sm" value={app.t_max.to_string()} oninput={ctx.link().callback(|e: InputEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into(); Msg::UpdateLimit(3, input.value())
                        })} />
                    </div>
                }

                <hr/>
                <h6>{ "Наборы данных" }</h6>
                {
                    for app.datasets.iter().enumerate().map(|(i, ds)| {
                        html! {
                            <div class="form-check">
                                <input class="form-check-input" type="checkbox" checked={ds.visible} onchange={ctx.link().callback(move |e: Event| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    Msg::ToggleDatasetVisibility(i, input.checked())
                                })} />
                                <label class="form-check-label">
                                    { format!("{} (точек: {})", ds.name, ds.points.len()) }
                                </label>
                            </div>
                        }
                    })
                }
            </div>
            <div class="col-md-9">
                <canvas id="plot_canvas" width="900" height="550" style="border:1px solid #ccc; cursor: grab;"
                    onwheel={ctx.link().callback(|e: WheelEvent| Msg::ZoomPlot(e))}
                    onmousedown={ctx.link().callback(|e: MouseEvent| Msg::PlotMouseDown(e.offset_x(), e.offset_y()))}
                    onmousemove={ctx.link().callback(|e: MouseEvent| Msg::PlotMouseMove(e.offset_x(), e.offset_y()))}
                    onmouseup={ctx.link().callback(|_| Msg::PlotMouseUp)}
                    onmouseleave={ctx.link().callback(|_| Msg::PlotMouseUp)}
                ></canvas>
                <div class="mt-2 text-muted">
                    <small>{ "💡 Используйте колесико мыши для масштабирования графика и левую кнопку для перетаскивания (при выключенном автомасштабе)." }</small>
                </div>
            </div>
        </div>
    }
}
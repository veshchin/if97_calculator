/* File: src/app.rs */
use yew::prelude::*;
use crate::ui::{single_calc::SingleCalcTab, table_calc::TableCalcTab, plots::PlotsTab, about_logs::AboutLogsTab};
use crate::types::{SavedItem, AppContext, PersistentState, StateContext};

#[derive(Clone, PartialEq)]
pub enum Tab { Single, Table, Plots, About }

#[function_component(App)]
pub fn app() -> Html {
    let active_tab = use_state(|| Tab::Single);

    // Глобальное состояние сохраненных данных (точек и таблиц)
    let saved_items = use_state(Vec::<SavedItem>::new);

    let persistent_state = use_state(|| PersistentState {
        s_mode: "pt".to_string(), s_v1: String::new(), s_v2: String::new(), s_res: None, s_error: None,
        t_input: String::new(), t_mode: "pt".to_string(), t_res: Vec::new(),
    });

    let set_tab = |tab: Tab| {
        let active_tab = active_tab.clone();
        Callback::from(move |_| active_tab.set(tab.clone()))
    };

    let get_tab_class = |tab: Tab, is_flex: bool| {
        if *active_tab == tab { if is_flex { "tab-active-flex" } else { "tab-active" } } else { "tab-hidden" }
    };

    html! {
        <ContextProvider<AppContext> context={saved_items.clone()}>
            <ContextProvider<StateContext> context={persistent_state.clone()}>
                <div class="app-container">
                    <aside class="sidebar">
                        <div class="sidebar-logo">{ "IF97 Calc Pro" }</div>
                        <nav class="sidebar-nav">
                            <button class={classes!("nav-btn", (*active_tab == Tab::Single).then_some("active"))} onclick={set_tab(Tab::Single)}>{ "📍 Одиночный расчет" }</button>
                            <button class={classes!("nav-btn", (*active_tab == Tab::Table).then_some("active"))} onclick={set_tab(Tab::Table)}>{ "📋 Табличный расчет" }</button>
                            <button class={classes!("nav-btn", (*active_tab == Tab::Plots).then_some("active"))} onclick={set_tab(Tab::Plots)}>{ "📈 Графики" }</button>
                            <button class={classes!("nav-btn", (*active_tab == Tab::About).then_some("active"))} onclick={set_tab(Tab::About)}>{ "ℹ️ О программе" }</button>
                        </nav>
                        <div class="spacer" />
                        <div class="sidebar-footer" style="padding: 20px; font-size: 0.85rem; color: var(--text-muted); border-top: 1px solid var(--border);">
                            { format!("💾 Сохранено наборов: {}", saved_items.len()) }
                        </div>
                    </aside>

                    <main class="main-content">
                        <div class={get_tab_class(Tab::Single, false)}><SingleCalcTab /></div>
                        <div class={get_tab_class(Tab::Table, true)}><TableCalcTab /></div>
                        <div class={get_tab_class(Tab::Plots, false)}><PlotsTab active={*active_tab == Tab::Plots} /></div>
                        <div class={get_tab_class(Tab::About, false)}><AboutLogsTab /></div>
                    </main>
                </div>
            </ContextProvider<StateContext>>
        </ContextProvider<AppContext>>
    }
}
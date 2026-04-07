/* File: src/app.rs */
use yew::prelude::*;
use crate::ui::{single_calc::SingleCalcTab, table_calc::TableCalcTab, plots::PlotsTab, about_logs::AboutLogsTab};
use crate::types::{SavedItem, AppContext, PersistentState, StateContext};
use std::collections::{HashSet, HashMap};

#[derive(Clone, PartialEq)]
pub enum Tab { Single, Table, Plots, About }

#[function_component(App)]
pub fn app() -> Html {
    let active_tab = use_state(|| Tab::Single);
    let saved_items = use_state(Vec::<SavedItem>::new);

    let persistent_state = use_state(|| PersistentState {
        s_mode: "pt".to_string(), s_v1: String::new(), s_v2: String::new(), s_res: None, s_error: None,
        t_input: String::new(), t_mode: "pt".to_string(), t_res: Vec::new(),
        t_gen_params: HashMap::new(),
        right_sidebar_open: false,
        plot_selected: HashSet::new(),
        is_dark_theme: false,
    });

    let set_tab = |tab: Tab| {
        let active_tab = active_tab.clone();
        Callback::from(move |_| active_tab.set(tab.clone()))
    };

    // ИСПРАВЛЕНИЕ ЗДЕСЬ: Возвращен класс "tab-active" для обычных (не-флекс) вкладок
    let get_tab_class = |tab: Tab, is_flex: bool| {
        if *active_tab == tab { if is_flex { "tab-active-flex" } else { "tab-active" } } else { "tab-hidden" }
    };

    let toggle_theme = {
        let state_ctx = persistent_state.clone();
        Callback::from(move |_| {
            let mut new_s = (*state_ctx).clone();
            new_s.is_dark_theme = !new_s.is_dark_theme;
            state_ctx.set(new_s);
        })
    };

    let toggle_data_panel = {
        let state_ctx = persistent_state.clone();
        Callback::from(move |_| {
            let mut new_s = (*state_ctx).clone();
            new_s.right_sidebar_open = !new_s.right_sidebar_open;
            state_ctx.set(new_s);
        })
    };

    let theme_class = if persistent_state.is_dark_theme { "theme-dark" } else { "theme-light" };

    html! {
        <ContextProvider<AppContext> context={saved_items.clone()}>
            <ContextProvider<StateContext> context={persistent_state.clone()}>
                <div class={classes!("app-container", theme_class)}>

                    <aside class="sidebar">
                        <button class={classes!("nav-btn", (*active_tab == Tab::Single).then_some("active"))} onclick={set_tab(Tab::Single)} title="Одиночный расчет">
                            {"📍"}
                        </button>
                        <button class={classes!("nav-btn", (*active_tab == Tab::Table).then_some("active"))} onclick={set_tab(Tab::Table)} title="Табличный расчет">
                            {"📋"}
                        </button>
                        <button class={classes!("nav-btn", (*active_tab == Tab::Plots).then_some("active"))} onclick={set_tab(Tab::Plots)} title="Графики">
                            {"📈"}
                        </button>
                        <button class={classes!("nav-btn", (*active_tab == Tab::About).then_some("active"))} onclick={set_tab(Tab::About)} title="О программе и Логи">
                            {"ℹ️"}
                        </button>

                        <div class="spacer" />

                        <button class={classes!("nav-btn", persistent_state.right_sidebar_open.then_some("active"))} onclick={toggle_data_panel} title="Управление сохраненными данными">
                            <svg viewBox="0 0 24 24" style="width: 22px; height: 22px; stroke: currentColor; stroke-width: 2; fill: none;">
                                <rect x="3" y="3" width="18" height="18" rx="4" />
                                <path d="M15 3v18" />
                                <rect x="17" y="5" width="2" height="14" rx="1" style={format!("fill: currentColor; transition: opacity 0.2s; opacity: {};", if persistent_state.right_sidebar_open { "1" } else { "0" })} />
                            </svg>
                        </button>

                        <button class="nav-btn" onclick={toggle_theme} title="Переключить тему" style="margin-bottom: 10px;">
                            { if persistent_state.is_dark_theme { "🌑" } else { "☀️" } }
                        </button>
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
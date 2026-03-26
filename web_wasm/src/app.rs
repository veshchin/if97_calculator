use yew::prelude::*;
use if97_core::domain::calculator::Calculator;
use if97_core::domain::state::WaterState;
use if97_core::app::router::Router;

use crate::types::*;
use crate::ui::view_tabs;
use crate::plot::draw_chart;

pub struct App {
    pub active_tab: usize,

    pub single_mode: i32,
    pub input_a: String,
    pub input_b: String,
    pub single_result_text: String,
    pub last_single_result: Option<WaterState>,

    pub batch_mode: i32,
    pub batch_input: String,
    pub batch_result_table: Vec<String>,

    pub datasets: Vec<SavedData>,
    pub plot_type: PlotType,
    pub show_dome: bool,
    pub swap_axes: bool,
    pub autoscale: bool,
    pub val_min: f64, pub val_max: f64,
    pub t_min: f64, pub t_max: f64,

    // Переменные для перетаскивания графика
    pub is_dragging: bool,
    pub last_mouse_pos_phys: (f64, f64),

    // Настройка точности вывода (знаков после запятой)
    pub precision: usize,
}

impl App {
    // Вспомогательная функция, чтобы убрать дублирование кода (Duplicated code fragment)
    fn run_calc(mode: i32, va: f64, vb: f64) -> Result<WaterState, String> {
        match mode {
            0 => Calculator::calculate_pt(va, vb).map_err(|e| e.to_string()),
            1 => Calculator::calculate_rhot(va, vb).map_err(|e| e.to_string()),
            2 => Router::calculate_ph(va, vb).map_err(|e| e.to_string()),
            3 => Router::calculate_ps(va, vb).map_err(|e| e.to_string()),
            _ => Calculator::calculate_px(va, vb).map_err(|e| e.to_string()),
        }
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            active_tab: 0,
            single_mode: 0,
            input_a: "0.1".to_string(),
            input_b: "300.0".to_string(),
            single_result_text: "Ожидание данных...".to_string(),
            last_single_result: None,
            batch_mode: 0,
            batch_input: "".to_string(),
            batch_result_table: vec![],
            datasets: vec![SavedData { name: "Текущая таблица".to_string(), points: vec![], visible: true }],
            plot_type: PlotType::PT,
            show_dome: true,
            swap_axes: false,
            autoscale: true,
            val_min: 0.0,
            val_max: 100.0,
            t_min: 273.15,
            t_max: 1000.0,
            is_dragging: false,
            last_mouse_pos_phys: (0.0, 0.0),
            precision: 4,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SwitchTab(idx) => { self.active_tab = idx; true }
            Msg::SetSingleMode(m) => { self.single_mode = m; true }
            Msg::UpdateInputA(val) => { self.input_a = val; true }
            Msg::UpdateInputB(val) => { self.input_b = val; true }
            Msg::UpdatePrecision(p) => { self.precision = p; true }

            Msg::CalculateSingle => {
                let va = self.input_a.replace(',', ".").parse::<f64>().unwrap_or(f64::NAN);
                let vb = self.input_b.replace(',', ".").parse::<f64>().unwrap_or(f64::NAN);

                if va.is_nan() || vb.is_nan() {
                    self.single_result_text = "Ошибка: Введите числа".to_string();
                    self.last_single_result = None;
                    return true;
                }

                match Self::run_calc(self.single_mode, va, vb) {
                    Ok(s) => {
                        let u = s.h - s.p * s.v * 1000.0;
                        let p = self.precision;
                        let pv = self.precision + 2;

                        // Исправлено форматирование на чистые позиционные аргументы
                        self.single_result_text = format!(
                            "Регион: {:?}\np = {:.*} МПа\nT = {:.*} К\nv = {:.*} м3/кг\nrho = {:.*} кг/м3\nh = {:.*} кДж/кг\ns = {:.*} кДж/(кг·К)\nu = {:.*} кДж/кг\ncp = {:.*} кДж/(кг·К)\nw = {:.*} м/с",
                            s.region,
                            p, s.p, p, s.t, pv, s.v, p, s.rho, p, s.h, p, s.s, p, u, p, s.cp, p, s.w
                        );
                        self.last_single_result = Some(s);
                    }
                    Err(e) => {
                        self.single_result_text = format!("Ошибка: {}", e);
                        self.last_single_result = None;
                    }
                }
                true
            }
            Msg::SaveSinglePoint => {
                let win = web_sys::window().unwrap();
                if let Some(s) = &self.last_single_result {
                    let exists = self.datasets.iter().any(|ds|
                        ds.points.len() == 1 && (ds.points[0].p - s.p).abs() < 1e-6 && (ds.points[0].t - s.t).abs() < 1e-6
                    );
                    if exists {
                        win.alert_with_message("Эта точка уже сохранена!").ok();
                    } else if let Ok(Some(name)) = win.prompt_with_message_and_default("Введите имя для точки:", &format!("Точка {:.2} МПа", s.p)) {
                        if !name.trim().is_empty() {
                            self.datasets.push(SavedData { name, points: vec![s.clone()], visible: true });
                            win.alert_with_message("Точка успешно сохранена!").ok();
                        }
                    }
                } else {
                    win.alert_with_message("Сначала выполните расчет!").ok();
                }
                true
            }

            Msg::SetBatchMode(m) => { self.batch_mode = m; true }
            Msg::UpdateBatchInput(val) => { self.batch_input = val; true }

            Msg::CalculateBatch => {
                let mut new_points = Vec::new();
                self.batch_result_table.clear();

                self.batch_result_table.push(
                    format!("{:<9} | {:<8} | {:<3} | {:<6} | {:<12} | {:<10} | {:<10} | {:<10}",
                            "p[МПа]", "T[К]", "Рег", "x", "v[м3/кг]", "rho[кг/м3]", "h[кДж/кг]", "s[кДж/кгК]")
                );
                self.batch_result_table.push("-".repeat(85));

                for line in self.batch_input.lines() {
                    let cleaned = line.replace(';', " ").replace(',', " ");
                    let parts: Vec<&str> = cleaned.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let (Ok(va), Ok(vb)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                            match Self::run_calc(self.batch_mode, va, vb) {
                                Ok(s) => {
                                    let reg = match s.region {
                                        if97_core::domain::state::Region::Region1 => "1",
                                        if97_core::domain::state::Region::Region2 => "2",
                                        if97_core::domain::state::Region::Region3 => "3",
                                        if97_core::domain::state::Region::Region4 => "4",
                                        if97_core::domain::state::Region::Region5 => "5",
                                        _ => "-",
                                    };
                                    let p = self.precision;
                                    let pv = self.precision + 2;
                                    let x_str = if self.batch_mode == 4 { format!("{:.*}", p, vb) } else { "-".into() };

                                    // Исправлено: только позиционные аргументы, чтобы избежать ошибки линтера
                                    self.batch_result_table.push(
                                        format!("{:<9.*} | {:<8.*} | {:<3} | {:<6} | {:<12.*} | {:<10.*} | {:<10.*} | {:<10.*}",
                                                p, s.p,
                                                p, s.t,
                                                reg,
                                                x_str,
                                                pv, s.v,
                                                p, s.rho,
                                                p, s.h,
                                                p, s.s
                                        )
                                    );
                                    new_points.push(s);
                                }
                                Err(_) => self.batch_result_table.push("Ошибка расчета\t-\t-\t-\t-\t-\t-\t-".to_string()),
                            }
                        }
                    }
                }
                self.datasets[0].points = new_points;
                true
            }
            Msg::SaveBatchTable => {
                let win = web_sys::window().unwrap();
                let current_points = self.datasets[0].points.clone();
                if current_points.is_empty() {
                    win.alert_with_message("Таблица пуста!").ok();
                } else if let Ok(Some(name)) = win.prompt_with_message_and_default("Введите имя набора данных:", "Набор 1") {
                    if !name.trim().is_empty() {
                        let exists = self.datasets.iter().skip(1).any(|ds|
                            ds.points.len() == current_points.len() &&
                                ds.points.first().map(|p| p.p) == current_points.first().map(|p| p.p)
                        );
                        if exists {
                            win.alert_with_message("Такой набор уже существует!").ok();
                        } else {
                            self.datasets.push(SavedData { name, points: current_points, visible: true });
                            win.alert_with_message("Набор сохранен!").ok();
                        }
                    }
                }
                true
            }

            Msg::SetPlotType(pt) => { self.plot_type = pt; true }
            Msg::ToggleDome(show) => { self.show_dome = show; true }
            Msg::ToggleSwapAxes(swap) => { self.swap_axes = swap; true }
            Msg::ToggleAutoscale(auto) => {
                self.autoscale = auto;
                if self.autoscale {
                    self.val_min = 0.0;
                    self.val_max = 100.0;
                    self.t_min = 273.15;
                    self.t_max = 1000.0;
                }
                true
            }
            Msg::UpdateLimit(idx, val) => {
                if let Ok(num) = val.replace(',', ".").parse::<f64>() {
                    match idx { 0 => self.val_min = num, 1 => self.val_max = num, 2 => self.t_min = num, 3 => self.t_max = num, _ => {} }
                }
                true
            }
            Msg::ToggleDatasetVisibility(idx, vis) => {
                if let Some(ds) = self.datasets.get_mut(idx) { ds.visible = vis; }
                true
            }

            Msg::ZoomPlot(e) => {
                e.prevent_default();
                if self.autoscale { return false; }

                let mx = e.offset_x() as f64;
                let my = e.offset_y() as f64;
                let width = 900.0;
                let height = 550.0;
                let rx = mx / width;
                let ry = my / height;

                let base_step = 0.05;
                let factor = if e.delta_y() > 0.0 { 1.0 + base_step } else { 1.0 - base_step };

                let current_x_span = self.val_max - self.val_min;
                let current_y_span = self.t_max - self.t_min;

                let new_x_span = current_x_span * factor;
                let new_y_span = current_y_span * factor;

                let x_cursor_shift = current_x_span - new_x_span;
                let y_cursor_shift = current_y_span - new_y_span;

                if !self.swap_axes {
                    self.val_min += x_cursor_shift * rx;
                    self.val_max = self.val_min + new_x_span;
                    self.t_max -= y_cursor_shift * ry;
                    self.t_min = self.t_max - new_y_span;
                } else {
                    self.t_min += x_cursor_shift * rx;
                    self.t_max = self.t_min + new_x_span;
                    self.val_max -= y_cursor_shift * ry;
                    self.val_min = self.val_max - new_y_span;
                }
                true
            }

            Msg::PlotMouseDown(x, y) => {
                if !self.autoscale {
                    self.is_dragging = true;
                    let canvas_scale_x = x as f64 / 900.0;
                    let canvas_scale_y = y as f64 / 550.0;

                    if !self.swap_axes {
                        self.last_mouse_pos_phys = (
                            self.val_min + canvas_scale_x * (self.val_max - self.val_min),
                            self.t_max - canvas_scale_y * (self.t_max - self.t_min)
                        );
                    } else {
                        self.last_mouse_pos_phys = (
                            self.t_min + canvas_scale_x * (self.t_max - self.t_min),
                            self.val_max - canvas_scale_y * (self.val_max - self.val_min)
                        );
                    }
                }
                false
            }

            Msg::PlotMouseMove(x, y) => {
                if self.is_dragging && !self.autoscale {
                    let mx = x as f64;
                    let my = y as f64;
                    let width = 900.0;
                    let height = 550.0;

                    let rx = mx / width;
                    let ry = my / height;

                    if !self.swap_axes {
                        let span_x = self.val_max - self.val_min;
                        let span_y = self.t_max - self.t_min;

                        self.val_min = self.last_mouse_pos_phys.0 - rx * span_x;
                        self.val_max = self.val_min + span_x;

                        self.t_max = self.last_mouse_pos_phys.1 + ry * span_y;
                        self.t_min = self.t_max - span_y;
                    } else {
                        let span_x = self.t_max - self.t_min;
                        let span_y = self.val_max - self.val_min;

                        self.t_min = self.last_mouse_pos_phys.0 - rx * span_x;
                        self.t_max = self.t_min + span_x;

                        self.val_max = self.last_mouse_pos_phys.1 + ry * span_y;
                        self.val_min = self.val_max - span_y;
                    }
                    return true;
                }
                false
            }

            Msg::PlotMouseUp => {
                if self.is_dragging { self.is_dragging = false; }
                false
            }
        }
    }

    // Исправлено: view теперь идет строго до rendered, как того требует Yew
    fn view(&self, ctx: &Context<Self>) -> Html {
        view_tabs(self, ctx)
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if self.active_tab == 2 {
            let _ = draw_chart(self, "plot_canvas");
        }
    }
}
//! Общие DTO и контракты приложения `if97_calculator`.
//!
//! Этот крейт содержит простые сериализуемые типы, которые разделяются между UI и backend
//! (FLTK/Yew/Tauri). Типы намеренно используют примитивы (`f64`, `String`), чтобы формат
//! данных оставался стабильным и не зависел от внутренних типов вычислительного ядра.
//!
//! ## Представление `f64` в JSON
//!
//! JSON не поддерживает `NaN` и `±Infinity`. Для совместимости `StateDto` кодирует такие
//! значения как строки: `"nan"`, `"inf"`, `"-inf"`. При десериализации допускается также
//! `null`, который интерпретируется как `+Infinity`.

use serde::{Deserialize, Serialize};

mod serde_f64_value {
    //! Серде-хелпер для кодирования невалидных для JSON значений `f64`.

    use serde::de::{Error as DeError, Visitor};
    use serde::{Deserializer, Serializer};
    use std::fmt;

    pub fn serialize<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if value.is_finite() {
            serializer.serialize_f64(*value)
        } else if value.is_infinite() {
            if value.is_sign_negative() {
                serializer.serialize_str("-inf")
            } else {
                serializer.serialize_str("inf")
            }
        } else {
            serializer.serialize_str("nan")
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct F64Visitor;

        impl<'de> Visitor<'de> for F64Visitor {
            type Value = f64;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("число, null или строку inf/-inf/nan")
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
                Ok(value)
            }

            fn visit_f32<E>(self, value: f32) -> Result<Self::Value, E> {
                Ok(value as f64)
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
                Ok(value as f64)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
                Ok(value as f64)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(f64::INFINITY)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(f64::INFINITY)
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                match value.trim().to_ascii_lowercase().as_str() {
                    "inf" | "+inf" | "infinity" | "+infinity" => Ok(f64::INFINITY),
                    "-inf" | "-infinity" => Ok(f64::NEG_INFINITY),
                    "nan" => Ok(f64::NAN),
                    other => other.parse::<f64>().map_err(E::custom),
                }
            }
        }

        deserializer.deserialize_any(F64Visitor)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Режим ввода параметров для расчета термодинамического состояния.
pub enum InputMode {
    /// Давление `p` (МПа) и температура `T` (K).
    Pt,
    /// Давление `p` (МПа) и энтальпия `h` (кДж/кг).
    Ph,
    /// Давление `p` (МПа) и энтропия `s` (кДж/(кг*K)).
    Ps,
    /// Давление `p` (МПа) и степень сухости `x` (0..=1).
    Px,
    /// Плотность `rho` (кг/м^3) и температура `T` (K).
    Rhot,
}

impl InputMode {
    /// Стабильное строковое представление для хранения/передачи в UI (`"pt"`, `"ph"`, ...).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pt => "pt",
            Self::Ph => "ph",
            Self::Ps => "ps",
            Self::Px => "px",
            Self::Rhot => "rhot",
        }
    }

    /// Подписи полей ввода для UI.
    pub fn labels(self) -> (&'static str, &'static str) {
        match self {
            Self::Pt => ("p (MPa)", "T (K)"),
            Self::Rhot => ("rho (kg/m^3)", "T (K)"),
            Self::Px => ("p (MPa)", "x"),
            Self::Ps => ("p (MPa)", "s (kJ/kgK)"),
            Self::Ph => ("p (MPa)", "h (kJ/kg)"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Тип термодинамической диаграммы для построения графиков.
pub enum DiagramKind {
    /// Диаграмма `T-s`.
    Ts,
    /// Диаграмма `h-s`.
    Hs,
    /// Диаграмма `p-h`.
    Ph,
    /// Диаграмма `T-v`.
    Tv,
    /// Диаграмма `p-v`.
    Pv,
    /// Диаграмма `p-T`.
    Pt,
    /// Диаграмма `p-s`.
    Ps,
    /// Диаграмма `T-h`.
    Th,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// Запрос на расчет одной точки.
pub struct SingleCalcRequest {
    /// Режим ввода.
    pub mode: InputMode,
    /// Первое значение (смысл и единицы зависят от `mode`).
    pub v1: f64,
    /// Второе значение (смысл и единицы зависят от `mode`).
    pub v2: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// Запрос на расчет таблицы, где входные значения представлены как текст.
pub struct TableCalcRequest {
    /// Режим ввода.
    pub mode: InputMode,
    /// Содержимое таблицы (строки, в каждой две колонки).
    pub input: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// Сериализуемое представление рассчитанного термодинамического состояния.
pub struct StateDto {
    #[serde(with = "serde_f64_value")]
    /// Давление `p` (МПа).
    pub p: f64,
    #[serde(with = "serde_f64_value")]
    /// Температура `T` (K).
    pub t: f64,
    #[serde(with = "serde_f64_value")]
    /// Удельный объем `v` (м^3/кг).
    pub v: f64,
    #[serde(with = "serde_f64_value")]
    /// Плотность `rho` (кг/м^3).
    pub rho: f64,
    #[serde(with = "serde_f64_value")]
    /// Энтальпия `h` (кДж/кг).
    pub h: f64,
    #[serde(with = "serde_f64_value")]
    /// Энтропия `s` (кДж/(кг*K)).
    pub s: f64,
    #[serde(with = "serde_f64_value")]
    /// Внутренняя энергия `u` (кДж/кг).
    pub u: f64,
    #[serde(with = "serde_f64_value")]
    /// Изобарная теплоемкость `cp` (кДж/(кг*K)).
    pub cp: f64,
    #[serde(with = "serde_f64_value")]
    /// Скорость звука `w` (м/с).
    pub w: f64,
    /// Строковое представление региона IF97 (например, `"Region1"`).
    pub region: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// Результат расчета одной строки из табличного ввода.
pub struct TableRowResult {
    /// Номер строки во входном тексте (1-based).
    pub line_no: usize,
    /// Рассчитанное состояние (если строка валидна и расчет успешен).
    pub state: Option<StateDto>,
    /// Текст ошибки (если парсинг или расчет не удались).
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// Параметры запроса купола насыщения для построения диаграмм.
pub struct DomeRequest {
    /// Тип диаграммы.
    pub chart_type: DiagramKind,
    /// Поменять оси местами (используется UI для удобства отображения).
    pub swap_axes: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
/// Точка для отрисовки графиков.
pub struct PlotPoint {
    /// Координата по оси X.
    pub x: f64,
    /// Координата по оси Y.
    pub y: f64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Сериализуемая запись журнала (для UI).
pub struct LogEntryDto {
    /// Источник (`"core"`, `"backend"`, `"frontend"` и т.п.).
    pub source: String,
    /// Уровень логирования (`"trace"`, `"debug"`, `"info"`, `"warn"`, `"error"`).
    pub level: String,
    /// Текст сообщения.
    pub message: String,
}

use serde::{Deserialize, Serialize};

mod serde_f64_value {
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
pub enum InputMode {
    Pt,
    Ph,
    Ps,
    Px,
    Rhot,
}

impl InputMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pt => "pt",
            Self::Ph => "ph",
            Self::Ps => "ps",
            Self::Px => "px",
            Self::Rhot => "rhot",
        }
    }

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
pub enum DiagramKind {
    Ts,
    Hs,
    Ph,
    Tv,
    Pv,
    Pt,
    Ps,
    Th,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SingleCalcRequest {
    pub mode: InputMode,
    pub v1: f64,
    pub v2: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TableCalcRequest {
    pub mode: InputMode,
    pub input: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StateDto {
    #[serde(with = "serde_f64_value")]
    pub p: f64,
    #[serde(with = "serde_f64_value")]
    pub t: f64,
    #[serde(with = "serde_f64_value")]
    pub v: f64,
    #[serde(with = "serde_f64_value")]
    pub rho: f64,
    #[serde(with = "serde_f64_value")]
    pub h: f64,
    #[serde(with = "serde_f64_value")]
    pub s: f64,
    #[serde(with = "serde_f64_value")]
    pub u: f64,
    #[serde(with = "serde_f64_value")]
    pub cp: f64,
    #[serde(with = "serde_f64_value")]
    pub w: f64,
    pub region: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TableRowResult {
    pub line_no: usize,
    pub state: Option<StateDto>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DomeRequest {
    pub chart_type: DiagramKind,
    pub swap_axes: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlotPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogEntryDto {
    pub source: String,
    pub level: String,
    pub message: String,
}

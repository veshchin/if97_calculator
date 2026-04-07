// File: src/domain/errors.rs

//! Модуль ошибок библиотеки.
//!
//! Предоставляет перечисление `If97Error` для типизации всех возможных
//! сбоев при вычислении теплофизических свойств.

use serde::{Deserialize, Serialize};

/// Ошибки, которые могут возникнуть при использовании алгоритмов IAPWS-IF97.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum If97Error {
    /// Переданные параметры выходят за границы применимости стандарта или конкретного региона.
    OutOfBounds(String),
    /// Вызванный метод расчета не реализован или физически не применим для данного региона.
    NotImplemented(String),
    /// Итерационный решатель (например, метод Ньютона-Рафсона) не смог достичь сходимости
    /// за отведенное число итераций.
    ConvergenceError(String),
    /// Ошибка при работе вблизи фазовых границ (например, попытка рассчитать однофазные свойства
    /// прямо на линии насыщения, где требуется степень сухости).
    PhaseBoundaryError(String),
    /// Некорректные входные данные (например, плотность <= 0 или степень сухости пара x < 0 или x > 1).
    InvalidInput(String),
}

impl std::fmt::Display for If97Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfBounds(msg) => write!(f, "Out of bounds: {}", msg),
            Self::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            Self::ConvergenceError(msg) => write!(f, "Convergence error: {}", msg),
            Self::PhaseBoundaryError(msg) => write!(f, "Phase boundary error: {}", msg),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}
impl std::error::Error for If97Error {}

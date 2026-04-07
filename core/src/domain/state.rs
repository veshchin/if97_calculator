// File: src/domain/state.rs

use crate::domain::units::*;
use serde::{Deserialize, Serialize};

/// Обозначение региона согласно стандарту IAPWS-IF97.
///
/// Регионы описывают различные фазовые состояния воды и пара, а также
/// определяют наборы уравнений, применяемых для расчета.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Region {
    /// Точка выходит за границы применимости стандарта IAPWS-IF97.
    #[default]
    OutOfBounds,
    /// Регион 1: Жидкая вода (субкритическая зона).
    Region1,
    /// Регион 2: Перегретый пар (субкритическая и надкритическая зоны).
    Region2,
    /// Регион 3: Околокритическая зона (жидкость и пар высокой плотности).
    Region3,
    /// Регион 4: Линия насыщения (двухфазная смесь жидкости и пара).
    Region4,
    /// Регион 5: Высокотемпературный пар (до 2273.15 K при давлениях до 50 МПа).
    Region5,
}

/// Структура, содержащая рассчитанные теплофизические свойства воды или пара.
///
/// Все поля используют строгие типы из модуля `units`, что предотвращает
/// ошибки с единицами измерения при использовании результатов.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct WaterState {
    /// Абсолютное давление (МПа).
    pub p: MegaPascal,
    /// Температура (К).
    pub t: Kelvin,
    /// Удельный объем (м³/кг).
    pub v: CubicMeterPerKilogram,
    /// Плотность (кг/м³).
    pub rho: KilogramPerCubicMeter,
    /// Удельная энтальпия (кДж/кг).
    pub h: KiloJoulePerKilogram,
    /// Удельная энтропия (кДж/(кг·К)).
    pub s: KiloJoulePerKilogramKelvin,
    /// Изобарная удельная теплоемкость (кДж/(кг·К)). Для Region 4 равно `NaN`.
    pub cp: KiloJoulePerKilogramKelvin,
    /// Скорость звука (м/с). Для Region 4 равно `NaN`.
    pub w: MeterPerSecond,
    /// Регион по стандарту IAPWS-IF97, в котором находилась точка.
    pub region: Region,
    /// Внутренняя энергия (кДж/кг), рассчитываемая как `u = h - p * v`.
    pub u: KiloJoulePerKilogram,
}

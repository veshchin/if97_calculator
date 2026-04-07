// File: src/calculator.rs

use crate::domain::errors::If97Error;
use crate::domain::state::WaterState;
use crate::domain::units::*;
use crate::engine::{CalculationEngine, CalculationRequest};
use tracing::{debug, instrument};

/// Варианты входных параметров для универсальной точки входа `calculate`.
pub enum InputParams {
    /// Абсолютное давление (МПа) и температура (К).
    PT(MegaPascal, Kelvin),
    /// Плотность (кг/м³) и температура (К).
    RhoT(KilogramPerCubicMeter, Kelvin),
}

/// Главный вычислительный фасад библиотеки.
///
/// Предоставляет публичные методы для расчета теплофизических свойств воды и пара
/// по различным входным парам параметров (p-T, p-h, p-s, p-x, rho-T).
/// Автоматически маршрутизирует запрос в нужный регион стандарта IAPWS-IF97.
pub struct If97;

impl If97 {
    /// Универсальная точка входа для расчета свойств воды и пара.
    ///
    /// Позволяет передавать параметры через перечисление `InputParams`,
    /// что удобно для создания обобщенных интерфейсов поверх библиотеки.
    #[instrument(level = "info", skip(input))]
    pub fn calculate(input: InputParams) -> Result<WaterState, If97Error> {
        match input {
            InputParams::PT(p, t) => {
                debug!("Вызван универсальный метод calculate с параметрами PT");
                CalculationEngine::calculate(CalculationRequest::PT(p, t))
            }
            InputParams::RhoT(rho, t) => {
                debug!("Вызван универсальный метод calculate с параметрами RhoT");
                CalculationEngine::calculate(CalculationRequest::RhoT(rho, t))
            }
        }
    }

    /// Расчет свойств по заданному давлению и температуре.
    ///
    /// # Arguments
    /// * `p` - Абсолютное давление в мегапаскалях (МПа).
    /// * `t` - Температура в кельвинах (К).
    ///
    /// # Returns
    /// Возвращает структуру `WaterState` со всеми рассчитанными теплофизическими свойствами.
    ///
    /// # Errors
    /// * `If97Error::OutOfBounds` - если параметры выходят за границы стандарта IAPWS-IF97.
    /// * `If97Error::PhaseBoundaryError` - если точка лежит точно на линии насыщения (Region 4). Для таких точек следует использовать метод `px`.
    #[instrument(level = "info")]
    pub fn pt(p: MegaPascal, t: Kelvin) -> Result<WaterState, If97Error> {
        CalculationEngine::calculate(CalculationRequest::PT(p, t))
    }

    /// Обратный расчет свойств по заданному давлению и удельной энтальпии.
    ///
    /// Метод определяет текущее фазовое состояние (жидкость, двухфазная смесь или пар) путем
    /// сравнения заданной энтальпии с энтальпиями на границах насыщения, после чего
    /// вызывает решатель соответствующего региона.
    ///
    /// # Arguments
    /// * `p` - Абсолютное давление в мегапаскалях (МПа).
    /// * `h` - Удельная энтальпия в кДж/кг.
    ///
    /// # Returns
    /// Возвращает структуру `WaterState`.
    ///
    /// # Errors
    /// Возвращает `If97Error::OutOfBounds`, если точка вне стандарта, или ошибку сходимости, если итерационный решатель не нашел решение.
    #[instrument(level = "debug")]
    pub fn ph(p: MegaPascal, h: KiloJoulePerKilogram) -> Result<WaterState, If97Error> {
        CalculationEngine::calculate(CalculationRequest::PH(p, h))
    }

    /// Обратный расчет свойств по заданному давлению и удельной энтропии.
    ///
    /// Работает аналогично методу `ph`, маршрутизируя запрос на основе сравнения
    /// текущей энтропии со значениями энтропии на граничных линиях стандарта.
    ///
    /// # Arguments
    /// * `p` - Абсолютное давление в мегапаскалях (МПа).
    /// * `s` - Удельная энтропия в кДж/(кг·К).
    ///
    /// # Returns
    /// Возвращает структуру `WaterState`.
    ///
    /// # Errors
    /// Возвращает ошибки выхода за границы или ошибки сходимости итерационных решателей.
    #[instrument(level = "debug")]
    pub fn ps(p: MegaPascal, s: KiloJoulePerKilogramKelvin) -> Result<WaterState, If97Error> {
        CalculationEngine::calculate(CalculationRequest::PS(p, s))
    }

    /// Расчет свойств влажного пара (двухфазная область, Region 4).
    ///
    /// # Arguments
    /// * `p` - Абсолютное давление в мегапаскалях (МПа).
    /// * `x` - Степень сухости пара (массовая доля пара в смеси) от 0.0 до 1.0.
    ///
    /// # Returns
    /// Свойства двухфазной смеси, усредненные по правилу аддитивности.
    /// Скорость звука и теплоемкость для влажного пара не рассчитываются (возвращается NaN).
    #[instrument(level = "info")]
    pub fn px(p: MegaPascal, x: VaporFraction) -> Result<WaterState, If97Error> {
        debug!(
            p = p.inner(),
            x = x.inner(),
            "Запрос расчета двухфазной области px"
        );
        CalculationEngine::calculate(CalculationRequest::PX(p, x))
    }

    /// Расчет свойств по заданной плотности и температуре.
    ///
    /// Использует метод секущих для нахождения давления, соответствующего целевой плотности.
    /// Является универсальным входом для задач гидродинамики, где плотность известна изначально.
    ///
    /// # Arguments
    /// * `rho` - Плотность в кг/м³.
    /// * `t` - Температура в кельвинах (К).
    #[instrument(level = "info")]
    pub fn rhot(rho: KilogramPerCubicMeter, t: Kelvin) -> Result<WaterState, If97Error> {
        CalculationEngine::calculate(CalculationRequest::RhoT(rho, t))
    }

    /// Расчет свойств метастабильного переохлажденного пара (расширение Region 2).
    ///
    /// Применяется для состояний пара, охлажденного ниже температуры насыщения без конденсации
    /// (например, в паровых турбинах при быстром расширении).
    ///
    /// # Arguments
    /// * `p` - Абсолютное давление в мегапаскалях (МПа).
    /// * `t` - Температура в кельвинах (К).
    ///
    /// # Errors
    /// Возвращает ошибку, если давление превышает 10 МПа (ограничение стандарта для метастабильной зоны).
    #[instrument(level = "info")]
    pub fn metastable_pt(p: MegaPascal, t: Kelvin) -> Result<WaterState, If97Error> {
        debug!(
            p = p.inner(),
            t = t.inner(),
            "Запрос расчета свойств метастабильного пара"
        );
        CalculationEngine::calculate(CalculationRequest::MetastablePT(p, t))
    }
}

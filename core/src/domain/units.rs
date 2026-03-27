// File: src/domain/units.rs

use serde::{Serialize, Deserialize};

macro_rules! define_unit {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
        pub struct $name(pub f64);

        impl $name {
            #[inline(always)]
            pub fn inner(self) -> f64 {
                self.0
            }
        }

        impl From<f64> for $name {
            #[inline(always)]
            fn from(val: f64) -> Self {
                Self(val)
            }
        }
    };
}

define_unit!(MegaPascal, "Давление в Мегапаскалях (MPa)");
define_unit!(Kelvin, "Температура в Кельвинах (K)");
define_unit!(KiloJoulePerKilogram, "Удельная энтальпия (kJ/kg)");
define_unit!(KiloJoulePerKilogramKelvin, "Удельная энтропия и теплоемкость (kJ/(kg·K))");
define_unit!(KilogramPerCubicMeter, "Плотность (kg/m^3)");
define_unit!(CubicMeterPerKilogram, "Удельный объем (m^3/kg)");
define_unit!(MeterPerSecond, "Скорость звука (m/s)");
define_unit!(VaporFraction, "Степень сухости пара (0.0 - 1.0)");
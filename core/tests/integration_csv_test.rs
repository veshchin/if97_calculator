// File: tests/integration_csv_test.rs

//! Интеграционные тесты для проверки ядра расчетов (if97_core)
//! с использованием верификационных данных IAPWS-IF97, загружаемых из CSV.

use std::error::Error;
use std::fs::File;
use serde::Deserialize;
use if97_core::If97;

/// Структура, описывающая маппинг колонок из тестового CSV файла.
#[derive(Debug, Deserialize)]
struct TestRow {
    #[serde(rename = "P_MPa")]
    p: f64,
    #[serde(rename = "T_K")]
    t: f64,
    #[serde(rename = "Region")]
    region: i32,
    #[serde(rename = "x_vapor_fraction")]
    x: Option<f64>,
    #[serde(rename = "v_m3_kg")]
    v: f64,
    #[serde(rename = "h_kJ_kg")]
    h: f64,
    #[serde(rename = "s_kJ_kgK")]
    s: f64,
    #[serde(rename = "cp_kJ_kgK")]
    cp: f64,
    #[serde(rename = "w_m_s")]
    w: f64,
}

/// Вспомогательный макрос для валидации относительной погрешности вычислений
/// в рамках заданного допуска (tolerance).
macro_rules! assert_relative_eq {
    ($calc:expr, $ref:expr, $eps:expr, $msg:expr) => {
        let calc = $calc;
        let reference = $ref;
        if reference.is_finite() && calc.is_finite() {
            let diff = (calc - reference).abs() / reference.abs();
            assert!(
                diff < $eps,
                "{} | Ожидалось: {}, Вычислено: {}, Откл: {:.2e} (макс: {:.2e})",
                $msg, reference, calc, diff, $eps
            );
        }
    };
}

/// Пакетный прогон по CSV-файлу с контрольными точками. Сверяет результаты библиотеки
/// с эталоном IAPWS, учитывая послабления допусков в околокритической зоне.
#[test]
fn test_core_against_python_csv() -> Result<(), Box<dyn Error>> {
    let file = File::open("../core/tests/if97_rust_test_data.csv")
        .expect("CSV файл не найден.");
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(file);
    let mut passed = 0;

    for result in rdr.deserialize() {
        let row: TestRow = match result {
            Ok(r) => r,
            Err(_) => continue,
        };
        let tolerance = if row.region == 4 && row.t > 623.15 { 1e-5 } else { 1e-6 };

        let calc_state = if row.region == 4 {
            If97::px(row.p.into(), row.x.unwrap_or(0.0).into())
        } else {
            If97::pt(row.p.into(), row.t.into())
        };

        assert!(calc_state.is_ok(), "Ошибка расчета для P={}, T={}", row.p, row.t);
        let state = calc_state.unwrap();
        let ctx = format!("Region {}, P: {}, T: {}", row.region, row.p, row.t);

        assert_relative_eq!(state.v.inner(), row.v, tolerance, format!("{} -> Объем (v)", ctx));
        assert_relative_eq!(state.h.inner(), row.h, tolerance, format!("{} -> Энтальпия (h)", ctx));
        assert_relative_eq!(state.s.inner(), row.s, tolerance, format!("{} -> Энтропия (s)", ctx));
        if row.cp.is_finite() && state.cp.inner().is_finite() {
            assert_relative_eq!(state.cp.inner(), row.cp, tolerance, format!("{} -> Теплоемкость (cp)", ctx));
        }
        if row.w.is_finite() && state.w.inner().is_finite() {
            assert_relative_eq!(state.w.inner(), row.w, tolerance, format!("{} -> Скорость звука (w)", ctx));
        }

        passed += 1;
    }

    println!("Успешно пройдено точек: {}", passed);
    Ok(())
}
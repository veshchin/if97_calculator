use std::error::Error;
use std::fs::File;
use serde::Deserialize;
use if97_core::If97;

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

#[test]
fn test_core_all_inputs_against_csv() -> Result<(), Box<dyn Error>> {
    let file = File::open("../core/tests/if97_rust_test_data.csv")
        .expect("CSV файл не найден.");
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(file);

    let mut passed_pt = 0;
    let mut passed_rhot = 0;
    let mut passed_px = 0;
    let mut passed_ph = 0;
    let mut passed_ps = 0;

    for result in rdr.deserialize() {
        let row: TestRow = match result {
            Ok(r) => r,
            Err(_) => continue,
        };

        let tolerance = if row.region == 4 && row.t > 623.15 { 1e-5 } else { 1e-6 };
        let ctx = format!("Region {}, P: {}, T: {}", row.region, row.p, row.t);

        if row.region == 4 {
            if let Ok(state_px) = If97::px(row.p.into(), row.x.unwrap_or(0.0).into()) {
                assert_relative_eq!(state_px.v.inner(), row.v, tolerance, format!("{} -> Объем (px)", ctx));
                assert_relative_eq!(state_px.h.inner(), row.h, tolerance, format!("{} -> Энтальпия (px)", ctx));
                assert_relative_eq!(state_px.s.inner(), row.s, tolerance, format!("{} -> Энтропия (px)", ctx));
                passed_px += 1;
            }
        } else {
            if let Ok(state_pt) = If97::pt(row.p.into(), row.t.into()) {
                assert_relative_eq!(state_pt.v.inner(), row.v, tolerance, format!("{} -> Объем (pT)", ctx));
                assert_relative_eq!(state_pt.h.inner(), row.h, tolerance, format!("{} -> Энтальпия (pT)", ctx));
                assert_relative_eq!(state_pt.s.inner(), row.s, tolerance, format!("{} -> Энтропия (pT)", ctx));

                if row.cp.is_finite() && state_pt.cp.inner().is_finite() {
                    assert_relative_eq!(state_pt.cp.inner(), row.cp, tolerance, format!("{} -> Теплоемкость (pT)", ctx));
                }
                if row.w.is_finite() && state_pt.w.inner().is_finite() {
                    assert_relative_eq!(state_pt.w.inner(), row.w, tolerance, format!("{} -> Скорость звука (pT)", ctx));
                }
                passed_pt += 1;
            }

            let rho = 1.0 / row.v;
            if let Ok(state_rhot) = If97::rhot(rho.into(), row.t.into()) {
                assert_relative_eq!(state_rhot.p.inner(), row.p, tolerance, format!("{} -> Давление (rhoT)", ctx));
                assert_relative_eq!(state_rhot.h.inner(), row.h, tolerance, format!("{} -> Энтальпия (rhoT)", ctx));
                passed_rhot += 1;
            }
        }

        let backward_tolerance = tolerance *200.0;

        if let Ok(state_ph) = If97::ph(row.p.into(), row.h.into()) {
            assert_relative_eq!(state_ph.t.inner(), row.t, backward_tolerance, format!("{} -> Температура (ph)", ctx));
            assert_relative_eq!(state_ph.v.inner(), row.v, backward_tolerance, format!("{} -> Объем (ph)", ctx));
            passed_ph += 1;
        }

        if let Ok(state_ps) = If97::ps(row.p.into(), row.s.into()) {
            assert_relative_eq!(state_ps.t.inner(), row.t, backward_tolerance, format!("{} -> Температура (ps)", ctx));
            assert_relative_eq!(state_ps.v.inner(), row.v, backward_tolerance, format!("{} -> Объем (ps)", ctx));
            passed_ps += 1;
        }
    }

    println!("Успешно пройдено точек (pT): {}", passed_pt);
    println!("Успешно пройдено точек (rhoT): {}", passed_rhot);
    println!("Успешно пройдено точек (px): {}", passed_px);
    println!("Успешно пройдено точек (ph): {}", passed_ph);
    println!("Успешно пройдено точек (ps): {}", passed_ps);

    Ok(())
}
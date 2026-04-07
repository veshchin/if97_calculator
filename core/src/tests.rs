// File: src/tests.rs

//! Набор unit-тестов для проверки корректности математических моделей
//! и фасадного API библиотеки по контрольным точкам стандарта IAPWS-IF97.

#[cfg(test)]
mod tests {
    // Внутри src/ мы используем пути через crate:: или super::
    use crate::If97;
    use crate::Region;
    use crate::models::boundaries::{boundary_2bc_enthalpy, determine_region};
    use crate::models::region_1::Region1;
    use crate::models::region_2::Region2;
    use crate::models::region_2_meta::Region2Meta;
    use crate::models::region_3::Region3;
    use crate::models::region_4::{saturation_pressure, saturation_temperature};
    use crate::models::region_5::Region5;
    use crate::models::traits::WaterRegionModel;

    /// Макрос для проверки равенства чисел с плавающей точкой с заданным допуском (абсолютная погрешность).
    macro_rules! assert_approx_eq {
        ($a:expr, $b:expr, $eps:expr) => {
            let (a, b) = ($a as f64, $b as f64);
            let diff = (a - b).abs();
            assert!(
                diff < $eps,
                "Assertion failed: `(left !== right)` (left: `{}`, right: `{}`, diff: `{}`)",
                a,
                b,
                diff
            );
        };
    }

    // ==========================================
    // ТЕСТЫ ОПРЕДЕЛЕНИЯ ГРАНИЦ И РЕГИОНОВ
    // ==========================================

    #[test]
    fn test_determine_region() {
        assert_eq!(determine_region(3.0, 300.0), Region::Region1);
        assert_eq!(determine_region(0.0035, 300.0), Region::Region2);
        assert_eq!(determine_region(40.0, 700.0), Region::Region3);
        assert_eq!(determine_region(30.0, 700.0), Region::Region2);
        assert_eq!(determine_region(30.0, 1500.0), Region::Region5);
        assert_eq!(determine_region(3.0, 2500.0), Region::OutOfBounds);

        assert_eq!(
            determine_region(16.74371590007485, 624.2019591836735),
            Region::Region4
        );
        assert_eq!(
            determine_region(18.372134407844708, 631.8333061224489),
            Region::Region4
        );
        assert_eq!(
            determine_region(20.136572088037845, 639.4646530612245),
            Region::Region4
        );

        assert_eq!(
            determine_region(22.06399999995171, 647.096),
            Region::Region4
        );
    }

    #[test]
    fn test_boundary_2bc() {
        let h_calc = boundary_2bc_enthalpy(100.0);
        assert_approx_eq!(h_calc, 3516.004323, 1e-4);
    }

    #[test]
    fn test_facade_ph_correct_regions() {
        assert_eq!(
            If97::ph(3.0.into(), 115.33.into()).unwrap().region,
            Region::Region1
        );
        assert_eq!(
            If97::ph(0.0035.into(), 2549.9.into()).unwrap().region,
            Region::Region2
        );
        assert_eq!(
            If97::ph(25.58.into(), 1863.4.into()).unwrap().region,
            Region::Region3
        );
    }

    #[test]
    fn test_facade_ps_correct_regions() {
        assert_eq!(
            If97::ps(3.0.into(), 0.392.into()).unwrap().region,
            Region::Region1
        );
        assert_eq!(
            If97::ps(0.0035.into(), 8.522.into()).unwrap().region,
            Region::Region2
        );
        assert_eq!(
            If97::ps(25.58.into(), 4.054.into()).unwrap().region,
            Region::Region3
        );
    }

    // ==========================================
    // ТЕСТЫ ПРЯМЫХ РАСЧЕТОВ (По p и T)
    // ==========================================

    #[test]
    fn test_region1_verification_table() {
        let state1 = Region1.calculate_pt(3.0, 300.0).unwrap();
        let u1 = state1.h.inner() - state1.p.inner() * state1.v.inner() * 1000.0;
        assert_approx_eq!(state1.v.inner(), 0.100215168e-2, 1e-9);
        assert_approx_eq!(state1.h.inner(), 0.115331273e3, 1e-6);
        assert_approx_eq!(u1, 0.112324818e3, 1e-6);
        assert_approx_eq!(state1.s.inner(), 0.392294792, 1e-8);
        assert_approx_eq!(state1.cp.inner(), 0.417301218e1, 1e-8);
        assert_approx_eq!(state1.w.inner(), 0.150773921e4, 1e-5);

        let state2 = Region1.calculate_pt(80.0, 300.0).unwrap();
        let u2 = state2.h.inner() - state2.p.inner() * state2.v.inner() * 1000.0;
        assert_approx_eq!(state2.v.inner(), 0.971180894e-3, 1e-9);
        assert_approx_eq!(state2.h.inner(), 0.184142828e3, 1e-6);
        assert_approx_eq!(u2, 0.106448356e3, 1e-6);
        assert_approx_eq!(state2.s.inner(), 0.368563852, 1e-8);
        assert_approx_eq!(state2.cp.inner(), 0.401008987e1, 1e-8);
        assert_approx_eq!(state2.w.inner(), 0.163469054e4, 1e-5);

        let state3 = Region1.calculate_pt(3.0, 500.0).unwrap();
        let u3 = state3.h.inner() - state3.p.inner() * state3.v.inner() * 1000.0;
        assert_approx_eq!(state3.v.inner(), 0.120241800e-2, 1e-9);
        assert_approx_eq!(state3.h.inner(), 0.975542239e3, 1e-6);
        assert_approx_eq!(u3, 0.971934985e3, 1e-6);
        assert_approx_eq!(state3.s.inner(), 0.258041912e1, 1e-8);
        assert_approx_eq!(state3.cp.inner(), 0.465580682e1, 1e-8);
        assert_approx_eq!(state3.w.inner(), 0.124071337e4, 1e-5);
    }

    #[test]
    fn test_region2_verification_table() {
        let state1 = Region2.calculate_pt(0.0035, 300.0).unwrap();
        assert_approx_eq!(state1.v.inner(), 39.4913866, 1e-7);
        assert_approx_eq!(state1.h.inner(), 2549.91145, 1e-5);

        let state2 = Region2.calculate_pt(0.0035, 700.0).unwrap();
        assert_approx_eq!(state2.v.inner(), 92.3015898, 1e-7);
        assert_approx_eq!(state2.h.inner(), 3335.68375, 1e-5);

        let state3 = Region2.calculate_pt(30.0, 700.0).unwrap();
        assert_approx_eq!(state3.v.inner(), 0.00542946619, 1e-9);
        assert_approx_eq!(state3.h.inner(), 2631.49474, 1e-5);
        assert_approx_eq!(state3.s.inner(), 5.17540298, 1e-8);
    }

    #[test]
    fn test_region3_verification_table() {
        let state1 = Region3.calculate_pt(25.5837018, 650.0).unwrap();
        let u1 = state1.h.inner() - state1.p.inner() * state1.v.inner() * 1000.0;
        assert_approx_eq!(state1.rho.inner(), 500.0, 1e-4);
        assert_approx_eq!(state1.h.inner(), 0.186343019e4, 1e-4);
        assert_approx_eq!(u1, 0.181226279e4, 1e-4);
        assert_approx_eq!(state1.s.inner(), 0.405427273e1, 1e-5);
        assert_approx_eq!(state1.cp.inner(), 0.138935717e2, 1e-4);
        assert_approx_eq!(state1.w.inner(), 0.502005554e3, 1e-3);

        let state2 = Region3.calculate_pt(22.2930643, 650.0).unwrap();
        let u2 = state2.h.inner() - state2.p.inner() * state2.v.inner() * 1000.0;
        assert_approx_eq!(state2.rho.inner(), 200.0, 1e-4);
        assert_approx_eq!(state2.h.inner(), 0.237512401e4, 1e-4);
        assert_approx_eq!(u2, 0.226365868e4, 1e-4);
        assert_approx_eq!(state2.s.inner(), 0.485438792e1, 1e-5);
        assert_approx_eq!(state2.cp.inner(), 0.446579342e2, 1e-4);
        assert_approx_eq!(state2.w.inner(), 0.383444594e3, 1e-3);
    }

    #[test]
    fn test_region5_verification_table() {
        let state1 = Region5.calculate_pt(0.5, 1500.0).unwrap();
        let u1 = state1.h.inner() - state1.p.inner() * state1.v.inner() * 1000.0;
        assert_approx_eq!(state1.v.inner(), 0.138455090e1, 1e-7);
        assert_approx_eq!(state1.h.inner(), 0.521976855e4, 1e-4);
        assert_approx_eq!(u1, 0.452749310e4, 1e-4);
        assert_approx_eq!(state1.s.inner(), 0.965408875e1, 1e-8);
        assert_approx_eq!(state1.cp.inner(), 0.261609445e1, 1e-8);
        assert_approx_eq!(state1.w.inner(), 0.917068690e3, 1e-5);

        let state2 = Region5.calculate_pt(30.0, 1500.0).unwrap();
        let u2 = state2.h.inner() - state2.p.inner() * state2.v.inner() * 1000.0;
        assert_approx_eq!(state2.v.inner(), 0.230761299e-1, 1e-8);
        assert_approx_eq!(state2.h.inner(), 0.516723514e4, 1e-4);
        assert_approx_eq!(u2, 0.447495124e4, 1e-4);
        assert_approx_eq!(state2.s.inner(), 0.772970133e1, 1e-8);
        assert_approx_eq!(state2.cp.inner(), 0.272724317e1, 1e-8);
        assert_approx_eq!(state2.w.inner(), 0.928548002e3, 1e-5);

        let state3 = Region5.calculate_pt(30.0, 2000.0).unwrap();
        let u3 = state3.h.inner() - state3.p.inner() * state3.v.inner() * 1000.0;
        assert_approx_eq!(state3.v.inner(), 0.311385219e-1, 1e-8);
        assert_approx_eq!(state3.h.inner(), 0.657122604e4, 1e-4);
        assert_approx_eq!(u3, 0.563707038e4, 1e-4);
        assert_approx_eq!(state3.s.inner(), 0.853640523e1, 1e-8);
        assert_approx_eq!(state3.cp.inner(), 0.288569882e1, 1e-8);
        assert_approx_eq!(state3.w.inner(), 0.106736948e4, 1e-4);
    }

    #[test]
    fn test_region4_saturation_lines() {
        assert_approx_eq!(saturation_pressure(300.0), 0.353658941e-2, 1e-10);
        assert_approx_eq!(saturation_pressure(500.0), 0.263889776e1, 1e-8);
        assert_approx_eq!(saturation_pressure(600.0), 0.123443146e2, 1e-7);

        let test_points = [(0.1, 372.755_919), (1.0, 453.035_632), (10.0, 584.149_488)];

        for (p, expected_t) in test_points.iter() {
            let t_calc = saturation_temperature(*p);
            assert_approx_eq!(t_calc, *expected_t, 1e-6);
        }
    }

    #[test]
    fn test_region2_metastable_official() {
        let state1 = Region2Meta.calculate_pt(1.0, 450.0).unwrap();
        let u1 = state1.h.inner() - state1.p.inner() * state1.v.inner() * 1000.0;
        assert_approx_eq!(state1.v.inner(), 0.192516540, 1e-7);
        assert_approx_eq!(state1.h.inner(), 2768.81115, 1e-4);
        assert_approx_eq!(u1, 2576.29461, 1e-4);
        assert_approx_eq!(state1.s.inner(), 6.56660377, 1e-6);
        assert_approx_eq!(state1.cp.inner(), 2.76349265, 1e-6);
        assert_approx_eq!(state1.w.inner(), 498.408101, 1e-4);

        let state2 = Region2Meta.calculate_pt(1.0, 440.0).unwrap();
        let u2 = state2.h.inner() - state2.p.inner() * state2.v.inner() * 1000.0;
        assert_approx_eq!(state2.v.inner(), 0.186212297, 1e-7);
        assert_approx_eq!(state2.h.inner(), 2740.15123, 1e-4);
        assert_approx_eq!(u2, 2553.93894, 1e-4);
        assert_approx_eq!(state2.s.inner(), 6.50218759, 1e-6);
        assert_approx_eq!(state2.cp.inner(), 2.98166443, 1e-6);
        assert_approx_eq!(state2.w.inner(), 489.363295, 1e-4);

        let state3 = Region2Meta.calculate_pt(1.5, 450.0).unwrap();
        let u3 = state3.h.inner() - state3.p.inner() * state3.v.inner() * 1000.0;
        assert_approx_eq!(state3.v.inner(), 0.121685206, 1e-7);
        assert_approx_eq!(state3.h.inner(), 2721.34539, 1e-4);
        assert_approx_eq!(u3, 2538.81758, 1e-4);
        assert_approx_eq!(state3.s.inner(), 6.29170440, 1e-6);
        assert_approx_eq!(state3.cp.inner(), 3.62795578, 1e-6);
        assert_approx_eq!(state3.w.inner(), 481.941819, 1e-4);
    }

    // ==========================================
    // ТЕСТЫ ОБРАТНЫХ РАСЧЕТОВ (По p-h и p-s)
    // ==========================================

    #[test]
    fn test_region1_backward_t_ps() {
        assert_approx_eq!(
            Region1.calculate_ps(3.0, 0.5).unwrap().t.inner(),
            307.842258,
            3e-2
        );
        assert_approx_eq!(
            Region1.calculate_ps(80.0, 0.5).unwrap().t.inner(),
            309.979785,
            3e-2
        );
        assert_approx_eq!(
            Region1.calculate_ps(80.0, 3.0).unwrap().t.inner(),
            565.899909,
            3e-2
        );
    }

    #[test]
    fn test_region2_backward_t_ph_official() {
        assert_approx_eq!(
            Region2.calculate_ph(0.001, 3000.0).unwrap().t.inner(),
            534.433241,
            3e-2
        );
        assert_approx_eq!(
            Region2.calculate_ph(3.0, 3000.0).unwrap().t.inner(),
            575.373370,
            3e-2
        );
        assert_approx_eq!(
            Region2.calculate_ph(3.0, 4000.0).unwrap().t.inner(),
            1010.77577,
            3e-2
        );

        assert_approx_eq!(
            Region2.calculate_ph(5.0, 3500.0).unwrap().t.inner(),
            801.299102,
            3e-2
        );
        assert_approx_eq!(
            Region2.calculate_ph(5.0, 4000.0).unwrap().t.inner(),
            1015.31583,
            3e-2
        );
        assert_approx_eq!(
            Region2.calculate_ph(25.0, 3500.0).unwrap().t.inner(),
            875.279054,
            3e-2
        );

        assert_approx_eq!(
            Region2.calculate_ph(40.0, 2700.0).unwrap().t.inner(),
            743.056411,
            3e-2
        );
        assert_approx_eq!(
            Region2.calculate_ph(60.0, 2700.0).unwrap().t.inner(),
            791.137067,
            3e-2
        );
        assert_approx_eq!(
            Region2.calculate_ph(60.0, 3200.0).unwrap().t.inner(),
            882.756860,
            3e-2
        );
    }

    #[test]
    fn test_region2_backward_t_ps_official() {
        assert_approx_eq!(
            Region2.calculate_ps(0.1, 7.5).unwrap().t.inner(),
            399.517097,
            3e-2
        );
        assert_approx_eq!(
            Region2.calculate_ps(0.1, 8.0).unwrap().t.inner(),
            514.127081,
            3e-2
        );
        assert_approx_eq!(
            Region2.calculate_ps(2.5, 8.0).unwrap().t.inner(),
            1039.84917,
            3e-2
        );

        assert_approx_eq!(
            Region2.calculate_ps(8.0, 6.0).unwrap().t.inner(),
            600.484040,
            3e-2
        );
        assert_approx_eq!(
            Region2.calculate_ps(8.0, 7.5).unwrap().t.inner(),
            1064.95556,
            3e-2
        );
        assert_approx_eq!(
            Region2.calculate_ps(90.0, 6.0).unwrap().t.inner(),
            1038.01126,
            3e-2
        );

        assert_approx_eq!(
            Region2.calculate_ps(20.0, 5.75).unwrap().t.inner(),
            697.992849,
            3e-2
        );
        assert_approx_eq!(
            Region2.calculate_ps(80.0, 5.25).unwrap().t.inner(),
            854.011484,
            3e-2
        );
        assert_approx_eq!(
            Region2.calculate_ps(80.0, 5.75).unwrap().t.inner(),
            949.017998,
            3e-2
        );
    }

    #[test]
    fn test_region3_backward_ph_solver() {
        let state1 = Region3.calculate_ph(25.5837018, 1863.43019).unwrap();
        assert_approx_eq!(state1.t.inner(), 650.0, 1e-4);
        assert_approx_eq!(state1.rho.inner(), 500.0, 1e-4);

        let state2 = Region3.calculate_ph(22.2930643, 2375.12401).unwrap();
        assert_approx_eq!(state2.t.inner(), 650.0, 1e-4);
        assert_approx_eq!(state2.rho.inner(), 200.0, 1e-4);

        let state3 = Region3.calculate_ph(78.3095639, 2258.68845).unwrap();
        assert_approx_eq!(state3.t.inner(), 750.0, 1e-4);
        assert_approx_eq!(state3.rho.inner(), 500.0, 1e-4);
    }

    #[test]
    fn test_region3_backward_ps_solver() {
        let state1 = Region3.calculate_ps(25.5837018, 4.05427273).unwrap();
        assert_approx_eq!(state1.t.inner(), 650.0, 1e-4);
        assert_approx_eq!(state1.rho.inner(), 500.0, 1e-4);

        let state2 = Region3.calculate_ps(22.2930643, 4.85438792).unwrap();
        assert_approx_eq!(state2.t.inner(), 650.0, 1e-4);
        assert_approx_eq!(state2.rho.inner(), 200.0, 1e-4);

        let state3 = Region3.calculate_ps(78.3095639, 4.46971906).unwrap();
        assert_approx_eq!(state3.t.inner(), 750.0, 1e-4);
        assert_approx_eq!(state3.rho.inner(), 500.0, 1e-4);
    }

    #[test]
    fn test_region5_backward_iterative() {
        assert_approx_eq!(
            Region5.calculate_ph(0.5, 5219.76855).unwrap().t.inner(),
            1500.0,
            1e-4
        );
        assert_approx_eq!(
            Region5.calculate_ph(30.0, 5167.23514).unwrap().t.inner(),
            1500.0,
            1e-4
        );
        assert_approx_eq!(
            Region5.calculate_ph(30.0, 6571.22604).unwrap().t.inner(),
            2000.0,
            1e-4
        );

        assert_approx_eq!(
            Region5.calculate_ps(0.5, 9.65408875).unwrap().t.inner(),
            1500.0,
            1e-4
        );
        assert_approx_eq!(
            Region5.calculate_ps(30.0, 7.72970133).unwrap().t.inner(),
            1500.0,
            1e-4
        );
        assert_approx_eq!(
            Region5.calculate_ps(30.0, 8.53640523).unwrap().t.inner(),
            2000.0,
            1e-4
        );
    }

    // ==========================================
    // ТЕСТЫ НЕПРЕРЫВНОСТИ РАССЧËТОВ
    // ==========================================

    #[test]
    fn test_continuity_boundary_b23() {
        let t = 750.0;
        let p_exact = crate::models::boundaries::b23_pressure(t);
        let eps = 1e-6;

        let state_below = If97::pt((p_exact - eps).into(), t.into()).unwrap();
        let state_exact = If97::pt(p_exact.into(), t.into()).unwrap();
        let state_above = If97::pt((p_exact + eps).into(), t.into()).unwrap();

        assert_eq!(state_below.region, Region::Region2);
        assert_eq!(state_above.region, Region::Region3);

        let tol = 0.2;

        assert_approx_eq!(state_below.v.inner(), state_exact.v.inner(), tol);
        assert_approx_eq!(state_above.v.inner(), state_exact.v.inner(), tol);

        assert_approx_eq!(state_below.h.inner(), state_exact.h.inner(), tol);
        assert_approx_eq!(state_above.h.inner(), state_exact.h.inner(), tol);

        assert_approx_eq!(state_below.s.inner(), state_exact.s.inner(), tol);
        assert_approx_eq!(state_above.s.inner(), state_exact.s.inner(), tol);
    }

    #[test]
    fn test_continuity_boundary_1_3() {
        let t_exact = 623.15;
        let p = 50.0;
        let eps = 1e-6;

        let state_below = If97::pt(p.into(), (t_exact - eps).into()).unwrap();
        let state_exact = If97::pt(p.into(), t_exact.into()).unwrap();
        let state_above = If97::pt(p.into(), (t_exact + eps).into()).unwrap();

        assert_eq!(state_below.region, Region::Region1);
        assert_eq!(state_above.region, Region::Region3);

        let tol = 0.2;

        assert_approx_eq!(state_below.v.inner(), state_exact.v.inner(), tol);
        assert_approx_eq!(state_above.v.inner(), state_exact.v.inner(), tol);

        assert_approx_eq!(state_below.h.inner(), state_exact.h.inner(), tol);
        assert_approx_eq!(state_above.h.inner(), state_exact.h.inner(), tol);
    }
}

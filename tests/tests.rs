#[cfg(test)]
mod tests {
    use if97_core::domain::calculator::Calculator;
    use if97_core::domain::state::Region;
    use if97_core::domain::boundaries::{determine_region, boundary_2bc_enthalpy};
    use if97_core::domain::models::region_4::{saturation_pressure, saturation_temperature};
    use if97_core::app::router::Router;

    macro_rules! assert_approx_eq {
        ($a:expr, $b:expr, $eps:expr) => {
            let (a, b) = ($a as f64, $b as f64);
            let diff = (a - b).abs();
            assert!(
                diff < $eps,
                "Assertion failed: `(left !== right)` (left: `{}`, right: `{}`, diff: `{}`)",
                a, b, diff
            );
        };
    }

    #[test]
    fn test_determine_region() {
        assert_eq!(determine_region(3.0, 300.0), Region::Region1);
        assert_eq!(determine_region(0.0035, 300.0), Region::Region2);
        assert_eq!(determine_region(40.0, 700.0), Region::Region3);
        assert_eq!(determine_region(30.0, 700.0), Region::Region2);
        assert_eq!(determine_region(30.0, 1500.0), Region::Region5);
        assert_eq!(determine_region(3.0, 2500.0), Region::OutOfBounds);
    }

    #[test]
    fn test_boundary_2bc() {
        let h_calc = boundary_2bc_enthalpy(100.0);
        assert_approx_eq!(h_calc, 3516.004323, 1e-4);
    }

    #[test]
    fn test_router_ph_correct_regions() {
        // Убеждаемся, что Router правильно раскидывает запросы по регионам
        assert_eq!(Router::calculate_ph(3.0, 115.33).unwrap().region, Region::Region1);
        assert_eq!(Router::calculate_ph(0.0035, 2549.9).unwrap().region, Region::Region2);
        assert_eq!(Router::calculate_ph(25.58, 1863.4).unwrap().region, Region::Region3);
    }

    #[test]
    fn test_router_ps_correct_regions() {
        assert_eq!(Router::calculate_ps(3.0, 0.392).unwrap().region, Region::Region1);
        assert_eq!(Router::calculate_ps(0.0035, 8.522).unwrap().region, Region::Region2);
        assert_eq!(Router::calculate_ps(25.58, 4.054).unwrap().region, Region::Region3);
    }

    // ==========================================
    // 2. ТЕСТЫ ПРЯМЫХ РАСЧЕТОВ (По p и T)
    // ==========================================
    #[test]
    fn test_region1_verification_table() {
        // Точка 1: T = 300 K, p = 3 MPa
        let state1 = Calculator::calculate_pt(3.0, 300.0).unwrap();
        let u1 = state1.h - state1.p * state1.v * 1000.0;
        assert_approx_eq!(state1.v, 0.100215168e-2, 1e-9);
        assert_approx_eq!(state1.h, 0.115331273e3, 1e-6);
        assert_approx_eq!(u1, 0.112324818e3, 1e-6);
        assert_approx_eq!(state1.s, 0.392294792, 1e-8);
        assert_approx_eq!(state1.cp, 0.417301218e1, 1e-8);
        assert_approx_eq!(state1.w, 0.150773921e4, 1e-5);

        // Точка 2: T = 300 K, p = 80 MPa
        let state2 = Calculator::calculate_pt(80.0, 300.0).unwrap();
        let u2 = state2.h - state2.p * state2.v * 1000.0;
        assert_approx_eq!(state2.v, 0.971180894e-3, 1e-9);
        assert_approx_eq!(state2.h, 0.184142828e3, 1e-6);
        assert_approx_eq!(u2, 0.106448356e3, 1e-6);
        assert_approx_eq!(state2.s, 0.368563852, 1e-8);
        assert_approx_eq!(state2.cp, 0.401008987e1, 1e-8);
        assert_approx_eq!(state2.w, 0.163469054e4, 1e-5);

        // Точка 3: T = 500 K, p = 3 MPa
        let state3 = Calculator::calculate_pt(3.0, 500.0).unwrap();
        let u3 = state3.h - state3.p * state3.v * 1000.0;
        assert_approx_eq!(state3.v, 0.120241800e-2, 1e-9);
        assert_approx_eq!(state3.h, 0.975542239e3, 1e-6);
        assert_approx_eq!(u3, 0.971934985e3, 1e-6);
        assert_approx_eq!(state3.s, 0.258041912e1, 1e-8);
        assert_approx_eq!(state3.cp, 0.465580682e1, 1e-8);
        assert_approx_eq!(state3.w, 0.124071337e4, 1e-5);
    }

    #[test]
    fn test_region2_verification_table() {

        // Point 1: 0.0035 MPa, 300 K
        let state1 = Calculator::calculate_pt(0.0035, 300.0).unwrap();
        assert_approx_eq!(state1.v, 39.4913866, 1e-7);
        assert_approx_eq!(state1.h, 2549.91145, 1e-5);

        // Point 2: 0.0035 MPa, 700 K
        let state2 = Calculator::calculate_pt(0.0035, 700.0).unwrap();
        assert_approx_eq!(state2.v, 92.3015898, 1e-7);
        assert_approx_eq!(state2.h, 3335.68375, 1e-5);

        // Point 3: Официальная точка 3 для Региона 2 — это 30.0 МПа, 700 К
        let state3 = Calculator::calculate_pt(30.0, 700.0).unwrap();
        assert_approx_eq!(state3.v, 0.00542946619, 1e-9);
        assert_approx_eq!(state3.h, 2631.49474, 1e-5);
        assert_approx_eq!(state3.s, 5.17540298, 1e-8);
    }

    #[test]
    fn test_region3_verification_table() {
        // Для Региона 3 проверяем работу публичного API.
        // Передаем p и T, проверяем, что решатель находит правильную плотность (rho) и остальные параметры.

        // Точка 1: T = 650 K, p = 25.5837018 MPa (Ожидается rho = 500)
        let state1 = Calculator::calculate_pt(25.5837018, 650.0).unwrap();
        let u1 = state1.h - state1.p * state1.v * 1000.0;
        assert_approx_eq!(state1.rho, 500.0, 1e-4);
        assert_approx_eq!(state1.h, 0.186343019e4, 1e-4);
        assert_approx_eq!(u1, 0.181226279e4, 1e-4);
        assert_approx_eq!(state1.s, 0.405427273e1, 1e-5);
        assert_approx_eq!(state1.cp, 0.138935717e2, 1e-4);
        assert_approx_eq!(state1.w, 0.502005554e3, 1e-3);

        // Точка 2: T = 650 K, p = 22.2930643 MPa (Ожидается rho = 200)
        let state2 = Calculator::calculate_pt(22.2930643, 650.0).unwrap();
        let u2 = state2.h - state2.p * state2.v * 1000.0;
        assert_approx_eq!(state2.rho, 200.0, 1e-4);
        assert_approx_eq!(state2.h, 0.237512401e4, 1e-4);
        assert_approx_eq!(u2, 0.226365868e4, 1e-4);
        assert_approx_eq!(state2.s, 0.485438792e1, 1e-5);
        assert_approx_eq!(state2.cp, 0.446579342e2, 1e-4);
        assert_approx_eq!(state2.w, 0.383444594e3, 1e-3);
    }

    #[test]
    fn test_region5_verification_table() {
        // Точка 1: T = 1500 K, p = 0.5 MPa
        let state1 = Calculator::calculate_pt(0.5, 1500.0).unwrap();
        let u1 = state1.h - state1.p * state1.v * 1000.0;
        assert_approx_eq!(state1.v, 0.138455090e1, 1e-7);
        assert_approx_eq!(state1.h, 0.521976855e4, 1e-4);
        assert_approx_eq!(u1, 0.452749310e4, 1e-4);
        assert_approx_eq!(state1.s, 0.965408875e1, 1e-8);
        assert_approx_eq!(state1.cp, 0.261609445e1, 1e-8);
        assert_approx_eq!(state1.w, 0.917068690e3, 1e-5);

        // Точка 2: T = 1500 K, p = 30 MPa
        let state2 = Calculator::calculate_pt(30.0, 1500.0).unwrap();
        let u2 = state2.h - state2.p * state2.v * 1000.0;
        assert_approx_eq!(state2.v, 0.230761299e-1, 1e-8);
        assert_approx_eq!(state2.h, 0.516723514e4, 1e-4);
        assert_approx_eq!(u2, 0.447495124e4, 1e-4);
        assert_approx_eq!(state2.s, 0.772970133e1, 1e-8);
        assert_approx_eq!(state2.cp, 0.272724317e1, 1e-8);
        assert_approx_eq!(state2.w, 0.928548002e3, 1e-5);

        // Точка 3: T = 2000 K, p = 30 MPa
        let state3 = Calculator::calculate_pt(30.0, 2000.0).unwrap();
        let u3 = state3.h - state3.p * state3.v * 1000.0;
        assert_approx_eq!(state3.v, 0.311385219e-1, 1e-8);
        assert_approx_eq!(state3.h, 0.657122604e4, 1e-4);
        assert_approx_eq!(u3, 0.563707038e4, 1e-4);
        assert_approx_eq!(state3.s, 0.853640523e1, 1e-8);
        assert_approx_eq!(state3.cp, 0.288569882e1, 1e-8);
        assert_approx_eq!(state3.w, 0.106736948e4, 1e-4);
    }

    #[test]
    fn test_region4_saturation_lines() {
        assert_approx_eq!(saturation_pressure(300.0), 0.353658941e-2, 1e-10);
        assert_approx_eq!(saturation_pressure(500.0), 0.263889776e1, 1e-8);
        assert_approx_eq!(saturation_pressure(600.0), 0.123443146e2, 1e-7);

        // Обратные точки IAPWS-IF97 (Таблица 36)
        let test_points = [
            (0.1, 372.755_919),
            (1.0, 453.035_632),
            (10.0, 584.149_488),
        ];

        for (p, expected_t) in test_points.iter() {
            let t_calc = saturation_temperature(*p);
            assert_approx_eq!(t_calc, *expected_t, 1e-6);
        }
    }

    #[test]
    fn test_region2_metastable_official() {
        // Точка 1: T = 450 K, p = 1 MPa
        let state1 = Calculator::calc_metastable(1.0, 450.0).unwrap();
        let u1 = state1.h - state1.p * state1.v * 1000.0;
        assert_approx_eq!(state1.v, 0.192516540, 1e-7);
        assert_approx_eq!(state1.h, 2768.81115, 1e-4);
        assert_approx_eq!(u1, 2576.29461, 1e-4);
        assert_approx_eq!(state1.s, 6.56660377, 1e-6);
        assert_approx_eq!(state1.cp, 2.76349265, 1e-6);
        assert_approx_eq!(state1.w, 498.408101, 1e-4);

        // Точка 2: T = 440 K, p = 1 MPa
        let state2 = Calculator::calc_metastable(1.0, 440.0).unwrap();
        let u2 = state2.h - state2.p * state2.v * 1000.0;
        assert_approx_eq!(state2.v, 0.186212297, 1e-7);
        assert_approx_eq!(state2.h, 2740.15123, 1e-4);
        assert_approx_eq!(u2, 2553.93894, 1e-4);
        assert_approx_eq!(state2.s, 6.50218759, 1e-6);
        assert_approx_eq!(state2.cp, 2.98166443, 1e-6);
        assert_approx_eq!(state2.w, 489.363295, 1e-4);

        // Точка 3: T = 450 K, p = 1.5 MPa
        let state3 = Calculator::calc_metastable(1.5, 450.0).unwrap();
        let u3 = state3.h - state3.p * state3.v * 1000.0;
        assert_approx_eq!(state3.v, 0.121685206, 1e-7);
        assert_approx_eq!(state3.h, 2721.34539, 1e-4);
        assert_approx_eq!(u3, 2538.81758, 1e-4);
        assert_approx_eq!(state3.s, 6.29170440, 1e-6);
        assert_approx_eq!(state3.cp, 3.62795578, 1e-6);
        assert_approx_eq!(state3.w, 481.941819, 1e-4);
    }
    #[test]
    fn test_region1_backward_t_ps() {
        assert_approx_eq!(Router::calculate_ps(3.0, 0.5).unwrap().t, 307.842258, 3e-2);
        assert_approx_eq!(Router::calculate_ps(80.0, 0.5).unwrap().t, 309.979785, 3e-2);
        assert_approx_eq!(Router::calculate_ps(80.0, 3.0).unwrap().t, 565.899909, 3e-2);
    }

    #[test]
    fn test_region2_backward_t_ph_official() {
        // Субрегион 2a
        assert_approx_eq!(Router::calculate_ph(0.001, 3000.0).unwrap().t, 534.433241, 3e-2);
        assert_approx_eq!(Router::calculate_ph(3.0, 3000.0).unwrap().t, 575.373370, 3e-2);
        assert_approx_eq!(Router::calculate_ph(3.0, 4000.0).unwrap().t, 1010.77577, 3e-2);

        // Субрегион 2b
        assert_approx_eq!(Router::calculate_ph(5.0, 3500.0).unwrap().t, 801.299102, 3e-2);
        assert_approx_eq!(Router::calculate_ph(5.0, 4000.0).unwrap().t, 1015.31583, 3e-2);
        assert_approx_eq!(Router::calculate_ph(25.0, 3500.0).unwrap().t, 875.279054, 3e-2);

        // Субрегион 2c
        assert_approx_eq!(Router::calculate_ph(40.0, 2700.0).unwrap().t, 743.056411, 3e-2);
        assert_approx_eq!(Router::calculate_ph(60.0, 2700.0).unwrap().t, 791.137067, 3e-2);
        assert_approx_eq!(Router::calculate_ph(60.0, 3200.0).unwrap().t, 882.756860, 3e-2);
    }

    #[test]
    fn test_region2_backward_t_ps_official() {
        // Субрегион 2a
        assert_approx_eq!(Router::calculate_ps(0.1, 7.5).unwrap().t, 399.517097, 3e-2);
        assert_approx_eq!(Router::calculate_ps(0.1, 8.0).unwrap().t, 514.127081, 3e-2);
        assert_approx_eq!(Router::calculate_ps(2.5, 8.0).unwrap().t, 1039.84917, 3e-2);

        // Субрегион 2b
        assert_approx_eq!(Router::calculate_ps(8.0, 6.0).unwrap().t, 600.484040, 3e-2);
        assert_approx_eq!(Router::calculate_ps(8.0, 7.5).unwrap().t, 1064.95556, 3e-2);
        assert_approx_eq!(Router::calculate_ps(90.0, 6.0).unwrap().t, 1038.01126, 3e-2);

        // Субрегион 2c
        assert_approx_eq!(Router::calculate_ps(20.0, 5.75).unwrap().t, 697.992849, 3e-2);
        assert_approx_eq!(Router::calculate_ps(80.0, 5.25).unwrap().t, 854.011484, 3e-2);
        assert_approx_eq!(Router::calculate_ps(80.0, 5.75).unwrap().t, 949.017998, 3e-2);
    }

    #[test]
    fn test_region3_backward_ph_solver() {
        // Ожидается: T = 650 K, rho = 500 kg/m^3
        let state1 = Router::calculate_ph(25.5837018, 1863.43019).unwrap();
        assert_approx_eq!(state1.t, 650.0, 1e-4);
        assert_approx_eq!(state1.rho, 500.0, 1e-4);

        // Ожидается: T = 650 K, rho = 200 kg/m^3
        let state2 = Router::calculate_ph(22.2930643, 2375.12401).unwrap();
        assert_approx_eq!(state2.t, 650.0, 1e-4);
        assert_approx_eq!(state2.rho, 200.0, 1e-4);

        // Ожидается: T = 750 K, rho = 500 kg/m^3
        let state3 = Router::calculate_ph(78.3095639, 2258.68845).unwrap();
        assert_approx_eq!(state3.t, 750.0, 1e-4);
        assert_approx_eq!(state3.rho, 500.0, 1e-4);
    }

    #[test]
    fn test_region3_backward_ps_solver() {
        // Ожидается: T = 650 K, rho = 500 kg/m^3
        let state1 = Router::calculate_ps(25.5837018, 4.05427273).unwrap();
        assert_approx_eq!(state1.t, 650.0, 1e-4);
        assert_approx_eq!(state1.rho, 500.0, 1e-4);

        // Ожидается: T = 650 K, rho = 200 kg/m^3
        let state2 = Router::calculate_ps(22.2930643, 4.85438792).unwrap();
        assert_approx_eq!(state2.t, 650.0, 1e-4);
        assert_approx_eq!(state2.rho, 200.0, 1e-4);

        // Ожидается: T = 750 K, rho = 500 kg/m^3
        let state3 = Router::calculate_ps(78.3095639, 4.46971906).unwrap();
        assert_approx_eq!(state3.t, 750.0, 1e-4);
        assert_approx_eq!(state3.rho, 500.0, 1e-4);
    }

    #[test]
    fn test_region5_backward_iterative() {
        assert_approx_eq!(Router::calculate_ph(0.5, 5219.76855).unwrap().t, 1500.0, 1e-4);
        assert_approx_eq!(Router::calculate_ph(30.0, 5167.23514).unwrap().t, 1500.0, 1e-4);
        assert_approx_eq!(Router::calculate_ph(30.0, 6571.22604).unwrap().t, 2000.0, 1e-4);

        assert_approx_eq!(Router::calculate_ps(0.5, 9.65408875).unwrap().t, 1500.0, 1e-4);
        assert_approx_eq!(Router::calculate_ps(30.0, 7.72970133).unwrap().t, 1500.0, 1e-4);
        assert_approx_eq!(Router::calculate_ps(30.0, 8.53640523).unwrap().t, 2000.0, 1e-4);
    }
}
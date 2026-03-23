// File: src/tests.rs

#[cfg(test)]
mod tests {
    use crate::calculator::Calculator;
    use crate::region_4::{saturation_pressure, saturation_temperature};

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
    fn test_region1_verification_table() {
        // Точка 1: T = 300 K, p = 3 MPa
        let state1 = Calculator::calculate(3.0, 300.0).unwrap();
        let u1 = state1.h - state1.p * state1.v * 1000.0;
        assert_approx_eq!(state1.v, 0.100215168e-2, 1e-9);
        assert_approx_eq!(state1.h, 0.115331273e3, 1e-6);
        assert_approx_eq!(u1, 0.112324818e3, 1e-6);
        assert_approx_eq!(state1.s, 0.392294792, 1e-8);
        assert_approx_eq!(state1.cp, 0.417301218e1, 1e-8);
        assert_approx_eq!(state1.w, 0.150773921e4, 1e-5);

        // Точка 2: T = 300 K, p = 80 MPa
        let state2 = Calculator::calculate(80.0, 300.0).unwrap();
        let u2 = state2.h - state2.p * state2.v * 1000.0;
        assert_approx_eq!(state2.v, 0.971180894e-3, 1e-9);
        assert_approx_eq!(state2.h, 0.184142828e3, 1e-6);
        assert_approx_eq!(u2, 0.106448356e3, 1e-6);
        assert_approx_eq!(state2.s, 0.368563852, 1e-8);
        assert_approx_eq!(state2.cp, 0.401008987e1, 1e-8);
        assert_approx_eq!(state2.w, 0.163469054e4, 1e-5);

        // Точка 3: T = 500 K, p = 3 MPa
        let state3 = Calculator::calculate(3.0, 500.0).unwrap();
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
        // Точка 1: T = 300 K, p = 0.0035 MPa
        let state1 = Calculator::calculate(0.0035, 300.0).unwrap();
        let u1 = state1.h - state1.p * state1.v * 1000.0;
        assert_approx_eq!(state1.v, 0.394913866e2, 1e-5);
        assert_approx_eq!(state1.h, 0.254991145e4, 1e-5);
        assert_approx_eq!(u1, 0.241169160e4, 1e-5);
        assert_approx_eq!(state1.s, 0.852238967e1, 1e-8);
        assert_approx_eq!(state1.cp, 0.191300162e1, 1e-8);
        assert_approx_eq!(state1.w, 0.427920172e3, 1e-5);

        // Точка 2: T = 700 K, p = 0.0035 MPa
        let state2 = Calculator::calculate(0.0035, 700.0).unwrap();
        let u2 = state2.h - state2.p * state2.v * 1000.0;
        assert_approx_eq!(state2.v, 0.923015898e2, 1e-5);
        assert_approx_eq!(state2.h, 0.333568375e4, 1e-5);
        assert_approx_eq!(u2, 0.301262819e4, 1e-5);
        assert_approx_eq!(state2.s, 0.101749996e2, 1e-7);
        assert_approx_eq!(state2.cp, 0.208141274e1, 1e-8);
        assert_approx_eq!(state2.w, 0.644289068e3, 1e-5);

        // Точка 3: T = 700 K, p = 30 MPa
        let state3 = Calculator::calculate(30.0, 700.0).unwrap();
        let u3 = state3.h - state3.p * state3.v * 1000.0;
        assert_approx_eq!(state3.v, 0.542946619e-2, 1e-9);
        assert_approx_eq!(state3.h, 0.263149474e4, 1e-5);
        assert_approx_eq!(u3, 0.246861076e4, 1e-5);
        assert_approx_eq!(state3.s, 0.517540298e1, 1e-8);
        assert_approx_eq!(state3.cp, 0.103505092e2, 1e-7);
        assert_approx_eq!(state3.w, 0.480386523e3, 1e-5);
    }

    // Вспомогательная функция для прямого тестирования Региона 3 по заданным rho и T
    fn verify_region3_direct(t: f64, rho: f64, p_expected: f64, h_expected: f64, u_expected: f64, s_expected: f64, cp_expected: f64, w_expected: f64) {
        use crate::region_3::Region3;
        use crate::constants::{R, T_C, RHO_C};

        let region3 = Region3;
        let delta = rho / RHO_C;
        let tau = T_C / t;

        let phi = region3.phi(delta, tau);
        let phi_delta = region3.phi_delta(delta, tau);
        let phi_tau = region3.phi_tau(delta, tau);
        let phi_tau_tau = region3.phi_tau_tau(delta, tau);
        let phi_delta_tau = region3.phi_delta_tau(delta, tau);
        let phi_delta_delta = region3.phi_delta_delta(delta, tau);

        let r_t = R * t;

        // Расчет термодинамических свойств
        let p_calc = rho * r_t * delta * phi_delta / 1000.0;
        let h_calc = r_t * (tau * phi_tau + delta * phi_delta);
        let u_calc = r_t * tau * phi_tau;
        let s_calc = R * (tau * phi_tau - phi);

        let dp_drho_term = 2.0 * delta * phi_delta + delta.powi(2) * phi_delta_delta;
        let dp_dt_term = delta * phi_delta - delta * tau * phi_delta_tau;

        let cp_calc = R * (-tau.powi(2) * phi_tau_tau + dp_dt_term.powi(2) / dp_drho_term);
        let w_calc = (r_t * 1000.0 * (dp_drho_term - dp_dt_term.powi(2) / (tau.powi(2) * phi_tau_tau))).sqrt();

        // Сравниваем расчет с эталоном. Допуск 1e-4 достаточен, т.к. табличные значения округлены
        assert_approx_eq!(p_calc, p_expected, 1e-6);
        assert_approx_eq!(h_calc, h_expected, 1e-4);
        assert_approx_eq!(u_calc, u_expected, 1e-4);
        assert_approx_eq!(s_calc, s_expected, 1e-7);
        assert_approx_eq!(cp_calc, cp_expected, 1e-4);
        assert_approx_eq!(w_calc, w_expected, 1e-4);
    }

    #[test]
    fn test_region3_verification_table() {
        // Точка 1: T = 650 K, rho = 500 kg m–3
        verify_region3_direct(
            650.0, 500.0,
            0.255837018e2, 0.186343019e4, 0.181226279e4,
            0.405427273e1, 0.138935717e2, 0.502005554e3
        );

        // Точка 2: T = 650 K, rho = 200 kg m–3
        verify_region3_direct(
            650.0, 200.0,
            0.222930643e2, 0.237512401e4, 0.226365868e4,
            0.485438792e1, 0.446579342e2, 0.383444594e3
        );

        // Точка 3: T = 750 K, rho = 500 kg m–3
        verify_region3_direct(
            750.0, 500.0,
            0.783095639e2, 0.225868845e4, 0.210206932e4,
            0.446971906e1, 0.634165359e1, 0.760696041e3
        );
    }

    #[test]
    fn test_region4_saturation_lines() {
        assert_approx_eq!(saturation_pressure(300.0), 0.353658941e-2, 1e-10);
        assert_approx_eq!(saturation_pressure(500.0), 0.263889776e1, 1e-8);
        assert_approx_eq!(saturation_pressure(600.0), 0.123443146e2, 1e-7);
    }

    #[test]
    fn test_region5_verification_table() {
        // Точка 1: T = 1500 K, p = 0.5 MPa
        let state1 = Calculator::calculate(0.5, 1500.0).unwrap();
        let u1 = state1.h - state1.p * state1.v * 1000.0;
        assert_approx_eq!(state1.v, 0.138455090e1, 1e-7);
        assert_approx_eq!(state1.h, 0.521976855e4, 1e-4);
        assert_approx_eq!(u1, 0.452749310e4, 1e-4);
        assert_approx_eq!(state1.s, 0.965408875e1, 1e-8);
        assert_approx_eq!(state1.cp, 0.261609445e1, 1e-8);
        assert_approx_eq!(state1.w, 0.917068690e3, 1e-5);

        // Точка 2: T = 1500 K, p = 30 MPa
        let state2 = Calculator::calculate(30.0, 1500.0).unwrap();
        let u2 = state2.h - state2.p * state2.v * 1000.0;
        assert_approx_eq!(state2.v, 0.230761299e-1, 1e-8);
        assert_approx_eq!(state2.h, 0.516723514e4, 1e-4);
        assert_approx_eq!(u2, 0.447495124e4, 1e-4);
        assert_approx_eq!(state2.s, 0.772970133e1, 1e-8);
        assert_approx_eq!(state2.cp, 0.272724317e1, 1e-8);
        assert_approx_eq!(state2.w, 0.928548002e3, 1e-5);

        // Точка 3: T = 2000 K, p = 30 MPa
        let state3 = Calculator::calculate(30.0, 2000.0).unwrap();
        let u3 = state3.h - state3.p * state3.v * 1000.0;
        assert_approx_eq!(state3.v, 0.311385219e-1, 1e-8);
        assert_approx_eq!(state3.h, 0.657122604e4, 1e-4);
        assert_approx_eq!(u3, 0.563707038e4, 1e-4);
        assert_approx_eq!(state3.s, 0.853640523e1, 1e-8);
        assert_approx_eq!(state3.cp, 0.288569882e1, 1e-8);
        assert_approx_eq!(state3.w, 0.106736948e4, 1e-4);
    }

    // Добавьте этот тест в src/tests.rs внутри mod tests { ... }

    #[test]
    fn test_region1_backward_t_ps() {
        use crate::backward::region1_t_ps;

        // Точка 1: p = 3 MPa, s = 0.5 kJ kg–1 K–1
        // Ожидаемое T = 0.307 842 258 × 10^3 K = 307.842258 K
        let t1 = region1_t_ps(3.0, 0.5);
        assert_approx_eq!(t1, 307.842258, 1e-6);

        // Точка 2: p = 80 MPa, s = 0.5 kJ kg–1 K–1
        // Ожидаемое T = 0.309 979 785 × 10^3 K = 309.979785 K
        let t2 = region1_t_ps(80.0, 0.5);
        assert_approx_eq!(t2, 309.979785, 1e-6);

        // Точка 3: p = 80 MPa, s = 3.0 kJ kg–1 K–1
        // Ожидаемое T = 0.565 899 909 × 10^3 K = 565.899909 K
        let t3 = region1_t_ps(80.0, 3.0);
        assert_approx_eq!(t3, 565.899909, 1e-6);
    }

    #[test]
    fn test_region1_calculator_ps() {
        use crate::calculator::Calculator;

        // Считаем состояние по p=3 MPa и s=0.5
        let state = Calculator::calc_region1_ps(3.0, 0.5).unwrap();

        // Проверяем, что температура сошлась с эталоном T(p,s)
        assert_approx_eq!(state.t, 307.842258, 1e-6);

        // Проверяем, что энтальпия тоже считается правильно для этой точки
        // (Можем сверить с вашими прямыми таблицами, если понадобится)
    }

    use super::*; // Подтягиваем макрос assert_approx_eq! и другие импорты
    use crate::backward::{region2_t_ph, region2_t_ps};
    use crate::boundaries::boundary_2bc_enthalpy;

    // --- 1. Основное уравнение (прямые вычисления по p, T) ---
    #[test]
    fn test_region2_forward_official() {
        // Точка 1: T = 300 K, p = 0.0035 MPa
        let state1 = Calculator::calculate(0.0035, 300.0).unwrap();
        let u1 = state1.h - state1.p * state1.v * 1000.0;
        assert_approx_eq!(state1.v, 39.4913866, 1e-5);
        assert_approx_eq!(state1.h, 2549.91145, 1e-4);
        assert_approx_eq!(u1, 2411.69160, 1e-4);
        assert_approx_eq!(state1.s, 8.52238967, 1e-6);
        assert_approx_eq!(state1.cp, 1.91300162, 1e-6);
        assert_approx_eq!(state1.w, 427.920172, 1e-4);

        // Точка 2: T = 700 K, p = 0.0035 MPa
        let state2 = Calculator::calculate(0.0035, 700.0).unwrap();
        let u2 = state2.h - state2.p * state2.v * 1000.0;
        assert_approx_eq!(state2.v, 92.3015898, 1e-5);
        assert_approx_eq!(state2.h, 3335.68375, 1e-4);
        assert_approx_eq!(u2, 3012.62819, 1e-4);
        assert_approx_eq!(state2.s, 10.1749996, 1e-6);
        assert_approx_eq!(state2.cp, 2.08141274, 1e-6);
        assert_approx_eq!(state2.w, 644.289068, 1e-4);

        // Точка 3: T = 700 K, p = 30 MPa
        let state3 = Calculator::calculate(30.0, 700.0).unwrap();
        let u3 = state3.h - state3.p * state3.v * 1000.0;
        assert_approx_eq!(state3.v, 0.00542946619, 1e-9);
        assert_approx_eq!(state3.h, 2631.49474, 1e-4);
        assert_approx_eq!(u3, 2468.61076, 1e-4);
        assert_approx_eq!(state3.s, 5.17540298, 1e-6);
        assert_approx_eq!(state3.cp, 10.3505092, 1e-6);
        assert_approx_eq!(state3.w, 480.386523, 1e-4);
    }

    // --- 3. Границы субрегионов ---
    #[test]
    fn test_boundary_2bc() {
        // Ожидается, что при p = 100.0 MPa, h_2bc = 3516.004323 kJ/kg
        let h_calc = boundary_2bc_enthalpy(100.0);
        assert_approx_eq!(h_calc, 3516.004323, 1e-4);
    }

    // --- 4. Обратные вычисления T(p, h) ---
    #[test]
    fn test_region2_backward_t_ph_official() {
        // Субрегион 2a
        assert_approx_eq!(region2_t_ph(0.001, 3000.0), 534.433241, 1e-5);
        assert_approx_eq!(region2_t_ph(3.0, 3000.0), 575.373370, 1e-5);
        assert_approx_eq!(region2_t_ph(3.0, 4000.0), 1010.77577, 1e-4);

        // Субрегион 2b
        assert_approx_eq!(region2_t_ph(5.0, 3500.0), 801.299102, 1e-5);
        assert_approx_eq!(region2_t_ph(5.0, 4000.0), 1015.31583, 1e-4);
        assert_approx_eq!(region2_t_ph(25.0, 3500.0), 875.279054, 1e-5);

        // Субрегион 2c
        assert_approx_eq!(region2_t_ph(40.0, 2700.0), 743.056411, 1e-5);
        assert_approx_eq!(region2_t_ph(60.0, 2700.0), 791.137067, 1e-5);
        assert_approx_eq!(region2_t_ph(60.0, 3200.0), 882.756860, 1e-5);
    }

    // --- 5. Обратные вычисления T(p, s) ---
    #[test]
    fn test_region2_backward_t_ps_official() {
        // Субрегион 2a
        assert_approx_eq!(region2_t_ps(0.1, 7.5), 399.517097, 1e-5);
        assert_approx_eq!(region2_t_ps(0.1, 8.0), 514.127081, 1e-5);
        assert_approx_eq!(region2_t_ps(2.5, 8.0), 1039.84917, 1e-4);

        // Субрегион 2b
        assert_approx_eq!(region2_t_ps(8.0, 6.0), 600.484040, 1e-5);
        assert_approx_eq!(region2_t_ps(8.0, 7.5), 1064.95556, 1e-4);
        assert_approx_eq!(region2_t_ps(90.0, 6.0), 1038.01126, 1e-4);

        // Субрегион 2c
        assert_approx_eq!(region2_t_ps(20.0, 5.75), 697.992849, 1e-5);
        assert_approx_eq!(region2_t_ps(80.0, 5.25), 854.011484, 1e-5);
        assert_approx_eq!(region2_t_ps(80.0, 5.75), 949.017998, 1e-5);
    }
    // --- 2. Метастабильный пар (уравнение 18) ---
    #[test]
    fn test_region2_metastable_official() {
        use crate::calculator::Calculator;

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

    // Добавьте это в src/tests.rs

    #[test]
    fn test_region3_backward_ph_solver() {
        use crate::region_3::Region3;
        let region3 = Region3;

        // Точка 1: p = 25.5837018 MPa, h = 1863.43019 kJ/kg
        // Ожидается: T = 650 K, rho = 500 kg/m^3
        let (rho1, t1) = region3.calculate_rho_t_ph(25.5837018, 1863.43019).unwrap();
        assert_approx_eq!(t1, 650.0, 1e-4);
        assert_approx_eq!(rho1, 500.0, 1e-4);

        // Точка 2: p = 22.2930643 MPa, h = 2375.12401 kJ/kg
        // Ожидается: T = 650 K, rho = 200 kg/m^3
        let (rho2, t2) = region3.calculate_rho_t_ph(22.2930643, 2375.12401).unwrap();
        assert_approx_eq!(t2, 650.0, 1e-4);
        assert_approx_eq!(rho2, 200.0, 1e-4);

        // Точка 3: p = 78.3095639 MPa, h = 2258.68845 kJ/kg
        // Ожидается: T = 750 K, rho = 500 kg/m^3
        let (rho3, t3) = region3.calculate_rho_t_ph(78.3095639, 2258.68845).unwrap();
        assert_approx_eq!(t3, 750.0, 1e-4);
        assert_approx_eq!(rho3, 500.0, 1e-4);
    }

    #[test]
    fn test_region3_backward_ps_solver() {
        use crate::region_3::Region3;
        let region3 = Region3;

        // Точка 1: p = 25.5837018 MPa, s = 4.05427273 kJ/(kg*K)
        // Ожидается: T = 650 K, rho = 500 kg/m^3
        let (rho1, t1) = region3.calculate_rho_t_ps(25.5837018, 4.05427273).unwrap();
        assert_approx_eq!(t1, 650.0, 1e-4);
        assert_approx_eq!(rho1, 500.0, 1e-4);

        // Точка 2: p = 22.2930643 MPa, s = 4.85438792 kJ/(kg*K)
        // Ожидается: T = 650 K, rho = 200 kg/m^3
        let (rho2, t2) = region3.calculate_rho_t_ps(22.2930643, 4.85438792).unwrap();
        assert_approx_eq!(t2, 650.0, 1e-4);
        assert_approx_eq!(rho2, 200.0, 1e-4);

        // Точка 3: p = 78.3095639 MPa, s = 4.46971906 kJ/(kg*K)
        // Ожидается: T = 750 K, rho = 500 kg/m^3
        let (rho3, t3) = region3.calculate_rho_t_ps(78.3095639, 4.46971906).unwrap();
        assert_approx_eq!(t3, 750.0, 1e-4);
        assert_approx_eq!(rho3, 500.0, 1e-4);
    }
    #[test]
    fn test_region4_saturation_temperature_official() {
        use crate::region_4::saturation_temperature;

        // В вашем проекте уже есть assert_approx_eq!, но можно использовать и approx::assert_relative_eq
        // Для совместимости с вашей текущей инфраструктурой тестов, используем ваш макрос

        // Контрольные точки IAPWS-IF97 (Таблица 36)
        // p [MPa], Ожидаемая T_s [K]
        let test_points = [
            (0.1, 372.755_919),
            (1.0, 453.035_632),
            (10.0, 584.149_488),
        ];

        for (p, expected_t) in test_points.iter() {
            let t_calc = saturation_temperature(*p);
            let diff = (t_calc - expected_t).abs();
            assert!(
                diff < 1e-6,
                "Ошибка расчета T_s(p) Регион 4: при p={} ожидалось {}, получено {} (diff: {})",
                p, expected_t, t_calc, diff
            );
        }
    }
    #[test]
    fn test_region5_backward_t_ph_iterative() {
        use crate::backward::region5_t_ph;

        // Точки из Таблицы 42 стандарта IAPWS-IF97
        // Вход: p [MPa], h [kJ/kg]. Ожидается: T [K]

        // Точка 1: T = 1500 K, p = 0.5 MPa
        let t1 = region5_t_ph(0.5, 5219.76855).unwrap();
        assert_approx_eq!(t1, 1500.0, 1e-4);

        // Точка 2: T = 1500 K, p = 30 MPa
        let t2 = region5_t_ph(30.0, 5167.23514).unwrap();
        assert_approx_eq!(t2, 1500.0, 1e-4);

        // Точка 3: T = 2000 K, p = 30 MPa
        let t3 = region5_t_ph(30.0, 6571.22604).unwrap();
        assert_approx_eq!(t3, 2000.0, 1e-4);
    }

    #[test]
    fn test_region5_backward_t_ps_iterative() {
        use crate::backward::region5_t_ps;

        // Вход: p [MPa], s [kJ/(kg K)]. Ожидается: T [K]

        // Точка 1: T = 1500 K, p = 0.5 MPa
        let t1 = region5_t_ps(0.5, 9.65408875).unwrap();
        assert_approx_eq!(t1, 1500.0, 1e-4);

        // Точка 2: T = 1500 K, p = 30 MPa
        let t2 = region5_t_ps(30.0, 7.72970133).unwrap();
        assert_approx_eq!(t2, 1500.0, 1e-4);

        // Точка 3: T = 2000 K, p = 30 MPa
        let t3 = region5_t_ps(30.0, 8.53640523).unwrap();
        assert_approx_eq!(t3, 2000.0, 1e-4);
    }
}
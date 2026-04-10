use if97_app_api::StateDto;

#[test]
fn state_dto_serializes_non_finite_as_strings() {
    let dto = StateDto {
        p: 1.0,
        t: 300.0,
        v: 0.001,
        rho: 1000.0,
        h: 100.0,
        s: 1.0,
        u: 50.0,
        cp: f64::INFINITY,
        w: f64::NAN,
        x: f64::NAN,
        region: "Test".to_string(),
    };

    let json = serde_json::to_value(&dto).expect("serialize");
    assert_eq!(json["cp"], "inf");
    assert_eq!(json["w"], "nan");
    assert!(json["p"].is_number());
    assert!(json["t"].is_number());
}

#[test]
fn state_dto_deserializes_non_finite_strings() {
    let payload = serde_json::json!({
        "p": 1.0,
        "t": 300.0,
        "v": 0.001,
        "rho": 1000.0,
        "h": 100.0,
        "s": 1.0,
        "u": 50.0,
        "cp": "inf",
        "w": "-inf",
        "x": "nan",
        "region": "Test"
    });

    let dto: StateDto = serde_json::from_value(payload).expect("deserialize");
    assert!(dto.cp.is_infinite() && !dto.cp.is_sign_negative());
    assert!(dto.w.is_infinite() && dto.w.is_sign_negative());
}

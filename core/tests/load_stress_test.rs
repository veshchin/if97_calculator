use if97_core::If97;
use serde::Deserialize;
use std::error::Error;
use std::hint::black_box;
use std::time::Instant;

#[derive(Debug, Deserialize, Clone)]
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
}

fn load_rows() -> Result<Vec<TestRow>, Box<dyn Error>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("if97_rust_test_data.csv");
    let file = std::fs::File::open(&path)?;
    let mut rdr = csv::ReaderBuilder::new().flexible(true).from_reader(file);
    let mut rows = Vec::new();
    for result in rdr.deserialize() {
        if let Ok(row) = result {
            rows.push(row);
        }
    }
    Ok(rows)
}

#[test]
#[ignore]
fn load_stress_core_csv() -> Result<(), Box<dyn Error>> {
    let rows = load_rows()?;
    if rows.is_empty() {
        return Err("CSV не содержит строк для нагрузки".into());
    }

    let iters = std::env::var("IF97_LOAD_ITERS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(50);

    let mut ok_calls: u64 = 0;
    let mut err_calls: u64 = 0;
    let mut checksum: u64 = 0;

    let start = Instant::now();
    for _ in 0..iters {
        for row in &rows {
            let p = row.p;
            let t = row.t;
            let h = row.h;
            let s = row.s;
            let rho = 1.0 / row.v;

            let primary = if row.region == 4 {
                If97::px(p.into(), row.x.unwrap_or(0.0).into())
            } else {
                If97::pt(p.into(), t.into())
            };

            for result in [
                primary,
                If97::rhot(rho.into(), t.into()),
                If97::ph(p.into(), h.into()),
                If97::ps(p.into(), s.into()),
            ] {
                match result {
                    Ok(state) => {
                        ok_calls += 1;
                        checksum = checksum.wrapping_add(black_box(state.p.inner().to_bits()));
                        checksum = checksum.wrapping_add(black_box(state.t.inner().to_bits()));
                        checksum = checksum.wrapping_add(black_box(state.h.inner().to_bits()));
                        checksum = checksum.wrapping_add(black_box(state.s.inner().to_bits()));
                    }
                    Err(_) => err_calls += 1,
                }
            }
        }
    }
    let elapsed = start.elapsed();

    let total_rows = (rows.len() as u64) * (iters as u64);
    let secs = elapsed.as_secs_f64().max(1e-9);
    let calls_per_sec = ((ok_calls + err_calls) as f64) / secs;
    let rows_per_sec = (total_rows as f64) / secs;

    println!(
        "load_stress_core_csv: rows={} iters={} total_rows={} elapsed={:.3}s rows/s={:.0} calls/s={:.0} ok={} err={} checksum={}",
        rows.len(),
        iters,
        total_rows,
        secs,
        rows_per_sec,
        calls_per_sec,
        ok_calls,
        err_calls,
        checksum
    );

    Ok(())
}


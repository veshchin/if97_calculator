use if97_core::If97;
use serde::Deserialize;
use std::hint::black_box;
use std::sync::Arc;
use std::thread;
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

fn parse_args() -> (usize, usize) {
    let mut iters: Option<usize> = None;
    let mut threads: Option<usize> = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--iters" => {
                if let Some(value) = args.next() {
                    iters = value.parse::<usize>().ok();
                }
            }
            "--threads" => {
                if let Some(value) = args.next() {
                    threads = value.parse::<usize>().ok();
                }
            }
            _ => {}
        }
    }

    let iters = iters
        .or_else(|| {
            std::env::var("IF97_LOAD_ITERS")
                .ok()
                .and_then(|value| value.parse::<usize>().ok())
        })
        .unwrap_or(50);

    let threads = threads
        .or_else(|| {
            std::env::var("IF97_LOAD_THREADS")
                .ok()
                .and_then(|value| value.parse::<usize>().ok())
        })
        .unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1));

    (iters.max(1), threads.max(1))
}

fn load_rows() -> Vec<TestRow> {
    let data = include_str!("../../../core/tests/if97_rust_test_data.csv");
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(data.as_bytes());

    let mut rows = Vec::new();
    for result in rdr.deserialize() {
        if let Ok(row) = result {
            rows.push(row);
        }
    }
    rows
}

fn main() {
    let (iters, threads) = parse_args();
    let rows = load_rows();
    if rows.is_empty() {
        eprintln!("loadtest: CSV не содержит строк");
        std::process::exit(2);
    }

    let rows = Arc::new(rows);
    let total_rows = rows.len() as u64 * iters as u64;
    let chunk_size = (rows.len() + threads - 1) / threads;

    let start = Instant::now();
    let mut handles = Vec::with_capacity(threads);

    for thread_idx in 0..threads {
        let rows = rows.clone();
        let start_idx = thread_idx * chunk_size;
        let end_idx = ((thread_idx + 1) * chunk_size).min(rows.len());
        if start_idx >= end_idx {
            continue;
        }

        handles.push(thread::spawn(move || {
            let mut ok_calls: u64 = 0;
            let mut err_calls: u64 = 0;
            let mut checksum: u64 = 0;

            for _ in 0..iters {
                for row in &rows[start_idx..end_idx] {
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

            (ok_calls, err_calls, checksum)
        }));
    }

    let mut ok_calls: u64 = 0;
    let mut err_calls: u64 = 0;
    let mut checksum: u64 = 0;
    for handle in handles {
        let (ok, err, sum) = handle.join().unwrap_or((0, 0, 0));
        ok_calls += ok;
        err_calls += err;
        checksum = checksum.wrapping_add(sum);
    }

    let elapsed = start.elapsed();
    let secs = elapsed.as_secs_f64().max(1e-9);
    let calls_per_sec = ((ok_calls + err_calls) as f64) / secs;
    let rows_per_sec = (total_rows as f64) / secs;

    println!(
        "if97_loadtest: rows={} iters={} threads={} total_rows={} elapsed={:.3}s rows/s={:.0} calls/s={:.0} ok={} err={} checksum={}",
        rows.len(),
        iters,
        threads,
        total_rows,
        secs,
        rows_per_sec,
        calls_per_sec,
        ok_calls,
        err_calls,
        checksum
    );
}


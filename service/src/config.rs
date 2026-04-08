//! Конфигурация gRPC-сервиса.
//!
//! Источник конфигурации: переменные окружения с префиксом `IF97_`.
//! Значения по умолчанию подобраны для типичного VPS/Kubernetes-окружения.

use std::net::SocketAddr;

#[derive(Clone, Copy, Debug)]
/// Выбор вычислительного ядра.
pub enum KernelKind {
    /// Реальные вычисления через `if97_core`.
    Core,
    /// Быстрая заглушка (для профилирования транспорта): `h = p + t`.
    Stub,
}

#[derive(Clone, Debug)]
/// Параметры запуска сервиса.
pub struct Config {
    /// Адрес gRPC-сервера (`IF97_GRPC_ADDR`, default `0.0.0.0:50051`).
    pub grpc_addr: SocketAddr,
    /// Количество потоков `tokio` runtime (`IF97_IO_THREADS`).
    pub io_threads: usize,
    /// Количество потоков CPU-пула (`IF97_CPU_THREADS`).
    ///
    /// По умолчанию значение вычисляется как число физических ядер, ограниченное cgroup quota/cpuset,
    /// чтобы корректно работать в Kubernetes без переподписки потоков.
    pub cpu_threads: usize,
    /// Лимит одновременно обрабатываемых батчей на одно gRPC-соединение (`IF97_MAX_IN_FLIGHT_PER_CONN`).
    pub max_in_flight_per_conn: usize,
    /// Глобальный лимит одновременно обрабатываемых батчей на процесс (`IF97_MAX_IN_FLIGHT_GLOBAL`).
    pub max_in_flight_global: usize,
    /// Максимальная длина батча (количество точек) (`IF97_MAX_BATCH_LEN`).
    pub max_batch_len: usize,
    /// Максимальный размер gRPC сообщения в байтах (`IF97_GRPC_MAX_MSG_BYTES`).
    pub max_msg_bytes: usize,
    /// Время drain (в секундах) после получения SIGTERM/SIGINT (`IF97_DRAIN_SECS`).
    pub drain_secs: u64,
    /// Используемое вычислительное ядро (`IF97_KERNEL`: `core|stub`).
    pub kernel: KernelKind,
}

fn parse_usize_env(key: &str) -> Option<usize> {
    std::env::var(key).ok()?.parse::<usize>().ok()
}

fn parse_addr_env(key: &str) -> Option<SocketAddr> {
    std::env::var(key).ok()?.parse::<SocketAddr>().ok()
}

fn parse_u64_env(key: &str) -> Option<u64> {
    std::env::var(key).ok()?.parse::<u64>().ok()
}

fn parse_cpuset_list(raw: &str) -> Option<usize> {
    let mut total: usize = 0;
    for part in raw.trim().split(',').map(str::trim) {
        if part.is_empty() {
            continue;
        }
        if let Some((a, b)) = part.split_once('-') {
            let start: usize = a.trim().parse().ok()?;
            let end: usize = b.trim().parse().ok()?;
            if end < start {
                return None;
            }
            total = total.saturating_add(end - start + 1);
        } else {
            let _cpu: usize = part.parse().ok()?;
            total = total.saturating_add(1);
        }
    }
    if total > 0 { Some(total) } else { None }
}

fn detect_cpuset_cpus() -> Option<usize> {
    let candidates = [
        "/sys/fs/cgroup/cpuset.cpus.effective",
        "/sys/fs/cgroup/cpuset.cpus",
        "/sys/fs/cgroup/cpuset/cpuset.cpus",
        "/sys/fs/cgroup/cpuset/cpuset.cpus.effective",
    ];
    for path in candidates {
        if let Ok(raw) = std::fs::read_to_string(path) {
            if let Some(n) = parse_cpuset_list(&raw) {
                return Some(n);
            }
        }
    }
    None
}

fn detect_cgroup_cpu_quota() -> Option<usize> {
    if let Ok(raw) = std::fs::read_to_string("/sys/fs/cgroup/cpu.max") {
        let mut it = raw.split_whitespace();
        let quota = it.next()?;
        let period = it.next()?;
        if quota == "max" {
            return None;
        }
        let quota: u64 = quota.parse().ok()?;
        let period: u64 = period.parse().ok()?;
        if quota == 0 || period == 0 {
            return None;
        }
        let cpus = ((quota + period - 1) / period) as usize;
        return Some(cpus.max(1));
    }

    let quota_raw = std::fs::read_to_string("/sys/fs/cgroup/cpu/cpu.cfs_quota_us").ok()?;
    let period_raw = std::fs::read_to_string("/sys/fs/cgroup/cpu/cpu.cfs_period_us").ok()?;

    let quota: i64 = quota_raw.trim().parse().ok()?;
    let period: i64 = period_raw.trim().parse().ok()?;
    if quota <= 0 || period <= 0 {
        return None;
    }

    let cpus = (quota + period - 1) / period;
    Some((cpus as usize).max(1))
}

fn default_cpu_threads() -> usize {
    let physical = num_cpus::get_physical().max(1);
    let quota = detect_cgroup_cpu_quota();
    let cpuset = detect_cpuset_cpus();

    let mut n = physical;
    if let Some(q) = quota {
        n = n.min(q);
    }
    if let Some(c) = cpuset {
        n = n.min(c);
    }
    n.max(1)
}

fn parse_kernel_env() -> Result<KernelKind, Box<dyn std::error::Error>> {
    let raw = std::env::var("IF97_KERNEL").unwrap_or_else(|_| "core".to_string());
    match raw.as_str() {
        "core" => Ok(KernelKind::Core),
        "stub" => Ok(KernelKind::Stub),
        _ => Err(format!("invalid IF97_KERNEL: {raw}").into()),
    }
}

impl Config {
    /// Загружает конфигурацию из переменных окружения.
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let cpu_threads = parse_usize_env("IF97_CPU_THREADS")
            .filter(|&n| n > 0)
            .unwrap_or_else(default_cpu_threads);

        let io_threads = parse_usize_env("IF97_IO_THREADS")
            .filter(|&n| n > 0)
            .unwrap_or_else(|| cpu_threads.min(2).max(1));

        let grpc_addr = parse_addr_env("IF97_GRPC_ADDR")
            .unwrap_or_else(|| SocketAddr::from(([0, 0, 0, 0], 50051)));

        let max_in_flight_per_conn = parse_usize_env("IF97_MAX_IN_FLIGHT_PER_CONN")
            .filter(|&n| n > 0)
            .unwrap_or(cpu_threads);

        let max_in_flight_global = parse_usize_env("IF97_MAX_IN_FLIGHT_GLOBAL")
            .filter(|&n| n > 0)
            .unwrap_or(cpu_threads);

        let max_batch_len = parse_usize_env("IF97_MAX_BATCH_LEN")
            .filter(|&n| n > 0)
            .unwrap_or(1_000_000);

        let max_msg_bytes = parse_usize_env("IF97_GRPC_MAX_MSG_BYTES")
            .filter(|&n| n > 0)
            .unwrap_or(32 * 1024 * 1024);

        let drain_secs = parse_u64_env("IF97_DRAIN_SECS").unwrap_or(2);

        Ok(Self {
            grpc_addr,
            io_threads,
            cpu_threads,
            max_in_flight_per_conn,
            max_in_flight_global,
            max_batch_len,
            max_msg_bytes,
            drain_secs,
            kernel: parse_kernel_env()?,
        })
    }
}

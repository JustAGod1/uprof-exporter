// src/main.rs
use prometheus::{Encoder, Gauge, Registry, TextEncoder};
use std::process::Command;
use std::fs;
use std::time::Duration;
use tokio::time;
use hyper::{
    server::Server,
    service::{make_service_fn, service_fn},
    Body, Request, Response, StatusCode,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct UProfMetrics {
    #[serde(rename = "Total Mem Bw (GB/s)")]
    total_mem_bw: Option<f64>,

    #[serde(rename = "Read Bw (GB/s)")]
    read_bw: Option<f64>,

    #[serde(rename = "Write Bw (GB/s)")]
    write_bw: Option<f64>,

    #[serde(rename = "L3 Miss %")]
    l3_miss: Option<f64>,

    #[serde(rename = "L2 Miss %")]
    l2_miss: Option<f64>,

    #[serde(rename = "L1 Miss %")]
    l1_miss: Option<f64>,
}

struct Metrics {
    registry: Registry,
    memory_bandwidth_total: Gauge,
    memory_bandwidth_read: Gauge,
    memory_bandwidth_write: Gauge,
    l3_miss_rate: Gauge,
    l2_miss_rate: Gauge,
    l1_miss_rate: Gauge,
}

impl Metrics {
    fn new() -> Self {
        let registry = Registry::new();

        let memory_bandwidth_total = Gauge::new(
            "amd_memory_bandwidth_total_gbps",
            "Total Memory Bandwidth GB/s"
        ).unwrap();

        let memory_bandwidth_read = Gauge::new(
            "amd_memory_bandwidth_read_gbps",
            "Memory Read Bandwidth GB/s"
        ).unwrap();

        let memory_bandwidth_write = Gauge::new(
            "amd_memory_bandwidth_write_gbps",
            "Memory Write Bandwidth GB/s"
        ).unwrap();

        let l3_miss_rate = Gauge::new(
            "amd_l3_cache_miss_rate_percent",
            "L3 Cache Miss Rate %"
        ).unwrap();

        let l2_miss_rate = Gauge::new(
            "amd_l2_cache_miss_rate_percent",
            "L2 Cache Miss Rate %"
        ).unwrap();

        let l1_miss_rate = Gauge::new(
            "amd_l1_cache_miss_rate_percent",
            "L1 Cache Miss Rate %"
        ).unwrap();

        registry.register(Box::new(memory_bandwidth_total.clone())).unwrap();
        registry.register(Box::new(memory_bandwidth_read.clone())).unwrap();
        registry.register(Box::new(memory_bandwidth_write.clone())).unwrap();
        registry.register(Box::new(l3_miss_rate.clone())).unwrap();
        registry.register(Box::new(l2_miss_rate.clone())).unwrap();
        registry.register(Box::new(l1_miss_rate.clone())).unwrap();

        Self {
            registry,
            memory_bandwidth_total,
            memory_bandwidth_read,
            memory_bandwidth_write,
            l3_miss_rate,
            l2_miss_rate,
            l1_miss_rate,
        }
    }

    fn update(&self, data: UProfMetrics) {
        if let Some(val) = data.total_mem_bw {
            self.memory_bandwidth_total.set(val);
        }
        if let Some(val) = data.read_bw {
            self.memory_bandwidth_read.set(val);
        }
        if let Some(val) = data.write_bw {
            self.memory_bandwidth_write.set(val);
        }
        if let Some(val) = data.l3_miss {
            self.l3_miss_rate.set(val);
        }
        if let Some(val) = data.l2_miss {
            self.l2_miss_rate.set(val);
        }
        if let Some(val) = data.l1_miss {
            self.l1_miss_rate.set(val);
        }
    }
}

async fn collect_metrics() -> Result<UProfMetrics, Box<dyn std::error::Error>> {
    let output_path = "/var/uprof/uprof_metrics.csv";

    // Запускаем AMDuProfPcm
    let output = Command::new("/opt/AMDuProf_Linux_x64_5.1.701/bin/AMDuProfPcm")
        .args(&[
            "-m", "memory,l1,l2,l3",
            "-a",
            "-d", "1",
            "-r",
            "-o", output_path,
            "--msr"
        ])
        .output()?;

    if !output.status.success() {
        return Err(format!("AMDuProfPcm failed: {}",
                           String::from_utf8_lossy(&output.stderr)).into());
    }

    // Читаем CSV
    let csv_content = fs::read_to_string(output_path)?;

    // Парсим CSV
    let mut reader = csv::Reader::from_reader(csv_content.as_bytes());

    // Берем последнюю строку с данными
    let mut last_record = None;
    for result in reader.deserialize() {
        let record: UProfMetrics = result?;
        last_record = Some(record);
    }

    // Удаляем временный файл
    let _ = fs::remove_file(output_path);

    last_record.ok_or("No data in CSV".into())
}

async fn metrics_handler(
    _req: Request<Body>,
    registry: Registry,
) -> Result<Response<Body>, hyper::Error> {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = vec![];

    encoder.encode(&metric_families, &mut buffer).unwrap();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", encoder.format_type())
        .body(Body::from(buffer))
        .unwrap())
}

#[tokio::main]
async fn main() {
    let metrics = Metrics::new();
    let registry = metrics.registry.clone();

    // Запускаем сборщик метрик в фоне
    let metrics_clone = std::sync::Arc::new(metrics);
    let collector_metrics = metrics_clone.clone();

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(10));

        loop {
            interval.tick().await;

            match collect_metrics().await {
                Ok(data) => {
                    collector_metrics.update(data);
                    println!("Metrics updated successfully");
                }
                Err(e) => {
                    eprintln!("Error collecting metrics: {}", e);
                }
            }
        }
    });

    // Запускаем HTTP сервер
    let addr = ([0, 0, 0, 0], 9100).into();

    let make_svc = make_service_fn(move |_| {
        let registry = registry.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                metrics_handler(req, registry.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("AMD uProf Exporter started on :9100");

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
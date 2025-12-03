use prometheus::{Encoder, GaugeVec, Registry, TextEncoder, Opts};
use std::process::Command;
use std::fs;
use std::time::Duration;
use tokio::time;
use hyper::{
    server::Server,
    service::{make_service_fn, service_fn},
    Body, Request, Response, StatusCode,
};

struct Metrics {
    registry: Registry,
    nodename: String,
    ic_fetch_miss_ratio: GaugeVec,
    op_cache_fetch_miss_ratio: GaugeVec,
    ic_access_pti: GaugeVec,
    ic_miss_pti: GaugeVec,
    dc_access_pti: GaugeVec,
    l2_access_pti: GaugeVec,
    l2_access_from_ic_miss_pti: GaugeVec,
    l2_access_from_dc_miss_pti: GaugeVec,
    l2_access_from_l2_hwpf_pti: GaugeVec,
    l2_miss_pti: GaugeVec,
    l2_miss_from_ic_miss_pti: GaugeVec,
    l2_miss_from_dc_miss_pti: GaugeVec,
    l2_miss_from_l2_hwpf_pti: GaugeVec,
    l2_hit_pti: GaugeVec,
    l2_hit_from_ic_miss_pti: GaugeVec,
    l2_hit_from_dc_miss_pti: GaugeVec,
    l2_hit_from_l2_hwpf_pti: GaugeVec,
    l3_access: GaugeVec,
    l3_miss: GaugeVec,
    l3_miss_percent: GaugeVec,
    l3_hit_percent: GaugeVec,
    ave_l3_miss_latency_ns: GaugeVec,
    total_mem_bw_gbps: GaugeVec,
    local_dram_read_data_bytes_gbps: GaugeVec,
    local_dram_write_data_bytes_gbps: GaugeVec,
    remote_dram_read_data_bytes_gbps: GaugeVec,
    remote_dram_write_data_bytes_gbps: GaugeVec,
    total_mem_rdbw_gbps: GaugeVec,
    total_mem_wrbw_gbps: GaugeVec,
}

fn get_host_hostname() -> String {
    // Попытка получить hostname из переменной окружения
    if let Ok(hostname) = std::env::var("HOST_HOSTNAME") {
        return hostname;
    }

    // Попытка прочитать из /host_hostname (если примонтирован файл)
    if let Ok(hostname) = fs::read_to_string("/host_hostname") {
        return hostname.trim().to_string();
    }

    // Попытка прочитать из /etc/host_hostname
    if let Ok(hostname) = fs::read_to_string("/etc/host_hostname") {
        return hostname.trim().to_string();
    }

    // Fallback: попытка получить hostname контейнера
    if let Ok(output) = Command::new("hostname").output() {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }

    "unknown".to_string()
}

impl Metrics {
    fn new() -> Self {
        let registry = Registry::new();
        let nodename = get_host_hostname();

        let ic_fetch_miss_ratio = GaugeVec::new(
            Opts::new("amd_ic_fetch_miss_ratio", "IC Fetch Miss Ratio"),
            &["nodename"]
        ).unwrap();
        let op_cache_fetch_miss_ratio = GaugeVec::new(
            Opts::new("amd_op_cache_fetch_miss_ratio", "Op Cache Fetch Miss Ratio"),
            &["nodename"]
        ).unwrap();
        let ic_access_pti = GaugeVec::new(
            Opts::new("amd_ic_access_pti", "IC Access (pti)"),
            &["nodename"]
        ).unwrap();
        let ic_miss_pti = GaugeVec::new(
            Opts::new("amd_ic_miss_pti", "IC Miss (pti)"),
            &["nodename"]
        ).unwrap();
        let dc_access_pti = GaugeVec::new(
            Opts::new("amd_dc_access_pti", "DC Access (pti)"),
            &["nodename"]
        ).unwrap();
        let l2_access_pti = GaugeVec::new(
            Opts::new("amd_l2_access_pti", "L2 Access (pti)"),
            &["nodename"]
        ).unwrap();
        let l2_access_from_ic_miss_pti = GaugeVec::new(
            Opts::new("amd_l2_access_from_ic_miss_pti", "L2 Access from IC Miss (pti)"),
            &["nodename"]
        ).unwrap();
        let l2_access_from_dc_miss_pti = GaugeVec::new(
            Opts::new("amd_l2_access_from_dc_miss_pti", "L2 Access from DC Miss (pti)"),
            &["nodename"]
        ).unwrap();
        let l2_access_from_l2_hwpf_pti = GaugeVec::new(
            Opts::new("amd_l2_access_from_l2_hwpf_pti", "L2 Access from L2 HWPF (pti)"),
            &["nodename"]
        ).unwrap();
        let l2_miss_pti = GaugeVec::new(
            Opts::new("amd_l2_miss_pti", "L2 Miss (pti)"),
            &["nodename"]
        ).unwrap();
        let l2_miss_from_ic_miss_pti = GaugeVec::new(
            Opts::new("amd_l2_miss_from_ic_miss_pti", "L2 Miss from IC Miss (pti)"),
            &["nodename"]
        ).unwrap();
        let l2_miss_from_dc_miss_pti = GaugeVec::new(
            Opts::new("amd_l2_miss_from_dc_miss_pti", "L2 Miss from DC Miss (pti)"),
            &["nodename"]
        ).unwrap();
        let l2_miss_from_l2_hwpf_pti = GaugeVec::new(
            Opts::new("amd_l2_miss_from_l2_hwpf_pti", "L2 Miss from L2 HWPF (pti)"),
            &["nodename"]
        ).unwrap();
        let l2_hit_pti = GaugeVec::new(
            Opts::new("amd_l2_hit_pti", "L2 Hit (pti)"),
            &["nodename"]
        ).unwrap();
        let l2_hit_from_ic_miss_pti = GaugeVec::new(
            Opts::new("amd_l2_hit_from_ic_miss_pti", "L2 Hit from IC Miss (pti)"),
            &["nodename"]
        ).unwrap();
        let l2_hit_from_dc_miss_pti = GaugeVec::new(
            Opts::new("amd_l2_hit_from_dc_miss_pti", "L2 Hit from DC Miss (pti)"),
            &["nodename"]
        ).unwrap();
        let l2_hit_from_l2_hwpf_pti = GaugeVec::new(
            Opts::new("amd_l2_hit_from_l2_hwpf_pti", "L2 Hit from L2 HWPF (pti)"),
            &["nodename"]
        ).unwrap();
        let l3_access = GaugeVec::new(
            Opts::new("amd_l3_access", "L3 Access"),
            &["nodename"]
        ).unwrap();
        let l3_miss = GaugeVec::new(
            Opts::new("amd_l3_miss", "L3 Miss"),
            &["nodename"]
        ).unwrap();
        let l3_miss_percent = GaugeVec::new(
            Opts::new("amd_l3_miss_percent", "L3 Miss %"),
            &["nodename"]
        ).unwrap();
        let l3_hit_percent = GaugeVec::new(
            Opts::new("amd_l3_hit_percent", "L3 Hit %"),
            &["nodename"]
        ).unwrap();
        let ave_l3_miss_latency_ns = GaugeVec::new(
            Opts::new("amd_ave_l3_miss_latency_ns", "Ave L3 Miss Latency (ns)"),
            &["nodename"]
        ).unwrap();
        let total_mem_bw_gbps = GaugeVec::new(
            Opts::new("amd_total_mem_bw_gbps", "Total Mem Bw (GB/s)"),
            &["nodename"]
        ).unwrap();
        let local_dram_read_data_bytes_gbps = GaugeVec::new(
            Opts::new("amd_local_dram_read_data_bytes_gbps", "Local DRAM Read Data Bytes(GB/s)"),
            &["nodename"]
        ).unwrap();
        let local_dram_write_data_bytes_gbps = GaugeVec::new(
            Opts::new("amd_local_dram_write_data_bytes_gbps", "Local DRAM Write Data Bytes(GB/s)"),
            &["nodename"]
        ).unwrap();
        let remote_dram_read_data_bytes_gbps = GaugeVec::new(
            Opts::new("amd_remote_dram_read_data_bytes_gbps", "Remote DRAM Read Data Bytes (GB/s)"),
            &["nodename"]
        ).unwrap();
        let remote_dram_write_data_bytes_gbps = GaugeVec::new(
            Opts::new("amd_remote_dram_write_data_bytes_gbps", "Remote DRAM Write Data Bytes (GB/s)"),
            &["nodename"]
        ).unwrap();
        let total_mem_rdbw_gbps = GaugeVec::new(
            Opts::new("amd_total_mem_rdbw_gbps", "Total Mem RdBw (GB/s)"),
            &["nodename"]
        ).unwrap();
        let total_mem_wrbw_gbps = GaugeVec::new(
            Opts::new("amd_total_mem_wrbw_gbps", "Total Mem WrBw (GB/s)"),
            &["nodename"]
        ).unwrap();

        registry.register(Box::new(ic_fetch_miss_ratio.clone())).unwrap();
        registry.register(Box::new(op_cache_fetch_miss_ratio.clone())).unwrap();
        registry.register(Box::new(ic_access_pti.clone())).unwrap();
        registry.register(Box::new(ic_miss_pti.clone())).unwrap();
        registry.register(Box::new(dc_access_pti.clone())).unwrap();
        registry.register(Box::new(l2_access_pti.clone())).unwrap();
        registry.register(Box::new(l2_access_from_ic_miss_pti.clone())).unwrap();
        registry.register(Box::new(l2_access_from_dc_miss_pti.clone())).unwrap();
        registry.register(Box::new(l2_access_from_l2_hwpf_pti.clone())).unwrap();
        registry.register(Box::new(l2_miss_pti.clone())).unwrap();
        registry.register(Box::new(l2_miss_from_ic_miss_pti.clone())).unwrap();
        registry.register(Box::new(l2_miss_from_dc_miss_pti.clone())).unwrap();
        registry.register(Box::new(l2_miss_from_l2_hwpf_pti.clone())).unwrap();
        registry.register(Box::new(l2_hit_pti.clone())).unwrap();
        registry.register(Box::new(l2_hit_from_ic_miss_pti.clone())).unwrap();
        registry.register(Box::new(l2_hit_from_dc_miss_pti.clone())).unwrap();
        registry.register(Box::new(l2_hit_from_l2_hwpf_pti.clone())).unwrap();
        registry.register(Box::new(l3_access.clone())).unwrap();
        registry.register(Box::new(l3_miss.clone())).unwrap();
        registry.register(Box::new(l3_miss_percent.clone())).unwrap();
        registry.register(Box::new(l3_hit_percent.clone())).unwrap();
        registry.register(Box::new(ave_l3_miss_latency_ns.clone())).unwrap();
        registry.register(Box::new(total_mem_bw_gbps.clone())).unwrap();
        registry.register(Box::new(local_dram_read_data_bytes_gbps.clone())).unwrap();
        registry.register(Box::new(local_dram_write_data_bytes_gbps.clone())).unwrap();
        registry.register(Box::new(remote_dram_read_data_bytes_gbps.clone())).unwrap();
        registry.register(Box::new(remote_dram_write_data_bytes_gbps.clone())).unwrap();
        registry.register(Box::new(total_mem_rdbw_gbps.clone())).unwrap();
        registry.register(Box::new(total_mem_wrbw_gbps.clone())).unwrap();

        Self {
            registry,
            nodename,
            ic_fetch_miss_ratio,
            op_cache_fetch_miss_ratio,
            ic_access_pti,
            ic_miss_pti,
            dc_access_pti,
            l2_access_pti,
            l2_access_from_ic_miss_pti,
            l2_access_from_dc_miss_pti,
            l2_access_from_l2_hwpf_pti,
            l2_miss_pti,
            l2_miss_from_ic_miss_pti,
            l2_miss_from_dc_miss_pti,
            l2_miss_from_l2_hwpf_pti,
            l2_hit_pti,
            l2_hit_from_ic_miss_pti,
            l2_hit_from_dc_miss_pti,
            l2_hit_from_l2_hwpf_pti,
            l3_access,
            l3_miss,
            l3_miss_percent,
            l3_hit_percent,
            ave_l3_miss_latency_ns,
            total_mem_bw_gbps,
            local_dram_read_data_bytes_gbps,
            local_dram_write_data_bytes_gbps,
            remote_dram_read_data_bytes_gbps,
            remote_dram_write_data_bytes_gbps,
            total_mem_rdbw_gbps,
            total_mem_wrbw_gbps,
        }
    }

    fn update(&self, values: Vec<f64>) {
        if values.len() >= 29 {
            self.ic_fetch_miss_ratio.with_label_values(&[&self.nodename]).set(values[0]);
            self.op_cache_fetch_miss_ratio.with_label_values(&[&self.nodename]).set(values[1]);
            self.ic_access_pti.with_label_values(&[&self.nodename]).set(values[2]);
            self.ic_miss_pti.with_label_values(&[&self.nodename]).set(values[3]);
            self.dc_access_pti.with_label_values(&[&self.nodename]).set(values[4]);
            self.l2_access_pti.with_label_values(&[&self.nodename]).set(values[5]);
            self.l2_access_from_ic_miss_pti.with_label_values(&[&self.nodename]).set(values[6]);
            self.l2_access_from_dc_miss_pti.with_label_values(&[&self.nodename]).set(values[7]);
            self.l2_access_from_l2_hwpf_pti.with_label_values(&[&self.nodename]).set(values[8]);
            self.l2_miss_pti.with_label_values(&[&self.nodename]).set(values[9]);
            self.l2_miss_from_ic_miss_pti.with_label_values(&[&self.nodename]).set(values[10]);
            self.l2_miss_from_dc_miss_pti.with_label_values(&[&self.nodename]).set(values[11]);
            self.l2_miss_from_l2_hwpf_pti.with_label_values(&[&self.nodename]).set(values[12]);
            self.l2_hit_pti.with_label_values(&[&self.nodename]).set(values[13]);
            self.l2_hit_from_ic_miss_pti.with_label_values(&[&self.nodename]).set(values[14]);
            self.l2_hit_from_dc_miss_pti.with_label_values(&[&self.nodename]).set(values[15]);
            self.l2_hit_from_l2_hwpf_pti.with_label_values(&[&self.nodename]).set(values[16]);
            self.l3_access.with_label_values(&[&self.nodename]).set(values[17]);
            self.l3_miss.with_label_values(&[&self.nodename]).set(values[18]);
            self.l3_miss_percent.with_label_values(&[&self.nodename]).set(values[19]);
            self.l3_hit_percent.with_label_values(&[&self.nodename]).set(values[20]);
            self.ave_l3_miss_latency_ns.with_label_values(&[&self.nodename]).set(values[21]);
            self.total_mem_bw_gbps.with_label_values(&[&self.nodename]).set(values[22]);
            self.local_dram_read_data_bytes_gbps.with_label_values(&[&self.nodename]).set(values[23]);
            self.local_dram_write_data_bytes_gbps.with_label_values(&[&self.nodename]).set(values[24]);
            self.remote_dram_read_data_bytes_gbps.with_label_values(&[&self.nodename]).set(values[25]);
            self.remote_dram_write_data_bytes_gbps.with_label_values(&[&self.nodename]).set(values[26]);
            self.total_mem_rdbw_gbps.with_label_values(&[&self.nodename]).set(values[27]);
            self.total_mem_wrbw_gbps.with_label_values(&[&self.nodename]).set(values[28]);
        }
    }
}

fn parse_uprof_output(content: &str) -> Option<Vec<f64>> {
    let lines: Vec<&str> = content.lines().collect();
    for line in lines.iter().rev() {
        if line.contains(',') && !line.contains("System") && !line.contains("METRICS") {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() > 28 {
                let mut values = Vec::new();
                for i in 0..29 {
                    if let Some(val) = parts.get(i) {
                        if let Ok(num) = val.trim().parse::<f64>() {
                            values.push(num);
                        } else {
                            values.push(0.0);
                        }
                    }
                }
                return Some(values);
            }
        }
    }
    None
}

async fn collect_metrics() -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    let output_path = "/var/uprof/uprof_metrics.csv";
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

    let content = fs::read_to_string(output_path)?;
    let _ = fs::remove_file(output_path);

    parse_uprof_output(&content).ok_or("Failed to parse output".into())
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
    println!("Using nodename: {}", metrics.nodename);

    let registry = metrics.registry.clone();
    let metrics_clone = std::sync::Arc::new(metrics);
    let collector_metrics = metrics_clone.clone();

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            match collect_metrics().await {
                Ok(values) => {
                    collector_metrics.update(values);
                }
                Err(e) => {
                    eprintln!("Error collecting metrics: {}", e);
                }
            }
        }
    });

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
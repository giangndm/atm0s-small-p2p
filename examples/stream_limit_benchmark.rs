use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    net::UdpSocket,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::Context;
use atm0s_small_p2p::{P2pNetwork, P2pNetworkConfig, P2pQuicStream, P2pServiceEvent, P2pServiceRequester, PeerAddress, PeerId, SharedKeyHandshake};
use clap::Parser;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const DEFAULT_CLUSTER_CERT: &[u8] = include_bytes!("../certs/dev.cluster.cert");
pub const DEFAULT_CLUSTER_KEY: &[u8] = include_bytes!("../certs/dev.cluster.key");
pub const DEFAULT_SECURE_KEY: &str = "atm0s";
const SERVICE_ID: u16 = 0;
const WRITE_TICK: Duration = Duration::from_millis(100);
const THROUGHPUT_PAYLOAD_BYTES: usize = 64 * 1024;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "benchmarks/stream_limit_profiles.yaml")]
    profiles: PathBuf,
    #[arg(long, default_value = "docs/stream_limit_benchmark_report.md")]
    report: PathBuf,
    #[arg(long)]
    profile: Option<String>,
    #[arg(long, default_value_t = 0)]
    min_run_seconds: u64,
}

#[derive(Debug, Deserialize)]
struct BenchmarkProfiles {
    profiles: Vec<BenchmarkProfile>,
}

#[derive(Clone, Debug, Deserialize)]
struct BenchmarkProfile {
    name: String,
    #[serde(default)]
    mode: BenchmarkMode,
    nodes: usize,
    source_peer: u64,
    attempts: usize,
    seed: u64,
    min_latency_ms: u64,
    max_latency_ms: u64,
    min_stream_kbps: usize,
    max_stream_kbps: usize,
    min_live_seconds: u64,
    max_live_seconds: u64,
    open_timeout_ms: u64,
    settle_ms: u64,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum BenchmarkMode {
    #[default]
    StreamLimit,
    Throughput,
}

impl BenchmarkMode {
    fn as_str(self) -> &'static str {
        match self {
            BenchmarkMode::StreamLimit => "stream-limit",
            BenchmarkMode::Throughput => "throughput",
        }
    }
}

#[derive(Default)]
struct StreamStats {
    inbound_streams: AtomicUsize,
    received_bytes: AtomicUsize,
    sent_bytes: AtomicUsize,
    write_errors: AtomicUsize,
    read_errors: AtomicUsize,
}

#[derive(Default)]
struct NodeStats {
    opened_streams: AtomicUsize,
    inbound_streams: AtomicUsize,
    sent_bytes: AtomicUsize,
    received_bytes: AtomicUsize,
}

#[derive(Clone, Debug)]
struct NodeStatsSnapshot {
    peer: PeerId,
    opened_streams: usize,
    inbound_streams: usize,
    sent_bytes: usize,
    received_bytes: usize,
}

#[derive(Clone, Copy, Debug)]
struct ResourceSample {
    at_seconds: u64,
    cpu_percent: f64,
    rss_kb: u64,
}

struct BenchmarkResult {
    iteration: usize,
    profile: BenchmarkProfile,
    opened: usize,
    failed: Vec<String>,
    inbound_streams: usize,
    sent_bytes: usize,
    received_bytes: usize,
    write_errors: usize,
    read_errors: usize,
    elapsed: Duration,
    max_latency_ms: u64,
    transfer_bandwidth: BandwidthSummary,
    node_stats: Vec<NodeStatsSnapshot>,
    resource_samples: Vec<ResourceSample>,
}

#[derive(Clone, Copy, Debug, Default)]
struct StreamTransfer {
    bytes: usize,
    elapsed: Duration,
}

impl StreamTransfer {
    fn kbps(self) -> f64 {
        let seconds = self.elapsed.as_secs_f64();
        if seconds <= 0.0 {
            0.0
        } else {
            self.bytes as f64 * 8.0 / seconds / 1000.0
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct BandwidthSummary {
    min_kbps: f64,
    max_kbps: f64,
    avg_kbps: f64,
    sum_kbps: f64,
}

async fn create_node(advertise: bool, peer_id: u64, seeds: Vec<PeerAddress>) -> anyhow::Result<(P2pNetwork<SharedKeyHandshake>, PeerAddress)> {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let priv_key: PrivatePkcs8KeyDer<'_> = PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec());
    let cert = CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec());

    let addr = {
        let socket = UdpSocket::bind("127.0.0.1:0").context("bind benchmark UDP socket")?;
        socket.local_addr().context("read benchmark UDP socket addr")?
    };
    let peer_id = PeerId::from(peer_id);
    let node = P2pNetwork::new(P2pNetworkConfig {
        peer_id,
        listen_addr: addr,
        advertise: advertise.then(|| addr.into()),
        priv_key,
        cert,
        tick_ms: 100,
        seeds,
        secure: DEFAULT_SECURE_KEY.into(),
    })
    .await
    .context("create benchmark node")?;

    Ok((node, (peer_id, addr.into()).into()))
}

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let input = fs::read_to_string(&args.profiles).with_context(|| format!("read profiles from {}", args.profiles.display()))?;
    let profiles: BenchmarkProfiles = serde_yaml::from_str(&input).with_context(|| format!("parse {}", args.profiles.display()))?;
    let profiles = select_profiles(profiles.profiles, args.profile.as_deref())?;
    if args.min_run_seconds > 0 && profiles.len() != 1 {
        anyhow::bail!("--min-run-seconds requires --profile so only one profile runs at a time");
    }

    let mut results = Vec::new();
    let min_run = Duration::from_secs(args.min_run_seconds);
    let started = Instant::now();
    let mut iteration = 1;
    loop {
        for profile in profiles.iter().cloned() {
            println!("running {} iteration {iteration}", profile.name);
            results.push(run_profile(profile, iteration).await?);
        }
        if started.elapsed() >= min_run {
            break;
        }
        iteration += 1;
    }

    let report = render_report(&results);
    if let Some(parent) = args.report.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create report directory {}", parent.display()))?;
    }
    fs::write(&args.report, report).with_context(|| format!("write report {}", args.report.display()))?;
    println!("wrote {}", args.report.display());
    Ok(())
}

fn select_profiles(profiles: Vec<BenchmarkProfile>, selected: Option<&str>) -> anyhow::Result<Vec<BenchmarkProfile>> {
    match selected {
        Some(name) => {
            let selected = profiles.into_iter().filter(|profile| profile.name == name).collect::<Vec<_>>();
            if selected.is_empty() {
                anyhow::bail!("profile {name} not found");
            }
            Ok(selected)
        }
        None => Ok(profiles),
    }
}

async fn run_profile(profile: BenchmarkProfile, iteration: usize) -> anyhow::Result<BenchmarkResult> {
    if !(3..=10).contains(&profile.nodes) {
        anyhow::bail!("profile {} nodes must be in 3..=10, got {}", profile.name, profile.nodes);
    }
    if profile.source_peer == 0 || profile.source_peer as usize > profile.nodes {
        anyhow::bail!("profile {} source_peer must be within node ids", profile.name);
    }
    if profile.min_latency_ms > profile.max_latency_ms {
        anyhow::bail!("profile {} min_latency_ms must be <= max_latency_ms", profile.name);
    }
    if profile.min_live_seconds == 0 || profile.min_live_seconds > profile.max_live_seconds {
        anyhow::bail!("profile {} live seconds must be non-zero and min_live_seconds <= max_live_seconds", profile.name);
    }
    if profile.min_stream_kbps == 0 || profile.min_stream_kbps > profile.max_stream_kbps {
        anyhow::bail!("profile {} stream kbps must be non-zero and min_stream_kbps <= max_stream_kbps", profile.name);
    }
    if profile.mode == BenchmarkMode::Throughput && profile.min_live_seconds < 300 {
        anyhow::bail!("throughput profile {} must run streams for at least 300 seconds", profile.name);
    }

    let started = Instant::now();
    let mut rng = StdRng::seed_from_u64(profile.seed);
    let source_idx = profile.source_peer as usize - 1;
    let latencies = random_latencies(&profile, &mut rng);
    let max_latency_ms = latencies.iter().flatten().copied().max().unwrap_or(0);
    let stats = Arc::new(StreamStats::default());
    let node_stats = (0..profile.nodes).map(|_| Arc::new(NodeStats::default())).collect::<Vec<_>>();
    let stop_sampler = Arc::new(AtomicBool::new(false));
    let resource_samples = Arc::new(Mutex::new(Vec::new()));
    let sampler = tokio::spawn(sample_process_resources(stop_sampler.clone(), resource_samples.clone()));

    let mut addrs: Vec<PeerAddress> = Vec::with_capacity(profile.nodes);
    let mut network_requesters = Vec::with_capacity(profile.nodes);
    let mut service_requesters = Vec::<P2pServiceRequester>::with_capacity(profile.nodes);
    let mut node_tasks = Vec::with_capacity(profile.nodes);

    for (id, node_stat) in node_stats.iter().enumerate() {
        let seeds = if id == 0 {
            vec![]
        } else {
            vec![addrs[0].clone()]
        };
        let (mut node, addr) = create_node(true, (id + 1) as u64, seeds).await?;
        let network_requester = node.requester();
        let mut service = node.create_service(SERVICE_ID.into());
        let service_requester = service.requester();
        let task_stats = stats.clone();
        let task_node_stats = node_stat.clone();

        let task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    event = node.recv() => {
                        if event.is_err() {
                            break;
                        }
                    }
                    service_event = service.recv() => {
                        match service_event {
                            Some(P2pServiceEvent::Stream(_, _, stream)) => {
                                task_stats.inbound_streams.fetch_add(1, Ordering::Relaxed);
                                task_node_stats.inbound_streams.fetch_add(1, Ordering::Relaxed);
                                tokio::spawn(drain_stream(stream, task_stats.clone(), task_node_stats.clone()));
                            }
                            Some(P2pServiceEvent::Unicast(_, _) | P2pServiceEvent::Broadcast(_, _) | P2pServiceEvent::PeerDisconnected(_)) => {}
                            None => break,
                        }
                    }
                }
            }
        });

        addrs.push(addr);
        network_requesters.push(network_requester);
        service_requesters.push(service_requester);
        node_tasks.push(task);
    }

    tokio::time::sleep(Duration::from_millis(profile.settle_ms)).await;
    for (idx, addr) in addrs.iter().enumerate() {
        if idx != source_idx {
            network_requesters[source_idx].try_connect(addr.clone());
        }
    }
    tokio::time::sleep(Duration::from_millis(profile.settle_ms)).await;

    let mut opened = 0;
    let mut failed = Vec::new();
    let mut writers = Vec::new();
    let source = service_requesters[source_idx].clone();
    let destinations: Vec<usize> = (0..profile.nodes).filter(|idx| *idx != source_idx).collect();

    for attempt in 0..profile.attempts {
        let dest_idx = destinations[rng.gen_range(0..destinations.len())];
        let latency_ms = latencies[source_idx][dest_idx];
        tokio::time::sleep(Duration::from_millis(latency_ms)).await;

        let meta = format!("{}:{attempt}:{}->{}", profile.name, profile.source_peer, dest_idx + 1).into_bytes();
        let open = tokio::time::timeout(Duration::from_millis(profile.open_timeout_ms), source.open_stream(addrs[dest_idx].peer_id(), meta)).await;
        match open {
            Ok(Ok(stream)) => {
                opened += 1;
                node_stats[source_idx].opened_streams.fetch_add(1, Ordering::Relaxed);
                let live_seconds = rng.gen_range(profile.min_live_seconds..=profile.max_live_seconds);
                let writer_seed = rng.gen();
                let duration = Duration::from_secs(live_seconds);
                let writer_stats = stats.clone();
                let writer_node_stats = node_stats[source_idx].clone();
                let mode = profile.mode;
                writers.push(tokio::spawn(async move {
                    match mode {
                        BenchmarkMode::StreamLimit => write_stream_throttled(stream, profile.min_stream_kbps, profile.max_stream_kbps, duration, writer_seed, writer_stats, writer_node_stats).await,
                        BenchmarkMode::Throughput => write_stream_unbounded(stream, duration, writer_stats, writer_node_stats).await,
                    }
                }));
            }
            Ok(Err(err)) => failed.push(format!("attempt {attempt} to peer {} failed: {err}", dest_idx + 1)),
            Err(_) => failed.push(format!("attempt {attempt} to peer {} timed out after {}ms", dest_idx + 1, profile.open_timeout_ms)),
        }
    }

    tokio::time::sleep(Duration::from_millis(profile.max_live_seconds * 1000 + max_latency_ms + 250)).await;
    let mut transfers = Vec::new();
    for writer in writers {
        if let Ok(transfer) = writer.await {
            transfers.push(transfer);
        }
    }
    for task in node_tasks {
        task.abort();
    }
    stop_sampler.store(true, Ordering::Relaxed);
    let _ = sampler.await;
    let node_stats = node_stats
        .iter()
        .enumerate()
        .map(|(idx, stats)| NodeStatsSnapshot {
            peer: PeerId::from((idx + 1) as u64),
            opened_streams: stats.opened_streams.load(Ordering::Relaxed),
            inbound_streams: stats.inbound_streams.load(Ordering::Relaxed),
            sent_bytes: stats.sent_bytes.load(Ordering::Relaxed),
            received_bytes: stats.received_bytes.load(Ordering::Relaxed),
        })
        .collect::<Vec<_>>();
    let resource_samples = resource_samples.lock().map(|samples| samples.clone()).unwrap_or_default();

    Ok(BenchmarkResult {
        iteration,
        profile,
        opened,
        failed,
        inbound_streams: stats.inbound_streams.load(Ordering::Relaxed),
        sent_bytes: stats.sent_bytes.load(Ordering::Relaxed),
        received_bytes: stats.received_bytes.load(Ordering::Relaxed),
        write_errors: stats.write_errors.load(Ordering::Relaxed),
        read_errors: stats.read_errors.load(Ordering::Relaxed),
        elapsed: started.elapsed(),
        max_latency_ms,
        transfer_bandwidth: summarize_bandwidth(&transfers),
        node_stats,
        resource_samples,
    })
}

fn random_latencies(profile: &BenchmarkProfile, rng: &mut StdRng) -> Vec<Vec<u64>> {
    let mut latencies = vec![vec![0; profile.nodes]; profile.nodes];
    for (from, row) in latencies.iter_mut().enumerate() {
        for (to, latency) in row.iter_mut().enumerate() {
            if from != to {
                *latency = rng.gen_range(profile.min_latency_ms..=profile.max_latency_ms);
            }
        }
    }
    latencies
}

async fn write_stream_throttled(
    mut stream: P2pQuicStream,
    min_stream_kbps: usize,
    max_stream_kbps: usize,
    duration: Duration,
    seed: u64,
    stats: Arc<StreamStats>,
    node_stats: Arc<NodeStats>,
) -> StreamTransfer {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut stream_kbps = rng.gen_range(min_stream_kbps..=max_stream_kbps);
    let mut next_rate_change = Instant::now() + Duration::from_secs(1);
    let started = Instant::now();
    let mut bytes = 0;

    while started.elapsed() < duration {
        let now = Instant::now();
        if now >= next_rate_change {
            stream_kbps = rng.gen_range(min_stream_kbps..=max_stream_kbps);
            next_rate_change = now + Duration::from_secs(1);
        }

        let bytes_per_second = stream_kbps.saturating_mul(1000) / 8;
        let bytes_per_tick = bytes_per_second.saturating_mul(WRITE_TICK.as_millis() as usize) / 1000;
        let payload = vec![0_u8; bytes_per_tick.max(1)];
        if stream.write_all(&payload).await.is_err() {
            stats.write_errors.fetch_add(1, Ordering::Relaxed);
            return StreamTransfer { bytes, elapsed: started.elapsed() };
        }
        bytes += payload.len();
        stats.sent_bytes.fetch_add(payload.len(), Ordering::Relaxed);
        node_stats.sent_bytes.fetch_add(payload.len(), Ordering::Relaxed);
        tokio::time::sleep(WRITE_TICK).await;
    }

    if stream.shutdown().await.is_err() {
        stats.write_errors.fetch_add(1, Ordering::Relaxed);
    }
    StreamTransfer { bytes, elapsed: started.elapsed() }
}

async fn write_stream_unbounded(mut stream: P2pQuicStream, duration: Duration, stats: Arc<StreamStats>, node_stats: Arc<NodeStats>) -> StreamTransfer {
    let payload = vec![0_u8; THROUGHPUT_PAYLOAD_BYTES];
    let started = Instant::now();
    let mut bytes = 0;

    while started.elapsed() < duration {
        if stream.write_all(&payload).await.is_err() {
            stats.write_errors.fetch_add(1, Ordering::Relaxed);
            return StreamTransfer { bytes, elapsed: started.elapsed() };
        }
        bytes += payload.len();
        stats.sent_bytes.fetch_add(payload.len(), Ordering::Relaxed);
        node_stats.sent_bytes.fetch_add(payload.len(), Ordering::Relaxed);
        tokio::task::yield_now().await;
    }

    if stream.shutdown().await.is_err() {
        stats.write_errors.fetch_add(1, Ordering::Relaxed);
    }
    StreamTransfer { bytes, elapsed: started.elapsed() }
}

fn summarize_bandwidth(transfers: &[StreamTransfer]) -> BandwidthSummary {
    let mut values = transfers.iter().map(|transfer| transfer.kbps()).collect::<Vec<_>>();
    if values.is_empty() {
        return BandwidthSummary::default();
    }
    values.sort_by(|a, b| a.total_cmp(b));
    let sum_kbps = values.iter().sum::<f64>();
    BandwidthSummary {
        min_kbps: values[0],
        max_kbps: values[values.len() - 1],
        avg_kbps: sum_kbps / values.len() as f64,
        sum_kbps,
    }
}

async fn drain_stream(mut stream: P2pQuicStream, stats: Arc<StreamStats>, node_stats: Arc<NodeStats>) {
    let mut buf = [0_u8; 8192];
    loop {
        match stream.read(&mut buf).await {
            Ok(0) => break,
            Ok(size) => {
                stats.received_bytes.fetch_add(size, Ordering::Relaxed);
                node_stats.received_bytes.fetch_add(size, Ordering::Relaxed);
            }
            Err(_) => {
                stats.read_errors.fetch_add(1, Ordering::Relaxed);
                break;
            }
        }
    }
}

async fn sample_process_resources(stop: Arc<AtomicBool>, samples: Arc<Mutex<Vec<ResourceSample>>>) {
    let started = Instant::now();
    let mut previous = read_process_ticks().map(|ticks| (Instant::now(), ticks));
    while !stop.load(Ordering::Relaxed) {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let now = Instant::now();
        let Some(ticks) = read_process_ticks() else {
            continue;
        };
        let rss_kb = read_process_rss_kb().unwrap_or(0);
        let cpu_percent = if let Some((previous_at, previous_ticks)) = previous {
            let elapsed = now.duration_since(previous_at).as_secs_f64();
            let tick_delta = ticks.saturating_sub(previous_ticks) as f64;
            let cpus = std::thread::available_parallelism().map_or(1.0, |cpus| cpus.get() as f64);
            if elapsed > 0.0 {
                tick_delta / 100.0 / elapsed / cpus * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };
        previous = Some((now, ticks));
        if let Ok(mut samples) = samples.lock() {
            samples.push(ResourceSample {
                at_seconds: started.elapsed().as_secs(),
                cpu_percent,
                rss_kb,
            });
        }
    }
}

fn read_process_ticks() -> Option<u64> {
    let stat = fs::read_to_string("/proc/self/stat").ok()?;
    let right = stat.rfind(')')?;
    let fields = stat.get(right + 2..)?.split_whitespace().collect::<Vec<_>>();
    let utime = fields.get(11)?.parse::<u64>().ok()?;
    let stime = fields.get(12)?.parse::<u64>().ok()?;
    Some(utime.saturating_add(stime))
}

fn read_process_rss_kb() -> Option<u64> {
    let status = fs::read_to_string("/proc/self/status").ok()?;
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            return rest.split_whitespace().next()?.parse::<u64>().ok();
        }
    }
    None
}

fn render_report(results: &[BenchmarkResult]) -> String {
    let generated = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or_else(|_| "unknown".to_string(), |duration| duration.as_secs().to_string());
    let repeated_short_iterations = has_repeated_profile_iterations(results);
    let rss_warnings = rss_growth_warnings(results);
    let rss_warning_profiles = rss_warnings.iter().map(|warning| warning.profile.as_str()).collect::<BTreeSet<_>>();
    let mut report = String::new();
    report.push_str("# Stream Limit Benchmark Report\n\n");
    report.push_str(&format!("Generated at unix timestamp: `{generated}`\n\n"));
    report.push_str("This benchmark opens streams from one source node to random peer nodes. Stream-limit profiles write at a random kbps target from the configured range, and that target changes once per second to approximate voice/video user traffic. Throughput profiles fork multiple streams and write as fast as the transport allows for at least five minutes per stream. Random latency is applied before each stream-open attempt to model different global-network paths.\n\n");
    report.push_str(
        "CPU and memory samples are process-level because the benchmark hosts all nodes inside one OS process. Per-node tables report stream and byte counters collected inside the benchmark.\n\n",
    );
    if repeated_short_iterations {
        report.push_str("Long-run mode note: this report contains repeated short-cluster iterations. Each table row is one fresh cluster run; it is not one continuous cluster unless a row's elapsed time covers the requested duration.\n\n");
    }
    if !rss_warnings.is_empty() {
        report.push_str("RSS growth warning: at least one profile's max RSS increased sharply across repeated iterations. Treat rows marked `resource-warning` as stability evidence that needs investigation, even when stream counters are clean.\n\n");
        for warning in &rss_warnings {
            report.push_str(&format!("- `{}` RSS grew from `{}` KiB to `{}` KiB.\n", warning.profile, warning.first_rss_kb, warning.max_rss_kb));
        }
        report.push('\n');
    }
    report.push_str("| Iteration | Profile | Mode | Nodes | Attempts | Opened | Failed | Inbound streams | Sent bytes | Received bytes | Min kbps | Max kbps | Avg kbps | Sum kbps | Max latency | Elapsed | Result |\n");
    report.push_str("| ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |\n");

    for result in results {
        let stream_ok = result.failed.is_empty() && result.opened == result.profile.attempts && result.write_errors == 0 && result.read_errors == 0;
        let status = if !stream_ok {
            "limited"
        } else if rss_warning_profiles.contains(result.profile.name.as_str()) {
            "resource-warning"
        } else {
            "pass"
        };
        report.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {:.2} | {:.2} | {:.2} | {:.2} | {}ms | {:.2}s | {} |\n",
            result.iteration,
            result.profile.name,
            result.profile.mode.as_str(),
            result.profile.nodes,
            result.profile.attempts,
            result.opened,
            result.failed.len(),
            result.inbound_streams,
            result.sent_bytes,
            result.received_bytes,
            result.transfer_bandwidth.min_kbps,
            result.transfer_bandwidth.max_kbps,
            result.transfer_bandwidth.avg_kbps,
            result.transfer_bandwidth.sum_kbps,
            result.max_latency_ms,
            result.elapsed.as_secs_f64(),
            status
        ));
    }

    report.push_str("\n## Resource Charts\n\n");
    for result in results {
        report.push_str(&format!("### {} iteration {}\n\n", result.profile.name, result.iteration));
        report.push_str("CPU percent:\n\n");
        report.push_str(&render_svg_chart(&result.resource_samples, "cpu_percent"));
        report.push_str("\n\nRSS memory KiB:\n\n");
        report.push_str(&render_svg_chart(&result.resource_samples, "rss_kb"));
        report.push_str("\n\n");
    }

    report.push_str("\n## Per-Node Counters\n\n");
    for result in results {
        report.push_str(&format!("### {} iteration {}\n\n", result.profile.name, result.iteration));
        report.push_str("| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |\n");
        report.push_str("| ---: | ---: | ---: | ---: | ---: |\n");
        for node in &result.node_stats {
            report.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                node.peer, node.opened_streams, node.inbound_streams, node.sent_bytes, node.received_bytes
            ));
        }
        report.push('\n');
    }

    report.push_str("\n## Profiles\n\n");
    for result in results {
        report.push_str(&format!(
            "- `{}`: mode `{}`, source peer `{}`, nodes `{}`, latency `{}..={}ms`, target stream rate `{}..={} kbps`, stream live range `{}..={}s`, open timeout `{}ms`, seed `{}`.\n",
            result.profile.name,
            result.profile.mode.as_str(),
            result.profile.source_peer,
            result.profile.nodes,
            result.profile.min_latency_ms,
            result.profile.max_latency_ms,
            result.profile.min_stream_kbps,
            result.profile.max_stream_kbps,
            result.profile.min_live_seconds,
            result.profile.max_live_seconds,
            result.profile.open_timeout_ms,
            result.profile.seed
        ));
    }

    report.push_str("\n## Failures\n\n");
    let mut any_failure = false;
    for result in results {
        if result.failed.is_empty() && result.write_errors == 0 && result.read_errors == 0 {
            continue;
        }
        any_failure = true;
        report.push_str(&format!("### {}\n\n", result.profile.name));
        report.push_str(&format!("- Write errors: `{}`\n", result.write_errors));
        report.push_str(&format!("- Read errors: `{}`\n", result.read_errors));
        for failure in result.failed.iter().take(20) {
            report.push_str(&format!("- {failure}\n"));
        }
        if result.failed.len() > 20 {
            report.push_str(&format!("- ... {} more failures omitted\n", result.failed.len() - 20));
        }
        report.push('\n');
    }
    if !any_failure {
        report.push_str("No stream open, write, or read failures were observed in this run.\n");
    }

    report.push_str("\n## Reproduce\n\n");
    report.push_str("```bash\n");
    report
        .push_str("RUST_LOG=error CARGO_BUILD_JOBS=8 cargo run --example stream_limit_benchmark -- --profile stream-limit-10-nodes --min-run-seconds 1800 --profiles benchmarks/stream_limit_profiles.yaml --report docs/stream_limit_benchmark_report.md\n");
    report
        .push_str("RUST_LOG=error CARGO_BUILD_JOBS=8 cargo run --example stream_limit_benchmark -- --profile throughput-10-nodes --profiles benchmarks/stream_limit_profiles.yaml --report docs/throughput_benchmark_report.md\n");
    report.push_str("```\n");
    report
}

fn has_repeated_profile_iterations(results: &[BenchmarkResult]) -> bool {
    let mut seen = BTreeSet::new();
    for result in results {
        if !seen.insert(result.profile.name.as_str()) {
            return true;
        }
    }
    false
}

#[derive(Debug)]
struct RssGrowthWarning {
    profile: String,
    first_rss_kb: u64,
    max_rss_kb: u64,
}

fn rss_growth_warnings(results: &[BenchmarkResult]) -> Vec<RssGrowthWarning> {
    const MIN_ABSOLUTE_GROWTH_KIB: u64 = 128 * 1024;
    const MIN_GROWTH_FACTOR: u64 = 2;

    let mut by_profile = BTreeMap::<&str, (Option<u64>, u64)>::new();
    for result in results {
        let profile = result.profile.name.as_str();
        let max_rss = result.resource_samples.iter().map(|sample| sample.rss_kb).max().unwrap_or(0);
        let entry = by_profile.entry(profile).or_insert((None, 0));
        if entry.0.is_none() && max_rss > 0 {
            entry.0 = Some(max_rss);
        }
        entry.1 = entry.1.max(max_rss);
    }

    by_profile
        .into_iter()
        .filter_map(|(profile, (first_rss_kb, max_rss_kb))| {
            let first_rss_kb = first_rss_kb?;
            let absolute_growth = max_rss_kb.saturating_sub(first_rss_kb);
            let factor_growth = max_rss_kb >= first_rss_kb.saturating_mul(MIN_GROWTH_FACTOR);
            if absolute_growth >= MIN_ABSOLUTE_GROWTH_KIB && factor_growth {
                Some(RssGrowthWarning {
                    profile: profile.to_string(),
                    first_rss_kb,
                    max_rss_kb,
                })
            } else {
                None
            }
        })
        .collect()
}

fn render_svg_chart(samples: &[ResourceSample], metric: &str) -> String {
    const WIDTH: f64 = 720.0;
    const HEIGHT: f64 = 180.0;
    const PAD: f64 = 24.0;
    if samples.is_empty() {
        return "<svg width=\"720\" height=\"80\" xmlns=\"http://www.w3.org/2000/svg\"><text x=\"16\" y=\"44\" font-family=\"monospace\" font-size=\"14\">no samples</text></svg>".to_string();
    }

    let max_x = samples.iter().map(|sample| sample.at_seconds).max().unwrap_or(1).max(1) as f64;
    let values = samples
        .iter()
        .map(|sample| {
            if metric == "rss_kb" {
                sample.rss_kb as f64
            } else {
                sample.cpu_percent
            }
        })
        .collect::<Vec<_>>();
    let max_y = values.iter().copied().fold(0.0_f64, f64::max).max(1.0);
    let points = samples
        .iter()
        .zip(values.iter())
        .map(|(sample, value)| {
            let x = PAD + sample.at_seconds as f64 / max_x * (WIDTH - PAD * 2.0);
            let y = HEIGHT - PAD - (*value / max_y * (HEIGHT - PAD * 2.0));
            format!("{x:.1},{y:.1}")
        })
        .collect::<Vec<_>>()
        .join(" ");

    let label = if metric == "rss_kb" {
        "RSS KiB"
    } else {
        "CPU %"
    };
    let last = values.last().copied().unwrap_or(0.0);
    format!(
        "<svg width=\"720\" height=\"180\" viewBox=\"0 0 720 180\" xmlns=\"http://www.w3.org/2000/svg\" role=\"img\" aria-label=\"{label} over time\"><rect width=\"720\" height=\"180\" fill=\"#fff\"/><line x1=\"24\" y1=\"156\" x2=\"696\" y2=\"156\" stroke=\"#bbb\"/><line x1=\"24\" y1=\"24\" x2=\"24\" y2=\"156\" stroke=\"#bbb\"/><polyline points=\"{points}\" fill=\"none\" stroke=\"#2563eb\" stroke-width=\"2\"/><text x=\"24\" y=\"16\" font-family=\"monospace\" font-size=\"12\">{label}, max {max_y:.2}, last {last:.2}</text><text x=\"24\" y=\"176\" font-family=\"monospace\" font-size=\"11\">0s</text><text x=\"650\" y=\"176\" font-family=\"monospace\" font-size=\"11\">{max_x:.0}s</text></svg>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_profile() -> BenchmarkProfile {
        BenchmarkProfile {
            name: "stream-limit-test".to_string(),
            mode: BenchmarkMode::StreamLimit,
            nodes: 3,
            source_peer: 1,
            attempts: 2,
            seed: 7,
            min_latency_ms: 1,
            max_latency_ms: 2,
            min_stream_kbps: 32,
            max_stream_kbps: 64,
            min_live_seconds: 1,
            max_live_seconds: 2,
            open_timeout_ms: 100,
            settle_ms: 1,
        }
    }

    fn test_result(iteration: usize, elapsed_seconds: u64, rss_kb: u64) -> BenchmarkResult {
        BenchmarkResult {
            iteration,
            profile: test_profile(),
            opened: 2,
            failed: Vec::new(),
            inbound_streams: 2,
            sent_bytes: 2000,
            received_bytes: 2000,
            write_errors: 0,
            read_errors: 0,
            elapsed: Duration::from_secs(elapsed_seconds),
            max_latency_ms: 2,
            transfer_bandwidth: BandwidthSummary {
                min_kbps: 80.0,
                max_kbps: 160.0,
                avg_kbps: 120.0,
                sum_kbps: 240.0,
            },
            node_stats: vec![NodeStatsSnapshot {
                peer: PeerId::from(1),
                opened_streams: 2,
                inbound_streams: 0,
                sent_bytes: 2000,
                received_bytes: 0,
            }],
            resource_samples: vec![ResourceSample {
                at_seconds: elapsed_seconds,
                cpu_percent: 1.0,
                rss_kb,
            }],
        }
    }

    #[test]
    fn report_labels_repeated_short_iterations_as_not_one_continuous_cluster() {
        let report = render_report(&[test_result(1, 30, 40_000), test_result(2, 30, 45_000)]);

        assert!(report.contains("repeated short-cluster iterations"));
        assert!(report.contains("not one continuous cluster"));
    }

    #[test]
    fn report_does_not_mark_large_rss_growth_as_plain_pass() {
        let report = render_report(&[test_result(1, 30, 40_000), test_result(2, 30, 700_000)]);

        assert!(report.contains("RSS growth warning"));
        assert!(!report.contains("| 2 | stream-limit-test | stream-limit | 3 | 2 | 2 | 0 | 2 | 2000 | 2000 | 80.00 | 160.00 | 120.00 | 240.00 | 2ms | 30.00s | pass |"));
    }

    #[test]
    fn report_includes_transfer_bandwidth_summary() {
        let report = render_report(&[test_result(1, 30, 40_000)]);

        assert!(report.contains("Min kbps"));
        assert!(report.contains("Max kbps"));
        assert!(report.contains("Avg kbps"));
        assert!(report.contains("Sum kbps"));
        assert!(report.contains("| 1 | stream-limit-test | stream-limit | 3 | 2 | 2 | 0 | 2 | 2000 | 2000 | 80.00 | 160.00 | 120.00 | 240.00 | 2ms | 30.00s | pass |"));
    }

    #[test]
    fn bandwidth_summary_uses_stream_min_max_average_and_sum() {
        let summary = summarize_bandwidth(&[
            StreamTransfer {
                bytes: 10_000,
                elapsed: Duration::from_secs(1),
            },
            StreamTransfer {
                bytes: 20_000,
                elapsed: Duration::from_secs(1),
            },
        ]);

        assert_eq!(summary.min_kbps, 80.0);
        assert_eq!(summary.max_kbps, 160.0);
        assert_eq!(summary.avg_kbps, 120.0);
        assert_eq!(summary.sum_kbps, 240.0);
    }
}

use std::{
    env,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use rand::{rngs::StdRng, Rng, SeedableRng};
use test_log::test;
use tokio::{sync::oneshot, task::JoinHandle};

use crate::{
    msg::{BroadcastMsgId, P2pServiceId, PeerMessage},
    P2pNetworkRequester, P2pServiceEvent, P2pServiceRequester, PeerAddress, PeerId, SharedCtx,
};

use super::create_node;

fn env_usize(name: &str, default: usize) -> usize {
    env::var(name).ok().and_then(|value| value.parse().ok()).unwrap_or(default)
}

fn env_u64(name: &str, default: u64) -> u64 {
    env::var(name).ok().and_then(|value| value.parse().ok()).unwrap_or(default)
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 4))]
async fn fuzz_random_node_actions_must_not_panic_connection_tasks() {
    run_random_node_action_fuzz(true, 120).await;
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 4))]
async fn fuzz_random_valid_node_actions_must_not_panic_connection_tasks() {
    run_random_node_action_fuzz(false, 300).await;
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 4))]
async fn fuzz_random_node_churn_actions_must_not_panic_connection_tasks() {
    run_random_node_churn_fuzz(true, 180).await;
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 4))]
async fn fuzz_random_valid_node_churn_actions_must_not_panic_connection_tasks() {
    run_random_node_churn_fuzz(false, 300).await;
}

async fn run_random_node_action_fuzz(include_known_invalid_service: bool, default_steps: usize) {
    let node_count = env_usize("P2P_FUZZ_NODES", 5).clamp(2, 8);
    let steps = env_usize("P2P_FUZZ_STEPS", default_steps);
    let seed = env_u64("P2P_FUZZ_SEED", 0x5eed);
    let mut rng = StdRng::seed_from_u64(seed);

    let background_panicked = Arc::new(AtomicBool::new(false));
    let previous_hook = std::panic::take_hook();
    let hook_flag = background_panicked.clone();
    std::panic::set_hook(Box::new(move |info| {
        hook_flag.store(true, Ordering::SeqCst);
        eprintln!("{info}");
    }));

    let mut addrs = Vec::with_capacity(node_count);
    let mut requesters = Vec::with_capacity(node_count);
    let mut service_requesters = Vec::with_capacity(node_count);
    let mut ctxs = Vec::with_capacity(node_count);

    for id in 0..node_count {
        let (mut node, addr) = create_node(true, (id + 1) as u64, vec![]).await;
        let mut service = node.create_service(P2pServiceId::from(0));
        requesters.push(node.requester());
        service_requesters.push(service.requester());
        ctxs.push(node.ctx.clone());
        addrs.push(addr);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    event = node.recv() => {
                        if event.is_err() {
                            break;
                        }
                    }
                    service_event = service.recv() => {
                        match service_event {
                            Some(P2pServiceEvent::Stream(_, _, _stream)) => {}
                            Some(P2pServiceEvent::Unicast(_, _)) | Some(P2pServiceEvent::Broadcast(_, _)) => {}
                            None => break,
                        }
                    }
                }
            }
        });
    }

    for step in 0..steps {
        let from = rng.gen_range(0..node_count);
        let mut to = rng.gen_range(0..node_count);
        if to == from {
            to = (to + 1) % node_count;
        }

        let actions = if include_known_invalid_service {
            11
        } else {
            10
        };
        match rng.gen_range(0..actions) {
            0 => {
                requesters[from].try_connect(addrs[to].clone());
            }
            1 => {
                let _ = tokio::time::timeout(Duration::from_millis(50), requesters[from].connect(addrs[to].clone())).await;
            }
            2 => {
                let data = format!("fuzz-unicast-{seed}-{step}-{from}-{to}").into_bytes();
                let _ = tokio::time::timeout(Duration::from_millis(50), service_requesters[from].send_unicast(addrs[to].peer_id(), data)).await;
            }
            3 => {
                let data = format!("fuzz-broadcast-{seed}-{step}-{from}").into_bytes();
                let _ = service_requesters[from].try_send_broadcast(data).await;
            }
            4 => {
                let meta = format!("fuzz-stream-{seed}-{step}-{from}-{to}").into_bytes();
                let _ = tokio::time::timeout(Duration::from_millis(100), service_requesters[from].open_stream(addrs[to].peer_id(), meta)).await;
            }
            5 => {
                if let Some(conn) = ctxs[from].conns().into_iter().next() {
                    let forged_peer = addrs[rng.gen_range(0..node_count)].peer_id();
                    let _ = conn.try_send(PeerMessage::PeerStopped(forged_peer));
                }
            }
            6 => {
                let burst = rng.gen_range(2..=6);
                for _ in 0..burst {
                    requesters[from].try_connect(addrs[to].clone());
                }
            }
            7 => {
                let data = format!("fuzz-try-unicast-{seed}-{step}-{from}-{to}").into_bytes();
                let _ = service_requesters[from].try_send_unicast(addrs[to].peer_id(), data).await;
            }
            8 => {
                let data = format!("fuzz-send-broadcast-{seed}-{step}-{from}").into_bytes();
                let _ = tokio::time::timeout(Duration::from_millis(50), service_requesters[from].send_broadcast(data)).await;
            }
            9 => {
                if let Some(conn) = ctxs[from].conns().into_iter().next() {
                    let source = PeerId::from(rng.gen_range(1_000_000..2_000_000));
                    let dest = addrs[to].peer_id();
                    let data = format!("fuzz-raw-unicast-{seed}-{step}-{from}-{to}").into_bytes();
                    let _ = conn.try_send(PeerMessage::Unicast(source, dest, P2pServiceId::from(0), data));
                }
            }
            _ if include_known_invalid_service => {
                if let Some(conn) = ctxs[from].conns().into_iter().next() {
                    let data = format!("fuzz-invalid-service-{seed}-{step}").into_bytes();
                    let _ = conn.try_send(PeerMessage::Broadcast(PeerId::from(999_999), P2pServiceId::from(256), BroadcastMsgId::rand(), data));
                }
            }
            _ => unreachable!("action count excludes known invalid service action"),
        }

        tokio::time::sleep(Duration::from_millis(5)).await;
        if background_panicked.load(Ordering::SeqCst) {
            break;
        }
    }

    tokio::time::sleep(Duration::from_millis(100)).await;
    std::panic::set_hook(previous_hook);

    assert!(
        !background_panicked.load(Ordering::SeqCst),
        "fuzz random node actions must not panic background connection/service tasks; seed={seed}, nodes={node_count}, steps={steps}"
    );
}

struct RunningFuzzNode {
    addr: PeerAddress,
    requester: P2pNetworkRequester,
    service_requester: P2pServiceRequester,
    ctx: SharedCtx,
    stop_tx: Option<oneshot::Sender<()>>,
    task: JoinHandle<()>,
}

impl RunningFuzzNode {
    fn abort(self) {
        self.task.abort();
    }
}

async fn spawn_fuzz_node(peer_id: u64) -> RunningFuzzNode {
    let (mut node, addr) = create_node(true, peer_id, vec![]).await;
    let requester = node.requester();
    let mut service = node.create_service(P2pServiceId::from(0));
    let service_requester = service.requester();
    let ctx = node.ctx.clone();
    let (stop_tx, mut stop_rx) = oneshot::channel();

    let task = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut stop_rx => {
                    node.shutdown_gracefully().await;
                    break;
                }
                event = node.recv() => {
                    if event.is_err() {
                        break;
                    }
                }
                service_event = service.recv() => {
                    match service_event {
                        Some(P2pServiceEvent::Stream(_, _, _stream)) => {}
                        Some(P2pServiceEvent::Unicast(_, _)) | Some(P2pServiceEvent::Broadcast(_, _)) => {}
                        None => break,
                    }
                }
            }
        }
    });

    RunningFuzzNode {
        addr,
        requester,
        service_requester,
        ctx,
        stop_tx: Some(stop_tx),
        task,
    }
}

async fn run_random_node_churn_fuzz(include_known_invalid_service: bool, default_steps: usize) {
    let node_count = env_usize("P2P_FUZZ_NODES", 5).clamp(2, 8);
    let steps = env_usize("P2P_FUZZ_STEPS", default_steps);
    let seed = env_u64("P2P_FUZZ_SEED", 0x51a7e);
    let mut rng = StdRng::seed_from_u64(seed);

    let background_panicked = Arc::new(AtomicBool::new(false));
    let previous_hook = std::panic::take_hook();
    let hook_flag = background_panicked.clone();
    std::panic::set_hook(Box::new(move |info| {
        hook_flag.store(true, Ordering::SeqCst);
        eprintln!("{info}");
    }));

    let mut nodes = Vec::with_capacity(node_count);
    for id in 0..node_count {
        nodes.push(Some(spawn_fuzz_node((id + 1) as u64).await));
    }

    for step in 0..steps {
        let from = rng.gen_range(0..node_count);
        let mut to = rng.gen_range(0..node_count);
        if to == from {
            to = (to + 1) % node_count;
        }

        let actions = if include_known_invalid_service {
            12
        } else {
            11
        };
        match rng.gen_range(0..actions) {
            0 => {
                if let (Some(from_node), Some(to_node)) = (&nodes[from], &nodes[to]) {
                    from_node.requester.try_connect(to_node.addr.clone());
                }
            }
            1 => {
                if let (Some(from_node), Some(to_node)) = (&nodes[from], &nodes[to]) {
                    let _ = tokio::time::timeout(Duration::from_millis(50), from_node.requester.connect(to_node.addr.clone())).await;
                }
            }
            2 => {
                if let (Some(from_node), Some(to_node)) = (&nodes[from], &nodes[to]) {
                    let data = format!("fuzz-churn-unicast-{seed}-{step}-{from}-{to}").into_bytes();
                    let _ = tokio::time::timeout(Duration::from_millis(50), from_node.service_requester.send_unicast(to_node.addr.peer_id(), data)).await;
                }
            }
            3 => {
                if let Some(from_node) = &nodes[from] {
                    let data = format!("fuzz-churn-broadcast-{seed}-{step}-{from}").into_bytes();
                    let _ = from_node.service_requester.try_send_broadcast(data).await;
                }
            }
            4 => {
                if let (Some(from_node), Some(to_node)) = (&nodes[from], &nodes[to]) {
                    let meta = format!("fuzz-churn-stream-{seed}-{step}-{from}-{to}").into_bytes();
                    let _ = tokio::time::timeout(Duration::from_millis(100), from_node.service_requester.open_stream(to_node.addr.peer_id(), meta)).await;
                }
            }
            5 => {
                if let Some(from_node) = &nodes[from] {
                    if let Some(conn) = from_node.ctx.conns().into_iter().next() {
                        let forged_peer = PeerId::from(rng.gen_range(1..=(node_count as u64 + 4)));
                        let _ = conn.try_send(PeerMessage::PeerStopped(forged_peer));
                    }
                }
            }
            6 => {
                if let Some(mut node) = nodes[from].take() {
                    if let Some(stop_tx) = node.stop_tx.take() {
                        let _ = stop_tx.send(());
                    }
                    let _ = tokio::time::timeout(Duration::from_millis(200), &mut node.task).await;
                }
            }
            7 => {
                if nodes[from].is_none() {
                    nodes[from] = Some(spawn_fuzz_node((from + 1) as u64).await);
                }
            }
            8 => {
                if let (Some(from_node), Some(to_node)) = (&nodes[from], &nodes[to]) {
                    for _ in 0..rng.gen_range(2..=6) {
                        from_node.requester.try_connect(to_node.addr.clone());
                    }
                }
            }
            9 => {
                if let (Some(from_node), Some(to_node)) = (&nodes[from], &nodes[to]) {
                    if let Some(conn) = from_node.ctx.conns().into_iter().next() {
                        let source = PeerId::from(rng.gen_range(1_000_000..2_000_000));
                        let data = format!("fuzz-churn-raw-unicast-{seed}-{step}-{from}-{to}").into_bytes();
                        let _ = conn.try_send(PeerMessage::Unicast(source, to_node.addr.peer_id(), P2pServiceId::from(0), data));
                    }
                }
            }
            10 if include_known_invalid_service => {
                if let Some(from_node) = &nodes[from] {
                    if let Some(conn) = from_node.ctx.conns().into_iter().next() {
                        let data = format!("fuzz-churn-invalid-service-{seed}-{step}").into_bytes();
                        let _ = conn.try_send(PeerMessage::Broadcast(PeerId::from(999_999), P2pServiceId::from(256), BroadcastMsgId::rand(), data));
                    }
                }
            }
            _ => {
                if let Some(from_node) = &nodes[from] {
                    let data = format!("fuzz-churn-send-broadcast-{seed}-{step}-{from}").into_bytes();
                    let _ = tokio::time::timeout(Duration::from_millis(50), from_node.service_requester.send_broadcast(data)).await;
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(5)).await;
        if background_panicked.load(Ordering::SeqCst) {
            break;
        }
    }

    tokio::time::sleep(Duration::from_millis(150)).await;
    for node in nodes.into_iter().flatten() {
        node.abort();
    }
    std::panic::set_hook(previous_hook);

    assert!(
        !background_panicked.load(Ordering::SeqCst),
        "fuzz random node churn actions must not panic background connection/service tasks; seed={seed}, nodes={node_count}, steps={steps}"
    );
}

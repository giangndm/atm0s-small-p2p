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

use crate::{
    msg::{BroadcastMsgId, P2pServiceId, PeerMessage},
    P2pServiceEvent, PeerId,
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

        let actions = if include_known_invalid_service { 11 } else { 10 };
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

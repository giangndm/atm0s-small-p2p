use std::{collections::HashSet, sync::Arc, time::Duration};

use super::create_node;
use crate::PeerAddress;
use parking_lot::Mutex;
use test_log::test;

#[test(tokio::test)]
async fn discovery_remain_node() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    log::info!("created node1 {addr1}");
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, addr2) = create_node(false, 2, vec![addr1]).await;
    log::info!("created node2 {addr2}");
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    let (mut node3, addr3) = create_node(false, 3, vec![addr2]).await;
    log::info!("created node3 {addr3}");
    let node3_neighbours = Arc::new(Mutex::new(HashSet::new()));
    let node3_neighbours_c = node3_neighbours.clone();
    tokio::spawn(async move {
        while let Ok(event) = node3.recv().await {
            match event {
                crate::P2pNetworkEvent::PeerConnected(_conn, peer) => {
                    node3_neighbours_c.lock().insert(peer);
                }
                crate::P2pNetworkEvent::PeerDisconnected(_conn, peer) => {
                    node3_neighbours_c.lock().remove(&peer);
                }
                crate::P2pNetworkEvent::Continue => {}
            }
        }
    });

    tokio::time::sleep(Duration::from_secs(1)).await;

    // after some cycle node3 should have node1 as neighbour
    assert_eq!(node3_neighbours.lock().len(), 2);
}

#[test(tokio::test)]
async fn graceful_shutdown_removes_stopped_non_seed() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, addr2) = create_node(true, 2, vec![addr1.clone()]).await;
    tokio::spawn(async move {
        let shutdown_at = tokio::time::sleep(Duration::from_millis(900));
        tokio::pin!(shutdown_at);
        loop {
            tokio::select! {
                _ = &mut shutdown_at => {
                    node2.shutdown_gracefully().await;
                    break;
                }
                res = node2.recv() => {
                    if res.is_err() {
                        break;
                    }
                }
            }
        }
    });

    let (mut node3, _addr3) = create_node(false, 3, vec![addr1]).await;
    let node3_neighbours = Arc::new(Mutex::new(HashSet::new()));
    let node3_neighbours_c = node3_neighbours.clone();
    tokio::spawn(async move {
        while let Ok(event) = node3.recv().await {
            match event {
                crate::P2pNetworkEvent::PeerConnected(_conn, peer) => {
                    node3_neighbours_c.lock().insert(peer);
                }
                crate::P2pNetworkEvent::PeerDisconnected(_conn, peer) => {
                    node3_neighbours_c.lock().remove(&peer);
                }
                crate::P2pNetworkEvent::Continue => {}
            }
        }
    });

    tokio::time::sleep(Duration::from_secs(2)).await;

    assert!(
        !node3_neighbours.lock().contains(&addr2.peer_id()),
        "node3 should remove a gracefully stopped non-seed peer and not keep reconnecting to it"
    );
}

#[test(tokio::test)]
async fn discovery_tick_connect_backlog_must_coalesce_duplicate_remotes() {
    const MAX_PENDING_PER_REMOTE: usize = 1;
    let seed: PeerAddress = "2@127.0.0.1:9".parse().expect("seed address should parse");
    let (mut node, _addr) = create_node(false, 1, vec![seed]).await;

    for now in 0..1025 {
        node.process_tick(now * 100).expect("tick should process");
    }

    assert!(
        node.control_rx.len() <= MAX_PENDING_PER_REMOTE,
        "discovery ticks must coalesce pending connects for the same remote, got {}",
        node.control_rx.len()
    );
}

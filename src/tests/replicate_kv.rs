use std::time::Duration;

use test_log::test;
use tokio::time::timeout;

use crate::replicate_kv_service::{KvEvent, ReplicatedKvService};

use super::create_node;

const WAIT: Duration = Duration::from_secs(3);

#[test(tokio::test)]
async fn single_node() {
    let (mut node1, _addr1) = create_node(true, 1, vec![]).await;
    let mut kv1: ReplicatedKvService<u16, u16> = ReplicatedKvService::new(node1.create_service(0.into()), 10, 3);
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    kv1.set(1, 1);

    assert_eq!(timeout(WAIT, kv1.recv()).await, Ok(Some(KvEvent::Set(None, 1, 1))));
}

#[test(tokio::test)]
async fn full_sync() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let (mut node2, addr2) = create_node(true, 2, vec![]).await;

    let node1_requester = node1.requester();

    let mut kv1: ReplicatedKvService<u16, u16> = ReplicatedKvService::new(node1.create_service(0.into()), 10, 3);
    let mut kv2: ReplicatedKvService<u16, u16> = ReplicatedKvService::new(node2.create_service(0.into()), 10, 3);

    tokio::spawn(async move {
        kv1.set(1, 1);
        kv1.set(2, 2);
        kv1.set(3, 3);
        while let Some(_event) = kv1.recv().await {}
    });

    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    tokio::time::sleep(Duration::from_millis(1000)).await;
    node1_requester.connect(addr2).await.expect("should connect success");

    assert_eq!(timeout(WAIT, kv2.recv()).await, Ok(Some(KvEvent::Set(Some(addr1.peer_id()), 1, 1))));
    assert_eq!(timeout(WAIT, kv2.recv()).await, Ok(Some(KvEvent::Set(Some(addr1.peer_id()), 2, 2))));
    assert_eq!(timeout(WAIT, kv2.recv()).await, Ok(Some(KvEvent::Set(Some(addr1.peer_id()), 3, 3))));
}

#[test(tokio::test)]
async fn replicated_kv_must_delete_remote_data_when_peer_gracefully_stops() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let (mut node2, addr2) = create_node(true, 2, vec![]).await;
    let node1_requester = node1.requester();
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();

    let mut kv1: ReplicatedKvService<u16, u16> = ReplicatedKvService::new(node1.create_service(0.into()), 10, 3);
    let mut kv2: ReplicatedKvService<u16, u16> = ReplicatedKvService::new(node2.create_service(0.into()), 10, 3);

    tokio::spawn(async move {
        kv1.set(7, 70);
        while kv1.recv().await.is_some() {}
    });

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => {
                    node1.shutdown_gracefully().await;
                    break;
                }
                res = node1.recv() => {
                    if res.is_err() {
                        break;
                    }
                }
            }
        }
    });
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    node1_requester.connect(addr2).await.expect("connect should succeed");

    assert_eq!(timeout(WAIT, kv2.recv()).await, Ok(Some(KvEvent::Set(Some(addr1.peer_id()), 7, 70))));

    shutdown_tx.send(()).expect("shutdown signal should send");

    let deleted = timeout(WAIT, async {
        loop {
            if let Some(KvEvent::Del(Some(peer), 7)) = kv2.recv().await {
                if peer == addr1.peer_id() {
                    break;
                }
            }
        }
    })
    .await;

    assert!(
        deleted.is_ok(),
        "replicated KV must delete a gracefully stopped peer's data promptly, not wait for the 10s idle timeout"
    );
}

// compose pkt smaller than slots count then FullSync will be split to multiple packets
#[test(tokio::test)]
async fn full_sync2() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let (mut node2, addr2) = create_node(true, 2, vec![]).await;

    let node1_requester = node1.requester();

    let mut kv1: ReplicatedKvService<u16, u16> = ReplicatedKvService::new(node1.create_service(0.into()), 10, 2);
    let mut kv2: ReplicatedKvService<u16, u16> = ReplicatedKvService::new(node2.create_service(0.into()), 10, 2);

    tokio::spawn(async move {
        kv1.set(1, 1);
        kv1.set(2, 2);
        kv1.set(3, 3);
        while let Some(_event) = kv1.recv().await {}
    });

    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    tokio::time::sleep(Duration::from_millis(1000)).await;
    node1_requester.connect(addr2).await.expect("should connect success");

    assert_eq!(timeout(WAIT, kv2.recv()).await, Ok(Some(KvEvent::Set(Some(addr1.peer_id()), 1, 1))));
    assert_eq!(timeout(WAIT, kv2.recv()).await, Ok(Some(KvEvent::Set(Some(addr1.peer_id()), 2, 2))));
    assert_eq!(timeout(WAIT, kv2.recv()).await, Ok(Some(KvEvent::Set(Some(addr1.peer_id()), 3, 3))));
}

#[test(tokio::test)]
async fn continuous_sync() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let (mut node2, _addr2) = create_node(true, 2, vec![addr1.clone()]).await;

    let mut kv1: ReplicatedKvService<u16, u16> = ReplicatedKvService::new(node1.create_service(0.into()), 10, 3);
    let mut kv2: ReplicatedKvService<u16, u16> = ReplicatedKvService::new(node2.create_service(0.into()), 10, 3);

    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    tokio::time::sleep(Duration::from_millis(1000)).await;

    tokio::spawn(async move {
        kv1.set(1, 1);
        kv1.set(2, 2);
        kv1.set(3, 3);
        while let Some(_event) = kv1.recv().await {}
    });

    assert_eq!(timeout(WAIT, kv2.recv()).await, Ok(Some(KvEvent::Set(Some(addr1.peer_id()), 1, 1))));
    assert_eq!(timeout(WAIT, kv2.recv()).await, Ok(Some(KvEvent::Set(Some(addr1.peer_id()), 2, 2))));
    assert_eq!(timeout(WAIT, kv2.recv()).await, Ok(Some(KvEvent::Set(Some(addr1.peer_id()), 3, 3))));
}

#[test]
fn fuzz_replicated_kv_convergence_under_network_gaps() {
    use std::collections::HashMap;
    use rand::{SeedableRng, Rng};
    use rand::rngs::StdRng;
    use crate::replicate_kv_service::{
        ReplicatedKvStore,
        messages::{Event, NetEvent, BroadcastEvent, BroadcastEventData}
    };
    use crate::PeerId;

    // Fixed seed for reproducible run
    let mut rng = StdRng::seed_from_u64(12345);

    let node1 = PeerId::from(1);
    let node2 = PeerId::from(2);

    let mut store1 = ReplicatedKvStore::new(100, 3);
    let mut store2 = ReplicatedKvStore::new(100, 3);

    struct Packet {
        from: PeerId,
        to: PeerId,
        event: NetEvent<PeerId, u16, u16>,
        deliver_at: usize,
    }
    let mut packet_queue: Vec<Packet> = Vec::new();

    let mut expected_kv: HashMap<u16, u16> = HashMap::new();

    let mut step = 0;
    let max_steps = 3000;

    while step < max_steps {
        // 1. Mutate node1 randomly during the first 1000 steps
        if step < 1000 && rng.gen_ratio(1, 15) {
            let key = rng.gen_range(0..20);
            let val = rng.gen_range(0..1000);
            store1.set(key, val);
            expected_kv.insert(key, val);
        }

        // 2. Pop events from store1 and queue them
        while let Some(event) = store1.pop() {
            if let Event::NetEvent(net_event) = event {
                let to = match &net_event {
                    NetEvent::Broadcast(_) => node2,
                    NetEvent::Unicast(dest, _) => *dest,
                };
                
                let is_unstable = step < 1500;
                let is_broadcast = matches!(net_event, NetEvent::Broadcast(_));
                let is_changed_broadcast = matches!(net_event, NetEvent::Broadcast(BroadcastEvent { data: BroadcastEventData::Changed(_), .. }));
                
                let drop_rate = if step >= 900 && is_changed_broadcast {
                    100
                } else if is_unstable && is_broadcast {
                    30
                } else {
                    0
                };
                let max_delay = if is_unstable { 100 } else { 0 };
                
                if !rng.gen_ratio(drop_rate, 100) {
                    let delay = rng.gen_range(0..=max_delay);
                    packet_queue.push(Packet {
                        from: node1,
                        to,
                        event: net_event,
                        deliver_at: step + delay,
                    });
                }
            }
        }

        // 3. Pop events from store2 and queue them
        while let Some(event) = store2.pop() {
            if let Event::NetEvent(net_event) = event {
                let to = match &net_event {
                    NetEvent::Broadcast(_) => node1,
                    NetEvent::Unicast(dest, _) => *dest,
                };
                
                let is_unstable = step < 1500;
                let is_broadcast = matches!(net_event, NetEvent::Broadcast(_));
                let is_changed_broadcast = matches!(net_event, NetEvent::Broadcast(BroadcastEvent { data: BroadcastEventData::Changed(_), .. }));
                
                let drop_rate = if step >= 900 && is_changed_broadcast {
                    100
                } else if is_unstable && is_broadcast {
                    30
                } else {
                    0
                };
                let max_delay = if is_unstable { 100 } else { 0 };
                
                if !rng.gen_ratio(drop_rate, 100) {
                    let delay = rng.gen_range(0..=max_delay);
                    packet_queue.push(Packet {
                        from: node2,
                        to,
                        event: net_event,
                        deliver_at: step + delay,
                    });
                }
            }
        }

        // 4. Tick both stores every 10 steps to simulate 1s tick
        if step % 10 == 0 {
            if step < 1000 {
                store1.on_tick();
            }
            store2.on_tick();
        }

        // 5. Deliver packets scheduled for this step
        let mut i = 0;
        while i < packet_queue.len() {
            if packet_queue[i].deliver_at <= step {
                let packet = packet_queue.remove(i);
                if packet.to == node1 {
                    store1.on_remote_event(packet.from, packet.event);
                } else if packet.to == node2 {
                    store2.on_remote_event(packet.from, packet.event);
                }
            } else {
                i += 1;
            }
        }

        step += 1;
    }

    // 6. Deliver all remaining packets and tick to settle down in stable phase
    for _ in 0..200 {
        while !packet_queue.is_empty() {
            let packet = packet_queue.remove(0);
            if packet.to == node1 {
                store1.on_remote_event(packet.from, packet.event);
            } else if packet.to == node2 {
                store2.on_remote_event(packet.from, packet.event);
            }
        }
        
        store2.on_tick();
        
        // Pop any new network packets generated by retries/ticks
        while let Some(event) = store1.pop() {
            if let Event::NetEvent(net_event) = event {
                let to = match &net_event {
                    NetEvent::Broadcast(_) => node2,
                    NetEvent::Unicast(dest, _) => *dest,
                };
                packet_queue.push(Packet {
                    from: node1,
                    to,
                    event: net_event,
                    deliver_at: 0,
                });
            }
        }
        while let Some(event) = store2.pop() {
            if let Event::NetEvent(net_event) = event {
                let to = match &net_event {
                    NetEvent::Broadcast(_) => node1,
                    NetEvent::Unicast(dest, _) => *dest,
                };
                packet_queue.push(Packet {
                    from: node2,
                    to,
                    event: net_event,
                    deliver_at: 0,
                });
            }
        }
    }

    // Assert that the replica (node2) has converged to the exact same state as node1
    let remote_store = store2.remotes.get(&node1).expect("remote store for node1 on node2 must exist");
    
    for (k, slot1) in &store1.local.slots {
        let slot2 = remote_store.ctx.slots.get(k).expect("key should exist on remote node2");
        assert_eq!(slot1.value, slot2.value, "Value mismatch for key {:?}", k);
        assert_eq!(slot1.version, slot2.version, "Version mismatch for key {:?}", k);
    }
    
    assert_eq!(remote_store.ctx.slots.len(), store1.local.slots.len(), "Remote storage has different number of keys");
}

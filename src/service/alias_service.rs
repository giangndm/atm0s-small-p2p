use std::{
    collections::{HashMap, HashSet, VecDeque},
    time::Duration,
};

use anyhow::anyhow;
use derive_more::derive::{Display, From};
use lru::LruCache;
use metrics::{counter, gauge};
use serde::{Deserialize, Serialize};
use tokio::{
    select,
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    time::Interval,
};

use crate::{
    stats::{P2P_ALIAS_CACHE_POP, P2P_ALIAS_CACHE_SIZE, P2P_ALIAS_CACHE_UPSERT},
    stream::P2pQuicStream,
    utils::{now_ms, ErrorExt, ErrorExt2},
    PeerId, P2P_ALIAS_FIND_REQUEST, P2P_ALIAS_LIVE_FIND_REQUEST,
};

use super::{P2pService, P2pServiceEvent, P2pServiceRequester};

const LRU_CACHE_SIZE: usize = 1_000_000;
const HINT_TIMEOUT_MS: u64 = 500;
const SCAN_TIMEOUT_MS: u64 = 1000;

#[derive(Debug, From, Display, Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Copy)]
pub struct AliasId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AliasFoundLocation {
    Local,
    Hint(PeerId),
    Scan(PeerId),
}

pub enum AliasStreamLocation {
    Local,
    Hint(P2pQuicStream),
    Scan(P2pQuicStream),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum AliasMessage {
    NotifySet(AliasId),
    NotifyDel(AliasId),
    Check(AliasId),
    Scan(AliasId),
    Found(AliasId),
    NotFound(AliasId),
    // when a node
    Shutdown,
}

enum AliasControl {
    Register(AliasId),
    Unregister(AliasId),
    Find(AliasId, oneshot::Sender<Option<AliasFoundLocation>>),
    Shutdown,
}

#[derive(Debug)]
pub struct AliasGuard {
    alias: AliasId,
    tx: UnboundedSender<AliasControl>,
}

impl Drop for AliasGuard {
    fn drop(&mut self) {
        log::info!("[AliasGuard] drop {} => auto unregister", self.alias);
        self.tx.send(AliasControl::Unregister(self.alias)).expect("alias service main channal should work");
    }
}

#[derive(Debug, Clone)]
pub struct AliasServiceRequester {
    tx: UnboundedSender<AliasControl>,
}

impl AliasServiceRequester {
    pub fn register<A: Into<AliasId>>(&self, alias: A) -> AliasGuard {
        let alias: AliasId = alias.into();
        log::info!("[AliasServiceRequester] register alias {alias}");
        self.tx.send(AliasControl::Register(alias)).expect("alias service main channal should work");

        AliasGuard { alias, tx: self.tx.clone() }
    }

    pub async fn find<A: Into<AliasId>>(&self, alias: A) -> Option<AliasFoundLocation> {
        let alias: AliasId = alias.into();
        log::info!("[AliasServiceRequester] find alias {alias}");
        let (tx, rx) = oneshot::channel();
        self.tx.send(AliasControl::Find(alias, tx)).expect("alias service main channal should work");
        let res = rx.await.ok()?;
        log::info!("[AliasServiceRequester] find alias {alias} => result {res:?}");
        res
    }

    pub async fn open_stream<A: Into<AliasId>>(&self, alias: A, over_service: P2pServiceRequester, meta: Vec<u8>) -> anyhow::Result<AliasStreamLocation> {
        match self.find(alias).await {
            Some(AliasFoundLocation::Local) => Ok(AliasStreamLocation::Local),
            Some(AliasFoundLocation::Hint(dest)) => over_service.open_stream(dest, meta).await.map(AliasStreamLocation::Hint),
            Some(AliasFoundLocation::Scan(dest)) => over_service.open_stream(dest, meta).await.map(AliasStreamLocation::Scan),
            None => Err(anyhow!("alias not found")),
        }
    }

    pub fn shutdown(&self) {
        log::info!("[AliasServiceRequester] shutdown");
        self.tx.send(AliasControl::Shutdown).expect("alias service main channal should work");
    }
}

enum FindRequestState {
    CheckHint(u64, HashSet<PeerId>),
    Scan(u64),
}

struct FindRequest {
    state: FindRequestState,
    waits: Vec<oneshot::Sender<Option<AliasFoundLocation>>>,
}

#[derive(Debug, PartialEq, Eq)]
enum InternalOutput {
    Broadcast(AliasMessage),
    Unicast(PeerId, AliasMessage),
}

struct AliasServiceInternal {
    local: HashMap<AliasId, u8>,
    cache: LruCache<AliasId, HashSet<PeerId>>,
    find_reqs: HashMap<AliasId, FindRequest>,
    outs: VecDeque<InternalOutput>,
}

pub struct AliasService {
    service: P2pService,
    tx: UnboundedSender<AliasControl>,
    rx: UnboundedReceiver<AliasControl>,
    internal: AliasServiceInternal,
    interval: Interval,
}

impl AliasService {
    pub fn new(service: P2pService) -> Self {
        let (tx, rx) = unbounded_channel();
        Self {
            service,
            tx,
            rx,
            internal: AliasServiceInternal {
                cache: LruCache::new(LRU_CACHE_SIZE.try_into().expect("")),
                find_reqs: HashMap::new(),
                outs: VecDeque::new(),
                local: HashMap::new(),
            },
            interval: tokio::time::interval(Duration::from_secs(1)),
        }
    }

    pub fn requester(&self) -> AliasServiceRequester {
        AliasServiceRequester { tx: self.tx.clone() }
    }

    pub async fn run_loop(&mut self) -> anyhow::Result<()> {
        loop {
            select! {
                _ = self.interval.tick() => {
                    self.on_tick().await;
                },
                event = self.service.recv() => match event.expect("service channel should work") {
                    P2pServiceEvent::Unicast(from, data) => {
                        if let Ok(msg) = bincode::deserialize::<AliasMessage>(&data) {
                            self.on_msg(from, msg).await;
                        }
                    }
                    P2pServiceEvent::Broadcast(from, data) => {
                        if let Ok(msg) = bincode::deserialize::<AliasMessage>(&data) {
                            self.on_msg(from, msg).await;
                        }
                    }
                    P2pServiceEvent::Stream(..) => {},
                },
                control = self.rx.recv() => {
                    self.on_control(control.expect("service channel should work")).await;
                }
            }
        }
    }

    async fn on_tick(&mut self) {
        self.internal.on_tick(now_ms());
        self.pop_internal().await;
    }

    async fn on_msg(&mut self, from: PeerId, msg: AliasMessage) {
        log::debug!("[AliasService] on msg from {from}, {msg:?}");
        self.internal.on_msg(now_ms(), from, msg);
        self.pop_internal().await;
    }

    async fn on_control(&mut self, control: AliasControl) {
        self.internal.on_control(now_ms(), control);
        self.pop_internal().await;
    }

    async fn pop_internal(&mut self) {
        while let Some(out) = self.internal.pop_output() {
            match out {
                InternalOutput::Broadcast(msg) => {
                    self.service.send_broadcast(bincode::serialize(&msg).expect("should serialie")).await;
                }
                InternalOutput::Unicast(dest, msg) => {
                    self.service
                        .send_unicast(dest, bincode::serialize(&msg).expect("should serialie"))
                        .await
                        .print_on_err("[AliasService] send unicast");
                }
            }
        }
    }
}

impl AliasServiceInternal {
    fn on_tick(&mut self, now: u64) {
        let mut timeout_reqs = vec![];
        for (alias_id, req) in self.find_reqs.iter_mut() {
            match req.state {
                FindRequestState::CheckHint(requested_at, ref mut _hash_set) => {
                    if requested_at + HINT_TIMEOUT_MS <= now {
                        log::info!("[AliasServiceInternal] check hint timeout {alias_id} => switch to scan");
                        self.outs.push_back(InternalOutput::Broadcast(AliasMessage::Scan(*alias_id)));
                        req.state = FindRequestState::Scan(now);
                    }
                }
                FindRequestState::Scan(requested_at) => {
                    if requested_at + SCAN_TIMEOUT_MS <= now {
                        log::info!("[AliasServiceInternal] find scan timeout {alias_id}");
                        timeout_reqs.push(*alias_id);
                        while let Some(tx) = req.waits.pop() {
                            tx.send(None).print_on_err2("");
                        }
                    }
                }
            }
        }

        for alias_id in timeout_reqs {
            if let Some(_) = self.find_reqs.remove(&alias_id) {
                gauge!(P2P_ALIAS_LIVE_FIND_REQUEST).decrement(1);
            }
        }

        self.collect_stats();
    }

    fn on_msg(&mut self, now: u64, from: PeerId, msg: AliasMessage) {
        log::info!("[AliasServiceInternal] on msg from {from}, {msg:?}");
        match msg {
            AliasMessage::NotifySet(alias_id) => {
                let slot = match self.cache.get_mut(&alias_id) {
                    Some(slot) => slot,
                    None => {
                        counter!(P2P_ALIAS_CACHE_UPSERT).increment(1);
                        self.cache.get_or_insert_mut(alias_id, HashSet::new)
                    }
                };
                slot.insert(from);
            }
            AliasMessage::NotifyDel(alias_id) => {
                if let Some(slot) = self.cache.get_mut(&alias_id) {
                    slot.remove(&from);
                    if slot.is_empty() {
                        counter!(P2P_ALIAS_CACHE_POP).increment(1);
                        self.cache.pop(&alias_id);
                    }
                }
            }
            AliasMessage::Check(alias_id) => {
                if self.local.contains_key(&alias_id) {
                    self.outs.push_back(InternalOutput::Unicast(from, AliasMessage::Found(alias_id)));
                } else {
                    self.outs.push_back(InternalOutput::Unicast(from, AliasMessage::NotFound(alias_id)));
                }
            }
            AliasMessage::Scan(alias_id) => {
                if self.local.contains_key(&alias_id) {
                    self.outs.push_back(InternalOutput::Unicast(from, AliasMessage::Found(alias_id)));
                }
            }
            AliasMessage::Found(alias_id) => {
                let slot = match self.cache.get_mut(&alias_id) {
                    Some(slot) => slot,
                    None => {
                        counter!(P2P_ALIAS_CACHE_UPSERT).increment(1);
                        self.cache.get_or_insert_mut(alias_id, HashSet::new)
                    }
                };
                slot.insert(from);

                if let Some(req) = self.find_reqs.remove(&alias_id) {
                    gauge!(P2P_ALIAS_LIVE_FIND_REQUEST).decrement(1);
                    let found = if matches!(req.state, FindRequestState::Scan(_)) {
                        AliasFoundLocation::Scan(from)
                    } else {
                        AliasFoundLocation::Hint(from)
                    };
                    for tx in req.waits {
                        tx.send(Some(found)).print_on_err2("[AliasServiceInternal] send query response");
                    }
                }
            }
            AliasMessage::NotFound(alias_id) => {
                if let Some(slot) = self.cache.get_mut(&alias_id) {
                    slot.remove(&from);
                    if slot.is_empty() {
                        counter!(P2P_ALIAS_CACHE_POP).increment(1);
                        self.cache.pop(&alias_id);
                    }
                }

                if let Some(req) = self.find_reqs.get_mut(&alias_id) {
                    match req.state {
                        FindRequestState::CheckHint(_, ref mut hint_peers) => {
                            hint_peers.remove(&from);
                            if hint_peers.is_empty() {
                                //not found => should switch to scan
                                req.state = FindRequestState::Scan(now);
                                self.outs.push_back(InternalOutput::Broadcast(AliasMessage::Scan(alias_id)));
                            }
                        }
                        FindRequestState::Scan(_) => {}
                    }
                }
            }
            AliasMessage::Shutdown => {
                let mut removed_alias_ids = vec![];
                for (k, _v) in &mut self.cache {
                    removed_alias_ids.push(*k);
                }
                for alias_id in removed_alias_ids {
                    if let Some(_) = self.cache.pop(&alias_id) {
                        counter!(P2P_ALIAS_CACHE_POP).increment(1);
                    }
                }
            }
        }
    }

    fn on_control(&mut self, now: u64, control: AliasControl) {
        match control {
            AliasControl::Register(alias_id) => {
                let ref_count = self.local.entry(alias_id).or_default();
                *ref_count += 1;
                self.outs.push_back(InternalOutput::Broadcast(AliasMessage::NotifySet(alias_id)));
            }
            AliasControl::Unregister(alias_id) => {
                if let Some(ref_count) = self.local.get_mut(&alias_id) {
                    *ref_count -= 1;
                    if *ref_count == 0 {
                        self.local.remove(&alias_id);
                        self.outs.push_back(InternalOutput::Broadcast(AliasMessage::NotifyDel(alias_id)));
                    }
                }
            }
            AliasControl::Find(alias_id, sender) => {
                if let Some(req) = self.find_reqs.get_mut(&alias_id) {
                    req.waits.push(sender);
                    return;
                }

                if self.local.contains_key(&alias_id) {
                    sender.send(Some(AliasFoundLocation::Local)).print_on_err2("[AliasServiceInternal] send query response");
                } else if let Some(slot) = self.cache.get(&alias_id) {
                    for peer in slot {
                        self.outs.push_back(InternalOutput::Unicast(*peer, AliasMessage::Check(alias_id)));
                    }
                    gauge!(P2P_ALIAS_LIVE_FIND_REQUEST).increment(1);
                    counter!(P2P_ALIAS_FIND_REQUEST).increment(1);
                    self.find_reqs.insert(
                        alias_id,
                        FindRequest {
                            state: FindRequestState::CheckHint(now, slot.clone()),
                            waits: vec![sender],
                        },
                    );
                } else {
                    self.outs.push_back(InternalOutput::Broadcast(AliasMessage::Scan(alias_id)));
                    gauge!(P2P_ALIAS_LIVE_FIND_REQUEST).increment(1);
                    counter!(P2P_ALIAS_FIND_REQUEST).increment(1);
                    self.find_reqs.insert(
                        alias_id,
                        FindRequest {
                            state: FindRequestState::Scan(now),
                            waits: vec![sender],
                        },
                    );
                }
            }
            AliasControl::Shutdown => {
                self.outs.push_back(InternalOutput::Broadcast(AliasMessage::Shutdown));
            }
        }
    }

    fn pop_output(&mut self) -> Option<InternalOutput> {
        self.outs.pop_front()
    }

    pub fn collect_stats(&self) {
        let cache_size = self.cache.len();
        gauge!(P2P_ALIAS_CACHE_SIZE).set(cache_size as f64);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{ctx::SharedCtx, msg::P2pServiceId, router::SharedRouterTable};
    use futures::FutureExt;

    struct TestContext {
        internal: AliasServiceInternal,
        now: u64,
    }

    impl TestContext {
        fn new() -> Self {
            Self {
                internal: AliasServiceInternal {
                    local: HashMap::new(),
                    cache: LruCache::new(LRU_CACHE_SIZE.try_into().expect("should create NoneZeroUsize")),
                    find_reqs: HashMap::new(),
                    outs: VecDeque::new(),
                },
                now: 1000,
            }
        }

        fn advance_time(&mut self, ms: u64) {
            self.now += ms;
        }

        fn collect_outputs(&mut self) -> Vec<InternalOutput> {
            let mut outputs = Vec::new();
            while let Some(output) = self.internal.pop_output() {
                outputs.push(output);
            }
            outputs
        }
    }

    fn test_service() -> AliasService {
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (service, _tx) = P2pService::build(P2pServiceId::from(0), ctx);
        AliasService::new(service)
    }

    #[tokio::test]
    async fn alias_internal_control_backlog_must_be_bounded() {
        const MAX_PENDING_CONTROLS: usize = 1024;
        let service = test_service();
        let requester = service.requester();
        let mut guards = Vec::new();

        for alias in 0..=MAX_PENDING_CONTROLS {
            guards.push(requester.register(AliasId(alias as u64 + 10)));
        }

        assert!(
            service.rx.len() <= MAX_PENDING_CONTROLS,
            "pending alias internal control messages must be bounded, got {}",
            service.rx.len()
        );
    }

    #[tokio::test]
    async fn alias_run_loop_after_base_service_close_must_not_panic() {
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (base_service, service_tx) = P2pService::build(P2pServiceId::from(0), ctx);
        let mut service = AliasService::new(base_service);
        drop(service_tx);

        let result = std::panic::AssertUnwindSafe(service.run_loop()).catch_unwind().await;

        assert!(
            matches!(result, Ok(Err(_))),
            "closed base service channel should make alias run_loop return Err instead of panicking, got {result:?}"
        );
    }

    #[tokio::test]
    async fn alias_run_loop_after_control_channel_close_must_not_panic() {
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (base_service, _service_tx) = P2pService::build(P2pServiceId::from(0), ctx);
        let mut service = AliasService::new(base_service);
        let (replacement_tx, _replacement_rx) = unbounded_channel();
        service.tx = replacement_tx;

        let result = std::panic::AssertUnwindSafe(service.run_loop()).catch_unwind().await;

        assert!(
            matches!(result, Ok(Err(_))),
            "closed alias control channel should make run_loop return Err instead of panicking, got {result:?}"
        );
    }

    #[test]
    fn test_register_alias() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);

        // Test registering an alias
        ctx.internal.on_control(ctx.now, AliasControl::Register(alias_id));

        // Verify local set contains the alias
        assert!(ctx.internal.local.contains_key(&alias_id));

        // Verify broadcast message
        let outputs = ctx.collect_outputs();
        assert_eq!(outputs.len(), 1);
        match &outputs[0] {
            InternalOutput::Broadcast(AliasMessage::NotifySet(id)) => assert_eq!(*id, alias_id),
            _ => panic!("Expected broadcast NotifySet message"),
        }
    }

    #[test]
    fn test_unregister_alias() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);

        // Register first
        ctx.internal.on_control(ctx.now, AliasControl::Register(alias_id));
        ctx.collect_outputs(); // Clear outputs

        // Test unregistering
        ctx.internal.on_control(ctx.now, AliasControl::Unregister(alias_id));

        // Verify local set doesn't contain the alias
        assert!(!ctx.internal.local.contains_key(&alias_id));

        // Verify broadcast message
        let outputs = ctx.collect_outputs();
        assert_eq!(outputs.len(), 1);
        match &outputs[0] {
            InternalOutput::Broadcast(AliasMessage::NotifyDel(id)) => assert_eq!(*id, alias_id),
            _ => panic!("Expected broadcast NotifyDel message"),
        }
    }

    #[test]
    fn registering_same_alias_many_times_must_not_overflow_refcount() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            for _ in 0..=u8::MAX {
                ctx.internal.on_control(ctx.now, AliasControl::Register(alias_id));
            }
        }));

        assert!(result.is_ok(), "alias registration refcount must not panic or wrap after 256 local guards");
    }

    #[test]
    fn test_find_local_alias() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);

        // Register alias locally
        ctx.internal.on_control(ctx.now, AliasControl::Register(alias_id));
        ctx.collect_outputs(); // Clear outputs

        // Create a oneshot channel for the find response
        let (tx, mut rx) = oneshot::channel();

        // Test finding the local alias
        ctx.internal.on_control(ctx.now, AliasControl::Find(alias_id, tx));

        // Verify response
        let response = rx.try_recv().expect("Should have a response");
        assert_eq!(response, Some(AliasFoundLocation::Local));

        // Verify no outputs (shouldn't need to broadcast for local find)
        let outputs = ctx.collect_outputs();
        assert!(outputs.is_empty());
    }

    #[test]
    fn test_find_cached_alias_found() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);
        let peer_addr = PeerId(1);

        // Add alias to cache
        ctx.internal.on_msg(ctx.now, peer_addr, AliasMessage::NotifySet(alias_id));

        // Create a oneshot channel for the find response
        let (tx, mut rx) = oneshot::channel();

        // Test finding the cached alias
        ctx.internal.on_control(ctx.now, AliasControl::Find(alias_id, tx));

        // Verify unicast message to check with cached peer
        let outputs = ctx.collect_outputs();
        assert_eq!(outputs, vec![InternalOutput::Unicast(peer_addr, AliasMessage::Check(alias_id))]);

        // Simulate peer response
        ctx.internal.on_msg(ctx.now, peer_addr, AliasMessage::Found(alias_id));

        // Verify find response
        let response = rx.try_recv().expect("Should have a response");
        assert_eq!(response, Some(AliasFoundLocation::Hint(peer_addr)));
    }

    #[test]
    fn cached_hint_find_must_ignore_found_from_unchecked_peer() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);
        let hinted_peer = PeerId(1);
        let unchecked_peer = PeerId(2);

        ctx.internal.on_msg(ctx.now, hinted_peer, AliasMessage::NotifySet(alias_id));

        let (tx, mut rx) = oneshot::channel();
        ctx.internal.on_control(ctx.now, AliasControl::Find(alias_id, tx));

        assert_eq!(ctx.collect_outputs(), vec![InternalOutput::Unicast(hinted_peer, AliasMessage::Check(alias_id))]);

        ctx.internal.on_msg(ctx.now, unchecked_peer, AliasMessage::Found(alias_id));

        assert!(rx.try_recv().is_err(), "cached hint lookup must not complete from a peer that was not checked");
        assert!(
            ctx.internal.find_reqs.contains_key(&alias_id),
            "cached hint lookup must remain pending after an unchecked peer replies Found"
        );
        assert!(
            !ctx.internal.cache.get(&alias_id).is_some_and(|peers| peers.contains(&unchecked_peer)),
            "unchecked Found responses must not poison the alias hint cache"
        );
    }

    #[test]
    fn unsolicited_found_must_not_create_alias_cache_hint() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);
        let unsolicited_peer = PeerId(2);

        ctx.internal.on_msg(ctx.now, unsolicited_peer, AliasMessage::Found(alias_id));

        assert!(
            !ctx.internal.cache.contains(&alias_id),
            "unsolicited Found messages without a pending lookup must not create alias cache hints"
        );
    }

    #[test]
    fn stale_not_found_must_not_evict_alias_cache_without_pending_check() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(7);
        let hinted_peer = PeerId(2);

        ctx.internal.on_msg(ctx.now, hinted_peer, AliasMessage::NotifySet(alias_id));
        assert!(
            ctx.internal.cache.get(&alias_id).is_some_and(|peers| peers.contains(&hinted_peer)),
            "test setup should cache the hinted peer"
        );
        assert!(
            !ctx.internal.find_reqs.contains_key(&alias_id),
            "test setup should have no active lookup for this alias"
        );

        ctx.internal.on_msg(ctx.now, hinted_peer, AliasMessage::NotFound(alias_id));

        assert!(
            ctx.internal.cache.get(&alias_id).is_some_and(|peers| peers.contains(&hinted_peer)),
            "stale NotFound without a matching pending CheckHint request must not evict a valid cached hint"
        );
    }

    #[test]
    fn stale_notify_set_must_not_resurrect_alias_after_newer_notify_del() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(7);
        let peer = PeerId(2);

        ctx.internal.on_msg(ctx.now, peer, AliasMessage::NotifySet(alias_id));
        assert!(ctx.internal.cache.get(&alias_id).is_some_and(|peers| peers.contains(&peer)));

        ctx.internal.on_msg(ctx.now + 1, peer, AliasMessage::NotifyDel(alias_id));
        assert!(!ctx.internal.cache.contains(&alias_id));

        ctx.internal.on_msg(ctx.now + 2, peer, AliasMessage::NotifySet(alias_id));

        assert!(
            !ctx.internal.cache.contains(&alias_id),
            "stale NotifySet must not resurrect an alias hint removed by a newer NotifyDel"
        );

        let (tx, _rx) = oneshot::channel();
        ctx.internal.on_control(ctx.now + 3, AliasControl::Find(alias_id, tx));

        assert_eq!(
            ctx.collect_outputs(),
            vec![InternalOutput::Broadcast(AliasMessage::Scan(alias_id))],
            "find must not use a resurrected stale hint"
        );
    }

    #[test]
    fn cached_alias_peer_hints_must_be_bounded() {
        const MAX_PEERS_PER_ALIAS: usize = 1024;
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);

        for peer in 0..=MAX_PEERS_PER_ALIAS {
            ctx.internal.on_msg(ctx.now, PeerId::from(peer as u64 + 10), AliasMessage::NotifySet(alias_id));
        }

        let cached_peers = ctx.internal.cache.get(&alias_id).expect("alias should be cached").len();

        assert!(cached_peers <= MAX_PEERS_PER_ALIAS, "cached peer hints for one alias must be bounded, got {cached_peers}");
    }

    #[test]
    fn test_find_cached_alias_not_found() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);
        let peer_addr = PeerId(1);

        // Add alias to cache
        ctx.internal.on_msg(ctx.now, peer_addr, AliasMessage::NotifySet(alias_id));

        // Create a oneshot channel for the find response
        let (tx, _rx) = oneshot::channel();

        // Test finding the cached alias
        ctx.internal.on_control(ctx.now, AliasControl::Find(alias_id, tx));

        // Verify unicast message to check with cached peer
        let outputs = ctx.collect_outputs();
        assert_eq!(outputs, vec![InternalOutput::Unicast(peer_addr, AliasMessage::Check(alias_id))]);

        // Simulate peer response
        ctx.internal.on_msg(ctx.now, peer_addr, AliasMessage::NotFound(alias_id));

        // Verify broadcast scan message
        let outputs = ctx.collect_outputs();
        assert_eq!(outputs, vec![InternalOutput::Broadcast(AliasMessage::Scan(alias_id))]);
    }

    #[test]
    fn shutdown_from_cached_hint_must_unblock_pending_find() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);
        let hinted_peer = PeerId(2);

        ctx.internal.on_msg(ctx.now, hinted_peer, AliasMessage::NotifySet(alias_id));

        let (tx, mut rx) = oneshot::channel();
        ctx.internal.on_control(ctx.now, AliasControl::Find(alias_id, tx));

        assert_eq!(ctx.collect_outputs(), vec![InternalOutput::Unicast(hinted_peer, AliasMessage::Check(alias_id))]);

        ctx.internal.on_msg(ctx.now, hinted_peer, AliasMessage::Shutdown);

        assert_eq!(
            ctx.collect_outputs(),
            vec![InternalOutput::Broadcast(AliasMessage::Scan(alias_id))],
            "shutdown from the only cached hint must immediately fail over instead of waiting for hint timeout"
        );
        assert!(
            rx.try_recv().is_err(),
            "lookup should remain pending while it fails over to scan, not complete from a stopped hint"
        );
    }

    #[test]
    fn pending_find_must_prefer_late_local_registration_over_remote_found() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);
        let remote_peer = PeerId(2);

        let (tx, mut rx) = oneshot::channel();
        ctx.internal.on_control(ctx.now, AliasControl::Find(alias_id, tx));
        assert_eq!(ctx.collect_outputs(), vec![InternalOutput::Broadcast(AliasMessage::Scan(alias_id))]);

        ctx.internal.on_control(ctx.now, AliasControl::Register(alias_id));
        ctx.collect_outputs();

        ctx.internal.on_msg(ctx.now, remote_peer, AliasMessage::Found(alias_id));

        assert_eq!(
            rx.try_recv().expect("pending find should complete"),
            Some(AliasFoundLocation::Local),
            "a pending alias find must resolve to Local once the alias is registered locally, not to a later remote Found"
        );
        assert!(
            !ctx.internal.find_reqs.contains_key(&alias_id),
            "local registration should clear the obsolete pending find request"
        );
    }

    #[test]
    fn test_find_cached_alias_timeout_switch_to_scan() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);
        let peer_addr = PeerId(1);

        // Add alias to cache
        ctx.internal.on_msg(ctx.now, peer_addr, AliasMessage::NotifySet(alias_id));

        // Create a oneshot channel for the find response
        let (tx, _rx) = oneshot::channel();

        // Test finding the cached alias
        ctx.internal.on_control(ctx.now, AliasControl::Find(alias_id, tx));

        // Verify unicast message to check with cached peer
        let outputs = ctx.collect_outputs();
        assert_eq!(outputs, vec![InternalOutput::Unicast(peer_addr, AliasMessage::Check(alias_id))]);

        // Simulate timeout
        ctx.advance_time(HINT_TIMEOUT_MS + 1);
        ctx.internal.on_tick(ctx.now);

        let outputs = ctx.collect_outputs();
        assert_eq!(outputs, vec![InternalOutput::Broadcast(AliasMessage::Scan(alias_id))]);
    }

    #[test]
    fn test_find_timeout() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);

        // Create a oneshot channel for the find response
        let (tx, mut rx) = oneshot::channel();

        // Test finding a non-existent alias
        ctx.internal.on_control(ctx.now, AliasControl::Find(alias_id, tx));

        // Verify broadcast scan message
        let outputs = ctx.collect_outputs();
        assert_eq!(outputs, vec![InternalOutput::Broadcast(AliasMessage::Scan(alias_id))]);

        // Advance time past timeout
        ctx.advance_time(SCAN_TIMEOUT_MS + 1);
        ctx.internal.on_tick(ctx.now);

        // Verify timeout response
        let response = rx.try_recv().expect("Should have a response");
        assert_eq!(response, None);
    }

    #[test]
    fn duplicate_find_waiters_for_same_alias_must_be_bounded() {
        const MAX_WAITERS_PER_ALIAS: usize = 1024;
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);

        for _ in 0..=MAX_WAITERS_PER_ALIAS {
            let (tx, _rx) = oneshot::channel();
            ctx.internal.on_control(ctx.now, AliasControl::Find(alias_id, tx));
        }

        let waiters = ctx.internal.find_reqs.get(&alias_id).expect("find request should exist").waits.len();

        assert!(waiters <= MAX_WAITERS_PER_ALIAS, "duplicate find waiters for one alias must be bounded, got {waiters}");
    }

    #[test]
    fn distinct_pending_find_requests_must_be_bounded() {
        const MAX_PENDING_FINDS: usize = 1024;
        let mut ctx = TestContext::new();

        for id in 0..=MAX_PENDING_FINDS {
            let (tx, _rx) = oneshot::channel();
            ctx.internal.on_control(ctx.now, AliasControl::Find(AliasId(id as u64), tx));
        }

        let pending_finds = ctx.internal.find_reqs.len();
        let pending_scans = ctx.internal.outs.len();

        assert!(pending_finds <= MAX_PENDING_FINDS, "pending alias find requests must be bounded, got {pending_finds}");
        assert!(pending_scans <= MAX_PENDING_FINDS, "pending alias scan fanout must be bounded, got {pending_scans}");
    }

    #[test]
    fn find_timeout_at_max_timestamp_must_not_overflow() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);
        let (tx, _rx) = oneshot::channel();

        ctx.internal.on_control(u64::MAX - 10, AliasControl::Find(alias_id, tx));

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ctx.internal.on_tick(u64::MAX);
        }));

        assert!(result.is_ok(), "alias find timeout arithmetic must not panic or wrap near u64::MAX");
    }

    #[test]
    fn local_shutdown_must_fail_pending_alias_finds() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);

        let (tx, mut rx) = oneshot::channel();
        ctx.internal.on_control(ctx.now, AliasControl::Find(alias_id, tx));

        assert_eq!(ctx.collect_outputs(), vec![InternalOutput::Broadcast(AliasMessage::Scan(alias_id))]);

        ctx.internal.on_control(ctx.now + 1, AliasControl::Shutdown);

        assert_eq!(
            rx.try_recv(),
            Ok(None),
            "local alias shutdown must immediately fail pending find waiters"
        );
        assert!(
            !ctx.internal.find_reqs.contains_key(&alias_id),
            "local alias shutdown must clear pending find state"
        );
        assert_eq!(ctx.collect_outputs(), vec![InternalOutput::Broadcast(AliasMessage::Shutdown)]);
    }

    #[test]
    fn test_shutdown() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);
        let peer_addr = PeerId(1);

        // Add some data to cache
        let mut peers = HashSet::new();
        peers.insert(peer_addr);
        ctx.internal.cache.put(alias_id, peers);

        // Test shutdown
        ctx.internal.on_control(ctx.now, AliasControl::Shutdown);

        // Verify broadcast shutdown message
        let outputs = ctx.collect_outputs();
        assert_eq!(outputs, vec![InternalOutput::Broadcast(AliasMessage::Shutdown)]);

        // Simulate receiving shutdown message
        ctx.internal.on_msg(ctx.now, peer_addr, AliasMessage::Shutdown);

        // Verify cache is cleared
        assert!(ctx.internal.cache.is_empty());
    }

    #[test]
    fn shutdown_from_one_peer_must_not_clear_aliases_from_other_peers() {
        let mut ctx = TestContext::new();
        let alias_from_peer1 = AliasId(1);
        let alias_from_peer2 = AliasId(2);
        let peer1 = PeerId(1);
        let peer2 = PeerId(2);

        ctx.internal.on_msg(ctx.now, peer1, AliasMessage::NotifySet(alias_from_peer1));
        ctx.internal.on_msg(ctx.now, peer2, AliasMessage::NotifySet(alias_from_peer2));

        ctx.internal.on_msg(ctx.now, peer1, AliasMessage::Shutdown);

        assert!(
            ctx.internal.cache.contains(&alias_from_peer2),
            "shutdown from one peer must not remove alias hints learned from other peers"
        );
    }
}

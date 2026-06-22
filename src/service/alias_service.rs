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
        mpsc::{channel, error::TrySendError, Receiver, Sender},
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
const ALIAS_LIFECYCLE_CACHE_SIZE: usize = 1_000_000;
pub(crate) const ALIAS_CONTROL_QUEUE_SIZE: usize = 1024;
const MAX_ALIAS_HINT_PEERS: usize = 1024;
const MAX_WAITERS_PER_ALIAS: usize = 1024;
const MAX_PENDING_FIND_REQUESTS: usize = 1024;
const HINT_TIMEOUT_MS: u64 = 500;
const SCAN_TIMEOUT_MS: u64 = 1000;

// If the deadline cannot be represented, keep the request alive instead of
// wrapping and expiring it early.
fn deadline_expired(requested_at: u64, timeout_ms: u64, now: u64) -> bool {
    requested_at.checked_add(timeout_ms).is_some_and(|deadline| deadline <= now)
}

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
    NotifySet(AliasId, u64),
    NotifyDel(AliasId, u64),
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
    tx: Sender<AliasControl>,
}

impl Drop for AliasGuard {
    fn drop(&mut self) {
        log::info!("[AliasGuard] drop {} => auto unregister", self.alias);
        try_send_alias_control(&self.tx, AliasControl::Unregister(self.alias), "guard drop unregister");
    }
}

#[derive(Debug, Clone)]
pub struct AliasServiceRequester {
    tx: Sender<AliasControl>,
}

impl AliasServiceRequester {
    pub fn register<A: Into<AliasId>>(&self, alias: A) -> anyhow::Result<AliasGuard> {
        let alias: AliasId = alias.into();
        log::info!("[AliasServiceRequester] register alias {alias}");
        send_alias_control(&self.tx, AliasControl::Register(alias), "requester register")?;

        Ok(AliasGuard { alias, tx: self.tx.clone() })
    }

    pub async fn find<A: Into<AliasId>>(&self, alias: A) -> Option<AliasFoundLocation> {
        let alias: AliasId = alias.into();
        log::info!("[AliasServiceRequester] find alias {alias}");
        let (tx, rx) = oneshot::channel();
        if !try_send_alias_control(&self.tx, AliasControl::Find(alias, tx), "requester find") {
            return None;
        }
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
        try_send_alias_control(&self.tx, AliasControl::Shutdown, "requester shutdown");
    }
}

fn try_send_alias_control(tx: &Sender<AliasControl>, msg: AliasControl, context: &str) -> bool {
    match send_alias_control(tx, msg, context) {
        Ok(()) => true,
        Err(err) => {
            log::debug!("[AliasService] {err}");
            false
        }
    }
}

fn send_alias_control(tx: &Sender<AliasControl>, msg: AliasControl, context: &str) -> anyhow::Result<()> {
    tx.try_send(msg).map_err(|err| match err {
        TrySendError::Full(_) => anyhow!("alias control queue full while handling {context}"),
        TrySendError::Closed(_) => anyhow!("alias control queue closed while handling {context}"),
    })
}

enum FindRequestState {
    CheckHint(u64, HashSet<PeerId>),
    Scan(u64),
}

struct FindRequest {
    state: FindRequestState,
    waits: Vec<oneshot::Sender<Option<AliasFoundLocation>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RemoteAliasState {
    generation: u64,
    active: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum InternalOutput {
    Broadcast(AliasMessage),
    Unicast(PeerId, AliasMessage),
}

struct AliasServiceInternal {
    local: HashMap<AliasId, usize>,
    local_generations: HashMap<AliasId, u64>,
    shutting_down: bool,
    cache: LruCache<AliasId, HashSet<PeerId>>,
    remote_lifecycle: LruCache<(AliasId, PeerId), RemoteAliasState>,
    find_reqs: HashMap<AliasId, FindRequest>,
    outs: VecDeque<InternalOutput>,
}

pub struct AliasService {
    service: P2pService,
    tx: Sender<AliasControl>,
    rx: Receiver<AliasControl>,
    internal: AliasServiceInternal,
    interval: Interval,
}

impl AliasService {
    pub fn new(service: P2pService) -> Self {
        let (tx, rx) = channel(ALIAS_CONTROL_QUEUE_SIZE);
        Self {
            service,
            tx,
            rx,
            internal: AliasServiceInternal {
                cache: LruCache::new(LRU_CACHE_SIZE.try_into().expect("")),
                remote_lifecycle: LruCache::new(ALIAS_LIFECYCLE_CACHE_SIZE.try_into().expect("")),
                find_reqs: HashMap::new(),
                outs: VecDeque::new(),
                local: HashMap::new(),
                local_generations: HashMap::new(),
                shutting_down: false,
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
                event = self.service.recv() => match event {
                    Some(P2pServiceEvent::Unicast(from, data)) => {
                        if let Ok(msg) = bincode::deserialize::<AliasMessage>(&data) {
                            self.on_msg(from, msg).await;
                        }
                    }
                    Some(P2pServiceEvent::Broadcast(from, data)) => {
                        if let Ok(msg) = bincode::deserialize::<AliasMessage>(&data) {
                            self.on_msg(from, msg).await;
                        }
                    }
                    Some(P2pServiceEvent::Stream(..)) => {},
                    Some(P2pServiceEvent::PeerDisconnected(peer)) => {
                        self.internal.on_peer_disconnected(now_ms(), peer);
                    }
                    None => anyhow::bail!("alias base service channel closed"),
                },
                control = self.rx.recv() => {
                    let Some(control) = control else {
                        anyhow::bail!("alias control channel closed");
                    };
                    self.on_control(control).await;
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
                    self.service
                        .send_broadcast(bincode::serialize(&msg).expect("should serialie"))
                        .await
                        .print_on_err("[AliasService] send broadcast");
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
                    if deadline_expired(requested_at, HINT_TIMEOUT_MS, now) {
                        log::info!("[AliasServiceInternal] check hint timeout {alias_id} => switch to scan");
                        self.outs.push_back(InternalOutput::Broadcast(AliasMessage::Scan(*alias_id)));
                        req.state = FindRequestState::Scan(now);
                    }
                }
                FindRequestState::Scan(requested_at) => {
                    if deadline_expired(requested_at, SCAN_TIMEOUT_MS, now) {
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
            AliasMessage::NotifySet(alias_id, generation) => {
                if self.accept_remote_lifecycle(alias_id, from, generation, true) {
                    self.insert_cache_hint(alias_id, from);
                }
            }
            AliasMessage::NotifyDel(alias_id, generation) => {
                if self.accept_remote_lifecycle(alias_id, from, generation, false) {
                    self.remove_cache_hint(alias_id, from);
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
                    let generation = *self.local_generations.entry(alias_id).or_default();
                    self.outs.push_back(InternalOutput::Unicast(from, AliasMessage::NotifySet(alias_id, generation)));
                    self.outs.push_back(InternalOutput::Unicast(from, AliasMessage::Found(alias_id)));
                }
            }
            AliasMessage::Found(alias_id) => {
                let found = match self.find_reqs.get(&alias_id).map(|req| &req.state) {
                    Some(FindRequestState::CheckHint(_, hint_peers)) if hint_peers.contains(&from) => Some(AliasFoundLocation::Hint(from)),
                    Some(FindRequestState::Scan(_)) if self.remote_lifecycle.get(&(alias_id, from)).is_some_and(|state| state.active) => Some(AliasFoundLocation::Scan(from)),
                    _ => None,
                };

                if let Some(found) = found {
                    self.insert_cache_hint(alias_id, from);
                    let Some(req) = self.find_reqs.remove(&alias_id) else {
                        return;
                    };
                    gauge!(P2P_ALIAS_LIVE_FIND_REQUEST).decrement(1);
                    for tx in req.waits {
                        tx.send(Some(found)).print_on_err2("[AliasServiceInternal] send query response");
                    }
                }
            }
            AliasMessage::NotFound(alias_id) => {
                let mut should_scan = false;
                let accepted = if let Some(req) = self.find_reqs.get_mut(&alias_id) {
                    match req.state {
                        FindRequestState::CheckHint(_, ref mut hint_peers) => {
                            if hint_peers.remove(&from) {
                                if hint_peers.is_empty() {
                                    //not found => should switch to scan
                                    req.state = FindRequestState::Scan(now);
                                    should_scan = true;
                                }
                                true
                            } else {
                                false
                            }
                        }
                        FindRequestState::Scan(_) => false,
                    }
                } else {
                    false
                };

                if accepted {
                    self.remove_cache_hint(alias_id, from);

                    if should_scan {
                        self.outs.push_back(InternalOutput::Broadcast(AliasMessage::Scan(alias_id)));
                    }
                }
            }
            AliasMessage::Shutdown => {
                self.on_peer_disconnected(now, from);
            }
        }
    }

    fn on_peer_disconnected(&mut self, now: u64, peer: PeerId) {
        let mut aliases = Vec::new();
        for ((alias_id, remote_peer), _state) in &self.remote_lifecycle {
            if *remote_peer == peer {
                aliases.push(*alias_id);
            }
        }

        for alias_id in aliases {
            self.remote_lifecycle.pop(&(alias_id, peer));
            self.remove_cache_hint(alias_id, peer);
        }

        let mut cached_aliases = Vec::new();
        for (alias_id, peers) in &self.cache {
            if peers.contains(&peer) {
                cached_aliases.push(*alias_id);
            }
        }

        for alias_id in cached_aliases {
            self.remove_cache_hint(alias_id, peer);
        }

        for (alias_id, req) in self.find_reqs.iter_mut() {
            if let FindRequestState::CheckHint(_, ref mut hint_peers) = req.state {
                hint_peers.remove(&peer);
                if hint_peers.is_empty() {
                    req.state = FindRequestState::Scan(now);
                    self.outs.push_back(InternalOutput::Broadcast(AliasMessage::Scan(*alias_id)));
                }
            }
        }
    }

    fn on_control(&mut self, now: u64, control: AliasControl) {
        match control {
            AliasControl::Register(alias_id) => {
                if self.shutting_down {
                    return;
                }
                let was_active = self.local.contains_key(&alias_id);
                let generation = if was_active {
                    *self.local_generations.entry(alias_id).or_default()
                } else {
                    self.increment_local_generation(alias_id)
                };
                let ref_count = self.local.entry(alias_id).or_default();
                *ref_count = ref_count.checked_add(1).expect("alias local refcount overflow");
                if let Some(req) = self.find_reqs.remove(&alias_id) {
                    gauge!(P2P_ALIAS_LIVE_FIND_REQUEST).decrement(1);
                    for tx in req.waits {
                        tx.send(Some(AliasFoundLocation::Local)).print_on_err2("[AliasServiceInternal] send query response");
                    }
                }
                self.outs.push_back(InternalOutput::Broadcast(AliasMessage::NotifySet(alias_id, generation)));
            }
            AliasControl::Unregister(alias_id) => {
                if let Some(ref_count) = self.local.get_mut(&alias_id) {
                    *ref_count -= 1;
                    if *ref_count == 0 {
                        self.local.remove(&alias_id);
                        let generation = self.increment_local_generation(alias_id);
                        self.outs.push_back(InternalOutput::Broadcast(AliasMessage::NotifyDel(alias_id, generation)));
                    }
                }
            }
            AliasControl::Find(alias_id, sender) => {
                if self.shutting_down {
                    sender.send(None).print_on_err2("[AliasServiceInternal] send shutdown find response");
                    return;
                }

                if let Some(req) = self.find_reqs.get_mut(&alias_id) {
                    if req.waits.len() >= MAX_WAITERS_PER_ALIAS {
                        sender.send(None).print_on_err2("[AliasServiceInternal] send find waiter overflow response");
                        return;
                    }
                    req.waits.push(sender);
                    return;
                }

                if self.local.contains_key(&alias_id) {
                    sender.send(Some(AliasFoundLocation::Local)).print_on_err2("[AliasServiceInternal] send query response");
                } else if self.find_reqs.len() >= MAX_PENDING_FIND_REQUESTS {
                    sender.send(None).print_on_err2("[AliasServiceInternal] send find backlog overflow response");
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
                if self.shutting_down {
                    return;
                }
                self.shutting_down = true;
                self.local.clear();
                for (_alias_id, req) in self.find_reqs.drain() {
                    gauge!(P2P_ALIAS_LIVE_FIND_REQUEST).decrement(1);
                    for tx in req.waits {
                        tx.send(None).print_on_err2("[AliasServiceInternal] send shutdown find response");
                    }
                }
                self.outs.push_back(InternalOutput::Broadcast(AliasMessage::Shutdown));
            }
        }
    }

    fn pop_output(&mut self) -> Option<InternalOutput> {
        self.outs.pop_front()
    }

    fn accept_remote_lifecycle(&mut self, alias_id: AliasId, from: PeerId, generation: u64, active: bool) -> bool {
        let key = (alias_id, from);
        if self.remote_lifecycle.get(&key).is_some_and(|state| generation <= state.generation) {
            return false;
        }

        self.remote_lifecycle.put(key, RemoteAliasState { generation, active });
        true
    }

    fn insert_cache_hint(&mut self, alias_id: AliasId, from: PeerId) {
        let slot = match self.cache.get_mut(&alias_id) {
            Some(slot) => slot,
            None => {
                counter!(P2P_ALIAS_CACHE_UPSERT).increment(1);
                self.cache.get_or_insert_mut(alias_id, HashSet::new)
            }
        };
        if !slot.contains(&from) && slot.len() >= MAX_ALIAS_HINT_PEERS {
            return;
        }
        slot.insert(from);
    }

    fn remove_cache_hint(&mut self, alias_id: AliasId, from: PeerId) {
        if let Some(slot) = self.cache.get_mut(&alias_id) {
            slot.remove(&from);
            if slot.is_empty() {
                counter!(P2P_ALIAS_CACHE_POP).increment(1);
                self.cache.pop(&alias_id);
            }
        }
    }

    fn increment_local_generation(&mut self, alias_id: AliasId) -> u64 {
        let generation = self.local_generations.entry(alias_id).or_default();
        *generation = generation.saturating_add(1);
        *generation
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
                    local_generations: HashMap::new(),
                    cache: LruCache::new(LRU_CACHE_SIZE.try_into().expect("should create NoneZeroUsize")),
                    remote_lifecycle: LruCache::new(ALIAS_LIFECYCLE_CACHE_SIZE.try_into().expect("should create NoneZeroUsize")),
                    find_reqs: HashMap::new(),
                    outs: VecDeque::new(),
                    shutting_down: false,
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

    fn expect_registered(result: anyhow::Result<AliasGuard>) -> AliasGuard {
        match result {
            Ok(guard) => guard,
            Err(err) => panic!("alias register should be admitted in test setup: {err}"),
        }
    }

    #[tokio::test]
    async fn alias_internal_control_backlog_must_be_bounded() {
        let service = test_service();
        let requester = service.requester();
        let mut guards = Vec::new();

        for alias in 0..ALIAS_CONTROL_QUEUE_SIZE {
            guards.push(expect_registered(requester.register(AliasId(alias as u64 + 10))));
        }
        let overflow = requester.register(AliasId(ALIAS_CONTROL_QUEUE_SIZE as u64 + 10));

        assert_eq!(
            service.rx.len(),
            ALIAS_CONTROL_QUEUE_SIZE,
            "pending alias internal control messages should stop at the bounded control queue size"
        );
        assert!(
            service.rx.len() <= ALIAS_CONTROL_QUEUE_SIZE,
            "pending alias internal control messages must be bounded, got {}",
            service.rx.len()
        );
        assert!(overflow.is_err(), "overflow alias registration must report admission failure");
    }

    #[tokio::test]
    async fn alias_find_returns_none_when_control_queue_full() {
        let service = test_service();
        let requester = service.requester();
        let mut guards = Vec::new();

        for alias in 0..ALIAS_CONTROL_QUEUE_SIZE {
            guards.push(expect_registered(requester.register(AliasId(alias as u64 + 10))));
        }

        assert_eq!(service.rx.len(), ALIAS_CONTROL_QUEUE_SIZE);

        let result = requester.find(AliasId(999_999)).await;

        assert_eq!(result, None, "find must fail closed instead of waiting on a oneshot that was never enqueued");
        assert_eq!(service.rx.len(), ALIAS_CONTROL_QUEUE_SIZE, "failed find admission must not grow the bounded control queue");
    }

    #[tokio::test]
    async fn alias_shutdown_when_control_queue_full_must_not_panic() {
        let service = test_service();
        let requester = service.requester();
        let mut guards = Vec::new();

        for alias in 0..ALIAS_CONTROL_QUEUE_SIZE {
            guards.push(expect_registered(requester.register(AliasId(alias as u64 + 10))));
        }

        assert_eq!(service.rx.len(), ALIAS_CONTROL_QUEUE_SIZE);
        requester.shutdown();
        assert_eq!(service.rx.len(), ALIAS_CONTROL_QUEUE_SIZE, "failed shutdown admission must not grow the bounded control queue");
    }

    #[tokio::test]
    async fn alias_guard_drop_when_control_queue_full_must_not_panic() {
        let service = test_service();
        let requester = service.requester();
        let mut guards = Vec::new();

        for alias in 0..ALIAS_CONTROL_QUEUE_SIZE {
            guards.push(expect_registered(requester.register(AliasId(alias as u64 + 10))));
        }

        assert_eq!(service.rx.len(), ALIAS_CONTROL_QUEUE_SIZE);
        drop(guards.pop());
        assert_eq!(
            service.rx.len(),
            ALIAS_CONTROL_QUEUE_SIZE,
            "failed unregister admission from Drop must not grow the bounded control queue"
        );
    }

    #[tokio::test]
    async fn alias_register_when_control_queue_full_must_not_return_live_guard() {
        let service = test_service();
        let requester = service.requester();
        let mut guards = Vec::new();

        for alias in 0..ALIAS_CONTROL_QUEUE_SIZE {
            guards.push(expect_registered(requester.register(AliasId(alias as u64 + 10))));
        }

        assert_eq!(service.rx.len(), ALIAS_CONTROL_QUEUE_SIZE);

        let overloaded_alias = AliasId(999_999);
        assert!(
            requester.register(overloaded_alias).is_err(),
            "register must report admission failure instead of returning a dead-on-arrival guard for {overloaded_alias}"
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
        let (replacement_tx, _replacement_rx) = channel(ALIAS_CONTROL_QUEUE_SIZE);
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
            InternalOutput::Broadcast(AliasMessage::NotifySet(id, generation)) => {
                assert_eq!(*id, alias_id);
                assert_eq!(*generation, 1);
            }
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
            InternalOutput::Broadcast(AliasMessage::NotifyDel(id, generation)) => {
                assert_eq!(*id, alias_id);
                assert_eq!(*generation, 2);
            }
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
    fn saturated_alias_refcount_must_not_unregister_while_guards_remain() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);

        for _ in 0..300 {
            ctx.internal.on_control(ctx.now, AliasControl::Register(alias_id));
        }

        for _ in 0..255 {
            ctx.internal.on_control(ctx.now, AliasControl::Unregister(alias_id));
        }

        assert!(
            ctx.internal.local.contains_key(&alias_id),
            "more than 255 live alias guards must not be truncated to 255 and unregister the alias while guards still remain"
        );
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
        ctx.internal.on_msg(ctx.now, peer_addr, AliasMessage::NotifySet(alias_id, 1));

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

        ctx.internal.on_msg(ctx.now, hinted_peer, AliasMessage::NotifySet(alias_id, 1));

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
    fn scan_found_must_require_advertised_alias_lifecycle() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);
        let unadvertised_peer = PeerId(2);

        let (tx, mut rx) = oneshot::channel();
        ctx.internal.on_control(ctx.now, AliasControl::Find(alias_id, tx));
        assert_eq!(ctx.collect_outputs(), vec![InternalOutput::Broadcast(AliasMessage::Scan(alias_id))]);

        ctx.internal.on_msg(ctx.now, unadvertised_peer, AliasMessage::Found(alias_id));

        assert!(rx.try_recv().is_err(), "scan lookup must not complete from a peer that never advertised active ownership of the alias");
        assert!(!ctx.internal.cache.contains(&alias_id), "unadvertised scan Found responses must not poison the alias hint cache");
        assert!(
            ctx.internal.find_reqs.contains_key(&alias_id),
            "scan lookup should remain pending until an advertised owner answers or the scan times out"
        );
    }

    #[test]
    fn stale_not_found_must_not_evict_alias_cache_without_pending_check() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(7);
        let hinted_peer = PeerId(2);

        ctx.internal.on_msg(ctx.now, hinted_peer, AliasMessage::NotifySet(alias_id, 1));
        assert!(
            ctx.internal.cache.get(&alias_id).is_some_and(|peers| peers.contains(&hinted_peer)),
            "test setup should cache the hinted peer"
        );
        assert!(!ctx.internal.find_reqs.contains_key(&alias_id), "test setup should have no active lookup for this alias");

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

        ctx.internal.on_msg(ctx.now, peer, AliasMessage::NotifySet(alias_id, 1));
        assert!(ctx.internal.cache.get(&alias_id).is_some_and(|peers| peers.contains(&peer)));

        ctx.internal.on_msg(ctx.now + 1, peer, AliasMessage::NotifyDel(alias_id, 2));
        assert!(!ctx.internal.cache.contains(&alias_id));

        ctx.internal.on_msg(ctx.now + 2, peer, AliasMessage::NotifySet(alias_id, 1));

        assert!(!ctx.internal.cache.contains(&alias_id), "stale NotifySet must not resurrect an alias hint removed by a newer NotifyDel");

        let (tx, _rx) = oneshot::channel();
        ctx.internal.on_control(ctx.now + 3, AliasControl::Find(alias_id, tx));

        assert_eq!(
            ctx.collect_outputs(),
            vec![InternalOutput::Broadcast(AliasMessage::Scan(alias_id))],
            "find must not use a resurrected stale hint"
        );
    }

    #[test]
    fn newer_notify_set_must_restore_alias_after_notify_del() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(7);
        let peer = PeerId(2);

        ctx.internal.on_msg(ctx.now, peer, AliasMessage::NotifySet(alias_id, 1));
        assert!(ctx.internal.cache.get(&alias_id).is_some_and(|peers| peers.contains(&peer)));

        ctx.internal.on_msg(ctx.now + 1, peer, AliasMessage::NotifyDel(alias_id, 2));
        assert!(!ctx.internal.cache.contains(&alias_id));

        ctx.internal.on_msg(ctx.now + 2, peer, AliasMessage::NotifySet(alias_id, 3));

        assert!(
            ctx.internal.cache.get(&alias_id).is_some_and(|peers| peers.contains(&peer)),
            "newer NotifySet generation must restore an alias hint after a prior NotifyDel"
        );

        let (tx, _rx) = oneshot::channel();
        ctx.internal.on_control(ctx.now + 3, AliasControl::Find(alias_id, tx));

        assert_eq!(
            ctx.collect_outputs(),
            vec![InternalOutput::Unicast(peer, AliasMessage::Check(alias_id))],
            "find must use the valid newer hint"
        );
    }

    #[test]
    fn alias_restart_with_reset_generation_must_restore_hint_after_disconnect() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(7);
        let peer = PeerId(2);

        ctx.internal.on_msg(ctx.now, peer, AliasMessage::NotifySet(alias_id, 2));
        assert!(ctx.internal.cache.get(&alias_id).is_some_and(|peers| peers.contains(&peer)));

        ctx.internal.on_msg(ctx.now + 1, peer, AliasMessage::NotifyDel(alias_id, 3));
        assert!(!ctx.internal.cache.contains(&alias_id));

        ctx.internal.on_peer_disconnected(ctx.now + 2, peer);
        ctx.internal.on_msg(ctx.now + 3, peer, AliasMessage::NotifySet(alias_id, 1));

        assert!(
            ctx.internal.cache.get(&alias_id).is_some_and(|peers| peers.contains(&peer)),
            "a restarted peer with reset alias generation must be able to restore a fresh hint"
        );

        let (tx, _rx) = oneshot::channel();
        ctx.internal.on_control(ctx.now + 4, AliasControl::Find(alias_id, tx));

        assert_eq!(
            ctx.collect_outputs(),
            vec![InternalOutput::Unicast(peer, AliasMessage::Check(alias_id))],
            "find must use the fresh post-restart alias hint"
        );
    }

    #[test]
    fn alias_peer_disconnect_must_clear_remote_lifecycle_and_cached_hint() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(7);
        let peer = PeerId(2);

        ctx.internal.on_msg(ctx.now, peer, AliasMessage::NotifySet(alias_id, 2));
        assert!(ctx.internal.cache.get(&alias_id).is_some_and(|peers| peers.contains(&peer)));
        assert!(ctx.internal.remote_lifecycle.contains(&(alias_id, peer)));

        let (tx, _rx) = oneshot::channel();
        ctx.internal.on_control(ctx.now + 1, AliasControl::Find(alias_id, tx));
        assert_eq!(ctx.collect_outputs(), vec![InternalOutput::Unicast(peer, AliasMessage::Check(alias_id))]);

        ctx.internal.on_peer_disconnected(ctx.now + 2, peer);

        assert!(!ctx.internal.cache.contains(&alias_id), "disconnect must remove cached alias hints for the peer");
        assert!(
            !ctx.internal.remote_lifecycle.contains(&(alias_id, peer)),
            "disconnect must remove alias lifecycle tombstones for the peer"
        );
        assert_eq!(
            ctx.collect_outputs(),
            vec![InternalOutput::Broadcast(AliasMessage::Scan(alias_id))],
            "pending cached-hint lookups must fail over when their checked peer disconnects"
        );
    }

    #[test]
    fn alias_peer_disconnect_must_clear_found_only_cached_hint() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(7);
        let peer = PeerId(2);

        let (tx, _rx) = oneshot::channel();
        ctx.internal.on_control(ctx.now, AliasControl::Find(alias_id, tx));
        assert_eq!(ctx.collect_outputs(), vec![InternalOutput::Broadcast(AliasMessage::Scan(alias_id))]);

        ctx.internal.on_msg(ctx.now + 1, peer, AliasMessage::Found(alias_id));
        assert!(
            !ctx.internal.cache.get(&alias_id).is_some_and(|peers| peers.contains(&peer)),
            "Found-only scan responses without advertised lifecycle must not create cache hints"
        );
        assert!(!ctx.internal.remote_lifecycle.contains(&(alias_id, peer)), "Found-only cache hints do not create lifecycle entries");

        ctx.internal.on_peer_disconnected(ctx.now + 2, peer);

        assert!(!ctx.internal.cache.contains(&alias_id), "disconnect must leave rejected Found-only cache hints absent");
    }

    #[test]
    fn cached_alias_peer_hints_must_be_bounded() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);

        for peer in 0..=MAX_ALIAS_HINT_PEERS {
            ctx.internal.on_msg(ctx.now, PeerId::from(peer as u64 + 10), AliasMessage::NotifySet(alias_id, 1));
        }

        let cached_peers = ctx.internal.cache.get(&alias_id).expect("alias should be cached").len();

        assert!(cached_peers <= MAX_ALIAS_HINT_PEERS, "cached peer hints for one alias must be bounded, got {cached_peers}");
    }

    #[test]
    fn cached_alias_existing_peer_refresh_must_work_when_hint_set_full() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);

        for peer in 0..MAX_ALIAS_HINT_PEERS {
            ctx.internal.on_msg(ctx.now, PeerId::from(peer as u64 + 10), AliasMessage::NotifySet(alias_id, 1));
        }

        let existing = PeerId::from(10);
        ctx.internal.on_msg(ctx.now + 1, existing, AliasMessage::NotifySet(alias_id, 2));

        let cached_peers = ctx.internal.cache.get(&alias_id).expect("alias should be cached");
        assert_eq!(cached_peers.len(), MAX_ALIAS_HINT_PEERS);
        assert!(cached_peers.contains(&existing), "existing cached peer must stay admitted at capacity");
        assert_eq!(
            ctx.internal.remote_lifecycle.get(&(alias_id, existing)),
            Some(&RemoteAliasState { generation: 2, active: true }),
            "existing peer lifecycle must refresh even when the hint set is full"
        );
    }

    #[test]
    fn found_response_must_not_exceed_alias_hint_cap() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);

        for peer in 0..MAX_ALIAS_HINT_PEERS {
            ctx.internal.on_msg(ctx.now, PeerId::from(peer as u64 + 10), AliasMessage::NotifySet(alias_id, 1));
        }

        let (tx, mut rx) = oneshot::channel();
        ctx.internal.find_reqs.insert(
            alias_id,
            FindRequest {
                state: FindRequestState::Scan(ctx.now),
                waits: vec![tx],
            },
        );
        gauge!(P2P_ALIAS_LIVE_FIND_REQUEST).increment(1);

        let found_peer = PeerId::from(20_000);
        ctx.internal.on_msg(ctx.now, found_peer, AliasMessage::NotifySet(alias_id, 1));
        assert!(
            !ctx.internal.cache.get(&alias_id).is_some_and(|peers| peers.contains(&found_peer)),
            "test setup should keep the advertised responder out of the full hint cache"
        );
        ctx.internal.on_msg(ctx.now, found_peer, AliasMessage::Found(alias_id));

        assert_eq!(rx.try_recv().expect("scan lookup should complete"), Some(AliasFoundLocation::Scan(found_peer)));
        let cached_peers = ctx.internal.cache.get(&alias_id).expect("alias should be cached");
        assert_eq!(cached_peers.len(), MAX_ALIAS_HINT_PEERS, "accepted Found must not grow a full alias hint set");
        assert!(!cached_peers.contains(&found_peer), "new Found peer must not be retained when the hint set is full");
    }

    #[test]
    fn test_find_cached_alias_not_found() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);
        let peer_addr = PeerId(1);

        // Add alias to cache
        ctx.internal.on_msg(ctx.now, peer_addr, AliasMessage::NotifySet(alias_id, 1));

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

        ctx.internal.on_msg(ctx.now, hinted_peer, AliasMessage::NotifySet(alias_id, 1));

        let (tx, mut rx) = oneshot::channel();
        ctx.internal.on_control(ctx.now, AliasControl::Find(alias_id, tx));

        assert_eq!(ctx.collect_outputs(), vec![InternalOutput::Unicast(hinted_peer, AliasMessage::Check(alias_id))]);

        ctx.internal.on_msg(ctx.now, hinted_peer, AliasMessage::Shutdown);

        assert_eq!(
            ctx.collect_outputs(),
            vec![InternalOutput::Broadcast(AliasMessage::Scan(alias_id))],
            "shutdown from the only cached hint must immediately fail over instead of waiting for hint timeout"
        );
        assert!(rx.try_recv().is_err(), "lookup should remain pending while it fails over to scan, not complete from a stopped hint");
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
        assert!(!ctx.internal.find_reqs.contains_key(&alias_id), "local registration should clear the obsolete pending find request");
    }

    #[test]
    fn test_find_cached_alias_timeout_switch_to_scan() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);
        let peer_addr = PeerId(1);

        // Add alias to cache
        ctx.internal.on_msg(ctx.now, peer_addr, AliasMessage::NotifySet(alias_id, 1));

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
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);

        for _ in 0..MAX_WAITERS_PER_ALIAS {
            let (tx, _rx) = oneshot::channel();
            ctx.internal.on_control(ctx.now, AliasControl::Find(alias_id, tx));
        }
        let (overflow_tx, mut overflow_rx) = oneshot::channel();
        ctx.internal.on_control(ctx.now, AliasControl::Find(alias_id, overflow_tx));

        let waiters = ctx.internal.find_reqs.get(&alias_id).expect("find request should exist").waits.len();

        assert!(waiters <= MAX_WAITERS_PER_ALIAS, "duplicate find waiters for one alias must be bounded, got {waiters}");
        assert_eq!(overflow_rx.try_recv(), Ok(None), "overflow find waiter must complete immediately with no result");
        assert_eq!(ctx.internal.outs.len(), 1, "duplicate and overflow finds must not create extra scan fanout");
    }

    #[test]
    fn distinct_pending_find_requests_must_be_bounded() {
        let mut ctx = TestContext::new();

        for id in 0..MAX_PENDING_FIND_REQUESTS {
            let (tx, _rx) = oneshot::channel();
            ctx.internal.on_control(ctx.now, AliasControl::Find(AliasId(id as u64), tx));
        }
        let (duplicate_tx, mut duplicate_rx) = oneshot::channel();
        ctx.internal.on_control(ctx.now, AliasControl::Find(AliasId(0), duplicate_tx));
        let (overflow_tx, mut overflow_rx) = oneshot::channel();
        ctx.internal.on_control(ctx.now, AliasControl::Find(AliasId(MAX_PENDING_FIND_REQUESTS as u64), overflow_tx));
        let local_alias = AliasId((MAX_PENDING_FIND_REQUESTS + 1) as u64);
        ctx.internal.local.insert(local_alias, 1);
        let (local_tx, mut local_rx) = oneshot::channel();
        ctx.internal.on_control(ctx.now, AliasControl::Find(local_alias, local_tx));

        let pending_finds = ctx.internal.find_reqs.len();
        let pending_scans = ctx.internal.outs.len();

        assert_eq!(pending_finds, MAX_PENDING_FIND_REQUESTS, "pending alias find requests must be bounded");
        assert_eq!(pending_scans, MAX_PENDING_FIND_REQUESTS, "pending alias scan fanout must be bounded");
        assert!(
            matches!(duplicate_rx.try_recv(), Err(oneshot::error::TryRecvError::Empty)),
            "duplicate find should join the existing request"
        );
        assert_eq!(ctx.internal.find_reqs.get(&AliasId(0)).expect("duplicate request should still exist").waits.len(), 2);
        assert_eq!(overflow_rx.try_recv(), Ok(None), "overflow distinct find must complete immediately with no result");
        assert_eq!(local_rx.try_recv(), Ok(Some(AliasFoundLocation::Local)), "pending-find cap must not reject immediate local hits");
    }

    #[test]
    fn find_timeout_at_max_timestamp_must_not_overflow() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);
        let (tx, mut rx) = oneshot::channel();

        ctx.internal.on_control(u64::MAX - 10, AliasControl::Find(alias_id, tx));

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ctx.internal.on_tick(u64::MAX);
        }));

        assert!(result.is_ok(), "alias find timeout arithmetic must not panic or wrap near u64::MAX");
        assert!(matches!(rx.try_recv(), Err(oneshot::error::TryRecvError::Empty)), "overflowed scan deadline must not expire early");
        assert!(ctx.internal.find_reqs.contains_key(&alias_id), "overflowed scan deadline must keep the pending find alive");
    }

    #[test]
    fn find_hint_timeout_at_max_timestamp_must_not_overflow() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);
        let hinted_peer = PeerId(2);
        ctx.internal.on_msg(u64::MAX - 20, hinted_peer, AliasMessage::NotifySet(alias_id, 1));
        let (tx, mut rx) = oneshot::channel();

        ctx.internal.on_control(u64::MAX - 10, AliasControl::Find(alias_id, tx));
        assert_eq!(ctx.internal.outs.pop_front(), Some(InternalOutput::Unicast(hinted_peer, AliasMessage::Check(alias_id))));

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ctx.internal.on_tick(u64::MAX);
        }));

        assert!(result.is_ok(), "alias hint timeout arithmetic must not panic or wrap near u64::MAX");
        assert_eq!(ctx.internal.outs.pop_front(), None, "overflowed hint deadline must not switch to scan early");
        assert!(
            matches!(rx.try_recv(), Err(oneshot::error::TryRecvError::Empty)),
            "overflowed hint deadline must keep the waiter pending"
        );
        assert!(
            matches!(ctx.internal.find_reqs.get(&alias_id).map(|req| &req.state), Some(FindRequestState::CheckHint(_, _))),
            "overflowed hint deadline must keep the request in CheckHint state"
        );
    }

    #[test]
    fn local_shutdown_must_fail_pending_alias_finds() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);

        let (tx, mut rx) = oneshot::channel();
        ctx.internal.on_control(ctx.now, AliasControl::Find(alias_id, tx));

        assert_eq!(ctx.collect_outputs(), vec![InternalOutput::Broadcast(AliasMessage::Scan(alias_id))]);

        ctx.internal.on_control(ctx.now + 1, AliasControl::Shutdown);

        assert_eq!(rx.try_recv(), Ok(None), "local alias shutdown must immediately fail pending find waiters");
        assert!(!ctx.internal.find_reqs.contains_key(&alias_id), "local alias shutdown must clear pending find state");
        assert_eq!(ctx.collect_outputs(), vec![InternalOutput::Broadcast(AliasMessage::Shutdown)]);
    }

    #[test]
    fn local_shutdown_must_stop_serving_local_aliases() {
        let mut ctx = TestContext::new();
        let alias_id = AliasId(1);

        ctx.internal.on_control(ctx.now, AliasControl::Register(alias_id));
        assert!(ctx.internal.local.contains_key(&alias_id), "test setup should register local alias ownership");
        assert_eq!(ctx.collect_outputs(), vec![InternalOutput::Broadcast(AliasMessage::NotifySet(alias_id, 1))]);

        ctx.internal.on_control(ctx.now + 1, AliasControl::Shutdown);
        assert_eq!(ctx.collect_outputs(), vec![InternalOutput::Broadcast(AliasMessage::Shutdown)]);

        let (tx, mut rx) = oneshot::channel();
        ctx.internal.on_control(ctx.now + 2, AliasControl::Find(alias_id, tx));

        assert_eq!(rx.try_recv(), Ok(None), "after local shutdown, alias service must not keep resolving aliases as local");
        assert!(!ctx.internal.local.contains_key(&alias_id), "local alias shutdown must clear local alias ownership");

        ctx.internal.on_control(ctx.now + 3, AliasControl::Register(alias_id));
        assert!(!ctx.internal.local.contains_key(&alias_id), "local alias shutdown must reject later local registrations");
        assert!(ctx.collect_outputs().is_empty(), "post-shutdown register must not broadcast NotifySet");
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

        ctx.internal.on_control(ctx.now + 1, AliasControl::Shutdown);
        assert!(ctx.collect_outputs().is_empty(), "repeated local shutdown must not broadcast again");

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
        let shared_alias = AliasId(3);
        let peer1 = PeerId(1);
        let peer2 = PeerId(2);

        ctx.internal.on_msg(ctx.now, peer1, AliasMessage::NotifySet(alias_from_peer1, 1));
        ctx.internal.on_msg(ctx.now, peer2, AliasMessage::NotifySet(alias_from_peer2, 1));
        ctx.internal.on_msg(ctx.now, peer1, AliasMessage::NotifySet(shared_alias, 1));
        ctx.internal.on_msg(ctx.now, peer2, AliasMessage::NotifySet(shared_alias, 1));

        ctx.internal.on_msg(ctx.now, peer1, AliasMessage::Shutdown);

        assert!(!ctx.internal.cache.contains(&alias_from_peer1), "shutdown from one peer must remove that peer's alias hints");
        assert!(
            ctx.internal.cache.contains(&alias_from_peer2),
            "shutdown from one peer must not remove alias hints learned from other peers"
        );
        assert!(
            !ctx.internal.remote_lifecycle.contains(&(alias_from_peer1, peer1)),
            "shutdown must remove lifecycle owned by the stopped peer"
        );
        assert!(
            ctx.internal.remote_lifecycle.contains(&(alias_from_peer2, peer2)),
            "shutdown must not remove lifecycle owned by other peers"
        );
        assert_eq!(
            ctx.internal.cache.get(&shared_alias).cloned(),
            Some(HashSet::from([peer2])),
            "shutdown from one peer must only remove that peer from shared alias hints"
        );
    }
}

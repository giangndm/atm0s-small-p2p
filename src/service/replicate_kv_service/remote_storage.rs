use std::{
    collections::{BTreeMap, VecDeque},
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    time::Instant,
};

use super::messages::{Action, BroadcastEvent, BroadcastEventData, Changed, Event, KvEvent, NetEvent, RpcEvent, RpcEventData, RpcReq, RpcRes, Slot, Version};

const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);
const MAX_SNAPSHOT_SLOTS_PER_PAGE: usize = 1024;
const MAX_STAGED_SNAPSHOT_SLOTS: usize = 16 * MAX_SNAPSHOT_SLOTS_PER_PAGE;
const MAX_PENDING_CHANGEDS: usize = 1024;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RemoteStoreState<N, K, V> {
    SyncFull(SyncFullState<N, K, V>),
    Working(WorkingState<N, K, V>),
    Destroy(DestroyState<N, K, V>),
}

impl<N, K, V> State<N, K, V> for RemoteStoreState<N, K, V>
where
    K: Debug + Hash + Ord + Eq + Clone,
    V: Debug + Eq + Clone,
    N: Debug + Clone,
{
    fn init(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant) {
        match self {
            RemoteStoreState::SyncFull(state) => state.init(ctx, now),
            RemoteStoreState::Working(state) => state.init(ctx, now),
            RemoteStoreState::Destroy(state) => state.init(ctx, now),
        }
    }

    fn on_tick(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant) {
        match self {
            RemoteStoreState::SyncFull(state) => state.on_tick(ctx, now),
            RemoteStoreState::Working(state) => state.on_tick(ctx, now),
            RemoteStoreState::Destroy(state) => state.on_tick(ctx, now),
        }
    }

    fn on_broadcast(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant, event: BroadcastEvent<K, V>) -> bool {
        match self {
            RemoteStoreState::SyncFull(state) => state.on_broadcast(ctx, now, event),
            RemoteStoreState::Working(state) => state.on_broadcast(ctx, now, event),
            RemoteStoreState::Destroy(state) => state.on_broadcast(ctx, now, event),
        }
    }

    fn on_rpc_res(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant, event: RpcRes<K, V>) -> bool {
        match self {
            RemoteStoreState::SyncFull(state) => state.on_rpc_res(ctx, now, event),
            RemoteStoreState::Working(state) => state.on_rpc_res(ctx, now, event),
            RemoteStoreState::Destroy(state) => state.on_rpc_res(ctx, now, event),
        }
    }
}

pub(crate) struct StateCtx<N, K, V> {
    pub(crate) remote: N,
    pub(crate) local_session_id: u64,
    pub(crate) slots: BTreeMap<K, Slot<V>>,
    pub(crate) outs: VecDeque<Event<N, K, V>>,
    pub(crate) next_state: Option<RemoteStoreState<N, K, V>>,
}

trait State<N, K, V> {
    fn init(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant);
    fn on_tick(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant);
    fn on_broadcast(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant, event: BroadcastEvent<K, V>) -> bool;
    fn on_rpc_res(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant, event: RpcRes<K, V>) -> bool;
}

pub struct RemoteStore<N, K, V> {
    pub(crate) session_id: u64,
    pub(crate) ctx: StateCtx<N, K, V>,
    pub(crate) state: RemoteStoreState<N, K, V>,
    pub(crate) last_active: Instant,
}

impl<N, K, V> RemoteStore<N, K, V>
where
    N: Debug + Clone,
    K: Debug + Hash + Ord + Eq + Clone,
    V: Debug + Eq + Clone,
{
    pub fn new(remote: N, local_session_id: u64, remote_session_id: u64) -> Self {
        let mut ctx = StateCtx {
            remote,
            local_session_id,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let mut state = SyncFullState::default();
        state.init(&mut ctx, Instant::now());

        Self {
            session_id: remote_session_id,
            ctx,
            state: RemoteStoreState::SyncFull(state),
            last_active: Instant::now(),
        }
    }

    pub fn on_tick(&mut self) {
        self.state.on_tick(&mut self.ctx, Instant::now());
    }

    pub fn destroy(&mut self) {
        let mut state = DestroyState { _tmp: PhantomData };
        state.init(&mut self.ctx, Instant::now());
        self.state = RemoteStoreState::Destroy(state);
    }

    pub fn last_active(&self) -> Instant {
        self.last_active
    }

    pub fn on_broadcast(&mut self, event: BroadcastEvent<K, V>) {
        let now = Instant::now();
        let accepted = self.state.on_broadcast(&mut self.ctx, now, event);
        if let Some(mut next_state) = self.ctx.next_state.take() {
            next_state.init(&mut self.ctx, now);
            self.state = next_state;
        }
        if accepted {
            self.last_active = now;
        }
    }

    pub fn on_rpc_res(&mut self, event: RpcRes<K, V>) {
        let now = Instant::now();
        let accepted = self.state.on_rpc_res(&mut self.ctx, now, event);
        if let Some(mut next_state) = self.ctx.next_state.take() {
            next_state.init(&mut self.ctx, now);
            self.state = next_state;
        }
        if accepted {
            self.last_active = now;
        }
    }

    pub fn pop_out(&mut self) -> Option<Event<N, K, V>> {
        self.ctx.outs.pop_front()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SyncFullState<N, K, V> {
    version: Option<Version>,
    catchup_to: Option<Version>,
    sending_req: Option<(Instant, NetEvent<N, K, V>)>,
    staged_slots: Option<BTreeMap<K, Slot<V>>>,
    skipped_newer: BTreeMap<K, Version>,
    _tmp: PhantomData<(N, K, V)>,
}

impl<N, K, V> Default for SyncFullState<N, K, V> {
    fn default() -> Self {
        Self {
            version: None,
            catchup_to: None,
            sending_req: None,
            staged_slots: Some(BTreeMap::new()),
            skipped_newer: BTreeMap::new(),
            _tmp: PhantomData,
        }
    }
}

impl<N, K, V> SyncFullState<N, K, V> {
    fn preserve_existing_until_complete() -> Self {
        Self {
            staged_slots: Some(BTreeMap::new()),
            ..Default::default()
        }
    }
}

impl<N, K, V> SyncFullState<N, K, V>
where
    K: Ord + Clone,
    V: Eq + Clone,
    N: Clone,
{
    fn commit_staged_slots(&mut self, ctx: &mut StateCtx<N, K, V>) {
        if let Some(mut staged_slots) = self.staged_slots.take() {
            for (k, slot) in ctx.slots.iter() {
                if staged_slots.contains_key(k) {
                    continue;
                }
                if self.skipped_newer.contains_key(k) {
                    staged_slots.insert(k.clone(), slot.clone());
                } else {
                    ctx.outs.push_back(Event::KvEvent(KvEvent::Del(Some(ctx.remote.clone()), k.clone())));
                }
            }
            for (k, slot) in staged_slots.iter() {
                if ctx.slots.get(k).is_none_or(|current| current.value != slot.value) {
                    ctx.outs.push_back(Event::KvEvent(KvEvent::Set(Some(ctx.remote.clone()), k.clone(), slot.value.clone())));
                }
            }
            ctx.slots = staged_slots;
        }
    }

    fn remember_catchup_to(&mut self, version: Version) {
        self.catchup_to = Some(self.catchup_to.map_or(version, |current| current.max(version)));
    }

    fn restart_full_sync(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant) {
        self.version = None;
        self.staged_slots = Some(BTreeMap::new());
        self.skipped_newer.clear();
        let req = NetEvent::Unicast(
            ctx.remote.clone(),
            RpcEvent {
                session_id: ctx.local_session_id,
                data: RpcEventData::RpcReq(RpcReq::FetchSnapshot {
                    from: None,
                    max_version: None,
                    max_items: MAX_SNAPSHOT_SLOTS_PER_PAGE as u64,
                }),
            },
        );
        self.sending_req = Some((now, req.clone()));
        ctx.outs.push_back(Event::NetEvent(req));
    }
}

impl<N, K, V> SyncFullState<N, K, V>
where
    K: Debug + Hash + Ord + Eq + Clone,
    V: Debug + Eq + Clone,
    N: Debug + Clone,
{
    fn transition_to_working(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant, version: Version) {
        self.commit_staged_slots(ctx);
        log::info!("[RemoteStore {:?}] switch to working with {} slots and version {version:?}", ctx.remote, ctx.slots.len());
        self.sending_req = None;
        let mut state = WorkingState::new(version);
        if let Some(catchup_to) = self.catchup_to.filter(|catchup_to| *catchup_to > version) {
            state.request_fetch_changed(ctx, now, version + 1, catchup_to - version);
        }
        ctx.next_state = Some(RemoteStoreState::Working(state));
    }
}

impl<N, K, V> State<N, K, V> for SyncFullState<N, K, V>
where
    K: Debug + Hash + Ord + Eq + Clone,
    V: Debug + Eq + Clone,
    N: Debug + Clone,
{
    fn init(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant) {
        log::info!("[RemoteStore {:?}] switch to syncFull", ctx.remote);
        let req = NetEvent::Unicast(
            ctx.remote.clone(),
            RpcEvent {
                session_id: ctx.local_session_id,
                data: RpcEventData::RpcReq(RpcReq::FetchSnapshot {
                    from: None,
                    max_version: None,
                    max_items: MAX_SNAPSHOT_SLOTS_PER_PAGE as u64,
                }),
            },
        );
        self.sending_req = Some((now, req.clone()));
        ctx.outs.push_back(Event::NetEvent(req));
    }

    fn on_tick(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant) {
        if let Some((sent_at, req)) = self.sending_req.as_mut() {
            if now - *sent_at >= REQUEST_TIMEOUT {
                log::warn!("[RemoteStore {:?}] syncFull timeout => resend", ctx.remote);
                *sent_at = now;
                ctx.outs.push_back(Event::NetEvent(req.clone()));
            }
        }
    }

    fn on_broadcast(&mut self, _ctx: &mut StateCtx<N, K, V>, _now: Instant, event: BroadcastEvent<K, V>) -> bool {
        match event.data {
            BroadcastEventData::Changed(changed) => self.remember_catchup_to(changed.version),
            BroadcastEventData::Version(version) => self.remember_catchup_to(version),
        }
        true
    }

    fn on_rpc_res(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant, event: RpcRes<K, V>) -> bool {
        match event {
            RpcRes::FetchChanged(_) => {
                false
            }
            RpcRes::FetchSnapshot(Some(snapshot), version) => {
                let Some((_, NetEvent::Unicast(_, RpcEvent { data: RpcEventData::RpcReq(RpcReq::FetchSnapshot { from: _, max_version, max_items }), .. }))) = self.sending_req.as_ref() else {
                    return false;
                };
                log::info!(
                    "[RemoteStore {:?}] got snapshot {} slots, current version {version:?}, next {:?}",
                    ctx.remote,
                    snapshot.slots.len(),
                    snapshot.next_key,
                );
                let page_items = snapshot.slots.len().saturating_add(snapshot.skipped_newer.len());
                if page_items > MAX_SNAPSHOT_SLOTS_PER_PAGE {
                    log::warn!(
                        "[RemoteStore {:?}] reject snapshot page with {page_items} scanned items over limit {MAX_SNAPSHOT_SLOTS_PER_PAGE}",
                        ctx.remote,
                    );
                    return false;
                }
                if page_items as u64 > *max_items {
                    log::warn!(
                        "[RemoteStore {:?}] reject snapshot page with {page_items} scanned items over requested max_items {max_items}",
                        ctx.remote,
                    );
                    return false;
                }
                if page_items == 0 && snapshot.next_key.is_some() {
                    log::warn!("[RemoteStore {:?}] reject empty non-terminal snapshot page with no scanned progress", ctx.remote,);
                    return false;
                }
                if max_version.is_some() && max_version != &Some(version) {
                    log::warn!(
                        "[RemoteStore {:?}] reject snapshot page version {version:?} not matching pending max_version {max_version:?}",
                        ctx.remote
                    );
                    self.restart_full_sync(ctx, now);
                    return true;
                }
                if let Some(staged_slots) = self.staged_slots.as_ref() {
                    if staged_slots.len().saturating_add(self.skipped_newer.len()).saturating_add(page_items) > MAX_STAGED_SNAPSHOT_SLOTS {
                        log::warn!(
                            "[RemoteStore {:?}] reject snapshot page because staged slots would exceed limit {MAX_STAGED_SNAPSHOT_SLOTS}",
                            ctx.remote
                        );
                        return false;
                    }
                }
                for (k, slot) in snapshot.slots.into_iter() {
                    if let Some(staged_slots) = self.staged_slots.as_mut() {
                        staged_slots.insert(k, slot);
                    } else {
                        ctx.outs.push_back(Event::KvEvent(KvEvent::Set(Some(ctx.remote.clone()), k.clone(), slot.value.clone())));
                        ctx.slots.insert(k, slot);
                    }
                }
                for (k, skipped_version) in snapshot.skipped_newer.into_iter() {
                    self.remember_catchup_to(skipped_version);
                    self.skipped_newer.insert(k, skipped_version);
                }
                if self.version.is_none() {
                    self.version = Some(version);
                }
                if let Some(next_key) = snapshot.next_key {
                    let max_version = self.version.expect("should have version");

                    log::info!("[RemoteStore {:?}] request more snapshot data with from {next_key:?}, max_version {max_version:?}", ctx.remote);
                    let req = NetEvent::Unicast(
                        ctx.remote.clone(),
                        RpcEvent {
                            session_id: ctx.local_session_id,
                            data: RpcEventData::RpcReq(RpcReq::FetchSnapshot {
                                from: Some(next_key),
                                max_version: Some(max_version),
                                max_items: MAX_SNAPSHOT_SLOTS_PER_PAGE as u64,
                            }),
                        },
                    );
                    self.sending_req = Some((now, req.clone()));
                    ctx.outs.push_back(Event::NetEvent(req));
                } else {
                    let version = self.version.expect("should have version");
                    self.transition_to_working(ctx, now, version);
                }
                true
            }
            RpcRes::FetchSnapshot(None, version) => {
                let Some((_, NetEvent::Unicast(_, RpcEvent { data: RpcEventData::RpcReq(RpcReq::FetchSnapshot { from, max_version, .. }), .. }))) = self.sending_req.as_ref() else {
                    return false;
                };
                if from.is_some() || max_version.is_some() {
                    self.restart_full_sync(ctx, now);
                    return true;
                }
                let version = self.version.unwrap_or(version);
                self.transition_to_working(ctx, now, version);
                true
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct WorkingState<N, K, V> {
    version: Version,
    // this is a list of changeds in order, we use it to detect discontinuity to send fetchChanged
    pendings: BTreeMap<Version, Changed<K, V>>,
    sending_req: Option<(Instant, NetEvent<N, K, V>)>,
    latest_seen_version: Option<Version>,
    _tmp: PhantomData<(N, V)>,
}

impl<N, K, V> WorkingState<N, K, V>
where
    K: Debug + Hash + Ord + Eq + Clone,
    V: Debug + Clone,
    N: Debug + Clone,
{
    fn new(version: Version) -> Self {
        Self {
            version,
            pendings: BTreeMap::new(),
            sending_req: None,
            latest_seen_version: None,
            _tmp: PhantomData,
        }
    }

    fn in_flight_fetch_changed(&self) -> Option<(Version, u64)> {
        let Some((_, NetEvent::Unicast(_, RpcEvent { data: RpcEventData::RpcReq(RpcReq::FetchChanged { from, count }), .. }))) = self.sending_req.as_ref() else {
            return None;
        };
        Some((*from, *count))
    }

    fn clear_satisfied_fetch_changed(&mut self) {
        let Some((from, count)) = self.in_flight_fetch_changed() else {
            return;
        };
        if count > 0 && self.version >= from + count.saturating_sub(1) {
            self.sending_req = None;
        }
    }

    fn request_fetch_changed(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant, from: Version, count: u64) {
        if count == 0 {
            return;
        }

        let requested_to = from + count.saturating_sub(1);
        if let Some((current_from, current_count)) = self.in_flight_fetch_changed() {
            let current_to = current_from + current_count.saturating_sub(1);
            if current_count > 0 && current_from <= from && current_to >= requested_to {
                log::debug!("[RemoteStore {:?}] fetch changed from {from:?} count {count} already covered by in-flight request", ctx.remote);
                return;
            }
        }

        let req = NetEvent::Unicast(
            ctx.remote.clone(),
            RpcEvent {
                session_id: ctx.local_session_id,
                data: RpcEventData::RpcReq(RpcReq::FetchChanged { from, count }),
            },
        );
        self.sending_req = Some((now, req.clone()));
        ctx.outs.push_back(Event::NetEvent(req));
    }

    fn apply_pendings(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant) {
        while let Some(entry) = self.pendings.first_entry() {
            if *entry.key() == self.version + 1 {
                self.version = self.version + 1;
                let (_, data) = self.pendings.pop_first().expect("should have data");
                match data.action {
                    Action::Set(value) => {
                        log::debug!("[RemoteStore {:?}] apply set k {:?} => version {:?}", ctx.remote, data.key, data.version);
                        ctx.outs.push_back(Event::KvEvent(KvEvent::Set(Some(ctx.remote.clone()), data.key.clone(), value.clone())));
                        ctx.slots.insert(data.key, Slot::new(value, data.version));
                    }
                    Action::Del => {
                        log::debug!("[RemoteStore {:?}] apply del k {:?} => version {:?}", ctx.remote, data.key, data.version);
                        if ctx.slots.remove(&data.key).is_some() {
                            ctx.outs.push_back(Event::KvEvent(KvEvent::Del(Some(ctx.remote.clone()), data.key.clone())));
                        }
                    }
                }
                self.clear_satisfied_fetch_changed();
            } else {
                let from = self.version + 1;
                let count = *entry.key() - self.version - 1;
                log::warn!("[RemoteStore {:?}] apply pendings discontinuity => request fetch changed from {from:?} count {count}", ctx.remote);
                self.request_fetch_changed(ctx, now, from, count);
                break;
            }
        }

        if self.pendings.is_empty() {
            if let Some(latest) = self.latest_seen_version {
                if latest > self.version {
                    let from = self.version + 1;
                    let count = latest - self.version;
                    log::warn!("[RemoteStore {:?}] pendings empty but have newer latest_seen_version => request catch up from {from:?} count {count}", ctx.remote);
                    self.request_fetch_changed(ctx, now, from, count);
                }
            }
        }
    }

    fn enqueue_pending_changed(&mut self, ctx: &mut StateCtx<N, K, V>, changed: Changed<K, V>) -> bool {
        if ctx.next_state.is_some() {
            return false;
        }
        if changed.version <= self.version {
            return false;
        }
        if self.pendings.contains_key(&changed.version) {
            return false;
        }
        if self.pendings.len() >= MAX_PENDING_CHANGEDS {
            log::warn!("[RemoteStore {:?}] pending changed cap {MAX_PENDING_CHANGEDS} exceeded => switch to full sync", ctx.remote);
            self.pendings.clear();
            self.sending_req = None;
            ctx.next_state = Some(RemoteStoreState::SyncFull(SyncFullState::preserve_existing_until_complete()));
            return true;
        }
        self.latest_seen_version = Some(self.latest_seen_version.map_or(changed.version, |v| v.max(changed.version)));
        self.pendings.insert(changed.version, changed);
        true
    }
}

impl<N, K, V> State<N, K, V> for WorkingState<N, K, V>
where
    K: Debug + Hash + Ord + Eq + Clone,
    V: Debug + Clone,
    N: Debug + Clone,
{
    fn init(&mut self, ctx: &mut StateCtx<N, K, V>, _now: Instant) {
        log::info!("[RemoteStore {:?}] switch to working", ctx.remote);
    }

    fn on_tick(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant) {
        if let Some((sent_at, req)) = self.sending_req.as_mut() {
            if now - *sent_at >= REQUEST_TIMEOUT {
                log::warn!("[RemoteStore {:?}] fetch changed timeout", ctx.remote);
                *sent_at = now;
                ctx.outs.push_back(Event::NetEvent(req.clone()));
            }
        }
    }

    fn on_broadcast(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant, event: BroadcastEvent<K, V>) -> bool {
        match event.data {
            BroadcastEventData::Changed(changed) => {
                if self.enqueue_pending_changed(ctx, changed) {
                    if ctx.next_state.is_none() {
                        self.apply_pendings(ctx, now);
                    }
                    true
                } else {
                    false
                }
            }
            BroadcastEventData::Version(version) => {
                if version > self.version {
                    self.latest_seen_version = Some(self.latest_seen_version.map_or(version, |v| v.max(version)));
                    if self.pendings.is_empty() {
                        let from = self.version + 1;
                        let count = version - self.version;
                        log::warn!("[RemoteStore {:?}] received discontinuity version => request fetch changed from {from:?} count {count}", ctx.remote);
                        self.request_fetch_changed(ctx, now, from, count);
                    }
                    true
                } else if version == self.version {
                    true
                } else {
                    false
                }
            }
        }
    }

    fn on_rpc_res(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant, event: RpcRes<K, V>) -> bool {
        match event {
            RpcRes::FetchChanged(Ok(changeds)) => {
                let Some((_, NetEvent::Unicast(_, RpcEvent { data: RpcEventData::RpcReq(RpcReq::FetchChanged { from, count }), .. }))) = self.sending_req.as_ref() else {
                    return false;
                };
                if *count == 0 {
                    return false;
                }
                let requested_to = *from + count.saturating_sub(1);

                log::info!("[RemoteStore {:?}] fetch changed success with {} changeds => apply", ctx.remote, changeds.len());
                for changed in changeds {
                    self.enqueue_pending_changed(ctx, changed);
                    if ctx.next_state.is_some() {
                        return true;
                    }
                }
                self.sending_req = None;
                self.apply_pendings(ctx, now);
                if self.sending_req.is_none() && self.version < requested_to {
                    let from = self.version + 1;
                    let count = requested_to - self.version;
                    log::warn!("[RemoteStore {:?}] partial fetch changed response => request remaining changed from {from:?} count {count}", ctx.remote);
                    self.request_fetch_changed(ctx, now, from, count);
                }
                true
            }
            RpcRes::FetchChanged(Err(err)) => {
                if self.sending_req.is_none() {
                    return false;
                }
                log::info!("[RemoteStore] fetch changed error: {err:?} => switch to resyncFull");
                self.sending_req = None;
                ctx.next_state = Some(RemoteStoreState::SyncFull(SyncFullState::preserve_existing_until_complete()));
                true
            }
            RpcRes::FetchSnapshot(_, _) => {
                false
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DestroyState<N, K, V> {
    _tmp: PhantomData<(N, K, V)>,
}

impl<N, K, V> State<N, K, V> for DestroyState<N, K, V>
where
    K: Debug + Hash + Ord + Eq + Clone,
    V: Clone,
    N: Clone,
{
    fn init(&mut self, ctx: &mut StateCtx<N, K, V>, _now: Instant) {
        while let Some((k, _v)) = ctx.slots.pop_first() {
            ctx.outs.push_back(Event::KvEvent(KvEvent::Del(Some(ctx.remote.clone()), k)));
        }
    }

    fn on_tick(&mut self, _ctx: &mut StateCtx<N, K, V>, _now: Instant) {
        // dont process here
    }

    fn on_broadcast(&mut self, _ctx: &mut StateCtx<N, K, V>, _now: Instant, _event: BroadcastEvent<K, V>) -> bool {
        // dont process here
        false
    }

    fn on_rpc_res(&mut self, _ctx: &mut StateCtx<N, K, V>, _now: Instant, _event: RpcRes<K, V>) -> bool {
        // dont process here
        false
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::super::messages::{FetchChangedError, SnapshotData};

    use super::*;

    /// restore with some data
    #[test]
    fn test_restore_full_single_pkt() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);

        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchSnapshot {
                    from: None,
                    max_version: None,
                    max_items: MAX_SNAPSHOT_SLOTS_PER_PAGE as u64
                }) }
            )))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        // we don't need to resend again because it too fast
        state.on_tick(&mut ctx, now);
        assert_eq!(ctx.outs.pop_front(), None);

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(1, Slot::new(1, Version(1)))],
                    skipped_newer: vec![],
                    next_key: None,
                }),
                Version(1),
            ),
        );

        assert_eq!(ctx.slots, BTreeMap::from([(1, Slot::new(1, Version(1)))]));
        assert_eq!(ctx.next_state, Some(RemoteStoreState::Working(WorkingState::new(Version(1)))));
        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Set(Some(1), 1, 1))));
        assert_eq!(ctx.outs.pop_front(), None);

        // we don't need to resend again because it already received answer
        state.on_tick(&mut ctx, now + REQUEST_TIMEOUT + Duration::from_millis(1));
        assert_eq!(ctx.outs.pop_front(), None);
    }



    #[test]
    fn initial_full_sync_must_not_emit_partial_snapshot_before_terminal_page() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(1, Slot::new(10, Version(1)))],
                    skipped_newer: vec![],
                    next_key: Some(2),
                }),
                Version(2),
            ),
        );

        assert!(ctx.slots.is_empty(), "initial full sync must not expose partial remote slots before the terminal snapshot page");
        assert!(
            !ctx.outs.iter().any(|event| matches!(event, Event::KvEvent(_))),
            "initial full sync must not emit visible KvEvent changes until the snapshot completes"
        );
    }

    #[test]
    fn full_sync_continuation_request_must_use_next_key_and_pivot_version() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(1, Slot::new(1, Version(1)))],
                    skipped_newer: vec![],
                    next_key: Some(2),
                }),
                Version(1),
            ),
        );

        assert_eq!(
            ctx.outs.pop_back(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchSnapshot {
                    from: Some(2),
                    max_version: Some(Version(1)),
                    max_items: MAX_SNAPSHOT_SLOTS_PER_PAGE as u64,
                }) }
            ))),
            "snapshot next_key must drive the next page request with the locked pivot version"
        );
    }

    #[test]
    fn full_sync_must_accept_empty_continuation_page_with_advancing_next_key() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(1, Slot::new(1, Version(1)))],
                    skipped_newer: vec![],
                    next_key: Some(2),
                }),
                Version(1),
            ),
        );
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![],
                    skipped_newer: vec![(2, Version(2))],
                    next_key: Some(3),
                }),
                Version(1),
            ),
        );

        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchSnapshot {
                    from: Some(3),
                    max_version: Some(Version(1)),
                    max_items: MAX_SNAPSHOT_SLOTS_PER_PAGE as u64,
                }) }
            )))
        );
        assert_eq!(ctx.outs.pop_front(), None);
        assert!(ctx.slots.is_empty(), "empty slot continuation pages must remain staged until the terminal page");
    }

    #[test]
    fn full_sync_must_reject_empty_no_progress_continuation_page() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(1, Slot::new(1, Version(1)))],
                    skipped_newer: vec![],
                    next_key: Some(2),
                }),
                Version(1),
            ),
        );
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![],
                    skipped_newer: vec![],
                    next_key: Some(3),
                }),
                Version(1),
            ),
        );

        assert_eq!(ctx.outs.pop_front(), None, "empty continuation with no scanned progress must not request another page");
        assert_eq!(ctx.next_state, None, "empty continuation with no scanned progress must not complete full sync");
    }

    #[test]
    fn full_sync_must_accept_terminal_empty_snapshot_with_nonzero_version() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![],
                    skipped_newer: vec![],
                    next_key: None,
                }),
                Version(1),
            ),
        );

        assert_eq!(ctx.next_state, Some(RemoteStoreState::Working(WorkingState::new(Version(1)))));
        assert!(ctx.slots.is_empty(), "terminal empty snapshot pages complete with no staged slots");
    }

    #[test]
    fn full_sync_must_accept_snapshot_slot_without_upper_key_bound() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(2, Slot::new(2, Version(1)))],
                    skipped_newer: vec![],
                    next_key: None,
                }),
                Version(1),
            ),
        );

        assert_eq!(ctx.slots, BTreeMap::from([(2, Slot::new(2, Version(1)))]));
        assert_eq!(ctx.next_state, Some(RemoteStoreState::Working(WorkingState::new(Version(1)))));
    }

    #[test]
    fn full_sync_snapshot_pages_must_be_bounded() {
        const MAX_SNAPSHOT_SLOTS_PER_PAGE: u16 = 1024;
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        let slots = (0..=MAX_SNAPSHOT_SLOTS_PER_PAGE).map(|key| (key, Slot::new(key, Version(key as u64 + 1)))).collect::<Vec<_>>();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots,
                    skipped_newer: vec![],
                    next_key: None,
                }),
                Version(MAX_SNAPSHOT_SLOTS_PER_PAGE as u64 + 1),
            ),
        );

        assert!(
            ctx.slots.len() <= MAX_SNAPSHOT_SLOTS_PER_PAGE as usize,
            "full-sync snapshot pages must be capped, got {} slots",
            ctx.slots.len()
        );
    }

    #[test]
    fn full_sync_staged_snapshot_slots_must_be_bounded_across_pages() {
        const MAX_STAGED_FOR_TEST: usize = 16 * MAX_SNAPSHOT_SLOTS_PER_PAGE;

        let mut ctx: StateCtx<u16, u64, u64> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        for page in 0..=16 {
            let first_key = (page * MAX_SNAPSHOT_SLOTS_PER_PAGE) as u64 + 1;
            let slots = (0..MAX_SNAPSHOT_SLOTS_PER_PAGE)
                .map(|offset| {
                    let key = first_key + offset as u64;
                    (key, Slot::new(key, Version(key)))
                })
                .collect::<Vec<_>>();

            state.on_rpc_res(
                &mut ctx,
                now,
                RpcRes::FetchSnapshot(
                    Some(SnapshotData {
                        slots,
                        skipped_newer: vec![],
                        next_key: Some(first_key + MAX_SNAPSHOT_SLOTS_PER_PAGE as u64),
                    }),
                    Version((MAX_STAGED_FOR_TEST + MAX_SNAPSHOT_SLOTS_PER_PAGE + 1) as u64),
                ),
            );
            ctx.outs.clear();
        }

        let staged_len = state.staged_slots.as_ref().map(BTreeMap::len).unwrap_or_default();

        assert!(
            staged_len <= MAX_STAGED_FOR_TEST,
            "full sync must cap total staged snapshot slots across continuation pages; got {staged_len}"
        );
        assert!(ctx.slots.is_empty(), "partial full-sync data must remain staged until the terminal page");
    }



    #[test]
    fn full_sync_mismatched_continuation_version_must_restart_sync() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(1, Slot::new(1, Version(1)))],
                    skipped_newer: vec![],
                    next_key: Some(2),
                }),
                Version(1),
            ),
        );
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(2, Slot::new(2, Version(2)))],
                    skipped_newer: vec![],
                    next_key: None,
                }),
                Version(2),
            ),
        );

        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent {
                    session_id: 1,
                    data: RpcEventData::RpcReq(RpcReq::FetchSnapshot {
                        from: None,
                        max_version: None,
                        max_items: MAX_SNAPSHOT_SLOTS_PER_PAGE as u64,
                    })
                }
            ))),
            "continuation mismatched version must restart full sync from scratch"
        );
        assert_eq!(ctx.outs.pop_front(), None);
        assert!(!ctx.slots.contains_key(&2), "continuation snapshot page with a different declared version must be rejected");
        assert_eq!(
            ctx.next_state, None,
            "full sync must not transition to WorkingState after accepting data newer than the locked snapshot version"
        );
    }



    #[test]
    fn full_sync_must_reject_stale_terminal_snapshot_after_continuation_request() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(1, Slot::new(10, Version(1)))],
                    skipped_newer: vec![],
                    next_key: Some(2),
                }),
                Version(3),
            ),
        );

        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchSnapshot {
                    from: Some(2),
                    max_version: Some(Version(3)),
                    max_items: MAX_SNAPSHOT_SLOTS_PER_PAGE as u64,
                }) }
            )))
        );
        assert_eq!(ctx.outs.pop_front(), None);
        assert!(ctx.slots.is_empty(), "initial full sync must stage partial snapshot pages until completion");

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(1, Slot::new(10, Version(1)))],
                    skipped_newer: vec![],
                    next_key: None,
                }),
                Version(1),
            ),
        );

        assert_eq!(
            ctx.next_state, None,
            "stale terminal snapshot responses must not complete full sync while a continuation range is outstanding"
        );
        assert_eq!(
            ctx.slots,
            BTreeMap::new(),
            "stale terminal snapshot responses must not hide keys still pending in the requested continuation range"
        );
    }

    #[test]
    fn test_restore_full_resend() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);

        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchSnapshot {
                    from: None,
                    max_version: None,
                    max_items: MAX_SNAPSHOT_SLOTS_PER_PAGE as u64,
                }) }
            )))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        // we need to resend again because it timeout
        state.on_tick(&mut ctx, now + REQUEST_TIMEOUT + Duration::from_millis(1));
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchSnapshot {
                    from: None,
                    max_version: None,
                    max_items: MAX_SNAPSHOT_SLOTS_PER_PAGE as u64,
                }) }
            )))
        );
        assert_eq!(ctx.outs.pop_front(), None);
    }

    /// restore with some data
    #[test]
    fn test_restore_multi_single_pkt() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchSnapshot {
                    from: None,
                    max_version: None,
                    max_items: MAX_SNAPSHOT_SLOTS_PER_PAGE as u64
                }) }
            )))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        // got first sync
        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(1, Slot::new(1, Version(1)))],
                    skipped_newer: vec![],
                    next_key: Some(2),
                }),
                Version(2),
            ),
        );

        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchSnapshot {
                    from: Some(2),
                    max_version: Some(Version(2)),
                    max_items: MAX_SNAPSHOT_SLOTS_PER_PAGE as u64
                }) }
            )))
        );
        assert_eq!(ctx.outs.pop_front(), None);
        assert!(ctx.slots.is_empty(), "initial full sync must stage partial snapshot pages until completion");

        // got last sync
        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(2, Slot::new(2, Version(2)))],
                    skipped_newer: vec![],
                    next_key: None,
                }),
                Version(2),
            ),
        );

        assert_eq!(ctx.slots, BTreeMap::from([(1, Slot::new(1, Version(1))), (2, Slot::new(2, Version(2)))]));
        assert_eq!(ctx.next_state, Some(RemoteStoreState::Working(WorkingState::new(Version(2)))));
        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Set(Some(1), 1, 1))));
        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Set(Some(1), 2, 2))));
        assert_eq!(ctx.outs.pop_front(), None);
    }

    /// start with zero
    #[test]
    fn test_working_state_zero() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent { session_id: 2, data: BroadcastEventData::Changed(Changed {
                key: 1,
                version: Version(1),
                action: Action::Set(1),
            }) },
        );

        assert_eq!(ctx.slots, BTreeMap::from([(1, Slot::new(1, Version(1)))]));
        assert_eq!(ctx.next_state, None);
        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Set(Some(1), 1, 1))));
        assert_eq!(ctx.outs.pop_front(), None);
    }

    #[test]
    fn remote_delete_for_absent_key_must_not_emit_delete_event() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent { session_id: 2, data: BroadcastEventData::Changed(Changed {
                key: 7,
                version: Version(1),
                action: Action::Del,
            }) },
        );

        assert_eq!(state.version, Version(1), "a valid ordered remote delete must still advance protocol version");
        assert_eq!(ctx.slots, BTreeMap::new());
        assert_eq!(ctx.outs.pop_front(), None, "remote delete for an absent key must not emit a visible delete event");
    }

    /// start with zero but got out of sync
    #[test]
    fn test_working_state_zero_out_of_sync() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(&mut ctx, now, BroadcastEvent { session_id: 2, data: BroadcastEventData::Version(Version(1)) });

        assert_eq!(ctx.slots, BTreeMap::new());
        assert_eq!(ctx.next_state, None);
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchChanged { from: Version(1), count: 1 }) })))
        );
        assert_eq!(ctx.outs.pop_front(), None);
    }

    /// After missing changed we got Changed event
    #[test]
    fn test_working_state_missing_changed() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent { session_id: 2, data: BroadcastEventData::Changed(Changed {
                key: 1,
                version: Version(2),
                action: Action::Set(1),
            }) },
        );

        assert_eq!(state.pendings.len(), 1);
        assert_eq!(ctx.slots, BTreeMap::new());
        assert_eq!(ctx.next_state, None);
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchChanged { from: Version(1), count: 1 }) })))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent { session_id: 2, data: BroadcastEventData::Changed(Changed {
                key: 1,
                version: Version(1),
                action: Action::Set(2),
            }) },
        );

        assert_eq!(state.pendings.len(), 0);
        assert_eq!(ctx.slots, BTreeMap::from([(1, Slot::new(1, Version(2)))]));
        assert_eq!(ctx.next_state, None);
        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Set(Some(1), 1, 2))));
        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Set(Some(1), 1, 1))));
        assert_eq!(ctx.outs.pop_front(), None);

        // after received FetchChanged, it should be rejected
        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchChanged(Ok(vec![Changed {
                key: 1,
                version: Version(1),
                action: Action::Set(2),
            }])),
        );
        assert_eq!(ctx.outs.pop_front(), None);

        // after received FetchChanged it should not be resend
        state.on_tick(&mut ctx, now + REQUEST_TIMEOUT + Duration::from_millis(1));
        assert_eq!(ctx.outs.pop_front(), None);
    }

    #[test]
    fn working_state_must_cancel_fetch_changed_when_broadcast_fills_gap() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent { session_id: 2, data: BroadcastEventData::Changed(Changed {
                key: 1,
                version: Version(2),
                action: Action::Set(2),
            }) },
        );
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchChanged { from: Version(1), count: 1 }) })))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent { session_id: 2, data: BroadcastEventData::Changed(Changed {
                key: 1,
                version: Version(1),
                action: Action::Set(1),
            }) },
        );
        assert_eq!(state.version, Version(2), "broadcast gap fill should advance the working version");
        while ctx.outs.pop_front().is_some() {}

        state.on_tick(&mut ctx, now + REQUEST_TIMEOUT + Duration::from_millis(1));

        assert_eq!(ctx.outs.pop_front(), None, "FetchChanged retry must be cancelled after broadcasts fill the missing gap");
    }

    /// After missing changed we got FetchChanged response
    #[test]
    fn test_working_state_missing_changed2() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent { session_id: 2, data: BroadcastEventData::Changed(Changed {
                key: 1,
                version: Version(2),
                action: Action::Set(1),
            }) },
        );

        assert_eq!(state.pendings.len(), 1);
        assert_eq!(ctx.slots, BTreeMap::new());
        assert_eq!(ctx.next_state, None);
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchChanged { from: Version(1), count: 1 }) })))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchChanged(Ok(vec![Changed {
                key: 1,
                version: Version(1),
                action: Action::Set(2),
            }])),
        );

        assert_eq!(state.pendings.len(), 0);
        assert_eq!(ctx.slots, BTreeMap::from([(1, Slot::new(1, Version(2)))]));
        assert_eq!(ctx.next_state, None);
        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Set(Some(1), 1, 2))));
        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Set(Some(1), 1, 1))));
        assert_eq!(ctx.outs.pop_front(), None);
    }

    #[test]
    fn working_state_must_reject_unsolicited_fetch_changed_success() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchChanged(Ok(vec![Changed {
                key: 1,
                version: Version(1),
                action: Action::Set(9),
            }])),
        );

        assert_eq!(state.version, Version(0), "unsolicited FetchChanged success must not advance the working version");
        assert_eq!(ctx.slots, BTreeMap::new(), "unsolicited FetchChanged success must not mutate replicated slots");
        assert_eq!(ctx.outs.pop_front(), None, "unsolicited FetchChanged success must not emit local KvEvent changes");
    }

    #[test]
    fn working_state_must_reject_unsolicited_fetch_changed_error() {
        let now = Instant::now();
        let mut remote: RemoteStore<u16, u16, u16> = RemoteStore { session_id: 2, ctx: StateCtx {
                remote: 1,
            local_session_id: 1,
                slots: BTreeMap::from([(7, Slot::new(70, Version(1)))]),
                outs: VecDeque::new(),
                next_state: None,
            },
            state: RemoteStoreState::Working(WorkingState::new(Version(1))),
            last_active: now,
        };

        remote.on_rpc_res(RpcRes::FetchChanged(Err(FetchChangedError::MissingData)));

        assert_eq!(
            remote.ctx.slots,
            BTreeMap::from([(7, Slot::new(70, Version(1)))]),
            "unsolicited FetchChanged errors must not clear existing remote slots"
        );
        assert_eq!(remote.pop_out(), None, "unsolicited FetchChanged errors must not emit deletes or start a full resync");
        assert_eq!(
            remote.state,
            RemoteStoreState::Working(WorkingState::new(Version(1))),
            "unsolicited FetchChanged errors must not leave WorkingState"
        );
    }

    #[test]
    fn solicited_full_resync_must_not_delete_existing_slots_before_snapshot_completes() {
        let now = Instant::now();
        let mut remote: RemoteStore<u16, u16, u16> = RemoteStore { session_id: 2, ctx: StateCtx {
                remote: 1,
            local_session_id: 1,
                slots: BTreeMap::from([(1, Slot::new(10, Version(1))), (2, Slot::new(20, Version(2)))]),
                outs: VecDeque::new(),
                next_state: None,
            },
            state: RemoteStoreState::Working(WorkingState::new(Version(2))),
            last_active: now,
        };

        remote.on_broadcast(BroadcastEvent { session_id: 2, data: BroadcastEventData::Version(Version(5)) });
        assert!(matches!(
            remote.pop_out(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchChanged { from: Version(3), count: 3 }) })))
        ));
        assert_eq!(remote.pop_out(), None);

        remote.on_rpc_res(RpcRes::FetchChanged(Err(FetchChangedError::MissingData)));

        assert_eq!(
            remote.ctx.slots,
            BTreeMap::from([(1, Slot::new(10, Version(1))), (2, Slot::new(20, Version(2)))]),
            "existing remote data should remain visible until the replacement full snapshot is complete"
        );
        assert_ne!(
            remote.pop_out(),
            Some(Event::KvEvent(KvEvent::Del(Some(1), 1))),
            "a legitimate full-resync fallback must not emit false deletes before it has replacement data"
        );
    }

    #[test]
    fn solicited_full_resync_commits_replacement_snapshot_on_completion() {
        let now = Instant::now();
        let mut remote: RemoteStore<u16, u16, u16> = RemoteStore { session_id: 2, ctx: StateCtx {
                remote: 1,
            local_session_id: 1,
                slots: BTreeMap::from([(1, Slot::new(10, Version(1))), (2, Slot::new(20, Version(2)))]),
                outs: VecDeque::new(),
                next_state: None,
            },
            state: RemoteStoreState::Working(WorkingState::new(Version(2))),
            last_active: now,
        };

        remote.on_broadcast(BroadcastEvent { session_id: 2, data: BroadcastEventData::Version(Version(5)) });
        assert!(matches!(
            remote.pop_out(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchChanged { from: Version(3), count: 3 }) })))
        ));
        remote.on_rpc_res(RpcRes::FetchChanged(Err(FetchChangedError::MissingData)));
        assert!(matches!(
            remote.pop_out(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchSnapshot {
                    from: None,
                    max_version: None,
                    max_items: _
                }) }
            )))
        ));
        assert_eq!(remote.pop_out(), None);

        remote.on_rpc_res(RpcRes::FetchSnapshot(
            Some(SnapshotData {
                slots: vec![(2, Slot::new(20, Version(4)))],
                skipped_newer: vec![],
                next_key: Some(3),
            }),
            Version(5),
        ));
        assert_eq!(
            remote.ctx.slots,
            BTreeMap::from([(1, Slot::new(10, Version(1))), (2, Slot::new(20, Version(2)))]),
            "partial replacement snapshots must remain staged until full resync completes"
        );
        assert!(matches!(
            remote.pop_out(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchSnapshot {
                    from: Some(3),
                    max_version: Some(Version(5)),
                    max_items: _
                }) }
            )))
        ));
        assert_eq!(remote.pop_out(), None);

        remote.on_rpc_res(RpcRes::FetchSnapshot(
            Some(SnapshotData {
                slots: vec![(3, Slot::new(30, Version(5)))],
                skipped_newer: vec![],
                next_key: None,
            }),
            Version(5),
        ));

        assert_eq!(remote.ctx.slots, BTreeMap::from([(2, Slot::new(20, Version(4))), (3, Slot::new(30, Version(5)))]));
        assert_eq!(remote.state, RemoteStoreState::Working(WorkingState::new(Version(5))));
        assert_eq!(remote.pop_out(), Some(Event::KvEvent(KvEvent::Del(Some(1), 1))));
        assert_eq!(remote.pop_out(), Some(Event::KvEvent(KvEvent::Set(Some(1), 3, 30))));
        assert_eq!(remote.pop_out(), None);
    }

    #[test]
    fn full_resync_must_not_delete_skipped_pivot_key_before_catchup() {
        let now = Instant::now();
        let mut remote: RemoteStore<u16, u16, u16> = RemoteStore { session_id: 2, ctx: StateCtx {
                remote: 1,
            local_session_id: 1,
                slots: BTreeMap::from([(1, Slot::new(10, Version(1))), (2, Slot::new(20, Version(2)))]),
                outs: VecDeque::new(),
                next_state: None,
            },
            state: RemoteStoreState::Working(WorkingState::new(Version(2))),
            last_active: now,
        };

        remote.on_broadcast(BroadcastEvent { session_id: 2, data: BroadcastEventData::Version(Version(5)) });
        assert!(matches!(
            remote.pop_out(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchChanged { from: Version(3), count: 3 }) })))
        ));
        remote.on_rpc_res(RpcRes::FetchChanged(Err(FetchChangedError::MissingData)));
        assert!(matches!(
            remote.pop_out(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchSnapshot {
                    from: None,
                    max_version: None,
                    max_items: _
                }) }
            )))
        ));

        remote.on_rpc_res(RpcRes::FetchSnapshot(
            Some(SnapshotData {
                slots: vec![(1, Slot::new(10, Version(1)))],
                skipped_newer: vec![],
                next_key: Some(2),
            }),
            Version(2),
        ));
        assert!(matches!(
            remote.pop_out(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchSnapshot {
                    from: Some(2),
                    max_version: Some(Version(2)),
                    max_items: _
                }) }
            )))
        ));

        remote.on_rpc_res(RpcRes::FetchSnapshot(
            Some(SnapshotData {
                slots: vec![],
                skipped_newer: vec![(2, Version(3))],
                next_key: None,
            }),
            Version(2),
        ));

        assert_eq!(
            remote.pop_out(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchChanged { from: Version(3), count: 1 }) }))),
            "a key skipped because it is newer than the pivot must schedule catch-up without first emitting a delete"
        );
        assert_eq!(remote.pop_out(), None);
    }

    #[test]
    fn full_sync_must_catch_up_broadcast_seen_during_snapshot() {
        let mut remote: RemoteStore<u16, u16, u16> = RemoteStore::new(1, 1, 2);
        assert!(matches!(
            remote.pop_out(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchSnapshot {
                    from: None,
                    max_version: None,
                    max_items: _
                }) }
            )))
        ));

        remote.on_rpc_res(RpcRes::FetchSnapshot(
            Some(SnapshotData {
                slots: vec![(1, Slot::new(10, Version(1)))],
                skipped_newer: vec![],
                next_key: Some(2),
            }),
            Version(2),
        ));
        assert!(matches!(
            remote.pop_out(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchSnapshot {
                    from: Some(2),
                    max_version: Some(Version(2)),
                    max_items: _
                }) }
            )))
        ));

        remote.on_broadcast(BroadcastEvent { session_id: 2, data: BroadcastEventData::Changed(Changed {
            key: 2,
            version: Version(3),
            action: Action::Set(21),
        }) });

        remote.on_rpc_res(RpcRes::FetchSnapshot(
            Some(SnapshotData {
                slots: vec![],
                skipped_newer: vec![],
                next_key: None,
            }),
            Version(2),
        ));
        assert_eq!(remote.pop_out(), Some(Event::KvEvent(KvEvent::Set(Some(1), 1, 10))));

        assert_eq!(
            remote.pop_out(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchChanged { from: Version(3), count: 1 }) }))),
            "a Changed broadcast observed during full sync must not be dropped without scheduling catch-up"
        );
    }

    #[test]
    fn ignored_rpc_response_must_not_refresh_remote_activity() {
        let stale = Instant::now() - Duration::from_secs(11);
        let mut remote: RemoteStore<u16, u16, u16> = RemoteStore { session_id: 2, ctx: StateCtx {
                remote: 1,
            local_session_id: 1,
                slots: BTreeMap::from([(7, Slot::new(70, Version(1)))]),
                outs: VecDeque::new(),
                next_state: None,
            },
            state: RemoteStoreState::Working(WorkingState::new(Version(1))),
            last_active: stale,
        };

        remote.on_rpc_res(RpcRes::FetchSnapshot(None, Version(99)));

        assert_eq!(
            remote.last_active(),
            stale,
            "ignored or unsolicited RPC responses must not refresh remote activity and prevent timeout cleanup"
        );
        assert_eq!(remote.pop_out(), None, "ignored RPC responses must not emit local events");
    }

    #[test]
    fn old_version_broadcast_must_not_refresh_remote_activity() {
        let stale = Instant::now() - Duration::from_secs(11);
        let mut remote: RemoteStore<u16, u16, u16> = RemoteStore { session_id: 2, ctx: StateCtx {
                remote: 1,
            local_session_id: 1,
                slots: BTreeMap::from([(7, Slot::new(70, Version(5)))]),
                outs: VecDeque::new(),
                next_state: None,
            },
            state: RemoteStoreState::Working(WorkingState::new(Version(5))),
            last_active: stale,
        };

        remote.on_broadcast(BroadcastEvent { session_id: 2, data: BroadcastEventData::Version(Version(4)) });

        assert_eq!(remote.last_active(), stale, "old-version broadcasts must not refresh remote activity and prevent timeout cleanup");
        assert_eq!(remote.pop_out(), None, "old-version broadcasts must not emit local events");
    }

    #[test]
    fn same_version_heartbeat_must_refresh_remote_activity() {
        let stale = Instant::now() - Duration::from_secs(11);
        let mut remote: RemoteStore<u16, u16, u16> = RemoteStore { session_id: 2, ctx: StateCtx {
                remote: 1,
            local_session_id: 1,
                slots: BTreeMap::from([(7, Slot::new(70, Version(5)))]),
                outs: VecDeque::new(),
                next_state: None,
            },
            state: RemoteStoreState::Working(WorkingState::new(Version(5))),
            last_active: stale,
        };

        remote.on_broadcast(BroadcastEvent { session_id: 2, data: BroadcastEventData::Version(Version(5)) });

        assert!(
            remote.last_active() > stale,
            "same-version heartbeats must refresh remote activity so idle but connected owners do not time out"
        );
        assert_eq!(remote.pop_out(), None, "same-version heartbeats must not emit local events");
    }



    #[test]
    fn working_state_must_not_cancel_repair_after_empty_fetch_changed_success() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(&mut ctx, now, BroadcastEvent { session_id: 2, data: BroadcastEventData::Version(Version(1)) });
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchChanged { from: Version(1), count: 1 }) })))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        state.on_rpc_res(&mut ctx, now, RpcRes::FetchChanged(Ok(vec![])));
        state.on_tick(&mut ctx, now + REQUEST_TIMEOUT + Duration::from_millis(1));

        assert!(
            ctx.next_state.is_some() || ctx.outs.pop_front().is_some(),
            "empty FetchChanged success must not cancel the in-flight repair without retrying or starting full resync"
        );
    }

    #[test]
    fn working_state_must_continue_repair_after_partial_fetch_changed_success() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(&mut ctx, now, BroadcastEvent { session_id: 2, data: BroadcastEventData::Version(Version(5)) });
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchChanged { from: Version(1), count: 5 }) })))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchChanged(Ok(vec![
                Changed {
                    key: 1,
                    version: Version(1),
                    action: Action::Set(10),
                },
                Changed {
                    key: 2,
                    version: Version(2),
                    action: Action::Set(20),
                },
            ])),
        );

        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Set(Some(1), 1, 10))));
        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Set(Some(1), 2, 20))));
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchChanged { from: Version(3), count: 3 }) }))),
            "partial FetchChanged success must continue repairing the remaining requested versions"
        );
    }

    #[test]
    fn working_state_must_not_let_stale_fetch_changed_response_cancel_newer_repair() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(&mut ctx, now, BroadcastEvent { session_id: 2, data: BroadcastEventData::Version(Version(1)) });
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchChanged { from: Version(1), count: 1 }) })))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        state.on_broadcast(&mut ctx, now, BroadcastEvent { session_id: 2, data: BroadcastEventData::Version(Version(5)) });
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchChanged { from: Version(1), count: 5 }) })))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchChanged(Ok(vec![Changed {
                key: 1,
                version: Version(1),
                action: Action::Set(10),
            }])),
        );

        while matches!(ctx.outs.pop_front(), Some(Event::KvEvent(_))) {}
        state.on_tick(&mut ctx, now + REQUEST_TIMEOUT + Duration::from_millis(1));

        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchChanged { from: Version(2), count: 4 }) }))),
            "stale response for the old narrow request must not cancel the newer repair for versions 2..=5"
        );
    }

    #[test]
    fn test_working_state_resend_timeout_fetch_changed() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent { session_id: 2, data: BroadcastEventData::Changed(Changed {
                key: 1,
                version: Version(2),
                action: Action::Set(1),
            }) },
        );

        assert_eq!(state.pendings.len(), 1);
        assert_eq!(ctx.slots, BTreeMap::new());
        assert_eq!(ctx.next_state, None);
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchChanged { from: Version(1), count: 1 }) })))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        // now after timeout we should resend
        state.on_tick(&mut ctx, now + REQUEST_TIMEOUT + Duration::from_millis(1));
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchChanged { from: Version(1), count: 1 }) })))
        );
        assert_eq!(ctx.outs.pop_front(), None);
    }

    #[test]
    fn working_state_must_not_duplicate_inflight_fetch_changed_for_same_gap() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));
        state.init(&mut ctx, now);

        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent { session_id: 2, data: BroadcastEventData::Changed(Changed {
                key: 10,
                version: Version(10),
                action: Action::Set(10),
            }) },
        );

        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchChanged { from: Version(1), count: 9 }) })))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        state.on_broadcast(
            &mut ctx,
            now + Duration::from_millis(10),
            BroadcastEvent { session_id: 2, data: BroadcastEventData::Changed(Changed {
                key: 11,
                version: Version(11),
                action: Action::Set(11),
            }) },
        );

        assert_eq!(
            ctx.outs.pop_front(),
            None,
            "same-gap FetchChanged repair is already in flight and must not be duplicated before timeout or response"
        );
    }

    #[test]
    fn working_state_must_cap_pending_future_changes() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        for version in 2..=2_050 {
            state.on_broadcast(
                &mut ctx,
                now,
                BroadcastEvent { session_id: 2, data: BroadcastEventData::Changed(Changed {
                    key: version as u16,
                    version: Version(version),
                    action: Action::Set(version as u16),
                }) },
            );
        }

        assert!(
            state.pendings.len() <= 1024,
            "future changed broadcasts from one remote must be capped to avoid unbounded memory growth"
        );
    }

    #[test]
    fn working_state_must_reject_duplicate_pending_changed_broadcast_versions() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent { session_id: 2, data: BroadcastEventData::Changed(Changed {
                key: 7,
                version: Version(2),
                action: Action::Set(20),
            }) },
        );
        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent { session_id: 2, data: BroadcastEventData::Changed(Changed {
                key: 7,
                version: Version(2),
                action: Action::Set(99),
            }) },
        );
        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent { session_id: 2, data: BroadcastEventData::Changed(Changed {
                key: 7,
                version: Version(1),
                action: Action::Set(10),
            }) },
        );

        assert_eq!(
            ctx.slots.get(&7),
            Some(&Slot::new(20, Version(2))),
            "duplicate pending Changed broadcasts must not overwrite the first accepted value for the same version"
        );
    }

    #[test]
    fn working_state_must_cap_pending_fetch_changed_response() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));
        state.on_broadcast(&mut ctx, now, BroadcastEvent { session_id: 2, data: BroadcastEventData::Version(Version(2_050)) });
        ctx.outs.clear();

        let changeds = (2..=2_050)
            .map(|version| Changed {
                key: version as u16,
                version: Version(version),
                action: Action::Set(version as u16),
            })
            .collect();

        state.on_rpc_res(&mut ctx, now, RpcRes::FetchChanged(Ok(changeds)));

        assert!(
            state.pendings.len() <= 1024,
            "future changed RPC responses from one remote must be capped to avoid unbounded memory growth, got {}",
            state.pendings.len()
        );
    }

    #[test]
    fn destroy_remote_should_clear_slots() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::from([(1, Slot::new(1, Version(1)))]),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = DestroyState { _tmp: PhantomData };
        state.init(&mut ctx, now);

        assert_eq!(ctx.slots, BTreeMap::new());
        assert_eq!(ctx.next_state, None);
        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Del(Some(1), 1))));
        assert_eq!(ctx.outs.pop_front(), None);
    }

    #[test]
    fn test_continuation_none_response_must_not_livelock() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        // Send first page of snapshot with next_key Some(2)
        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(1, Slot::new(10, Version(1)))],
                    skipped_newer: vec![],
                    next_key: Some(2),
                }),
                Version(2),
            ),
        );
        ctx.outs.clear();

        // Receive None for continuation snapshot request
        state.on_rpc_res(&mut ctx, now, RpcRes::FetchSnapshot(None, Version(2)));

        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent {
                    session_id: 1,
                    data: RpcEventData::RpcReq(RpcReq::FetchSnapshot {
                        from: None,
                        max_version: None,
                        max_items: MAX_SNAPSHOT_SLOTS_PER_PAGE as u64,
                    })
                }
            ))),
            "continuation None response must restart full sync from the first snapshot page"
        );
        assert_eq!(ctx.outs.pop_front(), None);

        // Advance time past timeout and trigger tick
        state.on_tick(&mut ctx, now + REQUEST_TIMEOUT + Duration::from_millis(1));

        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent {
                    session_id: 1,
                    data: RpcEventData::RpcReq(RpcReq::FetchSnapshot {
                        from: None,
                        max_version: None,
                        max_items: MAX_SNAPSHOT_SLOTS_PER_PAGE as u64,
                    })
                }
            ))),
            "timeout after restart must resend the first-page request, not the stale continuation"
        );
        assert_eq!(ctx.outs.pop_front(), None);
    }

    #[test]
    fn working_state_must_catch_up_new_version_broadcast_received_during_pending_gap() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            local_session_id: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(10));

        // 1. Receive BroadcastEvent::Changed for version 12.
        // Since version 11 is missing, this is a gap.
        // It should request FetchChanged from 11 count 1.
        state.on_broadcast(&mut ctx, now, BroadcastEvent { session_id: 2, data: BroadcastEventData::Changed(Changed {
            key: 1,
            version: Version(12),
            action: Action::Set(12),
        }) });

        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchChanged {
                    from: Version(11),
                    count: 1
                }) }
            )))
        );
        assert_eq!(ctx.outs.pop_front(), None);
        assert_eq!(state.version, Version(10));
        assert!(state.sending_req.is_some());
        assert!(!state.pendings.is_empty()); // contains 12

        // 2. Receive BroadcastEvent::Version for version 15.
        // Because self.pendings is not empty, this version heartbeat is currently ignored.
        state.on_broadcast(&mut ctx, now, BroadcastEvent { session_id: 2, data: BroadcastEventData::Version(Version(15)) });

        // 3. Receive the RPC response for version 11.
        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchChanged(Ok(vec![Changed {
                key: 1,
                version: Version(11),
                action: Action::Set(11),
            }])),
        );

        // After applying version 11 and 12, state.version must be 12.
        assert_eq!(state.version, Version(12));
        assert_eq!(state.pendings.len(), 0);

        // Clear any KvEvents (like Set) from outs to inspect NetEvents
        let net_events: Vec<_> = ctx.outs.iter().filter(|e| matches!(e, Event::NetEvent(_))).collect();
        assert_eq!(
            net_events,
            vec![&Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent { session_id: 1, data: RpcEventData::RpcReq(RpcReq::FetchChanged {
                    from: Version(13),
                    count: 3
                }) }
            ))],
            "Node B must schedule a catch-up request for the remaining versions 13-15 after resolving the gap"
        );
    }
}

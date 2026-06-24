use crate::PeerId;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub trait HandshakeProtocol: Send + Sync + 'static {
    fn create_request(&self, from: PeerId, to: PeerId, now: u64) -> Vec<u8>;
    fn verify_request(&self, data: Vec<u8>, expected_from: PeerId, expected_to: PeerId, now: u64) -> Result<(), String>;
    fn create_response(&self, from: PeerId, to: PeerId, now: u64) -> Vec<u8>;
    fn verify_response(&self, data: Vec<u8>, expected_from: PeerId, expected_to: PeerId, now: u64) -> Result<(), String>;
}

const HASH_SEED: &str = "atm0s-small-p2p";
const HANDSHAKE_TIMEOUT: u64 = 30_000;
const HANDSHAKE_MAX_FUTURE_SKEW: u64 = 1_000;
const HANDSHAKE_REPLAY_CACHE_MAX_ENTRIES: usize = 8192;
const HANDSHAKE_REPLAY_SEEN_BUCKETS: usize = 4;
const HANDSHAKE_REPLAY_SEEN_BITS: usize = 1 << 20;
const HANDSHAKE_REPLAY_SEEN_WORDS: usize = HANDSHAKE_REPLAY_SEEN_BITS / u64::BITS as usize;
const HANDSHAKE_REPLAY_SEEN_WINDOW_MS: u64 = HANDSHAKE_TIMEOUT + HANDSHAKE_MAX_FUTURE_SKEW;
const HANDSHAKE_REPLAY_SEEN_BUCKET_MS: u64 = HANDSHAKE_REPLAY_SEEN_WINDOW_MS.div_ceil(HANDSHAKE_REPLAY_SEEN_BUCKETS as u64);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct ReplayScope {
    from: PeerId,
    to: PeerId,
    is_initiator: bool,
}

#[derive(Debug, Clone, Copy)]
struct ReplayToken {
    expires_at: u64,
    accepted_at: u64,
}

#[derive(Debug)]
struct ReplaySeenBucket {
    started_at: u64,
    bits: Vec<u64>,
}

#[derive(Debug)]
struct ReplaySeenWindow {
    buckets: Vec<ReplaySeenBucket>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HandshakeMessage {
    payload: Vec<u8>,
    signature: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HandshakeData {
    from: PeerId,
    to: PeerId,
    timestamp: u64,
    is_initiator: bool,
    nonce: u128,
}

/// Simple secure_key protect with hash
/// Idea is we serialize HandshakeData to bytes with bincode then concat it with secure_key and a seed
/// Then compare received hash for ensuring two nodes have same secure_key
/// at_ts timestamp is used for avoiding relay attach, if it older than HANDSHAKE_TIMEOUT then we reject
pub struct SharedKeyHandshake {
    secure_key: String,
    accepted_tokens: Mutex<HashMap<ReplayScope, HashMap<[u8; 32], ReplayToken>>>,
    seen_tokens: Mutex<ReplaySeenWindow>,
}

impl From<&str> for SharedKeyHandshake {
    fn from(value: &str) -> Self {
        Self {
            secure_key: value.to_owned(),
            accepted_tokens: Mutex::new(HashMap::new()),
            seen_tokens: Mutex::new(ReplaySeenWindow::new()),
        }
    }
}

impl SharedKeyHandshake {
    fn generate_handshake(&self, from: PeerId, to: PeerId, is_client: bool, now: u64) -> Vec<u8> {
        let handshake_data = HandshakeData {
            from,
            to,
            timestamp: now,
            is_initiator: is_client,
            nonce: rand::random(),
        };

        let data = bincode::serialize(&handshake_data).unwrap();
        let mut hash_input = data.clone();
        hash_input.extend_from_slice(self.secure_key.as_bytes());
        hash_input.extend_from_slice(HASH_SEED.as_bytes());

        let hash = blake3::hash(&hash_input).as_bytes().to_vec();

        let handshake = HandshakeMessage { payload: data, signature: hash };
        bincode::serialize(&handshake).unwrap()
    }

    fn validate_handshake(&self, data: Vec<u8>, expected_from: PeerId, expected_to: PeerId, expected_is_client: bool, current_ts: u64) -> Result<(), String> {
        let handshake: HandshakeMessage = bincode::deserialize(&data).map_err(|_| "Invalid handshake format".to_string())?;

        let handshake_data: HandshakeData = bincode::deserialize(&handshake.payload).map_err(|_| "Invalid handshake data format".to_string())?;

        let Some(max_allowed_timestamp) = current_ts.checked_add(HANDSHAKE_MAX_FUTURE_SKEW) else {
            return Err(format!("Handshake verifier timestamp overflow {current_ts}"));
        };
        if handshake_data.timestamp > max_allowed_timestamp {
            return Err(format!("Handshake timestamp too far in future {} vs {}", current_ts, handshake_data.timestamp));
        }

        let Some(expires_at) = handshake_data.timestamp.checked_add(HANDSHAKE_TIMEOUT) else {
            return Err(format!("Handshake timestamp overflow {}", handshake_data.timestamp));
        };

        // Verify timestamp
        if current_ts > expires_at {
            return Err(format!("Handshake timeout {} vs {}", current_ts, handshake_data.timestamp));
        }

        // Verify peer IDs
        if handshake_data.from != expected_from || handshake_data.to != expected_to {
            return Err("Invalid peer IDs".to_string());
        }

        // Verify client/server role
        if handshake_data.is_initiator != expected_is_client {
            return Err("Invalid client/server role".to_string());
        }

        // Verify hash
        let mut hash_input = handshake.payload;
        hash_input.extend_from_slice(self.secure_key.as_bytes());
        hash_input.extend_from_slice(HASH_SEED.as_bytes());
        let expected_hash = blake3::hash(&hash_input).as_bytes().to_vec();

        if handshake.signature != expected_hash {
            return Err("Invalid handshake hash".to_string());
        }

        let token_id: [u8; 32] = expected_hash.as_slice().try_into().map_err(|_| "Invalid handshake hash length".to_string())?;
        let mut accepted_tokens = self.accepted_tokens.lock();
        let mut total_tokens = 0;
        accepted_tokens.retain(|_, tokens| {
            tokens.retain(|_, accepted_token| current_ts <= accepted_token.expires_at);
            total_tokens += tokens.len();
            !tokens.is_empty()
        });
        let scope = ReplayScope {
            from: handshake_data.from,
            to: handshake_data.to,
            is_initiator: handshake_data.is_initiator,
        };
        if accepted_tokens.get(&scope).is_some_and(|tokens| tokens.contains_key(&token_id)) {
            return Err("Handshake token replayed".to_string());
        }

        let mut seen_tokens = self.seen_tokens.lock();
        if seen_tokens.contains(current_ts, &token_id) {
            return Err("Handshake token replayed".to_string());
        }

        if total_tokens >= HANDSHAKE_REPLAY_CACHE_MAX_ENTRIES {
            Self::evict_oldest_replay_token(&mut accepted_tokens);
        }
        accepted_tokens.entry(scope).or_default().insert(token_id, ReplayToken { expires_at, accepted_at: current_ts });
        seen_tokens.insert(current_ts, &token_id);

        Ok(())
    }

    fn evict_oldest_replay_token(accepted_tokens: &mut HashMap<ReplayScope, HashMap<[u8; 32], ReplayToken>>) {
        let mut oldest: Option<(ReplayScope, [u8; 32], u64)> = None;
        for (scope, tokens) in accepted_tokens.iter() {
            for (token_id, token) in tokens.iter() {
                if oldest.is_none_or(|(_, _, accepted_at)| token.accepted_at < accepted_at) {
                    oldest = Some((*scope, *token_id, token.accepted_at));
                }
            }
        }
        if let Some((scope, token_id, _)) = oldest {
            if let Some(tokens) = accepted_tokens.get_mut(&scope) {
                tokens.remove(&token_id);
                if tokens.is_empty() {
                    accepted_tokens.remove(&scope);
                }
            }
        }
    }
}

impl ReplaySeenWindow {
    fn new() -> Self {
        Self {
            buckets: (0..HANDSHAKE_REPLAY_SEEN_BUCKETS)
                .map(|_| ReplaySeenBucket {
                    started_at: u64::MAX,
                    bits: vec![0; HANDSHAKE_REPLAY_SEEN_WORDS],
                })
                .collect(),
        }
    }

    fn contains(&mut self, now: u64, token_id: &[u8; 32]) -> bool {
        self.rotate(now);
        self.buckets
            .iter()
            .filter(|bucket| bucket.started_at != u64::MAX && now <= bucket.started_at.saturating_add(HANDSHAKE_REPLAY_SEEN_WINDOW_MS))
            .any(|bucket| Self::token_bits(token_id).into_iter().all(|bit| bucket.bit_is_set(bit)))
    }

    fn insert(&mut self, now: u64, token_id: &[u8; 32]) {
        self.rotate(now);
        let bucket = self.current_bucket(now);
        for bit in Self::token_bits(token_id) {
            bucket.set_bit(bit);
        }
    }

    fn rotate(&mut self, now: u64) {
        let bucket_start = Self::bucket_start(now);
        let index = Self::bucket_index(bucket_start);
        let bucket = &mut self.buckets[index];
        if bucket.started_at != bucket_start {
            bucket.started_at = bucket_start;
            bucket.bits.fill(0);
        }
    }

    fn current_bucket(&mut self, now: u64) -> &mut ReplaySeenBucket {
        let bucket_start = Self::bucket_start(now);
        let index = Self::bucket_index(bucket_start);
        &mut self.buckets[index]
    }

    fn bucket_start(now: u64) -> u64 {
        now / HANDSHAKE_REPLAY_SEEN_BUCKET_MS * HANDSHAKE_REPLAY_SEEN_BUCKET_MS
    }

    fn bucket_index(bucket_start: u64) -> usize {
        ((bucket_start / HANDSHAKE_REPLAY_SEEN_BUCKET_MS) as usize) % HANDSHAKE_REPLAY_SEEN_BUCKETS
    }

    fn token_bits(token_id: &[u8; 32]) -> [usize; 4] {
        let mut bits = [0; 4];
        for (idx, chunk) in token_id.chunks_exact(8).enumerate() {
            let value = u64::from_le_bytes(chunk.try_into().expect("token chunks are 8 bytes"));
            bits[idx] = value as usize % HANDSHAKE_REPLAY_SEEN_BITS;
        }
        bits
    }
}

impl ReplaySeenBucket {
    fn bit_is_set(&self, bit: usize) -> bool {
        let word = bit / u64::BITS as usize;
        let offset = bit % u64::BITS as usize;
        self.bits[word] & (1 << offset) != 0
    }

    fn set_bit(&mut self, bit: usize) {
        let word = bit / u64::BITS as usize;
        let offset = bit % u64::BITS as usize;
        self.bits[word] |= 1 << offset;
    }
}

impl HandshakeProtocol for SharedKeyHandshake {
    fn create_request(&self, from: PeerId, to: PeerId, now: u64) -> Vec<u8> {
        self.generate_handshake(from, to, true, now)
    }

    fn verify_request(&self, data: Vec<u8>, expected_from: PeerId, expected_to: PeerId, now: u64) -> Result<(), String> {
        self.validate_handshake(data, expected_from, expected_to, true, now)
    }

    fn create_response(&self, from: PeerId, to: PeerId, now: u64) -> Vec<u8> {
        self.generate_handshake(from, to, false, now)
    }

    fn verify_response(&self, data: Vec<u8>, expected_from: PeerId, expected_to: PeerId, now: u64) -> Result<(), String> {
        self.validate_handshake(data, expected_from, expected_to, false, now)
    }
}

#[cfg(test)]
mod tests {
    use crate::now_ms;

    use super::*;

    #[test]
    fn test_handshake_flow() {
        let secure = SharedKeyHandshake::from("test_key");
        let peer1 = PeerId::from(1);
        let peer2 = PeerId::from(2);

        // Test request handshake
        let request = secure.create_request(peer1, peer2, now_ms());
        assert!(secure.verify_request(request, peer1, peer2, now_ms()).is_ok());

        // Test response handshake
        let response = secure.create_response(peer2, peer1, now_ms());
        assert!(secure.verify_response(response, peer2, peer1, now_ms()).is_ok());
    }

    #[test]
    fn test_invalid_handshake() {
        let secure1 = SharedKeyHandshake::from("key1");
        let secure2 = SharedKeyHandshake::from("key2");
        let peer1 = PeerId::from(1);
        let peer2 = PeerId::from(2);

        let request = secure1.create_request(peer1, peer2, now_ms());
        assert!(secure2.verify_request(request, peer1, peer2, now_ms()).is_err());
    }

    #[test]
    fn test_handshake_timeout() {
        let secure = SharedKeyHandshake::from("test_key");
        let peer1 = PeerId::from(1);
        let peer2 = PeerId::from(2);

        // when date of peer2 is faster than peer1
        let request = secure.create_request(peer2, peer1, 1000);
        assert!(secure.verify_request(request, peer2, peer1, 980).is_ok());

        // when peer2 is too slow
        let request = secure.create_request(peer2, peer1, 1000);
        assert!(secure.verify_request(request, peer2, peer1, 1000 + HANDSHAKE_TIMEOUT + 1).is_err());
    }

    #[test]
    fn rejects_arbitrarily_future_request_timestamp() {
        let secure = SharedKeyHandshake::from("test_key");
        let peer1 = PeerId::from(1);
        let peer2 = PeerId::from(2);

        let request = secure.create_request(peer1, peer2, 1_000_000_000);

        assert!(
            secure.verify_request(request, peer1, peer2, 1_000).is_err(),
            "future-dated handshake tokens must not be accepted before their timestamp window"
        );
    }

    #[test]
    fn request_handshake_tokens_must_not_be_replayable() {
        let secure = SharedKeyHandshake::from("test_key");
        let peer1 = PeerId::from(1);
        let peer2 = PeerId::from(2);

        let request = secure.create_request(peer1, peer2, 1_000);

        assert!(secure.verify_request(request.clone(), peer1, peer2, 1_005).is_ok());
        assert!(
            secure.verify_request(request, peer1, peer2, 1_010).is_err(),
            "the same request token must not authenticate a second connection"
        );
    }

    #[test]
    fn response_handshake_tokens_must_not_be_replayable() {
        let secure = SharedKeyHandshake::from("test_key");
        let client = PeerId::from(1);
        let server = PeerId::from(2);

        let response = secure.create_response(server, client, 1_000);

        assert!(secure.verify_response(response.clone(), server, client, 1_005).is_ok());
        assert!(
            secure.verify_response(response, server, client, 1_010).is_err(),
            "the same response token must not authenticate a second outbound setup"
        );
    }

    #[test]
    fn replay_cache_exhaustion_must_not_reject_fresh_valid_handshake() {
        let secure = SharedKeyHandshake::from("test_key");
        let server = PeerId::from(1);

        for peer in 10..(10 + HANDSHAKE_REPLAY_CACHE_MAX_ENTRIES as u64) {
            let peer = PeerId::from(peer);
            let request = secure.create_request(peer, server, 1_000);
            assert!(secure.verify_request(request, peer, server, 1_000).is_ok(), "setup token should be accepted while filling replay cache");
        }

        let fresh_peer = PeerId::from(99_999);
        let fresh_request = secure.create_request(fresh_peer, server, 1_001);

        assert!(
            secure.verify_request(fresh_request, fresh_peer, server, 1_001).is_ok(),
            "replay protection must not let cache exhaustion reject a fresh valid handshake from an unrelated peer"
        );
    }

    #[test]
    fn replay_cache_many_scopes_must_remain_bounded() {
        let secure = SharedKeyHandshake::from("test_key");
        let server = PeerId::from(1);

        for peer_id in 10..(10 + HANDSHAKE_REPLAY_CACHE_MAX_ENTRIES as u64 + 32) {
            let peer = PeerId::from(peer_id);
            let now = 1_000 + peer_id;
            let request = secure.create_request(peer, server, now);
            assert!(secure.verify_request(request, peer, server, now).is_ok());
        }

        let accepted_tokens = secure.accepted_tokens.lock();
        let total_tokens: usize = accepted_tokens.values().map(HashMap::len).sum();
        assert!(total_tokens <= HANDSHAKE_REPLAY_CACHE_MAX_ENTRIES, "replay cache must stay globally bounded across many unique scopes");
    }

    #[test]
    fn handshake_replay_must_not_be_accepted_after_replay_cache_eviction_pressure() {
        let secure = SharedKeyHandshake::from("test_key");
        let victim = PeerId::from(1);
        let server = PeerId::from(2);

        let replayed = secure.create_request(victim, server, 1_000);
        assert!(secure.verify_request(replayed.clone(), victim, server, 1_000).is_ok());

        for idx in 0..HANDSHAKE_REPLAY_CACHE_MAX_ENTRIES {
            let peer = PeerId::from(10_000 + idx as u64);
            let request = secure.create_request(peer, server, 1_001);
            assert!(secure.verify_request(request, peer, server, 1_001).is_ok());
        }

        assert!(
            secure.verify_request(replayed, victim, server, 1_002).is_err(),
            "a live accepted handshake token must remain non-replayable even after replay-cache pressure"
        );
    }

    #[test]
    fn rejects_overflowing_request_timestamp_without_panic() {
        let secure = SharedKeyHandshake::from("test_key");
        let peer1 = PeerId::from(1);
        let peer2 = PeerId::from(2);

        let request = secure.create_request(peer1, peer2, u64::MAX);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| secure.verify_request(request, peer1, peer2, 1_000)));

        assert!(matches!(result, Ok(Err(_))), "overflowing handshake timestamps must be rejected without panicking or wrapping");
    }
}

use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
const ULID_LEN: usize = 26;
const RANDOM_MASK: u128 = (1u128 << 80) - 1;

static STATE: OnceLock<Mutex<UlidState>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdError {
    message: String,
}

impl IdError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for IdError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for IdError {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ulid(String);

impl Ulid {
    pub fn generate() -> Self {
        Self(generate_ulid())
    }

    pub fn parse(value: &str) -> Result<Self, IdError> {
        validate_ulid(value)?;
        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl Display for Ulid {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

pub fn generate_ulid() -> String {
    let mut state = STATE
        .get_or_init(|| Mutex::new(UlidState::default()))
        .lock()
        .expect("ulid state");
    state.next()
}

pub fn validate_ulid(value: &str) -> Result<(), IdError> {
    if value.len() != ULID_LEN {
        return Err(IdError::new("ULID must be 26 characters"));
    }

    for (index, byte) in value.bytes().enumerate() {
        if index == 0 && byte > b'7' {
            return Err(IdError::new("ULID timestamp prefix is out of range"));
        }
        if decode_char(byte).is_none() {
            return Err(IdError::new("ULID contains invalid characters"));
        }
    }

    Ok(())
}

#[derive(Default)]
struct UlidState {
    last_millis: u64,
    randomness: u128,
}

impl UlidState {
    fn next(&mut self) -> String {
        let millis = current_millis();
        if millis > self.last_millis {
            self.last_millis = millis;
            self.randomness = random_80_bits();
        } else {
            self.randomness = (self.randomness + 1) & RANDOM_MASK;
        }

        encode_ulid(self.last_millis, self.randomness)
    }
}

fn current_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default()
}

fn random_80_bits() -> u128 {
    let mut bytes = [0u8; 10];
    if File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut bytes))
        .is_ok()
    {
        return bytes
            .into_iter()
            .fold(0u128, |value, byte| (value << 8) | u128::from(byte));
    }

    let fallback = current_millis() as u128;
    let address = (&bytes as *const [u8; 10] as usize) as u128;
    ((fallback << 32) ^ address) & RANDOM_MASK
}

fn encode_ulid(millis: u64, randomness: u128) -> String {
    let value = (u128::from(millis & ((1u64 << 48) - 1)) << 80) | (randomness & RANDOM_MASK);
    let mut output = [b'0'; ULID_LEN];

    for index in (0..ULID_LEN).rev() {
        let shift = (ULID_LEN - 1 - index) * 5;
        output[index] = ALPHABET[((value >> shift) & 31) as usize];
    }

    String::from_utf8(output.to_vec()).expect("ulid alphabet")
}

fn decode_char(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'A'..=b'H' => Some(value - b'A' + 10),
        b'J'..=b'K' => Some(value - b'J' + 18),
        b'M'..=b'N' => Some(value - b'M' + 20),
        b'P'..=b'T' => Some(value - b'P' + 22),
        b'V'..=b'Z' => Some(value - b'V' + 27),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{generate_ulid, validate_ulid};
    use std::collections::BTreeSet;

    #[test]
    fn generated_ulids_are_canonical() {
        let id = generate_ulid();

        assert_eq!(id.len(), 26);
        validate_ulid(&id).expect("valid ulid");
    }

    #[test]
    fn generated_ulids_are_monotonic() {
        let ids = (0..128).map(|_| generate_ulid()).collect::<Vec<_>>();
        let sorted = ids.iter().cloned().collect::<BTreeSet<_>>();

        assert_eq!(ids.len(), sorted.len());
        assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn rejects_invalid_ulids() {
        assert!(validate_ulid("abc").is_err());
        assert!(validate_ulid("8ZZZZZZZZZZZZZZZZZZZZZZZZZ").is_err());
        assert!(validate_ulid("01ARZ3NDEKTSV4RRFFQ69G5FAI").is_err());
    }
}

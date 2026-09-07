use crate::{SpawnEvent, SpawnOutput, SpawnResult};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::io::{self, Read, Write};

pub(super) const VERSION: u32 = 1;
const MAX_FRAME: usize = 16 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
pub(super) enum Reply {
    Hello(u32),
    Ready(Option<u32>),
    Event(SpawnEvent),
    Finished(SpawnResult<SpawnOutput>),
}

pub(super) fn send(writer: &mut impl Write, value: &impl Serialize) -> io::Result<()> {
    let bytes = serde_json::to_vec(value)
        .map_err(|_| io::Error::other("cannot encode supervisor frame"))?;
    if bytes.len() > MAX_FRAME {
        return Err(io::Error::other("supervisor frame exceeds limit"));
    }
    writer.write_all(&(bytes.len() as u32).to_be_bytes())?;
    writer.write_all(&bytes)?;
    writer.flush()
}

pub(super) fn receive<T: DeserializeOwned>(reader: &mut impl Read) -> io::Result<T> {
    let mut length = [0; 4];
    reader.read_exact(&mut length)?;
    let length = u32::from_be_bytes(length) as usize;
    if length == 0 || length > MAX_FRAME {
        return Err(io::Error::other("invalid supervisor frame length"));
    }
    let mut bytes = vec![0; length];
    reader.read_exact(&mut bytes)?;
    serde_json::from_slice(&bytes).map_err(|_| io::Error::other("invalid supervisor frame"))
}

pub(super) fn remap(event: &mut SpawnEvent, id: u64) {
    match event {
        SpawnEvent::Started { spawn_id, .. }
        | SpawnEvent::Stdout { spawn_id, .. }
        | SpawnEvent::Stderr { spawn_id, .. }
        | SpawnEvent::Terminal { spawn_id, .. }
        | SpawnEvent::StdinClosed { spawn_id }
        | SpawnEvent::ResizeApplied { spawn_id, .. }
        | SpawnEvent::Exit { spawn_id, .. }
        | SpawnEvent::Timeout { spawn_id, .. }
        | SpawnEvent::Canceled { spawn_id, .. }
        | SpawnEvent::Error { spawn_id, .. } => *spawn_id = id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn protocol_is_bounded_and_diagnostics_never_echo_payloads() {
        let mut bytes = Vec::new();
        send(&mut bytes, &Reply::Hello(VERSION)).unwrap();
        assert!(matches!(
            receive::<Reply>(&mut bytes.as_slice()).unwrap(),
            Reply::Hello(VERSION)
        ));
        assert!(receive::<Reply>(&mut u32::MAX.to_be_bytes().as_slice()).is_err());
        let mut bad = Vec::new();
        send(&mut bad, &"private-secret").unwrap();
        let error = receive::<Reply>(&mut bad.as_slice()).err().unwrap();
        assert_eq!(error.to_string(), "invalid supervisor frame");
        assert!(receive::<Reply>(&mut bytes[..bytes.len() - 1].as_ref()).is_err());
    }
}

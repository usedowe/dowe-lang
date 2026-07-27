use dowe_compiler::RtpConfig;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct RtpPortPool {
    config: RtpConfig,
    available: BTreeSet<u16>,
    leased: BTreeSet<u16>,
}

impl RtpPortPool {
    pub fn new(config: RtpConfig) -> Self {
        let available = (config.min..=config.max).collect::<BTreeSet<_>>();
        Self {
            config,
            available,
            leased: BTreeSet::new(),
        }
    }

    pub fn bind(&self) -> &str {
        &self.config.bind
    }

    pub fn lease(&mut self) -> Option<u16> {
        let port = self.available.pop_first()?;
        self.leased.insert(port);
        Some(port)
    }

    pub fn release(&mut self, port: u16) -> bool {
        if !self.config.contains(port) || !self.leased.remove(&port) {
            return false;
        }
        self.available.insert(port);
        true
    }

    pub fn available_len(&self) -> usize {
        self.available.len()
    }

    pub fn leased_len(&self) -> usize {
        self.leased.len()
    }
}

#[cfg(test)]
mod tests {
    use super::RtpPortPool;
    use dowe_compiler::RtpConfig;

    #[test]
    fn leases_and_releases_ports_deterministically() {
        let mut pool = RtpPortPool::new(RtpConfig {
            bind: "0.0.0.0".to_string(),
            min: 40000,
            max: 40002,
        });

        assert_eq!(pool.bind(), "0.0.0.0");
        assert_eq!(pool.lease(), Some(40000));
        assert_eq!(pool.lease(), Some(40001));
        assert_eq!(pool.lease(), Some(40002));
        assert_eq!(pool.lease(), None);
        assert_eq!(pool.leased_len(), 3);
        assert!(pool.release(40001));
        assert!(!pool.release(40001));
        assert_eq!(pool.lease(), Some(40001));
        assert_eq!(pool.available_len(), 0);
    }
}

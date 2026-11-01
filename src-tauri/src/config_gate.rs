//! Master switch for the Config tab: when off, no Config automation or command touches hardware.
use std::sync::atomic::{AtomicBool, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(false);

pub fn enabled() -> bool {
    ENABLED.load(Ordering::SeqCst)
}

pub fn set(on: bool) {
    ENABLED.store(on, Ordering::SeqCst);
}

pub fn require() -> Result<(), String> {
    if enabled() {
        Ok(())
    } else {
        Err("System config is turned off. Enable it in Settings.".into())
    }
}

/// Reports the power source when automation should reapply a profile: on the first poll after
/// being enabled and whenever the source changes. Disabled polls read nothing and forget history.
#[cfg(any(windows, test))]
#[derive(Default)]
pub struct SourceWatch(Option<bool>);

#[cfg(any(windows, test))]
impl SourceWatch {
    pub fn changed(&mut self, enabled: bool, ac: impl FnOnce() -> Option<bool>) -> Option<bool> {
        if !enabled {
            self.0 = None;
            return None;
        }
        let ac = ac()?;
        (self.0.replace(ac) != Some(ac)).then_some(ac)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watch_applies_once_per_change_and_never_while_disabled() {
        let mut watch = SourceWatch::default();
        assert_eq!(watch.changed(false, || panic!("read while disabled")), None);
        assert_eq!(watch.changed(true, || Some(true)), Some(true));
        assert_eq!(watch.changed(true, || Some(true)), None);
        assert_eq!(watch.changed(true, || Some(false)), Some(false));
        assert_eq!(watch.changed(false, || panic!("read while disabled")), None);
        assert_eq!(watch.changed(true, || Some(false)), Some(false));
        assert_eq!(watch.changed(true, || None), None);
    }

    #[test]
    fn commands_are_rejected_while_disabled() {
        set(false);
        assert!(require().is_err());
        set(true);
        assert!(require().is_ok());
        set(false);
    }
}

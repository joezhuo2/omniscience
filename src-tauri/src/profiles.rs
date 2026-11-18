use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use tauri::{AppHandle, Manager, command};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::display::DisplayConfig;
use crate::fan_mode::{self, FanMode};
use crate::power_limits::PowerLimits;
use crate::power_mode::{PowerConfig, PowerMode};
use crate::settings::Profile;
use crate::shell::Shell;
use crate::telemetry::processes::normalize;

pub const MENU_PREFIX: &str = "profile:";

#[derive(Default, Serialize)]
pub struct ProfileResult {
    applied: Vec<String>,
    errors: Vec<String>,
}

#[derive(Debug, PartialEq)]
enum Step {
    Mode(PowerMode),
    Limits(u32, u32),
    Boost(u32),
    Refresh(u32),
    Fan(FanMode),
}

impl Step {
    fn name(&self) -> &'static str {
        match self {
            Step::Mode(_) => "Power mode",
            Step::Limits(..) => "Power limits",
            Step::Boost(_) => "CPU boost",
            Step::Refresh(_) => "Refresh rate",
            Step::Fan(_) => "Fan mode",
        }
    }

    fn run(&self, app: &AppHandle, ac: bool) -> Result<(), String> {
        match *self {
            Step::Mode(mode) => app.state::<Arc<PowerConfig>>().set(ac, mode).map(drop),
            Step::Limits(spl, sppt) => app.state::<Arc<PowerLimits>>().set(ac, spl, sppt).map(drop),
            Step::Boost(mode) => crate::cpu_boost::set(ac, mode),
            Step::Refresh(hz) => app
                .state::<Arc<DisplayConfig>>()
                .set_refresh(Some(hz))
                .map(drop),
            Step::Fan(mode) => fan_mode::write(mode),
        }
    }
}

fn steps(profile: &Profile) -> Vec<Step> {
    let mut steps = Vec::new();
    match profile.power_mode.as_deref() {
        Some("efficiency") => steps.push(Step::Mode(PowerMode::Efficiency)),
        Some("balanced") => steps.push(Step::Mode(PowerMode::Balanced)),
        Some("performance") => steps.push(Step::Mode(PowerMode::Performance)),
        _ => {}
    }
    if let (Some(spl), Some(sppt)) = (profile.spl_w, profile.sppt_w) {
        steps.push(Step::Limits(spl, sppt));
    }
    steps.extend(profile.cpu_boost.map(Step::Boost));
    steps.extend(profile.refresh_hz.map(Step::Refresh));
    match profile.fan_mode.as_deref() {
        Some("silent") => steps.push(Step::Fan(FanMode::Silent)),
        Some("performance") => steps.push(Step::Fan(FanMode::Performance)),
        Some("turbo") => steps.push(Step::Fan(FanMode::Turbo)),
        _ => {}
    }
    steps
}

fn run(app: &AppHandle, id: &str) -> Result<ProfileResult, String> {
    crate::config_gate::require()?;
    let shell = app.state::<Arc<Shell>>();
    let profile = shell
        .settings()
        .profiles
        .into_iter()
        .find(|p| p.id == id)
        .ok_or("That profile no longer exists.")?;
    let ac = crate::gpu_mode::ac_power()?;
    let mut result = ProfileResult::default();
    for step in steps(&profile) {
        match step.run(app, ac) {
            Ok(()) => result.applied.push(step.name().into()),
            Err(e) => result.errors.push(format!("{}: {e}", step.name())),
        }
    }
    let mut settings = shell.settings();
    settings.active_profile = id.into();
    shell.update_settings(app, settings)?;
    Ok(result)
}

pub fn run_logged(app: &AppHandle, id: &str) {
    match run(app, id) {
        Ok(result) => {
            for e in result.errors {
                crate::diagnostics::log_error(format_args!("profile {id}: {e}"));
            }
        }
        Err(e) => crate::diagnostics::log_error(format_args!("profile {id}: {e}")),
    }
}

#[command]
pub async fn apply_profile(app: AppHandle, id: String) -> Result<ProfileResult, String> {
    tauri::async_runtime::spawn_blocking(move || run(&app, &id))
        .await
        .map_err(|e| e.to_string())?
}

pub fn validate_hotkeys(profiles: &[Profile], show_hide: &str) -> Result<(), String> {
    let show_hide = show_hide.parse::<Shortcut>().ok();
    let mut seen: Vec<Shortcut> = Vec::new();
    for p in profiles.iter().filter(|p| !p.hotkey.is_empty()) {
        let shortcut = p
            .hotkey
            .parse::<Shortcut>()
            .map_err(|e| format!("hotkey {} for {}: {e}", p.hotkey, p.name))?;
        if Some(shortcut) == show_hide {
            return Err(format!(
                "hotkey {} for {} is the show/hide hotkey",
                p.hotkey, p.name
            ));
        }
        if seen.contains(&shortcut) {
            return Err(format!(
                "hotkey {} for {} is used by another profile",
                p.hotkey, p.name
            ));
        }
        seen.push(shortcut);
    }
    Ok(())
}

fn hotkeys(profiles: &[Profile]) -> Vec<(&str, &str)> {
    profiles
        .iter()
        .filter(|p| !p.hotkey.is_empty())
        .map(|p| (p.id.as_str(), p.hotkey.as_str()))
        .collect()
}

pub fn sync_hotkeys(app: &AppHandle, old: &[Profile], new: &[Profile]) -> Result<(), String> {
    if hotkeys(old) == hotkeys(new) {
        return Ok(());
    }
    let shortcuts = app.global_shortcut();
    for (_, key) in hotkeys(old) {
        let _ = shortcuts.unregister(key);
    }
    let register = |profiles: &[Profile]| -> Result<(), String> {
        for (id, key) in hotkeys(profiles) {
            let id = id.to_string();
            shortcuts
                .on_shortcut(key, move |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let (app, id) = (app.clone(), id.clone());
                        tauri::async_runtime::spawn_blocking(move || run_logged(&app, &id));
                    }
                })
                .map_err(|e| format!("hotkey {key}: {e}"))?;
        }
        Ok(())
    };
    register(new).inspect_err(|_| {
        for (_, key) in hotkeys(new) {
            let _ = shortcuts.unregister(key);
        }
        let _ = register(old);
    })
}

#[derive(Default)]
struct RuleState {
    matched: Option<String>,
    applied: Option<(String, String)>,
}

#[derive(Debug, PartialEq)]
enum Action {
    None,
    Apply(String),
    Restore(String),
}

fn decide(
    state: &mut RuleState,
    profiles: &[Profile],
    running: &HashSet<String>,
    active: &str,
) -> Action {
    let matched = profiles
        .iter()
        .find(|p| p.apps.iter().any(|a| running.contains(a)))
        .map(|p| p.id.clone());
    if matched == state.matched {
        return Action::None;
    }
    state.matched = matched.clone();
    match matched {
        Some(id) if id == active => Action::None,
        Some(id) => {
            let prev = state
                .applied
                .take()
                .map_or_else(|| active.to_string(), |(_, prev)| prev);
            state.applied = Some((id.clone(), prev));
            Action::Apply(id)
        }
        None => match state.applied.take() {
            Some((id, prev)) if id == active => Action::Restore(prev),
            _ => Action::None,
        },
    }
}

pub fn spawn_watcher(app: AppHandle) {
    std::thread::Builder::new()
        .name("profile-rules".into())
        .spawn(move || {
            let mut sys = System::new();
            let mut state = RuleState::default();
            loop {
                std::thread::sleep(Duration::from_secs(5));
                let shell = app.state::<Arc<Shell>>();
                let settings = shell.settings();
                if !crate::config_gate::enabled()
                    || settings.profiles.iter().all(|p| p.apps.is_empty())
                {
                    state = RuleState::default();
                    continue;
                }
                sys.refresh_processes_specifics(
                    ProcessesToUpdate::All,
                    true,
                    ProcessRefreshKind::nothing(),
                );
                let running: HashSet<String> = sys
                    .processes()
                    .values()
                    .map(|p| normalize(&p.name().to_string_lossy()))
                    .collect();
                match decide(
                    &mut state,
                    &settings.profiles,
                    &running,
                    &settings.active_profile,
                ) {
                    Action::None => {}
                    Action::Apply(id) => run_logged(&app, &id),
                    Action::Restore(id) if !id.is_empty() => run_logged(&app, &id),
                    Action::Restore(_) => {
                        let mut cleared = shell.settings();
                        cleared.active_profile.clear();
                        if let Err(e) = shell.update_settings(&app, cleared) {
                            crate::diagnostics::log_error(format_args!("profile rules: {e}"));
                        }
                    }
                }
            }
        })
        .expect("spawn profile rules watcher");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(id: &str, apps: &[&str], hotkey: &str) -> Profile {
        Profile {
            id: id.into(),
            name: id.into(),
            hotkey: hotkey.into(),
            apps: apps.iter().map(|a| a.to_string()).collect(),
            ..Profile::default()
        }
    }

    fn running(names: &[&str]) -> HashSet<String> {
        names.iter().map(|n| n.to_string()).collect()
    }

    #[test]
    fn steps_skip_unset_fields_and_need_both_limits() {
        assert!(steps(&Profile::default()).is_empty());
        let mut p = Profile {
            power_mode: Some("balanced".into()),
            spl_w: Some(45),
            cpu_boost: Some(0),
            refresh_hz: Some(60),
            fan_mode: Some("turbo".into()),
            ..Profile::default()
        };
        assert_eq!(
            steps(&p),
            vec![
                Step::Mode(PowerMode::Balanced),
                Step::Boost(0),
                Step::Refresh(60),
                Step::Fan(FanMode::Turbo)
            ]
        );
        p.sppt_w = Some(55);
        assert!(steps(&p).contains(&Step::Limits(45, 55)));
        assert_eq!(Step::Limits(45, 55).name(), "Power limits");
    }

    #[test]
    fn rules_act_only_on_transitions() {
        let profiles = [profile("a", &["game"], ""), profile("b", &["other"], "")];
        let mut s = RuleState::default();
        assert_eq!(decide(&mut s, &profiles, &running(&[]), "b"), Action::None);
        assert_eq!(
            decide(&mut s, &profiles, &running(&["game"]), "b"),
            Action::Apply("a".into())
        );
        assert_eq!(
            decide(&mut s, &profiles, &running(&["game"]), "b"),
            Action::None
        );
        assert_eq!(
            decide(&mut s, &profiles, &running(&["game"]), "b"),
            Action::None
        );
        assert_eq!(
            decide(&mut s, &profiles, &running(&[]), "a"),
            Action::Restore("b".into())
        );
        assert_eq!(decide(&mut s, &profiles, &running(&[]), "b"), Action::None);
    }

    #[test]
    fn rules_do_not_fight_a_manual_choice_or_restore_over_it() {
        let profiles = [profile("a", &["game"], "")];
        let mut s = RuleState::default();
        assert_eq!(
            decide(&mut s, &profiles, &running(&["game"]), ""),
            Action::Apply("a".into())
        );
        assert_eq!(
            decide(&mut s, &profiles, &running(&["game"]), "x"),
            Action::None
        );
        assert_eq!(decide(&mut s, &profiles, &running(&[]), "x"), Action::None);
        assert_eq!(
            decide(&mut s, &profiles, &running(&["game"]), "x"),
            Action::Apply("a".into())
        );
        assert_eq!(
            decide(&mut s, &profiles, &running(&[]), "a"),
            Action::Restore("x".into())
        );
    }

    #[test]
    fn rules_skip_an_already_active_profile_and_first_match_wins() {
        let profiles = [profile("a", &["one"], ""), profile("b", &["two"], "")];
        let mut s = RuleState::default();
        assert_eq!(
            decide(&mut s, &profiles, &running(&["one"]), "a"),
            Action::None
        );
        assert_eq!(decide(&mut s, &profiles, &running(&[]), "a"), Action::None);
        assert_eq!(
            decide(&mut s, &profiles, &running(&["one", "two"]), "x"),
            Action::Apply("a".into())
        );
    }

    #[test]
    fn hotkeys_must_parse_and_be_unique() {
        let show = "Ctrl+Alt+O";
        let ok = [profile("a", &[], "Ctrl+Alt+G"), profile("b", &[], "")];
        assert!(validate_hotkeys(&ok, show).is_ok());
        assert!(validate_hotkeys(&[profile("a", &[], "nonsense")], show).is_err());
        let dup = [
            profile("a", &[], "Ctrl+Alt+G"),
            profile("b", &[], "alt+ctrl+g"),
        ];
        assert!(validate_hotkeys(&dup, show).is_err());
        assert!(validate_hotkeys(&[profile("a", &[], "alt+control+o")], show).is_err());
    }
}

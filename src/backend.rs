use std::{
    collections::BTreeMap,
    ffi::{c_char, c_int, c_uchar, c_ulong, c_void, CString},
    fs,
    path::PathBuf,
    process::Command,
    thread,
    time::Duration,
};
#[derive(Clone, Debug)]
pub struct Rule {
    pub original: String,
    pub mapped: String,
    pub command: String,
    pub shortcut: String,
}
pub type Rules = BTreeMap<u16, Rule>;
fn run(cmd: &str, args: &[&str]) -> Result<String, String> {
    let o = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if o.status.success() {
        Ok(String::from_utf8_lossy(&o.stdout).trim().into())
    } else {
        Err(format!(
            "{}: {}",
            cmd,
            String::from_utf8_lossy(&o.stderr).trim()
        ))
    }
}
pub fn maps() -> Result<BTreeMap<u16, String>, String> {
    Ok(run("xmodmap", &["-pke"])?
        .lines()
        .filter_map(|l| {
            let (a, b) = l.split_once('=')?;
            Some((
                a.split_whitespace().nth(1)?.parse().ok()?,
                b.split_whitespace().collect::<Vec<_>>().join(" "),
            ))
        })
        .collect())
}
pub fn dir() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap()).join(".config"))
        .join("key-layout")
}
fn hex(s: &str) -> String {
    s.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
fn unhex(s: &str) -> Result<String, String> {
    if !s.is_ascii() || s.len() % 2 != 0 {
        return Err("Invalid config encoding".into());
    }
    String::from_utf8(
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?,
    )
    .map_err(|e| e.to_string())
}
pub fn load() -> Result<Rules, String> {
    let p = dir().join("rules.tsv");
    if !p.exists() {
        return Ok(Rules::new());
    }
    decode(&fs::read_to_string(p).map_err(|e| e.to_string())?)
}
fn decode(s: &str) -> Result<Rules, String> {
    let mut r = Rules::new();
    for l in s.lines() {
        let v: Vec<_> = l.split('\t').collect();
        if v.len() != 5 {
            return Err("Invalid saved rule".into());
        }
        let k = v[0].parse::<u16>().map_err(|e| e.to_string())?;
        if !(8..=255).contains(&k) || r.contains_key(&k) {
            return Err("Invalid or duplicate keycode".into());
        }
        r.insert(
            k,
            Rule {
                original: unhex(v[1])?,
                mapped: unhex(v[2])?,
                command: unhex(v[3])?,
                shortcut: unhex(v[4])?,
            },
        );
    }
    Ok(r)
}
pub fn save(r: &Rules) -> Result<(), String> {
    fs::create_dir_all(dir()).map_err(|e| e.to_string())?;
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(dir(), fs::Permissions::from_mode(0o700)).map_err(|e| e.to_string())?;
    let text = r
        .iter()
        .map(|(k, v)| {
            format!(
                "{}\t{}\t{}\t{}\t{}\n",
                k,
                hex(&v.original),
                hex(&v.mapped),
                hex(&v.command),
                hex(&v.shortcut)
            )
        })
        .collect::<String>();
    let p = dir().join("rules.tsv");
    let tmp = dir().join("rules.tmp");
    fs::write(&tmp, text).map_err(|e| e.to_string())?;
    fs::rename(tmp, p).map_err(|e| e.to_string())
}
fn mapping(k: u16, s: &str) -> Result<(), String> {
    run("xmodmap", &["-e", &format!("keycode {k} = {s}")]).map(|_| ())
}
fn property(s: &str) -> String {
    format!("/commands/custom/{s}")
}
fn shortcut(s: &str) -> Option<String> {
    run(
        "xfconf-query",
        &["-c", "xfce4-keyboard-shortcuts", "-p", &property(s)],
    )
    .ok()
}
fn remove_shortcut(r: &Rule) -> Result<(), String> {
    if r.command.is_empty() {
        return Ok(());
    }
    match shortcut(&r.shortcut) {
        None => Ok(()),
        Some(v) if v == r.command => run(
            "xfconf-query",
            &[
                "-c",
                "xfce4-keyboard-shortcuts",
                "-p",
                &property(&r.shortcut),
                "-r",
            ],
        )
        .map(|_| ()),
        _ => Err(
            "Shortcut was changed outside this app; preserve it and resolve the conflict first."
                .into(),
        ),
    }
}
fn set_shortcut(r: &Rule) -> Result<(), String> {
    if r.command.is_empty() {
        return Ok(());
    }
    if let Some(v) = shortcut(&r.shortcut) {
        if v == r.command {
            return Ok(());
        }
        return Err("This key already launches another command in XFCE.".into());
    }
    run(
        "xfconf-query",
        &[
            "-c",
            "xfce4-keyboard-shortcuts",
            "-p",
            &property(&r.shortcut),
            "-n",
            "-t",
            "string",
            "-s",
            &r.command,
        ],
    )
    .map(|_| ())
}

fn drifted_keycodes(current: &BTreeMap<u16, String>, rules: &Rules) -> Vec<u16> {
    rules
        .iter()
        .filter_map(|(keycode, rule)| {
            (current.get(keycode) != Some(&rule.mapped)).then_some(*keycode)
        })
        .collect()
}

pub fn reconcile(rules: &Rules) -> Result<usize, String> {
    let current = maps()?;
    let drifted = drifted_keycodes(&current, rules);
    let mut changed = 0;
    let mut errors = Vec::new();
    for (keycode, rule) in rules {
        if drifted.contains(keycode) {
            match mapping(*keycode, &rule.mapped) {
                Ok(()) => changed += 1,
                Err(error) => errors.push(format!("keycode {keycode}: {error}")),
            }
        }
        if let Err(error) = set_shortcut(rule) {
            errors.push(format!("keycode {keycode} shortcut: {error}"));
        }
    }
    if errors.is_empty() {
        Ok(changed)
    } else {
        Err(errors.join("; "))
    }
}

#[link(name = "libX11.so.6", kind = "dylib", modifiers = "+verbatim")]
extern "C" {
    fn XOpenDisplay(name: *const c_char) -> *mut c_void;
    fn XGetKeyboardMapping(
        display: *mut c_void,
        first_keycode: c_uchar,
        keycode_count: c_int,
        keysyms_per_keycode: *mut c_int,
    ) -> *mut c_ulong;
    fn XStringToKeysym(name: *const c_char) -> c_ulong;
    fn XFree(data: *mut c_void) -> c_int;
}

fn expected_keysyms(mapping: &str, width: usize) -> Result<Vec<c_ulong>, String> {
    let mut expected = mapping
        .split_whitespace()
        .map(|name| {
            if name == "NoSymbol" {
                return Ok(0);
            }
            let name = CString::new(name).map_err(|error| error.to_string())?;
            let keysym = unsafe { XStringToKeysym(name.as_ptr()) };
            if keysym == 0 {
                Err(format!(
                    "Unknown X11 key symbol: {}",
                    name.to_string_lossy()
                ))
            } else {
                Ok(keysym)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    if expected.len() > width {
        return Err("Saved mapping is wider than the current X11 keymap".into());
    }
    expected.resize(width, 0);
    Ok(expected)
}

fn live_drifted_keycodes(display: *mut c_void, rules: &Rules) -> Result<Vec<u16>, String> {
    let Some(first) = rules.keys().next().copied() else {
        return Ok(Vec::new());
    };
    let last = rules.keys().next_back().copied().unwrap();
    if last > u16::from(u8::MAX) {
        return Err(format!("Invalid X11 keycode: {last}"));
    }
    let count = i32::from(last - first + 1);
    let mut width = 0;
    let keysyms = unsafe { XGetKeyboardMapping(display, first as c_uchar, count, &mut width) };
    if keysyms.is_null() {
        return Err("Cannot read the current X11 keyboard mapping".into());
    }
    if width <= 0 {
        unsafe {
            XFree(keysyms.cast());
        }
        return Err("X11 returned an empty keyboard mapping".into());
    }
    let result = (|| {
        let width = width as usize;
        let mut drifted = Vec::new();
        for (keycode, rule) in rules {
            let expected = expected_keysyms(&rule.mapped, width)?;
            let offset = usize::from(*keycode - first) * width;
            let current = unsafe { std::slice::from_raw_parts(keysyms.add(offset), width) };
            if current != expected {
                drifted.push(*keycode);
            }
        }
        Ok(drifted)
    })();
    unsafe {
        XFree(keysyms.cast());
    }
    result
}

pub fn watch() -> Result<(), String> {
    let display = unsafe { XOpenDisplay(std::ptr::null()) };
    if display.is_null() {
        return Err("Cannot open X11 display for mapping monitoring".into());
    }
    // XFCE can still be restoring its own keyboard layout during login.
    thread::sleep(Duration::from_secs(3));
    reconcile(&load()?)?;
    let mut last_error = String::new();
    loop {
        thread::sleep(Duration::from_secs(1));
        let rules = load()?;
        match live_drifted_keycodes(display, &rules) {
            Ok(drifted) if drifted.is_empty() => last_error.clear(),
            Ok(_) => {
                // Let a layout switch or GUI save finish before restoring overrides.
                thread::sleep(Duration::from_millis(300));
                let rules = load()?;
                if !live_drifted_keycodes(display, &rules)?.is_empty() {
                    match reconcile(&rules) {
                        Ok(changed) => {
                            eprintln!("Reapplied {changed} saved key mapping(s).");
                            last_error.clear();
                        }
                        Err(error) if error != last_error => {
                            eprintln!("Could not reapply saved mappings: {error}");
                            last_error = error;
                        }
                        Err(_) => {}
                    }
                }
            }
            Err(error) if error != last_error => {
                eprintln!("Could not inspect saved mappings: {error}");
                last_error = error;
            }
            Err(_) => {}
        }
    }
}
pub fn available_launcher_symbol(
    current: &BTreeMap<u16, String>,
    bindings: &str,
) -> Result<String, String> {
    (13..=35)
        .map(|i| format!("F{i}"))
        .find(|sym| {
            !current
                .values()
                .any(|v| v.split_whitespace().any(|s| s == sym))
                && !bindings
                    .lines()
                    .any(|l| l.split_whitespace().next().unwrap_or("").ends_with(sym))
        })
        .ok_or_else(|| {
            "No unused launcher key is available. Restore an unused mapping first.".into()
        })
}
pub fn launcher_symbol(current: &BTreeMap<u16, String>) -> Result<String, String> {
    let bindings = run("xfconf-query", &["-c", "xfce4-keyboard-shortcuts", "-lv"])?;
    available_launcher_symbol(current, &bindings)
}
pub fn apply(k: u16, r: &Rule) -> Result<(), String> {
    let previous = maps()?.get(&k).cloned().ok_or("Unknown keycode")?;
    mapping(k, &r.mapped)?;
    if let Err(e) = set_shortcut(r) {
        mapping(k, &previous).map_err(|rollback| {
            format!("{e}; restoring the previous mapping also failed: {rollback}")
        })?;
        return Err(e);
    }
    Ok(())
}
pub fn restore(k: u16, r: &Rule) -> Result<(), String> {
    remove_shortcut(r)?;
    mapping(k, &r.original)
}
pub fn protected(s: &str) -> bool {
    s.split_whitespace().any(|s| {
        s.starts_with("Shift_")
            || s.starts_with("Control_")
            || s.starts_with("Alt_")
            || s.starts_with("Super_")
            || s.starts_with("Meta_")
            || s.contains("Lock")
            || s.starts_with("ISO_Level")
    })
}
pub fn change(rules: &mut Rules, k: u16, new: Option<Rule>) -> Result<(), String> {
    let old = rules.get(&k).cloned();
    if let Some(ref r) = old {
        restore(k, r)?;
    }
    if let Some(ref r) = new {
        if let Err(e) = apply(k, r) {
            if let Some(ref old) = old {
                let _ = apply(k, old);
            }
            return Err(e);
        }
    }
    match new {
        Some(r) => {
            rules.insert(k, r);
        }
        None => {
            rules.remove(&k);
        }
    }
    if let Err(e) = save(rules) {
        if let Some(r) = rules.get(&k) {
            let _ = restore(k, r);
        }
        match old {
            Some(r) => {
                let _ = apply(k, &r);
                rules.insert(k, r);
            }
            None => {
                rules.remove(&k);
            }
        }
        return Err(e);
    }
    Ok(())
}
pub fn startup() -> Result<(), String> {
    let p = dir().parent().unwrap().join("autostart");
    fs::create_dir_all(&p).map_err(|e| e.to_string())?;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe = exe
        .to_str()
        .ok_or("Invalid executable path")?
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('`', "\\`")
        .replace('$', "\\$")
        .replace('%', "%%");
    fs::write(p.join("key-layout.desktop"),format!("[Desktop Entry]\nType=Application\nName=Key Layout mappings\nExec=\"{exe}\" --watch\nOnlyShowIn=XFCE;\nTerminal=false\n")).map_err(|e|e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_command_roundtrip() {
        let s = "kitty --title '測試 space'\n% $";
        assert_eq!(unhex(&hex(s)).unwrap(), s);
    }
    #[test]
    fn reject_corrupt() {
        assert!(decode("102\t00").is_err());
        assert!(decode("999\t\t\t\t").is_err());
        assert!(unhex("zz").is_err());
        assert!(unhex("漢字").is_err());
    }
    #[test]
    fn protect_modifiers_only() {
        assert!(protected("Super_L NoSymbol"));
        assert!(!protected("Muhenkan NoSymbol Muhenkan"));
        assert!(!protected("BackSpace"));
    }
}

#[cfg(test)]
mod launcher_tests {
    use super::*;
    #[test]
    fn skips_mapped_and_bound_symbols() {
        let current = BTreeMap::from([(20, "F13 F14".into())]);
        assert_eq!(
            available_launcher_symbol(
                &current,
                "/commands/custom/F15 something\n/xfwm4/custom/<Super>F16 action"
            )
            .unwrap(),
            "F17"
        );
    }
    #[test]
    fn reports_exhaustion() {
        let current = BTreeMap::from([(
            20,
            (13..=35)
                .map(|i| format!("F{i}"))
                .collect::<Vec<_>>()
                .join(" "),
        )]);
        assert!(available_launcher_symbol(&current, "").is_err());
    }

    #[test]
    fn detects_only_drifted_mappings() {
        let current = BTreeMap::from([
            (102, "space space space space".into()),
            (132, "yen bar yen bar".into()),
        ]);
        let rules = BTreeMap::from([
            (
                102,
                Rule {
                    original: "Muhenkan NoSymbol Muhenkan".into(),
                    mapped: "space space space space".into(),
                    command: String::new(),
                    shortcut: String::new(),
                },
            ),
            (
                132,
                Rule {
                    original: "yen bar yen bar".into(),
                    mapped: "BackSpace BackSpace BackSpace BackSpace".into(),
                    command: String::new(),
                    shortcut: String::new(),
                },
            ),
        ]);
        assert_eq!(drifted_keycodes(&current, &rules), vec![132]);
    }
}

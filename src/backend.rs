use std::{collections::BTreeMap, fs, path::PathBuf, process::Command};
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
    fs::write(p.join("key-layout.desktop"),format!("[Desktop Entry]\nType=Application\nName=Key Layout mappings\nExec=\"{exe}\" --apply\nOnlyShowIn=XFCE;\nTerminal=false\n")).map_err(|e|e.to_string())
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
}

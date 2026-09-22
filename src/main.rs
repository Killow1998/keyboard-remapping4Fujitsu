mod backend;
use backend::*;
use std::{
    collections::BTreeMap,
    ffi::{c_char, c_int, c_void, CStr, CString},
    ptr,
};
type W = *mut c_void;
#[link(name = "libgtk-3.so.0", kind = "dylib", modifiers = "+verbatim")]
extern "C" {
    fn gtk_init_check(a: W, b: W) -> c_int;
    fn gtk_main();
    fn gtk_main_quit();
    fn gtk_window_new(t: c_int) -> W;
    fn gtk_window_set_title(w: W, s: *const c_char);
    fn gtk_window_set_default_size(w: W, x: c_int, y: c_int);
    fn gtk_container_add(w: W, c: W);
    fn gtk_container_set_border_width(w: W, n: u32);
    fn gtk_box_new(o: c_int, s: c_int) -> W;
    fn gtk_box_pack_start(w: W, c: W, e: c_int, f: c_int, p: u32);
    fn gtk_label_new(s: *const c_char) -> W;
    fn gtk_label_set_text(w: W, s: *const c_char);
    fn gtk_label_set_xalign(w: W, a: f32);
    fn gtk_label_set_line_wrap(w: W, b: c_int);
    fn gtk_button_new() -> W;
    fn gtk_button_new_with_label(s: *const c_char) -> W;
    fn gtk_widget_set_size_request(w: W, x: c_int, y: c_int);
    fn gtk_widget_set_tooltip_text(w: W, s: *const c_char);
    fn gtk_widget_set_sensitive(w: W, b: c_int);
    fn gtk_widget_show_all(w: W);
    fn gtk_widget_set_visible(w: W, b: c_int);
    fn gtk_widget_destroy(w: W);
    fn gtk_entry_new() -> W;
    fn gtk_entry_set_text(w: W, s: *const c_char);
    fn gtk_button_set_label(w: W, s: *const c_char);
    fn gtk_scrolled_window_new(a: W, b: W) -> W;
    fn gtk_widget_grab_focus(w: W);
    fn gtk_image_new_from_gicon(i: W, size: c_int) -> W;
    fn gtk_entry_get_text(w: W) -> *const c_char;
    fn gtk_entry_set_placeholder_text(w: W, s: *const c_char);
    fn gtk_combo_box_text_new() -> W;
    fn gtk_combo_box_text_append_text(w: W, s: *const c_char);
    fn gtk_combo_box_set_active(w: W, n: c_int);
    fn gtk_combo_box_get_active(w: W) -> c_int;
    fn gtk_dialog_new() -> W;
    fn gtk_window_set_transient_for(w: W, p: W);
    fn gtk_window_set_modal(w: W, b: c_int);
    fn gtk_dialog_get_content_area(w: W) -> W;
    fn gtk_dialog_add_button(w: W, s: *const c_char, r: c_int) -> W;
    fn gtk_dialog_run(w: W) -> c_int;
    fn gtk_dialog_response(w: W, r: c_int);
    fn gtk_separator_new(o: c_int) -> W;
    fn gtk_widget_get_style_context(w: W) -> W;
    fn gtk_style_context_add_class(w: W, s: *const c_char);
    fn gtk_style_context_remove_class(w: W, s: *const c_char);
    fn gtk_css_provider_new() -> W;
    fn gtk_css_provider_load_from_data(w: W, s: *const c_char, n: isize, e: W) -> c_int;
    fn gtk_style_context_add_provider_for_screen(s: W, p: W, n: u32);
    fn gtk_label_set_markup(w: W, s: *const c_char);
}
#[link(name = "libgobject-2.0.so.0", kind = "dylib", modifiers = "+verbatim")]
extern "C" {
    fn g_object_unref(p: W);
    fn g_signal_connect_data(
        w: W,
        s: *const c_char,
        f: unsafe extern "C" fn(),
        d: W,
        destroy: W,
        flags: c_int,
    ) -> u64;
}
#[link(name = "libgdk-3.so.0", kind = "dylib", modifiers = "+verbatim")]
extern "C" {
    fn gdk_event_get_keycode(e: W, k: *mut u16) -> c_int;
    fn gdk_screen_get_default() -> W;
}
#[link(name = "libglib-2.0.so.0", kind = "dylib", modifiers = "+verbatim")]
extern "C" {

    fn g_list_free(p: W);
    fn g_shell_parse_argv(s: *const c_char, n: *mut c_int, a: *mut *mut *mut c_char, e: W)
        -> c_int;
    fn g_strfreev(a: *mut *mut c_char);
    fn g_timeout_add(ms: u32, f: unsafe extern "C" fn(W) -> c_int, d: W) -> u32;
}
#[repr(C)]
struct GList {
    data: W,
    next: *mut GList,
    prev: *mut GList,
}
#[link(name = "libgio-2.0.so.0", kind = "dylib", modifiers = "+verbatim")]
extern "C" {
    fn g_desktop_app_info_new_from_filename(p: *const c_char) -> W;
    fn g_app_info_get_all() -> *mut GList;
    fn g_app_info_should_show(a: W) -> c_int;
    fn g_app_info_get_display_name(a: W) -> *const c_char;
    fn g_app_info_get_executable(a: W) -> *const c_char;
    fn g_desktop_app_info_get_filename(a: W) -> *const c_char;
    fn g_app_info_get_icon(a: W) -> W;
}
fn quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
unsafe fn text(p: *const c_char) -> String {
    if p.is_null() {
        String::new()
    } else {
        CStr::from_ptr(p).to_string_lossy().into()
    }
}
struct AppSearch {
    dialog: W,
    rows: Vec<(W, String)>,
    chosen: Option<(String, String)>,
}
struct AppPick {
    search: *mut AppSearch,
    name: String,
    command: String,
}
unsafe extern "C" fn filter_apps(w: W, d: W) {
    let search = &mut *(d as *mut AppSearch);
    let q = entry(w).to_lowercase();
    for (w, name) in &search.rows {
        gtk_widget_set_visible(*w, name.contains(&q) as i32);
    }
}
unsafe extern "C" fn pick_app(_: W, d: W) {
    let p = &*(d as *mut AppPick);
    (*p.search).chosen = Some((p.name.clone(), p.command.clone()));
    gtk_dialog_response((*p.search).dialog, 1);
}
unsafe extern "C" fn search_apps(_: W, d: W) {
    let u = &mut *(d as *mut Ui);
    let dialog = gtk_dialog_new();
    gtk_window_set_title(dialog, cs("Choose an application").as_ptr());
    gtk_window_set_transient_for(dialog, u.window);
    gtk_window_set_modal(dialog, 1);
    gtk_window_set_default_size(dialog, 520, 480);
    gtk_container_set_border_width(dialog, 16);
    let area = gtk_dialog_get_content_area(dialog);
    let query = gtk_entry_new();
    gtk_entry_set_placeholder_text(query, cs("Search apps… Firefox, Kitty, Settings").as_ptr());
    pack(area, query);
    let scroll = gtk_scrolled_window_new(ptr::null_mut(), ptr::null_mut());
    gtk_box_pack_start(area, scroll, 1, 1, 8);
    let list = gtk_box_new(1, 4);
    gtk_container_add(scroll, list);
    gtk_dialog_add_button(dialog, cs("Cancel").as_ptr(), 0);
    let mut search = AppSearch {
        dialog,
        rows: vec![],
        chosen: None,
    };
    let all = g_app_info_get_all();
    let mut node = all;
    let mut items = vec![];
    while !node.is_null() {
        let a = (*node).data;
        if g_app_info_should_show(a) != 0 {
            let name = text(g_app_info_get_display_name(a));
            let path = text(g_desktop_app_info_get_filename(a));
            if !path.is_empty() {
                items.push((
                    name,
                    text(g_app_info_get_executable(a)),
                    path,
                    g_app_info_get_icon(a),
                ));
            }
        }
        node = (*node).next;
    }
    items.sort_by_key(|x| x.0.to_lowercase());
    let mut picks = vec![];
    for (name, exe, path, icon) in items {
        let button = gtk_button_new();
        let row = gtk_box_new(0, 12);
        if !icon.is_null() {
            pack(row, gtk_image_new_from_gicon(icon, 3));
        }
        pack(row, label(&name));
        gtk_container_add(button, row);
        pack(list, button);
        let p = Box::into_raw(Box::new(AppPick {
            search: &mut search,
            name: name.clone(),
            command: format!("gio launch {}", quote(&path)),
        }));
        connect(button, "clicked", pick_app as *const (), p as W);
        picks.push(p);
        search
            .rows
            .push((button, format!("{name} {exe}").to_lowercase()));
    }
    connect(
        query,
        "changed",
        filter_apps as *const (),
        &mut search as *mut _ as W,
    );
    gtk_widget_show_all(dialog);
    gtk_widget_grab_focus(query);
    gtk_dialog_run(dialog);
    if let Some((name, cmd)) = search.chosen.take() {
        gtk_entry_set_text(u.command, cs(&cmd).as_ptr());
        gtk_button_set_label(u.app_pick, cs(&format!("Change app: {name}…")).as_ptr());
        set(u.status, &format!("Selected {name}. Click Apply to save."));
    }
    gtk_widget_destroy(dialog);
    for p in picks {
        drop(Box::from_raw(p));
    }
    let mut node = all;
    while !node.is_null() {
        g_object_unref((*node).data);
        node = (*node).next;
    }
    g_list_free(all as W);
}
unsafe fn target_button(u: &Ui) {
    let i = gtk_combo_box_get_active(u.target);
    let label = u
        .copy
        .get(i as usize)
        .map(|(name, _)| format!("Copy: {name}…"))
        .unwrap_or_else(|| "Pick from keyboard…".into());
    gtk_button_set_label(u.pointer, cs(&label).as_ptr());
}
unsafe extern "C" fn pointer_target(_: W, d: W) {
    let u = &mut *(d as *mut Ui);
    u.picking = !u.picking;
    gtk_button_set_label(
        u.pointer,
        cs(if u.picking {
            "Cancel picking"
        } else {
            "Pick from keyboard…"
        })
        .as_ptr(),
    );
    set(
        u.status,
        if u.picking {
            "Click the key you want to COPY on the keyboard above."
        } else {
            "Choose what this key should do."
        },
    );
}
unsafe fn class(w: W, s: &str, on: bool) {
    let c = gtk_widget_get_style_context(w);
    if on {
        gtk_style_context_add_class(c, cs(s).as_ptr());
    } else {
        gtk_style_context_remove_class(c, cs(s).as_ptr());
    }
}
fn cs(s: &str) -> CString {
    CString::new(s.replace('\0', "")).unwrap()
}
unsafe fn label(s: &str) -> W {
    let w = gtk_label_new(cs(s).as_ptr());
    gtk_label_set_xalign(w, 0.);
    w
}
unsafe fn pack(b: W, w: W) {
    gtk_box_pack_start(b, w, 0, 0, 0)
}
unsafe fn set(w: W, s: &str) {
    gtk_label_set_text(w, cs(s).as_ptr())
}
unsafe fn entry(w: W) -> String {
    CStr::from_ptr(gtk_entry_get_text(w))
        .to_string_lossy()
        .into()
}
unsafe fn connect(w: W, s: &str, f: *const (), d: W) {
    g_signal_connect_data(
        w,
        cs(s).as_ptr(),
        std::mem::transmute(f),
        d,
        ptr::null_mut(),
        0,
    );
}
struct Ui {
    window: W,
    selected: W,
    status: W,
    list: W,
    mode: W,
    target: W,
    command: W,
    code: u16,
    rules: Rules,
    baseline: BTreeMap<u16, String>,
    copy: Vec<(String, String)>,
    keys: BTreeMap<u16, W>,
    names: BTreeMap<u16, String>,
    last: W,
    details: W,
    pointer: W,
    app_pick: W,
    picking: bool,
}
struct Click {
    ui: *mut Ui,
    code: u16,
}
unsafe fn action_name(r: &Rule) -> String {
    if r.command.is_empty() {
        return r
            .mapped
            .split_whitespace()
            .next()
            .unwrap_or("Unassigned")
            .replace("BackSpace", "Backspace")
            .replace("space", "Space");
    }
    let mut n = 0;
    let mut argv = ptr::null_mut();
    let mut name = "App".to_string();
    if g_shell_parse_argv(cs(&r.command).as_ptr(), &mut n, &mut argv, ptr::null_mut()) != 0 {
        if n == 3 && text(*argv) == "gio" && text(*argv.add(1)) == "launch" {
            let app = g_desktop_app_info_new_from_filename(*argv.add(2));
            if !app.is_null() {
                name = text(g_app_info_get_display_name(app));
                g_object_unref(app);
            }
        } else if n > 0 {
            name = std::path::Path::new(&text(*argv))
                .file_name()
                .map(|s| s.to_string_lossy().into())
                .unwrap_or(name);
        }
        g_strfreev(argv);
    }
    name
}
unsafe extern "C" fn clear_pressed(_: W, _: W, d: W) -> c_int {
    let u = &mut *(d as *mut Ui);
    for w in u.keys.values() {
        class(*w, "pressed-key", false);
    }
    0
}
unsafe fn refresh(u: &mut Ui) {
    let name = u
        .names
        .get(&u.code)
        .cloned()
        .unwrap_or_else(|| format!("Key {}", u.code));
    set(u.selected, &format!("Change: {}", name.replace('\n', " ")));
    for (k, w) in &u.keys {
        class(*w, "selected-key", *k == u.code);
        let original = u.names.get(k).cloned().unwrap_or_else(|| k.to_string());
        if let Some(r) = u.rules.get(k) {
            let action = action_name(r);
            let short: String = action.chars().take(14).collect();
            gtk_button_set_label(*w, cs(&format!("{original}\n→ {short}")).as_ptr());
            gtk_widget_set_tooltip_text(*w, cs(&format!("{original} → {action}")).as_ptr());
        } else {
            gtk_button_set_label(*w, cs(&original).as_ptr());
        }
    }
    let original = u
        .rules
        .get(&u.code)
        .map(|r| r.original.as_str())
        .or_else(|| u.baseline.get(&u.code).map(String::as_str))
        .unwrap_or("");
    set(u.details,&format!("Keycode {} · Original: {}\nX11 / XFCE. Modifier and Fn keys are not editable.\nSaved mappings are monitored and restored automatically.",u.code,original));
    let s = u
        .rules
        .iter()
        .map(|(k, r)| {
            format!(
                "{} → {}",
                u.names
                    .get(k)
                    .cloned()
                    .unwrap_or_else(|| k.to_string())
                    .replace('\n', " "),
                if r.command.is_empty() {
                    r.mapped.split_whitespace().next().unwrap_or("").to_string()
                } else {
                    r.command.clone()
                }
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    set(
        u.list,
        if s.is_empty() {
            "No saved changes."
        } else {
            &s
        },
    );
}
unsafe extern "C" fn key_light(_: W, e: W, d: W) -> c_int {
    let u = &mut *(d as *mut Ui);
    let mut k = 0;
    if gdk_event_get_keycode(e, &mut k) != 0 {
        if let Some(w) = u.keys.get(&k) {
            class(*w, "pressed-key", true);
        }
        set(
            u.last,
            &format!(
                "Pressed: {}",
                u.names
                    .get(&k)
                    .cloned()
                    .unwrap_or_else(|| format!("Key {k}"))
                    .replace('\n', " ")
            ),
        );
    }
    0
}
unsafe extern "C" fn clear_light(d: W) -> c_int {
    class(d, "pressed-key", false);
    0
}
unsafe extern "C" fn key_release(_: W, e: W, d: W) -> c_int {
    let u = &mut *(d as *mut Ui);
    let mut k = 0;
    if gdk_event_get_keycode(e, &mut k) != 0 {
        if let Some(w) = u.keys.get(&k) {
            g_timeout_add(180, clear_light, *w);
        }
    }
    0
}
unsafe fn sync_editor(u: &mut Ui) {
    if let Some(r) = u.rules.get(&u.code).cloned() {
        if r.command.is_empty() {
            let index = if let Some(i) = u.copy.iter().position(|(_, s)| *s == r.mapped) {
                i
            } else {
                let name = action_name(&r);
                gtk_combo_box_text_append_text(u.target, cs(&name).as_ptr());
                u.copy.push((name, r.mapped));
                u.copy.len() - 1
            };
            gtk_combo_box_set_active(u.target, index as i32);
            gtk_combo_box_set_active(u.mode, 0);
        } else {
            gtk_entry_set_text(u.command, cs(&r.command).as_ptr());
            gtk_button_set_label(
                u.app_pick,
                cs(&format!("Change app: {}…", action_name(&r))).as_ptr(),
            );
            gtk_combo_box_set_active(u.mode, 1);
        }
    } else {
        gtk_combo_box_set_active(u.mode, 0);
        gtk_combo_box_set_active(u.target, 0);
        gtk_entry_set_text(u.command, cs("").as_ptr());
        gtk_button_set_label(u.app_pick, cs("Search apps…").as_ptr());
    }
    mode_changed(ptr::null_mut(), u as *mut _ as W);
}
unsafe extern "C" fn choose(_: W, d: W) {
    let d = &*(d as *mut Click);
    let u = &mut *d.ui;
    if u.picking {
        let k = d.code;
        let sym = u
            .rules
            .get(&k)
            .map(|r| r.original.clone())
            .or_else(|| u.baseline.get(&k).cloned())
            .unwrap_or_default();
        if sym.is_empty() || protected(&sym) {
            set(
                u.status,
                "Choose an ordinary key, such as Space or Backspace.",
            );
            return;
        }
        let name = u.names.get(&k).cloned().unwrap_or_else(|| k.to_string());
        gtk_combo_box_text_append_text(u.target, cs(&name).as_ptr());
        u.copy.push((name.clone(), sym));
        gtk_combo_box_set_active(u.target, (u.copy.len() - 1) as i32);
        u.picking = false;
        target_button(u);
        set(u.status, &format!("Copy {name}. Click Apply to save."));
        return;
    }
    u.code = d.code;
    refresh(u);
    sync_editor(u);
    set(u.status, "Choose what this key should do.");
}
struct Capture {
    dialog: W,
    code: u16,
}
unsafe extern "C" fn captured(_: W, e: W, d: W) -> c_int {
    let c = &mut *(d as *mut Capture);
    let mut k = 0;
    if gdk_event_get_keycode(e, &mut k) != 0 {
        c.code = k;
        gtk_dialog_response(c.dialog, 1);
    }
    1
}
unsafe fn capture(parent: W) -> Option<u16> {
    let d = gtk_dialog_new();
    gtk_window_set_title(d, cs("Identify a physical key").as_ptr());
    gtk_window_set_transient_for(d, parent);
    gtk_window_set_modal(d, 1);
    gtk_container_set_border_width(d, 24);
    pack(gtk_dialog_get_content_area(d),label("Press the physical key once. Use the mouse to cancel.\nGlobal desktop shortcuts may be handled before this window."));
    gtk_dialog_add_button(d, cs("Cancel").as_ptr(), 0);
    let mut c = Capture { dialog: d, code: 0 };
    connect(
        d,
        "key-press-event",
        captured as *const (),
        &mut c as *mut _ as W,
    );
    gtk_widget_show_all(d);
    let result = gtk_dialog_run(d);
    gtk_widget_destroy(d);
    if result == 1 {
        Some(c.code)
    } else {
        None
    }
}
unsafe extern "C" fn identify(_: W, d: W) {
    let u = &mut *(d as *mut Ui);
    if let Some(k) = capture(u.window) {
        u.code = k;
        refresh(u);
        set(
            u.status,
            "Physical key identified. Select its new action below.",
        );
    }
}
unsafe extern "C" fn target_capture(_: W, d: W) {
    let u = &mut *(d as *mut Ui);
    if let Some(k) = capture(u.window) {
        let map = u
            .rules
            .get(&k)
            .map(|r| r.original.clone())
            .or_else(|| u.baseline.get(&k).cloned());
        if let Some(s) = map {
            if protected(&s) {
                set(
                    u.status,
                    "Modifier and lock keys are not supported in this version.",
                );
                return;
            }
            let name = format!("Copy key {k}: {s}");
            gtk_combo_box_text_append_text(u.target, cs(&name).as_ptr());
            u.copy.push((name, s));
            gtk_combo_box_set_active(u.target, (u.copy.len() - 1) as i32);
            gtk_combo_box_set_active(u.mode, 0);
            target_button(u);
        }
    }
}
unsafe extern "C" fn mode_changed(_: W, d: W) {
    let u = &mut *(d as *mut Ui);
    let copy = gtk_combo_box_get_active(u.mode) == 0;
    gtk_widget_set_sensitive(u.target, copy as i32);
    gtk_widget_set_visible(u.target, 0);
    gtk_widget_set_visible(u.pointer, copy as i32);
    gtk_widget_set_visible(u.app_pick, (!copy) as i32);
    u.picking = false;
    target_button(u);
    gtk_widget_set_visible(u.command, 0);
    gtk_widget_set_sensitive(u.command, (!copy) as i32);
}
unsafe extern "C" fn apply_key(_: W, d: W) {
    let u = &mut *(d as *mut Ui);
    let result = (|| -> Result<(), String> {
        let current = maps()?;
        let original = u
            .rules
            .get(&u.code)
            .map(|r| r.original.clone())
            .or_else(|| u.baseline.get(&u.code).cloned())
            .ok_or("Unknown physical key")?;
        if protected(&original) {
            return Err(
                "Modifier and lock source keys are not supported. Choose an ordinary key.".into(),
            );
        }
        let mut r = Rule {
            original: original.clone(),
            mapped: original.clone(),
            command: String::new(),
            shortcut: String::new(),
        };
        if gtk_combo_box_get_active(u.mode) == 0 {
            let i = gtk_combo_box_get_active(u.target);
            r.mapped = u
                .copy
                .get(i as usize)
                .ok_or("Select a target key")?
                .1
                .clone();
        } else {
            r.command = entry(u.command).trim().into();
            if r.command.is_empty() {
                return Err("Click Search apps to choose an application.".into());
            }
            if r.command.contains('\n') {
                return Err("Use a single-line application command.".into());
            }
            if let Some(symbol) = original
                .split_whitespace()
                .next()
                .filter(|s| *s != "NoSymbol")
            {
                r.shortcut = symbol.into();
            } else {
                r.shortcut = launcher_symbol(&current)?;
                r.mapped = std::iter::repeat(r.shortcut.as_str())
                    .take(4)
                    .collect::<Vec<_>>()
                    .join(" ");
            }
            if current
                .iter()
                .any(|(k, v)| *k != u.code && v.split_whitespace().any(|s| s == r.shortcut))
            {
                return Err("Another key shares this symbol. A launcher would affect both; choose another key.".into());
            }
        }
        startup()?;
        change(&mut u.rules, u.code, Some(r))
    })();
    match result {
        Ok(()) => {
            refresh(u);
            set(u.status, "✓ Saved. Try your key below.")
        }
        Err(e) => set(u.status, &e),
    }
}
unsafe extern "C" fn restore_key(_: W, d: W) {
    let u = &mut *(d as *mut Ui);
    match change(&mut u.rules, u.code, None) {
        Ok(()) => {
            refresh(u);
            set(u.status, "Selected key restored.")
        }
        Err(e) => set(u.status, &e),
    }
}
unsafe extern "C" fn restore_all(_: W, d: W) {
    let u = &mut *(d as *mut Ui);
    for k in u.rules.keys().copied().collect::<Vec<_>>() {
        if let Err(e) = change(&mut u.rules, k, None) {
            set(u.status, &e);
            refresh(u);
            return;
        }
    }
    refresh(u);
    set(
        u.status,
        "All mappings created by this app have been restored.",
    );
}
unsafe extern "C" fn reapply(_: W, d: W) {
    let u = &mut *(d as *mut Ui);
    for (k, r) in &u.rules {
        if let Err(e) = apply(*k, r) {
            set(u.status, &e);
            return;
        }
    }
    set(
        u.status,
        "Saved mappings reapplied to the current X11 session.",
    );
}
unsafe extern "C" fn quit(_: W, _: W) {
    gtk_main_quit()
}
unsafe fn button(b: W, s: &str, f: *const (), u: *mut Ui) {
    let w = gtk_button_new_with_label(cs(s).as_ptr());
    pack(b, w);
    connect(w, "clicked", f, u as W);
}
fn main() {
    if let Err(e) = start() {
        eprintln!("{e}");
        std::process::exit(1)
    }
}
fn start() -> Result<(), String> {
    let rules = load()?;
    if std::env::args().any(|a| a == "--watch") {
        return watch();
    }
    if std::env::args().any(|a| a == "--apply") {
        std::thread::sleep(std::time::Duration::from_secs(3));
        reconcile(&rules)?;
        return Ok(());
    }
    if std::env::args().any(|a| a == "--restore") {
        let mut rules = rules;
        for k in rules.keys().copied().collect::<Vec<_>>() {
            change(&mut rules, k, None)?;
        }
        return Ok(());
    }
    if std::env::var("XDG_SESSION_TYPE").unwrap_or_default() == "wayland" {
        return Err("This version requires an X11 desktop session.".into());
    }
    let baseline = maps()?;
    unsafe {
        if gtk_init_check(ptr::null_mut(), ptr::null_mut()) == 0 {
            return Err("Cannot open desktop display".into());
        }
        let window = gtk_window_new(0);
        gtk_window_set_title(window, cs("Key Layout").as_ptr());
        gtk_window_set_default_size(window, 1140, 640);
        gtk_container_set_border_width(window, 20);
        let root = gtk_box_new(1, 12);
        gtk_container_add(window, root);
        let title = label("");
        gtk_label_set_markup(
            title,
            cs("<span size='xx-large' weight='bold'>Make room for your keys.</span>").as_ptr(),
        );
        pack(root, title);
        pack(root, label("Pick a key below. Give it a new job."));
        let css = gtk_css_provider_new();
        gtk_css_provider_load_from_data(css,cs("button.keycap { transition: background 120ms ease, color 120ms ease; border-radius: 8px; } button.spare-key { background-image: none; background-color: #e8e5ff; color: #413a75; } button.selected-key { border: 2px solid #7162d9; } button.keycap:disabled { background-image: none; background-color: #eeeeee; color: #aaaaaa; border-color: #dddddd; box-shadow: none; } button.pressed-key:not(:disabled) { background-image: none; background-color: #7162d9; color: white; box-shadow: 0 0 8px alpha(#7162d9,0.5); } button.apply-action { background-image: none; background-color: #6554c0; color: white; font-weight: bold; } ").as_ptr(),-1,ptr::null_mut());
        gtk_style_context_add_provider_for_screen(gdk_screen_get_default(), css, 600);
        let selected = label("");
        let status = label("Choose a key to get started.");
        gtk_label_set_line_wrap(status, 1);
        let details = label("");
        let last = label("Press a key to see it light up.");
        let list = label("");
        gtk_label_set_line_wrap(list, 1);
        let mode = gtk_combo_box_text_new();
        for s in ["Use as a key", "Open an app"] {
            gtk_combo_box_text_append_text(mode, cs(s).as_ptr());
        }
        gtk_combo_box_set_active(mode, 0);
        let target = gtk_combo_box_text_new();
        let pointer = gtk_button_new_with_label(cs("Pick from keyboard…").as_ptr());
        let app_pick = gtk_button_new_with_label(cs("Search apps…").as_ptr());
        let command = gtk_entry_new();
        gtk_entry_set_placeholder_text(
            command,
            cs("Application name or command — or use Search apps").as_ptr(),
        );
        gtk_widget_set_sensitive(command, 0);
        let mut copy = vec![];
        for (name, sym) in [
            ("Space", "space"),
            ("Backspace", "BackSpace"),
            ("Enter", "Return"),
            ("Tab", "Tab"),
            ("Escape", "Escape"),
            ("Delete", "Delete"),
            ("Left", "Left"),
            ("Right", "Right"),
            ("Up", "Up"),
            ("Down", "Down"),
            ("Home", "Home"),
            ("End", "End"),
            ("Page Up", "Prior"),
            ("Page Down", "Next"),
        ] {
            copy.push((name.into(), format!("{sym} {sym} {sym} {sym}")));
            gtk_combo_box_text_append_text(target, cs(name).as_ptr());
        }
        gtk_combo_box_set_active(target, 0);
        let u = Box::into_raw(Box::new(Ui {
            window,
            selected,
            status,
            list,
            mode,
            target,
            command,
            code: 102,
            rules,
            baseline,
            copy,
            keys: BTreeMap::new(),
            names: BTreeMap::new(),
            last,
            details,
            pointer,
            app_pick,
            picking: false,
        }));
        let rows: Vec<Vec<(&str, u16, i32)>> = vec![
            vec![
                ("Esc", 9, 55),
                ("F1", 67, 50),
                ("F2", 68, 50),
                ("F3", 69, 50),
                ("F4", 70, 50),
                ("F5", 71, 50),
                ("F6", 72, 50),
                ("F7", 73, 50),
                ("F8", 74, 50),
                ("F9", 75, 50),
                ("F10", 76, 50),
                ("F11", 95, 50),
                ("F12", 96, 50),
                ("PrtSc", 107, 55),
                ("Pause", 127, 55),
                ("Ins", 118, 50),
                ("Del", 119, 50),
            ],
            vec![
                ("` ~", 49, 55),
                ("1", 10, 50),
                ("2", 11, 50),
                ("3", 12, 50),
                ("4", 13, 50),
                ("5", 14, 50),
                ("6", 15, 50),
                ("7", 16, 50),
                ("8", 17, 50),
                ("9", 18, 50),
                ("0", 19, 50),
                ("-", 20, 50),
                ("=", 21, 50),
                ("JIS ¥", 132, 55),
                ("Backspace", 22, 112),
            ],
            vec![
                ("Tab", 23, 80),
                ("Q", 24, 50),
                ("W", 25, 50),
                ("E", 26, 50),
                ("R", 27, 50),
                ("T", 28, 50),
                ("Y", 29, 50),
                ("U", 30, 50),
                ("I", 31, 50),
                ("O", 32, 50),
                ("P", 33, 50),
                ("[", 34, 50),
                ("]", 35, 50),
                ("Enter", 36, 110),
            ],
            vec![
                ("Caps", 66, 90),
                ("A", 38, 50),
                ("S", 39, 50),
                ("D", 40, 50),
                ("F", 41, 50),
                ("G", 42, 50),
                ("H", 43, 50),
                ("J", 44, 50),
                ("K", 45, 50),
                ("L", 46, 50),
                (";", 47, 50),
                ("'", 48, 50),
                ("\\", 51, 50),
            ],
            vec![
                ("Shift", 50, 110),
                ("Z", 52, 50),
                ("X", 53, 50),
                ("C", 54, 50),
                ("V", 55, 50),
                ("B", 56, 50),
                ("N", 57, 50),
                ("M", 58, 50),
                (",", 59, 50),
                (".", 60, 50),
                ("/", 61, 50),
                ("JIS ろ", 97, 60),
                ("Shift", 62, 110),
            ],
            vec![
                ("Ctrl", 37, 65),
                ("Fn", 0, 50),
                ("Win", 133, 55),
                ("Alt", 64, 55),
                ("Blank 1", 102, 100),
                ("Space", 65, 175),
                ("Blank 2", 100, 100),
                ("Blank 3", 101, 100),
                ("Menu", 135, 55),
                ("Ctrl", 105, 55),
                ("←", 113, 50),
                ("↑", 111, 50),
                ("↓", 116, 50),
                ("→", 114, 50),
            ],
        ];
        for row in rows {
            let b = gtk_box_new(0, 4);
            for (name, code, width) in row {
                let w = gtk_button_new_with_label(cs(name).as_ptr());
                gtk_widget_set_size_request(w, width, 44);
                gtk_box_pack_start(b, w, 1, 1, 0);
                class(w, "keycap", true);
                if [100, 101, 102].contains(&code) {
                    class(w, "spare-key", true);
                }
                (*u).keys.insert(code, w);
                (*u).names.insert(code, name.to_string());
                let original = (*u)
                    .rules
                    .get(&code)
                    .map(|r| r.original.as_str())
                    .or_else(|| (*u).baseline.get(&code).map(String::as_str))
                    .unwrap_or("");
                if code == 0 || protected(original) {
                    gtk_widget_set_sensitive(w, 0);
                } else {
                    let d = Box::into_raw(Box::new(Click { ui: u, code }));
                    connect(w, "clicked", choose as *const (), d as W);
                }
            }
            pack(root, b);
        }
        pack(root, last);
        pack(root, gtk_separator_new(0));
        let actions = gtk_box_new(0, 10);
        pack(actions, selected);
        pack(actions, label("→"));
        pack(actions, mode);

        pack(actions, pointer);
        pack(actions, app_pick);
        connect(pointer, "clicked", pointer_target as *const (), u as W);
        connect(app_pick, "clicked", search_apps as *const (), u as W);

        let apply = gtk_button_new_with_label(cs("Apply").as_ptr());
        class(apply, "apply-action", true);
        pack(actions, apply);
        connect(apply, "clicked", apply_key as *const (), u as W);
        pack(root, actions);
        connect(mode, "changed", mode_changed as *const (), u as W);
        let test = gtk_entry_new();
        gtk_entry_set_placeholder_text(test, cs("Try your keys here…").as_ptr());
        gtk_widget_set_size_request(test, -1, 44);
        pack(root, test);
        pack(root, status);
        let tools = gtk_box_new(0, 8);
        button(tools, "Find a physical key…", identify as *const (), u);
        button(
            tools,
            "Copy a different key…",
            target_capture as *const (),
            u,
        );
        button(tools, "Undo this key", restore_key as *const (), u);
        button(tools, "Restore all", restore_all as *const (), u);
        button(tools, "Reapply", reapply as *const (), u);
        pack(actions, tools);
        connect(window, "key-press-event", key_light as *const (), u as W);
        connect(
            window,
            "key-release-event",
            key_release as *const (),
            u as W,
        );
        refresh(&mut *u);
        connect(
            window,
            "focus-out-event",
            clear_pressed as *const (),
            u as W,
        );
        connect(window, "destroy", quit as *const (), ptr::null_mut());
        gtk_widget_show_all(window);
        sync_editor(&mut *u);
        gtk_main();
    }
    Ok(())
}

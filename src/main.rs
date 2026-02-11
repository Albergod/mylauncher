use freedesktop_desktop_entry::{DesktopEntry, Iter};
use gtk4::gdk;
use gtk4::glib;
use gtk4::{
    prelude::*, Application, ApplicationWindow, Box, Entry, Image, Label, ListItem, ListView,
    ScrolledWindow, SignalListItemFactory, SingleSelection, StringList,
};
use shellexpand;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

const MAX_HISTORY: usize = 4;

const APP_ID: &str = "com.github.mylauncher";

#[derive(Clone)]
struct AppEntry {
    name: String,
    exec: String,
    icon: String,
    description: String,
}

fn main() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        if let Some(window) = app.active_window() {
            if window.is_visible() {
                window.close();
            } else {
                window.present();
            }
            return;
        }
        build_ui(app);
    });
    app.run();
}

fn build_ui(app: &Application) {
    let apps = load_desktop_entries();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("MyLauncher")
        .default_width(700)
        .default_height(500)
        .modal(true)
        .resizable(true)
        .decorated(false)
        .build();

    window.add_css_class("launcher-window");
    window.present();

    // Create main content box
    let main_box = Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(12)
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .build();

    // Header with title
    let header_label = Label::builder()
        .label("Launch Application")
        .css_classes(["title-1"])
        .margin_bottom(10)
        .build();
    main_box.append(&header_label);

    // Search entry with modern styling
    let search_entry = Entry::builder()
        .placeholder_text("Search applications...")
        .margin_bottom(15)
        .css_classes(["search-entry"])
        .build();

    // Create scrolled window with custom styling
    let scrolled = ScrolledWindow::builder()
        .vexpand(true)
        .min_content_height(350)
        .css_classes(["scrolled"])
        .build();

    // Create list model for applications
    let string_list = StringList::new(&[]);
    let selection_model = SingleSelection::new(Some(string_list.clone()));

    // Create factory for list items
    let factory = SignalListItemFactory::new();

    factory.connect_setup(move |_, list_item| {
        let item = list_item
            .downcast_ref::<ListItem>()
            .expect("ListItem expected");

        let hbox = Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(12)
            .margin_start(8)
            .margin_end(8)
            .margin_top(4)
            .margin_bottom(4)
            .build();

        let icon = Image::builder()
            .pixel_size(32)
            .build();

        let vbox = Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(2)
            .build();

        let name_label = Label::builder()
            .xalign(0.0)
            .css_classes(["app-name"])
            .build();

        let desc_label = Label::builder()
            .xalign(0.0)
            .css_classes(["app-description"])
            .build();

        vbox.append(&name_label);
        vbox.append(&desc_label);
        hbox.append(&icon);
        hbox.append(&vbox);
        item.set_child(Some(&hbox));
    });

    let apps_for_bind = apps.clone();
    factory.connect_bind(move |_, list_item| {
        let item = list_item
            .downcast_ref::<ListItem>()
            .expect("ListItem expected");

        if let Some(child) = item.child() {
            let hbox = child.downcast::<gtk4::Box>().unwrap();
            let icon = hbox.first_child().unwrap().downcast::<Image>().unwrap();
            let vbox = hbox.last_child().unwrap().downcast::<gtk4::Box>().unwrap();
            let name_label = vbox.first_child().unwrap().downcast::<Label>().unwrap();
            let desc_label = vbox.last_child().unwrap().downcast::<Label>().unwrap();

            if let Some(string_obj) = item.item() {
                let text_obj: glib::Object = string_obj;
                let text = text_obj.property::<String>("string");

                if let Some(pos) = text.find(" - ") {
                    let name = &text[..pos];
                    let desc = &text[pos + 3..];

                    name_label.set_label(name);
                    desc_label.set_label(desc);

                    if let Some(app) = apps_for_bind
                        .iter()
                        .find(|app| text.starts_with(&format!("{} - ", app.name)))
                    {
                        load_app_icon(&icon, &app.icon);
                    }
                } else {
                    name_label.set_label(&text);
                    desc_label.set_label("Application");
                }
            }
        }
    });

    // Create list view with the factory
    let list_view = ListView::new(Some(selection_model.clone()), Some(factory));

    list_view.add_css_class("app-list");
    scrolled.set_child(Some(&list_view));

    // Filter function
    let apps_clone = apps.clone();
    let string_list_clone = string_list.clone();
    let selection_model_clone = selection_model.clone();

    search_entry.connect_changed(move |entry| {
        let query = entry.text().to_lowercase();
        let filtered: Vec<String> = apps_clone
            .iter()
            .filter(|app| {
                query.is_empty()
                    || app.name.to_lowercase().contains(&query)
                    || app.description.to_lowercase().contains(&query)
            })
            .map(|app| format!("{} - {}", app.name, app.description))
            .collect();

        string_list_clone.splice(
            0,
            string_list_clone.n_items(),
            &filtered.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
        );

        if !filtered.is_empty() {
            selection_model_clone.set_selected(0);
        }
    });

    // Handle item activation
    let apps_for_activation = apps.clone();
    let window_for_activation = window.clone();
    let selection_model_for_activation = selection_model.clone();

    list_view.connect_activate(move |_, position| {
        if let Some(item) = selection_model_for_activation.item(position) {
            let string_obj = item.downcast::<glib::Object>().unwrap();
            let text = string_obj.property::<String>("string");

            if let Some(app) = apps_for_activation
                .iter()
                .find(|app| text.starts_with(&format!("{} - ", app.name)))
            {
                launch_app(&app.exec);
                window_for_activation.close();
            }
        }
    });

    // Handle Enter key in search entry
    let apps_for_enter = apps.clone();
    let window_for_enter = window.clone();

    search_entry.connect_activate(move |entry| {
        let query = entry.text().to_lowercase();

        // Find exact match first
        if let Some(app) = apps_for_enter
            .iter()
            .find(|app| app.name.to_lowercase() == query)
        {
            launch_app(&app.exec);
            window_for_enter.close();
            return;
        }

        // Find first partial match
        if let Some(app) = apps_for_enter
            .iter()
            .find(|app| app.name.to_lowercase().contains(&query))
        {
            launch_app(&app.exec);
            window_for_enter.close();
        }
    });

    // Handle Escape key
    let window_for_escape = window.clone();
    let key_controller = gtk4::EventControllerKey::new();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape {
            window_for_escape.close();
            return glib::signal::Propagation::Stop;
        }
        glib::signal::Propagation::Proceed
    });
    window.add_controller(key_controller);

    // Add widgets to main box
    main_box.append(&search_entry);
    main_box.append(&scrolled);

    main_box.add_css_class("launcher-content");
    window.set_child(Some(&main_box));

    // Apply custom CSS for modern appearance
    apply_custom_css();

    // Initial population
    let initial_apps: Vec<String> = apps
        .iter()
        .take(8) // Show first 8 apps initially
        .map(|app| format!("{} - {}", app.name, app.description))
        .collect();

    string_list.splice(
        0,
        string_list.n_items(),
        &initial_apps
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>(),
    );

    if !initial_apps.is_empty() {
        selection_model.set_selected(0);
    }

    window.show();
    search_entry.grab_focus();
}

fn load_app_icon(image: &Image, icon_name: &str) {
    if icon_name.starts_with('/') && std::path::Path::new(icon_name).exists() {
        image.set_from_file(Some(icon_name));
    } else {
        image.set_from_icon_name(Some(icon_name));
    }
}

fn history_path() -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(format!("{}/.config", std::env::var("HOME").unwrap_or_default())))
        .join("mylauncher");
    let _ = fs::create_dir_all(&dir);
    dir.join("history")
}

fn load_history() -> Vec<String> {
    fs::read_to_string(history_path())
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect()
}

fn save_to_history(app_name: &str) {
    let mut history = load_history();
    history.retain(|n| n != app_name);
    history.insert(0, app_name.to_string());
    history.truncate(MAX_HISTORY);
    if let Ok(mut f) = fs::File::create(history_path()) {
        for name in &history {
            let _ = writeln!(f, "{}", name);
        }
    }
}

fn sort_apps_with_history(apps: &[AppEntry]) -> Vec<AppEntry> {
    let history = load_history();
    let mut recent: Vec<AppEntry> = Vec::new();
    for name in &history {
        if let Some(app) = apps.iter().find(|a| &a.name == name) {
            recent.push(app.clone());
        }
    }
    let mut rest: Vec<AppEntry> = apps.iter()
        .filter(|a| !history.contains(&a.name))
        .cloned()
        .collect();
    rest.sort_by(|a, b| a.name.cmp(&b.name));
    recent.extend(rest);
    recent
}

fn get_gtk3_theme_colors() -> std::collections::HashMap<String, String> {
    let mut colors = std::collections::HashMap::new();

    colors.insert("bg".into(), "#23252e".into());
    colors.insert("fg".into(), "#eeeeec".into());
    colors.insert("base".into(), "#272a34".into());
    colors.insert("selected_bg".into(), "#962ac3".into());
    colors.insert("selected_fg".into(), "#ffffff".into());
    colors.insert("borders".into(), "#13151a".into());

    let settings = gtk4::Settings::default();
    let theme_name = settings
        .map(|s| s.gtk_theme_name().map(|n| n.to_string()))
        .flatten()
        .unwrap_or_default();

    if theme_name.is_empty() {
        return colors;
    }

    let search_paths = [
        format!("{}/.themes/{}/gtk-3.0/gtk.css", std::env::var("HOME").unwrap_or_default(), theme_name),
        format!("{}/.themes/{}/gtk-3.0/gtk-dark.css", std::env::var("HOME").unwrap_or_default(), theme_name),
        format!("/usr/share/themes/{}/gtk-3.0/gtk.css", theme_name),
        format!("/usr/share/themes/{}/gtk-3.0/gtk-dark.css", theme_name),
    ];

    for path in &search_paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let line = line.trim();
                if !line.starts_with("@define-color") {
                    continue;
                }
                let parts: Vec<&str> = line.splitn(3, ' ').collect();
                if parts.len() < 3 {
                    continue;
                }
                let name = parts[1];
                let value = parts[2].trim_end_matches(';').trim();
                if !value.starts_with('#') && !value.starts_with("rgb") {
                    continue;
                }
                match name {
                    "theme_bg_color" => { colors.insert("bg".into(), value.into()); }
                    "theme_fg_color" => { colors.insert("fg".into(), value.into()); }
                    "theme_base_color" => { colors.insert("base".into(), value.into()); }
                    "theme_selected_bg_color" => { colors.insert("selected_bg".into(), value.into()); }
                    "theme_selected_fg_color" => { colors.insert("selected_fg".into(), value.into()); }
                    "borders" => { colors.insert("borders".into(), value.into()); }
                    _ => {}
                }
            }
            break;
        }
    }

    colors
}

fn apply_custom_css() {
    let c = get_gtk3_theme_colors();
    let bg = c.get("bg").unwrap();
    let fg = c.get("fg").unwrap();
    let base = c.get("base").unwrap();
    let sel_bg = c.get("selected_bg").unwrap();
    let sel_fg = c.get("selected_fg").unwrap();
    let borders = c.get("borders").unwrap();

    let css = format!(r#"
        window.background,
        window.background > *,
        .background {{
            border-radius: 20px;
        }}

        .launcher-window {{
            background: {bg};
            border-radius: 20px;
            border: 1px solid alpha({borders}, 0.6);
        }}

        .launcher-content {{
            background: {bg};
            border-radius: 20px;
        }}

        .app-list {{
            background: transparent;
            border-radius: 14px;
        }}

        .app-list > row {{
            border-radius: 10px;
            margin: 2px 4px;
            padding: 8px;
            transition: all 200ms ease;
        }}

        .app-list > row:hover {{
            background: alpha({sel_bg}, 0.3);
        }}

        .app-list > row:selected {{
            background: {sel_bg};
            color: {sel_fg};
        }}

        .app-name {{
            font-weight: 600;
            font-size: 0.95em;
            color: {fg};
        }}

        .app-description {{
            font-size: 0.85em;
            opacity: 0.7;
            color: {fg};
        }}

        entry.search-entry {{
            font-size: 1.1em;
            padding: 12px 16px;
            border-radius: 12px;
            background: {base};
            color: {fg};
            border: 1px solid {borders};
            outline: none;
            box-shadow: none;
        }}

        entry.search-entry > text {{
            background: transparent;
            color: {fg};
        }}

        entry.search-entry:focus-within {{
            border: 2px solid {sel_bg};
            outline: none;
            box-shadow: none;
        }}

        .title-1 {{
            font-weight: 700;
            font-size: 1.3em;
            color: {fg};
        }}

        .scrolled {{
            border-radius: 14px;
        }}
    "#);

    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data(&css);
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().unwrap(),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn load_desktop_entries() -> Vec<AppEntry> {
    let mut apps = Vec::new();
    let paths = vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
        PathBuf::from("/var/lib/flatpak/exports/share/applications"),
        PathBuf::from("/home/trevaldev/.local/share/applications"),
    ];

    let iter = Iter::new(paths);
    for path in iter {
        if path.extension().map_or(false, |ext| ext == "desktop") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(desktop_entry) = DesktopEntry::decode(&path, &content) {
                    if let (Some(name), Some(exec)) =
                        (desktop_entry.name(None), desktop_entry.exec())
                    {
                        // Skip NoDisplay=true entries
                        if desktop_entry.no_display() {
                            continue;
                        }

                        // Only show GUI applications (skip terminal apps)
                        if desktop_entry.terminal() {
                            continue;
                        }

                        apps.push(AppEntry {
                            name: name.to_string(),
                            exec: exec.to_string(),
                            icon: desktop_entry
                                .icon()
                                .unwrap_or("application-x-executable")
                                .to_string(),
                            description: match desktop_entry.comment(None) {
                                Some(cow) => cow.to_string(),
                                None => "Application".to_string(),
                            },
                        });
                    }
                }
            }
        }
    }

    // Sort by name and remove duplicates
    apps.sort_by(|a, b| a.name.cmp(&b.name));
    apps.dedup_by(|a, b| a.name == b.name);
    apps
}

fn launch_app(exec_cmd: &str) {
    // Parse the exec command (handle %f, %u, %U, %F, etc.)
    let clean_cmd = exec_cmd
        .split_whitespace()
        .filter(|arg| !arg.starts_with('%'))
        .collect::<Vec<&str>>()
        .join(" ");

    // Expand environment variables
    let clean_cmd_clone = clean_cmd.clone();
    let expanded_cmd = shellexpand::full(&clean_cmd)
        .unwrap_or_else(|_| clean_cmd_clone.into())
        .to_string();

    // Parse command and arguments
    let parts: Vec<&str> = expanded_cmd.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    let cmd = parts[0];
    let args = &parts[1..];

    // Launch the application using system-run for better integration
    let result = Command::new("systemd-run")
        .args(&["--user", "--scope", cmd])
        .args(args)
        .spawn();

    match result {
        Ok(_) => println!("Launched: {}", exec_cmd),
        Err(_e) => {
            // Fallback to direct execution
            let result = Command::new(cmd).args(args).spawn();

            match result {
                Ok(_) => println!("Launched (fallback): {}", exec_cmd),
                Err(e) => eprintln!("Failed to launch {}: {}", exec_cmd, e),
            }
        }
    }
}

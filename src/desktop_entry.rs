use std::collections::HashSet;

pub use freedesktop_desktop_entry::DesktopEntry;

pub fn get_all_applications(locales: &[String]) -> Vec<DesktopEntry> {
    let mut seen = HashSet::new();
    let mut apps: Vec<DesktopEntry> = freedesktop_desktop_entry::desktop_entries(locales)
        .into_iter()
        .filter(|entry| {
            matches!(entry.type_(), Some("Application")) && !entry.no_display() && !entry.hidden()
        })
        .filter(|entry| seen.insert(entry.appid.clone()))
        .collect();

    apps.sort_by(|a, b| {
        let a_name = a.name(locales).unwrap_or_default();
        let b_name = b.name(locales).unwrap_or_default();
        a_name.to_lowercase().cmp(&b_name.to_lowercase())
    });

    apps
}

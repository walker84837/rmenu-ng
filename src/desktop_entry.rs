// src/desktop.rs

use serde::de::{self, Deserializer};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::deserialize_boolean_from_string;
use std::collections::BTreeMap;
use std::fmt;

/// Represents a semicolon‐separated list (e.g. "AudioVideo;Video;Player;")
/// and always serializes with a trailing semicolon if non‐empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemicolonList(pub Vec<String>);

impl<'de> Deserialize<'de> for SemicolonList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // Split on ';', drop empty trailing, but keep empty items if interior.
        let items: Vec<String> = s
            .split(';')
            .filter_map(|part| {
                if part.is_empty() {
                    None
                } else {
                    Some(part.to_string())
                }
            })
            .collect();
        Ok(SemicolonList(items))
    }
}

impl Serialize for SemicolonList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.0.is_empty() {
            serializer.serialize_str("") // empty string
        } else {
            let mut joined = self.0.join(";");
            // ensure trailing semicolon
            if !joined.ends_with(';') {
                joined.push(';');
            }
            serializer.serialize_str(&joined)
        }
    }
}

/// Represents a set of localized strings under a single key name.
/// On disk, keys look like:
///   Name=Foo Viewer
///   Name[de]=Fu Betrachter
///   Name[fr]=Visionneuse Foo
///
/// We collect them into a Map<String, String> where the empty
/// locale ("") is the un‐localized default.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocaleMap(pub BTreeMap<String, String>);

impl<'de> Deserialize<'de> for LocaleMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // We expect the whole section of keys to come in, but serde cannot
        // know ahead of time how to map "Name" vs "Name[de]". Instead, we use
        // `#[serde(flatten)]` on a BTreeMap<String,String> in the parent, then
        // in the custom `deserialize` of that parent struct, manually pull out
        // keys that match this field’s prefix.
        // Because of that complexity, we will not implement Deserialize directly
        // here; rather, we assume the parent struct’s `#[serde(deserialize_with)]`
        // will route into a helper. To keep things simple in this example,
        // we only show how to collect keys that start with one prefix. See
        // `deserialize_localized()` below.
        Err(de::Error::custom(
            "LocaleMap should be deserialized with a custom helper in the parent struct",
        ))
    }
}

impl Serialize for LocaleMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // On serialize, we need to emit multiple keys: `Name=…`, `Name[de]=…`, etc.
        // But since `serde` only serializes struct fields in order, we cannot
        // directly “flatten” this map. Instead, we rely on the parent struct’s
        // `#[serde(flatten)]` on a BTreeMap<String,String>` to pick up these pairs.
        // So here, we implement Serialize by returning an “empty” map; the parent
        // must have already inserted the correct keys into its flatten map.
        Err(serde::ser::Error::custom(
            "LocaleMap should be flattened by the parent struct’s serialize implementation",
        ))
    }
}

/// Pull out all keys matching `prefix` or `prefix[<locale>]` from a flatten map.
fn deserialize_localized<'de, D>(
    prefix: &str,
    map: &mut BTreeMap<String, String>,
) -> Option<LocaleMap> {
    // collect any entry whose key == prefix or key starts with prefix + "[".
    let mut loc_map = LocaleMap(BTreeMap::new());
    let mut to_remove = Vec::new();

    for key in map.keys() {
        if key == prefix {
            // default
            to_remove.push(key.clone());
        } else if key.starts_with(&format!("{}[", prefix)) && key.ends_with(']') {
            to_remove.push(key.clone());
        }
    }
    if to_remove.is_empty() {
        return None;
    }
    for full_key in to_remove {
        if let Some(value) = map.remove(&full_key) {
            if full_key == prefix {
                loc_map.0.insert("".into(), value);
            } else if let Some(start) = full_key.find('[') {
                // e.g. Name[de] => locale = "de"
                let locale = full_key[start + 1..full_key.len() - 1].to_string();
                loc_map.0.insert(locale, value);
            }
        }
    }
    Some(loc_map)
}

/// The `[Desktop Entry]` section.  Corresponds to "Table 2. Standard Keys".
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DesktopEntry {
    /// Type=Application | Link | Directory
    #[serde(rename = "Type")]
    pub entry_type: String,

    /// Version=1.1   (optional)
    #[serde(rename = "Version", default)]
    pub version: Option<String>,

    /// Name=…   (localized)
    /// We remove all `Name*` keys from the flatten‐map and put them here.
    #[serde(skip)] // custom‐handled
    pub name: LocaleMap,

    /// GenericName=…   (localized)
    #[serde(skip)]
    pub generic_name: Option<LocaleMap>,

    /// NoDisplay=true/false
    #[serde(
        rename = "NoDisplay",
        default,
        deserialize_with = "deserialize_boolean_from_string"
    )]
    pub no_display: Option<bool>,

    /// Comment=…   (localized)
    #[serde(skip)]
    pub comment: Option<LocaleMap>,

    /// Icon=…   (iconstring, localized or not)
    #[serde(skip)]
    pub icon: Option<LocaleMap>,

    /// Hidden=true/false
    #[serde(
        rename = "Hidden",
        default,
        deserialize_with = "deserialize_boolean_from_string"
    )]
    pub hidden: Option<bool>,

    /// OnlyShowIn=… (semicolon list)
    #[serde(
        rename = "OnlyShowIn",
        default,
        deserialize_with = "deserialize_semicolon_list"
    )]
    pub only_show_in: Option<SemicolonList>,

    /// NotShowIn=… (semicolon list)
    #[serde(
        rename = "NotShowIn",
        default,
        deserialize_with = "deserialize_semicolon_list"
    )]
    pub not_show_in: Option<SemicolonList>,

    /// DBusActivatable=true/false
    #[serde(
        rename = "DBusActivatable",
        default,
        deserialize_with = "deserialize_boolean_from_string"
    )]
    pub dbus_activatable: Option<bool>,

    /// TryExec=“/usr/bin/foo”
    #[serde(rename = "TryExec", default)]
    pub try_exec: Option<String>,

    /// Exec=…   (command line with placeholders)
    #[serde(rename = "Exec", default)]
    pub exec: Option<String>,

    /// Path=/working/dir
    #[serde(rename = "Path", default)]
    pub path: Option<String>,

    /// Terminal=true/false
    #[serde(
        rename = "Terminal",
        default,
        deserialize_with = "deserialize_boolean_from_string"
    )]
    pub terminal: Option<bool>,

    /// Actions=…;  (semicolon list of action IDs)
    #[serde(
        rename = "Actions",
        default,
        deserialize_with = "deserialize_semicolon_list"
    )]
    pub actions: Option<SemicolonList>,

    /// MimeType=…;  (semicolon list)
    #[serde(
        rename = "MimeType",
        default,
        deserialize_with = "deserialize_semicolon_list"
    )]
    pub mime_type: Option<SemicolonList>,

    /// Categories=…;  (semicolon list)
    #[serde(
        rename = "Categories",
        default,
        deserialize_with = "deserialize_semicolon_list"
    )]
    pub categories: Option<SemicolonList>,

    /// Implements=…;  (semicolon list)
    #[serde(
        rename = "Implements",
        default,
        deserialize_with = "deserialize_semicolon_list"
    )]
    pub implements: Option<SemicolonList>,

    /// Keywords=…;  (semicolon list, localized––treated as localized if suffixed)
    #[serde(skip)]
    pub keywords: Option<LocaleMap>,

    /// StartupNotify=true/false
    #[serde(
        rename = "StartupNotify",
        default,
        deserialize_with = "deserialize_boolean_from_string"
    )]
    pub startup_notify: Option<bool>,

    /// StartupWMClass=… (string)
    #[serde(rename = "StartupWMClass", default)]
    pub startup_wm_class: Option<String>,

    /// URL=… (when Type=Link)
    #[serde(rename = "URL", default)]
    pub url: Option<String>,

    /// PrefersNonDefaultGPU=true/false
    #[serde(
        rename = "PrefersNonDefaultGPU",
        default,
        deserialize_with = "deserialize_boolean_from_string"
    )]
    pub prefers_non_default_gpu: Option<bool>,

    /// Catch‐all for any unknown keys (including X-… or KDE-specific)
    #[serde(flatten)]
    pub other: BTreeMap<String, String>,
}

/// The `[Desktop Action <ActionID>]` section.  Corresponds to Table 3.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DesktopAction {
    /// Name=…  (localized)
    #[serde(skip)]
    pub name: LocaleMap,

    /// Icon=…  (iconstring; optional)
    #[serde(skip)]
    pub icon: Option<LocaleMap>,

    /// Exec=…  (string; optional if DBusActivatable=true)
    #[serde(rename = "Exec", default)]
    pub exec: Option<String>,

    #[serde(flatten)]
    pub other: BTreeMap<String, String>,
}

/// A single INI‐style section. We attempt to parse "Desktop Entry" into `Section::Entry`,
/// "Desktop Action <ID>" into `Section::Action { id, data }`, and anything else into
/// `Section::Other`, which just stores a flatten‐map of keys/values unchanged.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Section {
    Entry {
        #[serde(rename = "Desktop Entry")]
        #[serde(deserialize_with = "deserialize_desktop_entry")]
        #[serde(serialize_with = "serialize_desktop_entry")]
        pub desktop_entry: DesktopEntry,
    },

    Action {
        #[serde(rename = "Desktop Action")]
        #[serde(deserialize_with = "deserialize_desktop_action")]
        #[serde(serialize_with = "serialize_desktop_action")]
        pub action: (String /*action_id*/, DesktopAction),
    },

    Other {
        #[serde(flatten)]
        pub raw: BTreeMap<String, String>,
    },
}

/// The top‐level .desktop file: a map from section‐name to `Section`.
/// For example:
///   "Desktop Entry"               => Section::Entry
///   "Desktop Action Gallery"      => Section::Action("Gallery", DesktopAction)
///   "X-KDE-SomeGroup"             => Section::Other { … }
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DesktopFile {
    #[serde(flatten)]
    pub sections: BTreeMap<String, Section>,
}

fn deserialize_semicolon_list<'de, D>(deserializer: D) -> Result<Option<SemicolonList>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    if let Some(s) = opt {
        Ok(Some(
            SemicolonList::deserialize(serde_json::Value::String(s)).map_err(de::Error::custom)?,
        ))
    } else {
        Ok(None)
    }
}

/// Deserialize the `[Desktop Entry]` section out of a flatten‐map of key -> value.
fn deserialize_desktop_entry<'de, D>(deserializer: D) -> Result<DesktopEntry, D::Error>
where
    D: Deserializer<'de>,
{
    // first, deserialize into a temporary map of String -> String
    let mut raw_map: BTreeMap<String, String> = BTreeMap::deserialize(deserializer)?;

    // manually extract all localized fields:
    let name = deserialize_localized("Name", &mut raw_map)
        .ok_or_else(|| de::Error::missing_field("Name"))?;
    let generic_name = deserialize_localized("GenericName", &mut raw_map);
    let comment = deserialize_localized("Comment", &mut raw_map);
    let icon = deserialize_localized("Icon", &mut raw_map);
    let keywords = deserialize_localized("Keywords", &mut raw_map);

    // for all remaining keys, let serde build the rest
    #[derive(Deserialize)]
    struct TempEntry {
        #[serde(rename = "Type")]
        pub entry_type: String,
        #[serde(rename = "Version")]
        pub version: Option<String>,
        #[serde(
            rename = "NoDisplay",
            default,
            deserialize_with = "deserialize_opt_bool"
        )]
        pub no_display: Option<bool>,
        #[serde(rename = "Hidden", default, deserialize_with = "deserialize_opt_bool")]
        pub hidden: Option<bool>,
        #[serde(
            rename = "OnlyShowIn",
            default,
            deserialize_with = "deserialize_opt_semicolon_list"
        )]
        pub only_show_in: Option<SemicolonList>,
        #[serde(
            rename = "NotShowIn",
            default,
            deserialize_with = "deserialize_opt_semicolon_list"
        )]
        pub not_show_in: Option<SemicolonList>,
        #[serde(
            rename = "DBusActivatable",
            default,
            deserialize_with = "deserialize_opt_bool"
        )]
        pub dbus_activatable: Option<bool>,
        #[serde(rename = "TryExec")]
        pub try_exec: Option<String>,
        #[serde(rename = "Exec")]
        pub exec: Option<String>,
        #[serde(rename = "Path")]
        pub path: Option<String>,
        #[serde(
            rename = "Terminal",
            default,
            deserialize_with = "deserialize_opt_bool"
        )]
        pub terminal: Option<bool>,
        #[serde(
            rename = "Actions",
            default,
            deserialize_with = "deserialize_opt_semicolon_list"
        )]
        pub actions: Option<SemicolonList>,
        #[serde(
            rename = "MimeType",
            default,
            deserialize_with = "deserialize_opt_semicolon_list"
        )]
        pub mime_type: Option<SemicolonList>,
        #[serde(
            rename = "Categories",
            default,
            deserialize_with = "deserialize_opt_semicolon_list"
        )]
        pub categories: Option<SemicolonList>,
        #[serde(
            rename = "Implements",
            default,
            deserialize_with = "deserialize_opt_semicolon_list"
        )]
        pub implements: Option<SemicolonList>,
        #[serde(
            rename = "StartupNotify",
            default,
            deserialize_with = "deserialize_opt_bool"
        )]
        pub startup_notify: Option<bool>,
        #[serde(rename = "StartupWMClass")]
        pub startup_wm_class: Option<String>,
        #[serde(rename = "URL")]
        pub url: Option<String>,
        #[serde(
            rename = "PrefersNonDefaultGPU",
            default,
            deserialize_with = "deserialize_opt_bool"
        )]
        pub prefers_non_default_gpu: Option<bool>,

        // Anything we did not mention becomes “other”
        #[serde(flatten)]
        pub other: BTreeMap<String, String>,
    }

    let temp: TempEntry = TempEntry::deserialize(raw_map.clone()).map_err(de::Error::custom)?;

    Ok(DesktopEntry {
        entry_type: temp.entry_type,
        version: temp.version,
        name,
        generic_name,
        no_display: temp.no_display,
        comment,
        icon,
        hidden: temp.hidden,
        only_show_in: temp.only_show_in,
        not_show_in: temp.not_show_in,
        dbus_activatable: temp.dbus_activatable,
        try_exec: temp.try_exec,
        exec: temp.exec,
        path: temp.path,
        terminal: temp.terminal,
        actions: temp.actions,
        mime_type: temp.mime_type,
        categories: temp.categories,
        implements: temp.implements,
        keywords,
        startup_notify: temp.startup_notify,
        startup_wm_class: temp.startup_wm_class,
        url: temp.url,
        prefers_non_default_gpu: temp.prefers_non_default_gpu,
        other: temp.other,
    })
}

/// Serialize the `DesktopEntry` back into a flatten‐map of key → value.
fn serialize_desktop_entry<S>(entry: &DesktopEntry, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Build a single BTreeMap<String, String> with all keys in the right order
    let mut map = BTreeMap::new();

    map.insert("Type".into(), entry.entry_type.clone());
    if let Some(v) = &entry.version {
        map.insert("Version".into(), v.clone());
    }

    // Insert localized “Name” keys:
    for (locale, text) in &entry.name.0 {
        if locale.is_empty() {
            map.insert("Name".into(), text.clone());
        } else {
            map.insert(format!("Name[{}]", locale), text.clone());
        }
    }
    if let Some(gen) = &entry.generic_name {
        for (locale, text) in &gen.0 {
            if locale.is_empty() {
                map.insert("GenericName".into(), text.clone());
            } else {
                map.insert(format!("GenericName[{}]", locale), text.clone());
            }
        }
    }

    if let Some(no) = entry.no_display {
        map.insert("NoDisplay".into(), no.to_string());
    }
    if let Some(com) = &entry.comment {
        for (locale, text) in &com.0 {
            if locale.is_empty() {
                map.insert("Comment".into(), text.clone());
            } else {
                map.insert(format!("Comment[{}]", locale), text.clone());
            }
        }
    }
    if let Some(ic) = &entry.icon {
        for (locale, text) in &ic.0 {
            if locale.is_empty() {
                map.insert("Icon".into(), text.clone());
            } else {
                map.insert(format!("Icon[{}]", locale), text.clone());
            }
        }
    }
    if let Some(h) = entry.hidden {
        map.insert("Hidden".into(), h.to_string());
    }
    if let Some(only) = &entry.only_show_in {
        let s = serde_json::to_string(only).map_err(serde::ser::Error::custom)?;
        // `serde_json::to_string` wraps in quotes; strip them
        let s = s.trim_matches('"').to_string();
        map.insert("OnlyShowIn".into(), s);
    }
    if let Some(not) = &entry.not_show_in {
        let s = serde_json::to_string(not).map_err(serde::ser::Error::custom)?;
        let s = s.trim_matches('"').to_string();
        map.insert("NotShowIn".into(), s);
    }
    if let Some(d) = entry.dbus_activatable {
        map.insert("DBusActivatable".into(), d.to_string());
    }
    if let Some(te) = &entry.try_exec {
        map.insert("TryExec".into(), te.clone());
    }
    if let Some(e) = &entry.exec {
        map.insert("Exec".into(), e.clone());
    }
    if let Some(p) = &entry.path {
        map.insert("Path".into(), p.clone());
    }
    if let Some(t) = entry.terminal {
        map.insert("Terminal".into(), t.to_string());
    }
    if let Some(actions) = &entry.actions {
        let s = serde_json::to_string(actions).map_err(serde::ser::Error::custom)?;
        let s = s.trim_matches('"').to_string();
        map.insert("Actions".into(), s);
    }
    if let Some(m) = &entry.mime_type {
        let s = serde_json::to_string(m).map_err(serde::ser::Error::custom)?;
        let s = s.trim_matches('"').to_string();
        map.insert("MimeType".into(), s);
    }
    if let Some(cats) = &entry.categories {
        let s = serde_json::to_string(cats).map_err(serde::ser::Error::custom)?;
        let s = s.trim_matches('"').to_string();
        map.insert("Categories".into(), s);
    }
    if let Some(imp) = &entry.implements {
        let s = serde_json::to_string(imp).map_err(serde::ser::Error::custom)?;
        let s = s.trim_matches('"').to_string();
        map.insert("Implements".into(), s);
    }
    if let Some(kw) = &entry.keywords {
        for (locale, text) in &kw.0 {
            if locale.is_empty() {
                map.insert("Keywords".into(), text.clone());
            } else {
                map.insert(format!("Keywords[{}]", locale), text.clone());
            }
        }
    }
    if let Some(sn) = entry.startup_notify {
        map.insert("StartupNotify".into(), sn.to_string());
    }
    if let Some(wm) = &entry.startup_wm_class {
        map.insert("StartupWMClass".into(), wm.clone());
    }
    if let Some(u) = &entry.url {
        map.insert("URL".into(), u.clone());
    }
    if let Some(gpu) = entry.prefers_non_default_gpu {
        map.insert("PrefersNonDefaultGPU".into(), gpu.to_string());
    }

    // Finally, insert any “other” keys the user didn’t specifically declare:
    for (k, v) in &entry.other {
        map.insert(k.clone(), v.clone());
    }

    // Wrap as a single‐pair “Desktop Entry” section:
    let mut wrapper = BTreeMap::new();
    wrapper.insert("Desktop Entry".to_string(), map);

    wrapper.serialize(serializer)
}

/// Deserialize any `[Desktop Action <ID>]` section.
fn deserialize_desktop_action<'de, D>(deserializer: D) -> Result<(String, DesktopAction), D::Error>
where
    D: Deserializer<'de>,
{
    // we deserialize into a temporary map key→value
    let mut raw_map: BTreeMap<String, String> = BTreeMap::deserialize(deserializer)?;

    // The “section name” is something like "Desktop Action Gallery".
    // Serde will have already given us the section’s entire name. We need
    // to extract the `<ID>` part (“Gallery” in this example) from the caller.
    // However, in an untagged enum, serde dispatches based on matching the
    // key “Desktop Action” in the field attribute.  Unfortunately, serde
    // does not by default give us the “Gallery” part.  The trick is: in
    // your top‐level map you should have inserted the section as:
    //    "Desktop Action Gallery" => Section::Action
    // so here, we only know that debiasing the action ID must be done upstream.
    // For simplicity, let’s assume serde gives us a special key "__action_id"
    // in raw_map.  In reality, most INI backends allow you to grab the section
    // name exactly.  For clarity in this example, we’ll pull the action ID out
    // of a special field.  In a real implementation you would capture the section
    // header from your INI reader directly.
    let action_id = raw_map
        .remove("__action_id")
        .ok_or_else(|| de::Error::custom("Missing action ID"))?;

    // now extract localized fields out of raw_map
    let name = deserialize_localized("Name", &mut raw_map)
        .ok_or_else(|| de::Error::missing_field(&format!("Desktop Action {} → Name", action_id)))?;
    let icon = deserialize_localized("Icon", &mut raw_map);

    let temp = DesktopAction {
        name,
        icon,
        exec: raw_map.remove("Exec"),
        other: raw_map,
    };
    Ok((action_id, temp))
}

/// serialize a `(action_id, DesktopAction)` into a section named `"Desktop Action <ID>"`.
fn serialize_desktop_action<S>(
    pair: &(String, DesktopAction),
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let (action_id, action) = pair;

    // build a flatten map of all keys in the action
    let mut map = BTreeMap::new();
    // localized Name
    for (locale, text) in &action.name.0 {
        if locale.is_empty() {
            map.insert("Name".into(), text.clone());
        } else {
            map.insert(format!("Name[{}]", locale), text.clone());
        }
    }
    // localized icon
    if let Some(ic) = &action.icon {
        for (locale, text) in &ic.0 {
            if locale.is_empty() {
                map.insert("Icon".into(), text.clone());
            } else {
                map.insert(format!("Icon[{}]", locale), text.clone());
            }
        }
    }
    if let Some(exec) = &action.exec {
        map.insert("Exec".into(), exec.clone());
    }
    for (k, v) in &action.other {
        map.insert(k.clone(), v.clone());
    }

    let section_name = format!("Desktop Action {}", action_id);
    let mut wrapper = BTreeMap::new();
    wrapper.insert(section_name, map);
    wrapper.serialize(serializer)
}

fn deserialize_opt_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    if let Some(s) = opt {
        // accept "true"/"false" or "0"/"1"
        let s_lower = s.to_lowercase();
        match s_lower.as_str() {
            "true" | "1" => Ok(Some(true)),
            "false" | "0" => Ok(Some(false)),
            other => Err(D::Error::custom(format!(
                "Invalid boolean string: {}",
                other
            ))),
        }
    } else {
        Ok(None)
    }
}

fn deserialize_opt_semicolon_list<'de, D>(
    deserializer: D,
) -> Result<Option<SemicolonList>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    if let Some(s) = opt {
        // Reuse SemicolonList’s Deserialize
        Ok(Some(
            SemicolonList::deserialize(serde_json::Value::String(s)).map_err(de::Error::custom)?,
        ))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_ini::{from_str, to_string};

    const EXAMPLE: &str = r#"
# This is a comment
[Desktop Entry]
Version=1.0
Type=Application
Name=Foo Viewer
Name[de]=Foo Betrachter
Comment=The best viewer for Foo objects available!
TryExec=fooview
Exec=fooview %F
Icon=fooview
MimeType=image/x-foo;
Actions=Gallery;Create;

[Desktop Action Gallery]
Name=Browse Gallery
Exec=fooview --gallery

[Desktop Action Create]
Name=Create a new Foo!
Icon=fooview-new
Exec=fooview --create-new
"#;

    #[test]
    fn roundtrip_example() {
        // deserialize into our DesktopFile
        let df: DesktopFile = from_str(EXAMPLE).expect("Failed to parse example");

        // check that we got the "Name[de]" localized entry:
        if let Section::Entry { ref desktop_entry } = df.sections.get("Desktop Entry").unwrap() {
            assert_eq!(
                desktop_entry.name.0.get("de").map(String::as_str).unwrap(),
                "Foo Betrachter"
            );
            assert_eq!(
                desktop_entry.name.0.get("").map(String::as_str).unwrap(),
                "Foo Viewer"
            );
        } else {
            panic!("Desktop Entry not found or wrong variant");
        }

        // serialize back to a string
        let out = to_string(&df).expect("Failed to serialize back");

        // we expect at least the keys to be present again
        assert!(out.contains("Name=Foo Viewer"));
        assert!(out.contains("Name[de]=Foo Betrachter"));
        assert!(out.contains("[Desktop Action Gallery]"));
        assert!(out.contains("Exec=fooview --gallery"));
    }
}

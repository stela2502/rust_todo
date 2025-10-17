// todo_list.rs
use std::path::PathBuf;
use rust_yaml::yaml::Yaml;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;



#[derive(Debug, Clone)]
pub struct ToDoItem {
    pub yaml: Yaml, // always a Hash
}

impl fmt::Display for ToDoItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = self.yaml.get_str("type").unwrap_or("Unknown");
        let godot_path = self.yaml.get_str("godot_path").unwrap_or("<no path>");
        let status = self.yaml.get_str("status").unwrap_or("unprocessed");

        write!(f, "[{}] → {} ({})", kind, godot_path, status)
    }
}

impl ToDoItem {
    pub fn new(kind: &str, fields: Vec<(&str, &str)>) -> Self {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert("type".into(), Yaml::Value(kind.into()));

        for (k, v) in fields {
            map.insert(k.into(), Yaml::Value(v.into()));
        }

        // default new fields
        map.entry("status".into()).or_insert(Yaml::Value("Open".into()));
        map.entry("reason".into()).or_insert(Yaml::Value("New conversion task".into()));
        map.entry("info".into()).or_insert(Yaml::Value(String::new()));

        Self { yaml: Yaml::Hash(map) }
    }

    pub fn to_yaml(&self) -> &Yaml {
        &self.yaml
    }

    /// Try to build a ToDoItem from a Yaml::Hash
    pub fn from_yaml(y: &Yaml) -> Option<Self> {
        match y {
            Yaml::Hash(map) => {
                if let Some(Yaml::Value(kind)) = map.get("type") {
                    // Minimal validation
                    let required = Self::required_keys(kind);
                    for key in required {
                        if !map.contains_key(*key) {
                            eprintln!("⚠️ Missing key '{}' for ToDoItem type {}", key, kind);
                            return None;
                        }
                    }
                    Some(Self { yaml: y.clone() })
                } else {
                    eprintln!("⚠️ ToDoItem missing 'type' key");
                    None
                }
            }
            _ => None,
        }
    }

    pub fn kind(&self) -> Option<&str> {
        if let Yaml::Hash(map) = &self.yaml {
            if let Some(Yaml::Value(v)) = map.get("type") {
                return Some(v);
            }
        }
        None
    }

    pub fn required_keys(kind: &str) -> &'static [&'static str] {
        match kind {
            "Shader" => &["unity_path", "godot_path", "instruction"],
            "Material" => &["unity_path", "godot_path", "instruction"],
            "Prefab" => &["unity_path", "godot_path"],
            "Animation" => &["unity_path", "godot_path"],
            "Script" => &["unity_path", "godot_path"],
            "Other" => &["unity_path", "godot_path"],
            _ => &[],
        }
    }

    pub fn set_status(&mut self, new_status: &str) {
        if let Yaml::Hash(map) = &mut self.yaml {
            map.insert("status".into(), Yaml::Value(new_status.into()));
        }
    }

    pub fn status(&self) -> Option<&str> {
        self.yaml.get_str("status")
    }

    pub fn mark_done(&mut self) { self.set_status("Done"); }
    pub fn mark_failed(&mut self) { self.set_status("Failed"); }
    pub fn reopen(&mut self) { self.set_status("Open"); }

    pub fn set_info(&mut self, msg: &str) {
        if let Yaml::Hash(map) = &mut self.yaml {
            map.insert("info".into(), Yaml::Value(msg.into()));
        }
    }
    pub fn get(&self, key:&str ) -> Option<&str>{
        self.yaml.get_str( key )
    }
}

#[derive(Debug, Default, Clone)]
pub struct ToDoList {
    pub items: std::collections::HashMap<String, ToDoItem>,
}

impl ToDoList {

    pub fn new() -> Self {
        Self { items: std::collections::HashMap::new() }
    }

    pub fn insert(&mut self, guid: &str, item: ToDoItem) {
        self.items.insert(guid.to_string(), item);
    }

    pub fn update_status(&mut self, guid: &str, status: &str, info: &str) {
        if let Some(item) = self.items.get_mut(guid) {
            item.set_status(status);
            item.set_info(info);
        }
    }

    pub fn mark_done(&mut self, guid: &str) {
        self.update_status(guid, "Done", "✅ Conversion verified in Godot");
    }

    pub fn contains(&self, guid: &str) -> bool {
        self.items.contains_key(guid)
    }

    pub fn to_yaml(&self) -> Yaml {
        use std::collections::HashMap;
        let mut map = HashMap::new();

        let mut todo_hash = HashMap::new();
        for (guid, item) in &self.items {
            todo_hash.insert(guid.clone(), item.to_yaml().clone());
        }

        map.insert("todo_list".into(), Yaml::Hash(todo_hash));
        Yaml::Hash(map)
    }

    pub fn from_yaml(y: &Yaml) -> Self {
        let mut items = std::collections::HashMap::new();

        if let Yaml::Hash(map) = y {
            if let Some(Yaml::Hash(todo_map)) = map.get("todo_list") {
                for (guid, entry) in todo_map {
                    if let Some(item) = ToDoItem::from_yaml(entry) {
                        items.insert(guid.clone(), item);
                    }
                }
            }
        }

        Self { items }
    }        
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let yaml = self.to_yaml();
        yaml.save_to_file(path)
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let yaml = Yaml::load_from_file( path )?;
        Ok(Self::from_yaml( &yaml ))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn tmp_file(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("{}_todo_test.yaml", name));
        path
    }

    #[test]
    fn test_todoitem_missing_required_fields() {
        let kinds = ["Shader", "Material", "Prefab", "Animation", "Script", "Other"];

        for kind in kinds {
            let required = ToDoItem::required_keys(kind);
            assert!(!required.is_empty(), "Expected required keys for {}", kind);

            // omit last required key → should be invalid
            let incomplete_fields: Vec<_> = required[..required.len() - 1]
                .iter()
                .map(|&k| (k, "dummy"))
                .collect();

            let item = ToDoItem::new(kind, incomplete_fields.clone());

            // simulate validation check
            let is_complete = required.iter().all(|&k| item.yaml.get_str(k).is_some());

            assert!(
                !is_complete,
                "Item for kind '{}' should be invalid when missing fields: {:?}",
                kind,
                incomplete_fields
            );
        }
    }

    #[test]
    fn test_todoitem_complete_required_fields() {
        let kinds = ["Shader", "Material", "Prefab", "Animation", "Script", "Other"];

        for kind in kinds {
            let required = ToDoItem::required_keys(kind);
            let fields: Vec<_> = required.iter().map(|&k| (k, "ok")).collect();
            let item = ToDoItem::new(kind, fields.clone());

            for (k, _) in fields {
                assert!(
                    item.yaml.get_str(k).is_some(),
                    "Expected key '{}' in ToDoItem of kind '{}'",
                    k,
                    kind
                );
            }

            // also check defaults are present
            for k in ["status", "reason", "info"] {
                assert!(
                    item.yaml.get_str(k).is_some(),
                    "Expected default field '{}' in ToDoItem",
                    k
                );
            }
        }
    }

    #[test]
    fn test_todolist_save_and_load() {
        let mut list = ToDoList::new();

        let item1 = ToDoItem::new(
            "Shader",
            vec![
                ("unity_path", "Assets/Shaders/Example.shader"),
                ("godot_path", "res://shaders/example.gdshader"),
                ("instruction", "translate"),
            ],
        );
        let item2 = ToDoItem::new(
            "Prefab",
            vec![
                ("unity_path", "Assets/Prefabs/Player.prefab"),
                ("godot_path", "res://scenes/Player.tscn"),
            ],
        );

        list.insert("guid1", item1.clone());
        list.insert("guid2", item2.clone());

        let tmp = tmp_file("save_load");
        list.save_to_file(&tmp).expect("Failed to save file");

        let loaded = ToDoList::load_from_file(&tmp).expect("Failed to load file");

        // Check that keys and counts match
        assert_eq!(list.items.len(), loaded.items.len());
        assert!(loaded.items.contains_key("guid1"));
        assert!(loaded.items.contains_key("guid2"));

        // Check some representative field values
        let shader = &loaded.items["guid1"].yaml;
        assert_eq!(shader.get_str("godot_path"), Some("res://shaders/example.gdshader".into()));

        let prefab = &loaded.items["guid2"].yaml;
        assert_eq!(prefab.get_str("unity_path"), Some("Assets/Prefabs/Player.prefab".into()));

        // cleanup
        fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_required_keys_for_all_known_kinds() {
        let kinds = ["Shader", "Material", "Prefab", "Animation", "Script", "Other", "Unknown"];

        for kind in kinds {
            let keys = ToDoItem::required_keys(kind);
            match kind {
                "Shader" | "Material" => assert_eq!(keys, &["unity_path", "godot_path", "instruction"]),
                "Prefab" | "Animation" | "Script" | "Other" => assert_eq!(keys, &["unity_path", "godot_path"]),
                _ => assert!(keys.is_empty(), "Unknown kind '{}' should have no required keys", kind),
            }
        }
    }
}
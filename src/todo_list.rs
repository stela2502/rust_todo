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

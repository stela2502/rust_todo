# 🗂️ ToDoList Library

A small Rust utility library for managing **YAML-based ToDo items** — designed to track translation or conversion tasks (e.g. Unity → Godot shader/material/script conversions).  
Each `ToDoItem` is stored as a `Yaml::Hash`, supporting serialization and persistence via the [`rust_yaml`](https://github.com/stela2502/rust_yaml) crate.

---

## ✨ Features

- 📄 Simple `ToDoItem` type backed by YAML for easy serialization.
- 🧩 Built-in validation of required keys per item type (`Shader`, `Material`, `Prefab`, etc.).
- 🔄 `ToDoList` type for collection management and persistence.
- 💾 Save and load entire task lists from disk.
- ✅ Rich helper API for task state management (`Open`, `Done`, `Failed`, etc.).
- 🗣️ Clean `Display` output for user-friendly terminal logs.

---

## 🧱 Structure

```rust
use rust_yaml::yaml::Yaml;

pub struct ToDoItem {
    pub yaml: Yaml, // always a Hash
}

pub struct ToDoList {
    pub items: HashMap<String, ToDoItem>,
}
```

---

## 🚀 Example Usage

```rust
use rust_yaml::yaml::Yaml;
use todo_list::{ToDoItem, ToDoList};

fn main() -> std::io::Result<()> {
    // Create a new ToDoItem
    let item = ToDoItem::new(
        "Shader",
        vec![
            ("unity_path", "Assets/Shaders/Heatmap.shader"),
            ("godot_path", "res://shaders/heatmap.gdshader"),
            ("instruction", "Convert to Godot 4 shader language"),
        ],
    );

    // Display it
    println!("{}", item);
    // Output: [Shader] → res://shaders/heatmap.gdshader (Open)

    // Create a ToDoList and add the item
    let mut list = ToDoList::new();
    list.insert("860a66ca4259b384f9ec01b687f71c3c", item);

    // Save to file
    list.save_to_file("todo_list.yaml")?;

    // Later: load it again
    let loaded = ToDoList::load_from_file("todo_list.yaml")?;
    println!("Loaded {} items", loaded.items.len());

    Ok(())
}
```

---

## 🧩 Supported `type` Values

| Type       | Required Keys                                  |
|-------------|-----------------------------------------------|
| `Shader`    | `unity_path`, `godot_path`, `instruction`     |
| `Material`  | `unity_path`, `godot_path`, `instruction`     |
| `Prefab`    | `unity_path`, `godot_path`                    |
| `Animation` | `unity_path`, `godot_path`                    |
| `Script`    | `unity_path`, `godot_path`                    |
| `Other`     | `unity_path`, `godot_path`                    |

Each item automatically gains:
- `status` (default `"Open"`)
- `reason` (default `"New conversion task"`)
- `info` (default empty string)

---

## ⚙️ API Overview

### `ToDoItem`
| Method | Description |
|--------|--------------|
| `new(kind, fields)` | Construct a new ToDo item with `type` and other key-value pairs. |
| `from_yaml(yaml)` | Load a ToDo item from a YAML hash, validating required keys. |
| `to_yaml()` | Return a reference to the underlying YAML. |
| `status()` | Get current status (`Open`, `Done`, `Failed`, …). |
| `set_status(new_status)` | Manually change the item status. |
| `mark_done()` | Set status to `"Done"`. |
| `mark_failed()` | Set status to `"Failed"`. |
| `reopen()` | Reset to `"Open"`. |
| `set_info(msg)` | Set the `"info"` field. |
| `get(key)` | Retrieve a string value from the YAML by key. |

### `ToDoList`
| Method | Description |
|--------|--------------|
| `new()` | Create an empty ToDoList. |
| `insert(guid, item)` | Add or replace a ToDo item under a GUID. |
| `contains(guid)` | Check if a GUID exists in the list. |
| `update_status(guid, status, info)` | Update both status and info fields. |
| `mark_done(guid)` | Convenience wrapper for marking as done. |
| `to_yaml()` | Export list to a `Yaml::Hash`. |
| `from_yaml(yaml)` | Construct list from YAML structure. |
| `save_to_file(path)` | Save list to file as YAML. |
| `load_from_file(path)` | Load list from YAML file. |

---

## 🧾 Example YAML Format

```yaml
todo_list:
  860a66ca4259b384f9ec01b687f71c3c:
    type: Shader
    unity_path: Assets/Shaders/Heatmap.shader
    godot_path: res://shaders/heatmap.gdshader
    instruction: Convert to Godot 4 shader language
    status: Open
    reason: New conversion task
    info: ""
```

---

## 🧰 Dependencies

```toml
[dependencies]
rust_yaml = { git = "https://github.com/stela2502/rust_yaml", branch = "main" }
```

---

## 📜 License

BSD 3-Clause (same as the rest of your `scenebridge_rs` project).

---

## 🧠 Notes

This library is part of a larger **Unity → Godot translation system**, where ToDo items represent intermediate translation tasks (e.g., converting a Unity material or shader).  
It can also be reused as a general-purpose YAML-backed task tracker in Rust projects.

---

## 🧑‍💻 Author

**Stefan Lang**  
Bioinformatician, Rust enthusiast, and developer of the Unity → Godot converter toolkit.


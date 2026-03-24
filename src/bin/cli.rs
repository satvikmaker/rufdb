//! rufdb CLI — command-line interface for interacting with JSON databases.
//!
//! Usage:
//!   rufdb <file> get [key.path]        Read the whole DB or a nested key
//!   rufdb <file> set <key.path> <json>  Set a value at a key path
//!   rufdb <file> delete <key.path>      Delete a key
//!   rufdb <file> init [json]            Initialize a new database
//!
//! Examples:
//!   rufdb db.json init '{"posts":[],"users":[]}'
//!   rufdb db.json get
//!   rufdb db.json get posts
//!   rufdb db.json set posts '[{"id":1,"title":"Hello"}]'
//!   rufdb db.json delete posts.0

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        print_usage();
        process::exit(1);
    }

    let file = &args[1];
    let command = args[2].as_str();

    let result = match command {
        "init" => cmd_init(file, args.get(3).map(|s| s.as_str())),
        "get" => cmd_get(file, args.get(3).map(|s| s.as_str())),
        "set" => {
            if args.len() < 5 {
                eprintln!("Error: 'set' requires <key.path> and <json>");
                process::exit(1);
            }
            cmd_set(file, &args[3], &args[4])
        }
        "delete" => {
            if args.len() < 4 {
                eprintln!("Error: 'delete' requires <key.path>");
                process::exit(1);
            }
            cmd_delete(file, &args[3])
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn print_usage() {
    eprintln!(
        "rufdb - A fast, file-system based JSON database (Rust implementation of lowdb)

Usage:
  rufdb <file> init [json]            Initialize a new database
  rufdb <file> get [key.path]         Read the whole DB or a nested key
  rufdb <file> set <key.path> <json>  Set a value at a key path
  rufdb <file> delete <key.path>      Delete a key

Examples:
  rufdb db.json init '{{\"posts\":[],\"users\":[]}}'
  rufdb db.json get
  rufdb db.json get posts
  rufdb db.json set posts.0.title '\"Updated Title\"'
  rufdb db.json delete posts.0"
    );
}

fn cmd_init(
    file: &str,
    default_json: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let default = default_json.unwrap_or("{}");
    let data: serde_json::Value = serde_json::from_str(default)?;

    let adapter = rufdb::adapters::JSONFileSync::<serde_json::Value>::new(file);
    let db = rufdb::LowSync::new(adapter, data);
    db.write()?;

    println!("Initialized {}", file);
    Ok(())
}

fn cmd_get(
    file: &str,
    key_path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let adapter = rufdb::adapters::JSONFileSync::<serde_json::Value>::new(file);
    let default = serde_json::Value::Object(serde_json::Map::new());
    let mut db = rufdb::LowSync::new(adapter, default);
    db.read()?;

    let value = match key_path {
        None => &db.data,
        Some(path) => resolve_path(&db.data, path)
            .ok_or_else(|| format!("Key '{}' not found", path))?,
    };

    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn cmd_set(
    file: &str,
    key_path: &str,
    json_value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let new_value: serde_json::Value = serde_json::from_str(json_value)?;

    let adapter = rufdb::adapters::JSONFileSync::<serde_json::Value>::new(file);
    let default = serde_json::Value::Object(serde_json::Map::new());
    let mut db = rufdb::LowSync::new(adapter, default);
    db.read()?;

    set_path(&mut db.data, key_path, new_value)?;
    db.write()?;

    println!("Updated '{}'", key_path);
    Ok(())
}

fn cmd_delete(
    file: &str,
    key_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let adapter = rufdb::adapters::JSONFileSync::<serde_json::Value>::new(file);
    let default = serde_json::Value::Object(serde_json::Map::new());
    let mut db = rufdb::LowSync::new(adapter, default);
    db.read()?;

    delete_path(&mut db.data, key_path)?;
    db.write()?;

    println!("Deleted '{}'", key_path);
    Ok(())
}

/// Resolve a dot-separated path like "posts.0.title" to a JSON value reference.
fn resolve_path<'a>(data: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let mut current = data;
    for key in path.split('.') {
        current = if let Ok(idx) = key.parse::<usize>() {
            current.get(idx)?
        } else {
            current.get(key)?
        };
    }
    Some(current)
}

/// Set a value at a dot-separated path.
fn set_path(
    data: &mut serde_json::Value,
    path: &str,
    value: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let keys: Vec<&str> = path.split('.').collect();
    if keys.is_empty() {
        return Err("Empty path".into());
    }

    let mut current = data;
    for key in &keys[..keys.len() - 1] {
        current = if let Ok(idx) = key.parse::<usize>() {
            current
                .get_mut(idx)
                .ok_or_else(|| format!("Index {} out of bounds", idx))?
        } else {
            current
                .get_mut(*key)
                .ok_or_else(|| format!("Key '{}' not found", key))?
        };
    }

    let last_key = keys[keys.len() - 1];
    if let Ok(idx) = last_key.parse::<usize>() {
        if let Some(arr) = current.as_array_mut() {
            if idx < arr.len() {
                arr[idx] = value;
            } else {
                arr.push(value);
            }
        } else {
            return Err(format!("Cannot index into non-array at '{}'", last_key).into());
        }
    } else if let Some(obj) = current.as_object_mut() {
        obj.insert(last_key.to_string(), value);
    } else {
        return Err(format!("Cannot set key '{}' on non-object", last_key).into());
    }

    Ok(())
}

/// Delete a value at a dot-separated path.
fn delete_path(
    data: &mut serde_json::Value,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let keys: Vec<&str> = path.split('.').collect();
    if keys.is_empty() {
        return Err("Empty path".into());
    }

    let mut current = data;
    for key in &keys[..keys.len() - 1] {
        current = if let Ok(idx) = key.parse::<usize>() {
            current
                .get_mut(idx)
                .ok_or_else(|| format!("Index {} out of bounds", idx))?
        } else {
            current
                .get_mut(*key)
                .ok_or_else(|| format!("Key '{}' not found", key))?
        };
    }

    let last_key = keys[keys.len() - 1];
    if let Ok(idx) = last_key.parse::<usize>() {
        if let Some(arr) = current.as_array_mut() {
            if idx < arr.len() {
                arr.remove(idx);
            } else {
                return Err(format!("Index {} out of bounds", idx).into());
            }
        } else {
            return Err(format!("Cannot index into non-array at '{}'", last_key).into());
        }
    } else if let Some(obj) = current.as_object_mut() {
        obj.remove(last_key)
            .ok_or_else(|| format!("Key '{}' not found", last_key))?;
    } else {
        return Err(format!("Cannot delete key '{}' from non-object", last_key).into());
    }

    Ok(())
}

//! Python bindings for rsdb via PyO3.
//!
//! Exposes a Pythonic API:
//!
//! ```python
//! from rsdb_py import RsDb
//!
//! db = RsDb("db.json", {"posts": [], "users": []})
//! db.read()
//! data = db.get_data()
//! data["posts"].append({"id": 1, "title": "Hello"})
//! db.set_data(data)
//! db.write()
//! ```

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyList};
use std::sync::RwLock;

use ::rsdb::adapters::JSONFileSync;
use ::rsdb::LowSync;
use ::rsdb::SyncAdapter;

/// Convert a Python object to serde_json::Value.
fn py_to_json(obj: &Bound<'_, PyAny>) -> PyResult<serde_json::Value> {
    if obj.is_none() {
        Ok(serde_json::Value::Null)
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(serde_json::Value::Bool(b))
    } else if let Ok(i) = obj.extract::<i64>() {
        Ok(serde_json::json!(i))
    } else if let Ok(f) = obj.extract::<f64>() {
        Ok(serde_json::json!(f))
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(serde_json::Value::String(s))
    } else if let Ok(list) = obj.downcast::<PyList>() {
        let mut arr = Vec::new();
        for item in list.iter() {
            arr.push(py_to_json(&item)?);
        }
        Ok(serde_json::Value::Array(arr))
    } else if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut map = serde_json::Map::new();
        for (key, value) in dict.iter() {
            let k: String = key.extract()?;
            let v = py_to_json(&value)?;
            map.insert(k, v);
        }
        Ok(serde_json::Value::Object(map))
    } else {
        let s: String = obj.str()?.extract()?;
        Ok(serde_json::Value::String(s))
    }
}

/// Convert a serde_json::Value to a Python object.
fn json_to_py(py: Python<'_>, value: &serde_json::Value) -> PyResult<PyObject> {
    match value {
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Bool(b) => Ok(b.into_pyobject(py)?.to_owned().into_any().unbind()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_pyobject(py)?.to_owned().into_any().unbind())
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_pyobject(py)?.to_owned().into_any().unbind())
            } else {
                Ok(py.None())
            }
        }
        serde_json::Value::String(s) => Ok(s.into_pyobject(py)?.to_owned().into_any().unbind()),
        serde_json::Value::Array(arr) => {
            let list = PyList::empty(py);
            for item in arr {
                list.append(json_to_py(py, item)?)?;
            }
            Ok(list.into_any().unbind())
        }
        serde_json::Value::Object(map) => {
            let dict = PyDict::new(py);
            for (k, v) in map {
                dict.set_item(k, json_to_py(py, v)?)?;
            }
            Ok(dict.into_any().unbind())
        }
    }
}

fn map_rsdb_err(e: ::rsdb::RsdbError) -> PyErr {
    pyo3::exceptions::PyIOError::new_err(e.to_string())
}

/// A synchronous JSON file database.
#[pyclass]
struct RsDb {
    inner: RwLock<LowSync<serde_json::Value, JSONFileSync<serde_json::Value>>>,
}

#[pymethods]
impl RsDb {
    #[new]
    fn new(path: String, default_data: &Bound<'_, PyAny>) -> PyResult<Self> {
        let data = py_to_json(default_data)?;
        let adapter = JSONFileSync::new(&path);
        let db = LowSync::new(adapter, data);
        Ok(Self {
            inner: RwLock::new(db),
        })
    }

    fn read(&self) -> PyResult<()> {
        self.inner
            .write()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?
            .read()
            .map_err(map_rsdb_err)
    }

    fn write(&self) -> PyResult<()> {
        self.inner
            .read()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?
            .write()
            .map_err(map_rsdb_err)
    }

    fn get_data(&self, py: Python<'_>) -> PyResult<PyObject> {
        let guard = self
            .inner
            .read()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        json_to_py(py, &guard.data)
    }

    fn set_data(&self, data: &Bound<'_, PyAny>) -> PyResult<()> {
        let json_data = py_to_json(data)?;
        self.inner
            .write()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?
            .data = json_data;
        Ok(())
    }

    fn update(&self, patch: &Bound<'_, PyAny>) -> PyResult<()> {
        let patch_json = py_to_json(patch)?;
        let mut inner = self
            .inner
            .write()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        json_merge_patch(&mut inner.data, &patch_json);
        inner.write().map_err(map_rsdb_err)
    }
}

#[pyfunction]
fn read_db(py: Python<'_>, path: String, default_data: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let data = py_to_json(default_data)?;
    let adapter = JSONFileSync::<serde_json::Value>::new(&path);
    let mut db = LowSync::new(adapter, data);
    db.read().map_err(map_rsdb_err)?;
    json_to_py(py, &db.data)
}

#[pyfunction]
fn write_db(path: String, data: &Bound<'_, PyAny>) -> PyResult<()> {
    let json_data = py_to_json(data)?;
    let adapter = JSONFileSync::<serde_json::Value>::new(&path);
    adapter.write(&json_data).map_err(map_rsdb_err)
}

#[pymodule]
fn rsdb_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<RsDb>()?;
    m.add_function(wrap_pyfunction!(read_db, m)?)?;
    m.add_function(wrap_pyfunction!(write_db, m)?)?;
    Ok(())
}

fn json_merge_patch(target: &mut serde_json::Value, patch: &serde_json::Value) {
    if let serde_json::Value::Object(patch_obj) = patch {
        if !target.is_object() {
            *target = serde_json::Value::Object(serde_json::Map::new());
        }
        if let serde_json::Value::Object(target_obj) = target {
            for (key, value) in patch_obj {
                if value.is_null() {
                    target_obj.remove(key);
                } else if value.is_object() {
                    let entry = target_obj
                        .entry(key.clone())
                        .or_insert(serde_json::Value::Object(serde_json::Map::new()));
                    json_merge_patch(entry, value);
                } else {
                    target_obj.insert(key.clone(), value.clone());
                }
            }
        }
    } else {
        *target = patch.clone();
    }
}

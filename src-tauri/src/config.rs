use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

pub struct Config(pub Mutex<HashMap<String, serde_json::Value>>);

impl Config {
    pub fn load(dir: &Path) -> Result<Self, crate::db::AppError> {
        fs::create_dir_all(dir)?;
        let path = dir.join("config.json");
        let map: HashMap<String, serde_json::Value> = if path.exists() {
            let content = fs::read_to_string(&path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };
        Ok(Config(Mutex::new(map)))
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let map = self.0.lock().unwrap();
        map.get(key).and_then(|v| match v {
            serde_json::Value::String(s) => Some(s.clone()),
            other => Some(other.to_string()),
        })
    }

    pub fn set(&self, key: &str, value: &str) -> Result<(), crate::db::AppError> {
        let parsed: serde_json::Value = serde_json::from_str(value)
            .unwrap_or(serde_json::Value::String(value.to_string()));
        let mut map = self.0.lock().unwrap();
        map.insert(key.to_string(), parsed);
        Ok(())
    }

    pub fn save(&self, dir: &Path) -> Result<(), crate::db::AppError> {
        let path = dir.join("config.json");
        let map = self.0.lock().unwrap();
        let content = serde_json::to_string_pretty(&*map)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn get_value(&self, key: &str) -> Option<serde_json::Value> {
        let map = self.0.lock().unwrap();
        map.get(key).cloned()
    }
}

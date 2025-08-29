use dashmap::DashMap;
use std::hash::Hash;
use std::time::Duration;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use serde_json;
use std::fs::File;
use std::io::BufReader;
use serde::de::DeserializeOwned;

static DURATION: Duration = Duration::from_secs(100);
static CAPACITY: usize = 100;


#[derive(Clone, Serialize, Deserialize)]
pub struct CacheEntry<V> {
    value: V,
    expires_at: u64,
    frequency: u32,
}
pub struct Cache<K, V> {
    map: DashMap<K, CacheEntry<V>>, //CacheEntry<V> allows us to store multiple values associated with a key
}

impl<K, V> Cache<K, V> where
    K: Eq + Hash + Clone + std::fmt::Display + std::marker::Send + std::marker::Sync + Serialize + DeserializeOwned + 'static, // defines the type of key and value
    V: Clone + std::marker::Send + std::marker::Sync + std::fmt::Display + Serialize + DeserializeOwned + 'static,
{
    pub fn new () -> Self {
        Cache {
            map: DashMap::new(),
        }
    }

    pub async fn clean_lfu(self: Arc<Self>){
        tokio::spawn(async move{
            loop{
                tokio::time::sleep(Duration::from_secs(10)).await;
                self.lfu();
            }

        });
    }
    pub fn insert(&self, key: K, value: V){
        let entry = CacheEntry {
            value,
            expires_at: now_epoch_seconds() + DURATION.as_secs(),
            frequency: 0,
        };
        if self.map.len() >= CAPACITY {
            self.lfu();
        }
        self.map.insert(key, entry);
    }

    pub fn get_value(&self, key: &K) -> Option<V>{

        if let Some(mut self_ref) = self.map.get_mut(key) {//gets direct object so it can be modified in place
            if now_epoch_seconds() >= self_ref.expires_at {
                let cloned_key = key.clone();

                drop(self_ref);//drops the reference/lock so it can be removed from the map

                self.map.remove(key);
                println!("Removed expired key: {}", cloned_key);

                self.write_to_file("deleted_keys.txt");
                return None;
            }
            self_ref.frequency += 1;
            Some(self_ref.value.clone())
        }else{
            None
        }
    }

    pub fn lfu(&self){

        let mut min_freq = u32::MAX;
        let mut min_key = None;

        for kv in self.map.iter() {
            let entry = kv.value();
            if entry.frequency < min_freq {
                min_freq = entry.frequency;
                min_key = Some(kv.key().clone());
            }
        }
        if let Some(key) = min_key {
            self.map.remove(&key);
        }

    }
    pub fn read_from_file(&self, path: &str) {
        let file = File::open(path).expect("file not found");
        let reader = BufReader::new(file);

        let entries: Vec<(K, CacheEntry<V>)> = serde_json::from_reader(reader).unwrap_or_default();

        for(k, v) in entries {
            self.map.insert(k, v);
        }
        println!("Finished reading from file, length of vector: {}", self.map.len())
    }
    pub fn write_to_file(&self, path: &str) {
        println!("Writing to file");
        let entries: Vec<(_, _)> = self.map.iter().map(|kv| (kv.key().clone(), kv.value().clone())).collect();

        let serialized = serde_json::to_string_pretty(&entries).unwrap();

        std::fs::write(path, serialized).unwrap();
    }
}

pub fn now_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}


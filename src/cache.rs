use dashmap::DashMap;
use std::hash::Hash;
use std::time::Duration;
use std::sync::Arc;
use std::net::SocketAddr;
use axum::{routing::get, Extension, Router};
use axum::{
    extract::{Path},
};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use serde_json;
use tokio::signal;
use std::fs::File;
use std::io::BufReader;
use axum::routing::post;
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

pub fn build_http(cache: Arc<Cache<String, String>>) -> Router {
    Router::new()
        .route("/info/{key}", get(get_value_http))
        .route("/put/{key}/{value}", post(put_value_http))
        .layer(Extension(cache))

}
pub async fn get_value_http(Path(key): Path<String>, Extension(cache): Extension<Arc<Cache<String, String>>>)
                        -> String {
    cache.get_value(&key).unwrap_or_else(|| "Not found".to_string())
}
pub async fn put_value_http(Path((key, value)): Path<(String, String)>, Extension(cache): Extension<Arc<Cache<String, String>>>){
    cache.insert(key.clone(), value.clone());
    println!("Key={}, Value={}", key, value)

}
pub fn now_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// #[tokio::main]
// async fn main(){
// 
// 
//     let cache = Arc::new(Cache::<String, String>::new());//arcs allows cache to be shared between threads
//     cache.read_from_file("dashmap.txt");
//     cache.clone().clean_lfu();
//     let cache_clone = cache.clone();
// 
//     let _ctrlc_listener = tokio::spawn(async move {
//         println!("CTRL+C pressed");
//         signal::ctrl_c().await.expect("failed to install CTRL+C handler");
//         cache_clone.write_to_file("dashmap.txt");
// 
//         std::process::exit(0);
//     });
// 
// 
//     cache.insert("foo".to_string(), "bar".to_string());
//     cache.insert("hello".to_string(), "world".to_string());
//     cache.insert("user".to_string(), "alice".to_string());
//     cache.insert("session".to_string(), "xyz123".to_string());
//     cache.insert("token".to_string(), "abc987".to_string());
//     cache.insert("http".to_string(), "lol".to_string());
//     cache.insert("config".to_string(), "enabled".to_string());
//     cache.insert("env".to_string(), "production".to_string());
//     cache.insert("item1".to_string(), "value1".to_string());
//     cache.insert("item2".to_string(), "value2".to_string());
// 
//     let app = build_http(cache.clone());
//     let addr = SocketAddr::from(([127, 0, 0, 1], 5000));//http://127.0.0.1:5000/
//     println!("listening on {}", addr);
// 
//     let server_handle = tokio::spawn(async move {
//         axum_server::bind(addr)
//             .serve(app.into_make_service())
//             .await
//             .unwrap();
//     });
// 
// 
//     if let Some(val) = cache.get_value(&"foo".to_string()) {
//         println!("Got immediately: {}", val);
//     } else {
//         println!("Missing immediately!");
//     }
// 
//     tokio::time::sleep(DURATION + Duration::from_secs(1)).await;
// 
//     if let Some(val) = cache.get_value(&"foo".to_string()) {
//         println!("Got after wait: {}", val);
//     }
//     server_handle.await.unwrap();
// 
//     println!("Exiting")
// }


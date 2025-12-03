use anyhow::Context;
use anyhow::Result;
use log::error;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::hash_map::OccupiedEntry;
use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::hash::Hash;
use std::path::PathBuf;
use trait_set::trait_set;

trait_set! {
    pub trait CacheKey = Serialize + for<'de> Deserialize<'de> + Eq + Hash;
    pub trait CacheValue = Serialize + for<'de> Deserialize<'de>;
}

pub struct Cache<K: CacheKey, V: CacheValue> {
    path: PathBuf,
    file: File,
    pub map: HashMap<K, V>, // TODO: do it properly
}

impl<K: CacheKey, V: CacheValue> Cache<K, V> {
    pub fn new(path: PathBuf) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&path)
            .context("Failed to open cache file")?;

        let contents = fs::read_to_string(&path).context("Failed to read cache file")?;
        let map = if contents.is_empty() {
            HashMap::new()
        } else {
            serde_json::from_str(&contents).context("Failed to parse cache file")?
        };
        Ok(Self { path, file, map })
    }

    // pub fn filled(
    //     mut self,
    //     keys: impl IntoIterator<Item = K>,
    //     filling_function: impl Fn(&K) -> Result<V>,
    //     callback: impl Fn(Result<OccupiedEntry<K, V>>),
    // ) {
    //     for key in keys {
    //         callback(match self.app_to_path.entry(key) {
    //             Occupied(entry) => Ok(entry),
    //             Vacant(entry) => {
    //                 filling_function(&entry.key()).and_then(|value| Ok(entry.insert_entry(value)))
    //             }
    //         });
    //     }
    // }
    pub fn get_or_insert_with<'a>(
        &'a mut self,
        key: K,
        f: impl Fn() -> Result<V>,
    ) -> Result<&'a mut V> {
        match self.map.entry(key) {
            Vacant(entry) => f().and_then(|value| Ok(entry.insert(value))),
            Occupied(entry) => Ok(entry.into_mut()),
        }
    }

    fn _flush(&self, fs_sync: bool) -> Result<()> {
        serde_json::to_writer(&self.file, &self.map).context("Failed to serialize cache")?;
        if fs_sync {
            self.file.sync_all().context("Failed to sync cache file")?;
        }
        Ok(())
    }

    pub fn flush(&self) -> Result<()> {
        self._flush(true)
    }
}

impl<K: CacheKey, V: CacheValue> Drop for Cache<K, V> {
    fn drop(&mut self) {
        // False because dropping the file will flush it anyway
        if let Err(e) = self._flush(false) {
            error!("Failed to flush cache: {}", e);
        }
    }
}

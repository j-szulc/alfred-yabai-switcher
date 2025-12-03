use crate::error::{AllOrNone, LogError};
use anyhow::{Context, Result};
use early_returns::some_or_return;
use log::error;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::marker::PhantomData;

#[derive(PartialEq, Eq)]
pub enum ShouldCache {
    Yes,
    No,
}

pub struct Cache<U: Serialize, V: Serialize + for<'a> Deserialize<'a>, F: Fn(U) -> (ShouldCache, V)>
{
    fun: F,
    db: Option<Db>,
    _phantom_input: PhantomData<U>,
    _phantom_output: PhantomData<V>,
}

impl<U: Serialize, V: Serialize + for<'a> Deserialize<'a>, F: Fn(U) -> (ShouldCache, V)>
    Cache<U, V, F>
{
    pub fn new(fun: F, path: &str) -> Self {
        Self {
            fun,
            db: sled::open(path)
                .context("Failed to open cache database. Cache will not be used.")
                .log_error(),
            _phantom_input: PhantomData,
            _phantom_output: PhantomData,
        }
    }

    fn get_key(&self, input: &U) -> Result<String> {
        serde_json::to_string(&input).context("Failed to serialize input as cache key")
    }

    fn lookup_value(&self, key: &String) -> Result<Option<V>> {
        let db = some_or_return!(self.db.as_ref(), Ok(None));

        let value_bytes = some_or_return!(
            db.get(key.as_bytes())
                .context("Failed to get value from cache")?,
            Ok(None)
        );

        let value =
            serde_json::from_slice(&value_bytes).context("Failed to parse value from cache")?;
        Ok(Some(value))
    }

    fn insert_value(&self, key: &String, value: &V) -> Result<()> {
        let db = some_or_return!(self.db.as_ref(), Ok(()));

        let value_bytes =
            serde_json::to_vec(&value).context("Failed to serialize value as cache value")?;

        db.insert(key.as_bytes(), value_bytes)
            .context("Failed to insert value into cache")?;

        Ok(())
    }

    pub fn call(&mut self, input: U) -> V {
        let key = some_or_return!(self.get_key(&input).log_error(), (self.fun)(input).1);

        let new_value = match self.lookup_value(&key).log_error() {
            Some(Some(found_value)) => return found_value,
            _ => (self.fun)(input),
        };

        if new_value.0 == ShouldCache::Yes {
            self.insert_value(&key, &new_value.1).log_error();
        }
        new_value.1
    }
}

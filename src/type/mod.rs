mod bool;
pub use bool::*;
mod r#enum;
pub use r#enum::*;
mod float;
pub use float::*;
mod integer;
pub use integer::*;
mod option;
pub use option::*;
pub mod string;
mod vector;
pub use vector::*;
mod btreemap;
pub use btreemap::*;
mod hashmap;
pub use hashmap::*;
mod third;
#[cfg(any(feature = "third", feature = "ordered_float"))]
pub use third::*;

mod r#macro;

use dashmap::DashMap;
use std::{
    any::TypeId,
    sync::{LazyLock, OnceLock},
};

// due to the ICF could be wrongly implemented by the compiler, we have to use a global cache to store the prompt schema and root name for each type, and use the type id as the key to access the cache.
// although this may cause a performance issue, but it is the only way to ensure the correctness of the prompt schema and root name for each type.
// the case is that generate a big prompt is really time consuming, so we have to cache the prompt schema and root name for each type to avoid generating the prompt schema and root name for each type every time we need to use it.
// maybe future we can use a more efficient way to store the prompt schema and root name for each type, but for now, this is the only way to ensure the correctness of the prompt schema and root name for each type.
// Now is RUST 1.92

pub struct CacheInner {
    pub prompt_schema: OnceLock<String>,
    pub root_name: OnceLock<String>,
}

static CACHE_HOLDER: LazyLock<DashMap<TypeId, &'static CacheInner>> = LazyLock::new(DashMap::new);

pub struct Cache<T>(std::marker::PhantomData<T>);

impl<T: 'static> Cache<T> {
    pub fn get() -> &'static CacheInner {
        let tid = TypeId::of::<T>();

        if let Some(inner) = CACHE_HOLDER.get(&tid) {
            return *inner;
        }

        *CACHE_HOLDER.entry(tid).or_insert_with(|| {
            Box::leak(Box::new(CacheInner {
                prompt_schema: OnceLock::new(),
                root_name: OnceLock::new(),
            }))
        })
    }
}

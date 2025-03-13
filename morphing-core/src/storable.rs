use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use dyn_clone::DynClone;
use dyn_eq::DynEq;
use dyn_hash::DynHash;

pub trait SlotKeyGenerator: 'static + Send + Sync {
    type SlotKey: 'static + Clone;

    fn new() -> Self;
    fn generate_slot_key(&mut self) -> Self::SlotKey;
}

pub struct SharableSlotKeyGenerator;

impl SlotKeyGenerator for SharableSlotKeyGenerator {
    type SlotKey = ();

    fn new() -> Self {
        Self
    }

    fn generate_slot_key(&mut self) -> Self::SlotKey {
        ()
    }
}

pub struct VecSlotKeyGenerator(usize);

impl SlotKeyGenerator for VecSlotKeyGenerator {
    type SlotKey = usize;

    fn new() -> Self {
        Self(0)
    }

    fn generate_slot_key(&mut self) -> Self::SlotKey {
        let key = self.0;
        self.0 += 1;
        key
    }
}

pub trait Slot: 'static + Send + Sync {
    type Value;
    type SlotKeyGenerator: SlotKeyGenerator;

    fn new() -> Self;
    fn get(
        &self,
        slot_key: &<Self::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
    ) -> Option<&Self::Value>;
    fn get_or_insert_with<F>(
        &mut self,
        slot_key: &<Self::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
        f: F,
    ) -> &mut Self::Value
    where
        F: FnOnce() -> Self::Value;
    fn expire(&mut self);
}

pub struct SharableSlot<V>(Option<Arc<V>>);

impl<V> Slot for SharableSlot<V>
where
    V: 'static + Send + Sync,
{
    type Value = Arc<V>;
    type SlotKeyGenerator = SharableSlotKeyGenerator;

    fn new() -> Self {
        Self(None)
    }

    fn get(
        &self,
        _slot_key: &<Self::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
    ) -> Option<&Self::Value> {
        self.0.as_ref()
    }

    fn get_or_insert_with<F>(
        &mut self,
        _slot_key: &<Self::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
        f: F,
    ) -> &mut Self::Value
    where
        F: FnOnce() -> Self::Value,
    {
        self.0.get_or_insert_with(f)
    }

    fn expire(&mut self) {
        self.0.take();
    }
}

pub struct VecSlot<V>(Vec<Option<V>>);

impl<V> Slot for VecSlot<V>
where
    V: 'static + Send + Sync,
{
    type Value = V;
    type SlotKeyGenerator = VecSlotKeyGenerator;

    fn new() -> Self {
        Self(Vec::new())
    }

    fn get(
        &self,
        slot_key: &<Self::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
    ) -> Option<&Self::Value> {
        self.0.get(*slot_key).as_ref().unwrap().as_ref()
    }

    fn get_or_insert_with<F>(
        &mut self,
        slot_key: &<Self::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
        f: F,
    ) -> &mut Self::Value
    where
        F: FnOnce() -> Self::Value,
    {
        if self.0.len() <= *slot_key {
            self.0.resize_with(slot_key + 1, || None);
        }
        self.0.get_mut(*slot_key).unwrap().get_or_insert_with(f)
    }

    fn expire(&mut self) {
        self.0.clear()
    }
}

pub struct SwapSlot<S> {
    active: S,
    inactive: S,
}

impl<S> Slot for SwapSlot<S>
where
    S: Slot,
{
    type Value = S::Value;
    type SlotKeyGenerator = S::SlotKeyGenerator;

    fn new() -> Self {
        Self {
            active: S::new(),
            inactive: S::new(),
        }
    }

    fn get(
        &self,
        slot_key: &<Self::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
    ) -> Option<&Self::Value> {
        self.active.get(slot_key)
    }

    fn get_or_insert_with<F>(
        &mut self,
        slot_key: &<Self::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
        f: F,
    ) -> &mut Self::Value
    where
        F: FnOnce() -> Self::Value,
    {
        self.active.get_or_insert_with(slot_key, f)
    }

    fn expire(&mut self) {
        let _ = std::mem::replace(
            &mut self.inactive,
            std::mem::replace(&mut self.active, S::new()),
        );
    }
}

pub trait DynKey: 'static + Send + Sync + DynClone + DynEq + DynHash {}

impl<K> DynKey for K where K: 'static + Clone + Eq + Hash + Send + Sync {}

dyn_clone::clone_trait_object!(DynKey);
dyn_eq::eq_trait_object!(DynKey);
dyn_hash::hash_trait_object!(DynKey);

pub trait Storable: 'static + Send + Sync {
    type StorableKey: Clone + Eq + Hash + Send + Sync;
    type Slot: Slot;

    fn key(
        &self,
        storable_key_fn: &fn(&dyn serde_traitobject::Serialize) -> Box<dyn DynKey>,
    ) -> Self::StorableKey;
}

#[derive(Clone)]
pub struct StorageKey<K, SK> {
    storable_key: K,
    slot_key: SK,
}

struct SlotKeyGeneratorWrapper<K, S>(K, S);

impl<K, S> typemap_rev::TypeMapKey for SlotKeyGeneratorWrapper<K, S>
where
    K: 'static + Send + Sync,
    S: Slot,
{
    type Value = HashMap<K, S::SlotKeyGenerator>;
}

pub struct SlotKeyGeneratorTypeMap {
    type_map: typemap_rev::TypeMap,
    storable_key_fn: fn(&dyn serde_traitobject::Serialize) -> Box<dyn DynKey>,
}

impl SlotKeyGeneratorTypeMap {
    pub fn new(storable_key_fn: fn(&dyn serde_traitobject::Serialize) -> Box<dyn DynKey>) -> Self {
        Self {
            type_map: typemap_rev::TypeMap::new(),
            storable_key_fn,
        }
    }

    pub fn allocate<S>(
        &mut self,
        storable: &S,
    ) -> StorageKey<
        S::StorableKey,
        <<S::Slot as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
    >
    where
        S: Storable,
    {
        let storable_key = storable.key(&self.storable_key_fn);
        StorageKey {
            storable_key: storable_key.clone(),
            slot_key: self
                .type_map
                .entry::<SlotKeyGeneratorWrapper<S::StorableKey, S::Slot>>()
                .or_insert_with(HashMap::new)
                .entry(storable_key)
                .or_insert_with(<S::Slot as Slot>::SlotKeyGenerator::new)
                .generate_slot_key(),
        }
    }

    pub fn expire(&mut self) {
        self.type_map.clear();
    }
}

struct StorageWrapper<K, S>(K, S);

impl<K, S> typemap_rev::TypeMapKey for StorageWrapper<K, S>
where
    K: 'static + Send + Sync,
    S: Slot,
{
    type Value = HashMap<K, S>;
}

trait Expire: Send + Sync {
    fn expire(&mut self);
}

impl<K, S> Expire for HashMap<K, S>
where
    K: Send + Sync,
    S: Slot,
{
    fn expire(&mut self) {
        self.values_mut().for_each(Slot::expire);
    }
}

impl<K, S> typemap_rev::IntoBox<dyn Expire> for HashMap<K, S>
where
    K: 'static + Send + Sync,
    S: Slot,
{
    fn into_box(self) -> Box<dyn Expire> {
        Box::new(self)
    }
}

pub struct StorageTypeMap {
    type_map: typemap_rev::TypeMap<dyn Expire>,
}

impl StorageTypeMap {
    pub fn new() -> Self {
        Self {
            type_map: typemap_rev::TypeMap::custom(),
        }
    }

    pub fn get_or_insert_with<K, S, F>(
        &mut self,
        storage_key: &StorageKey<K, <S::SlotKeyGenerator as SlotKeyGenerator>::SlotKey>,
        f: F,
    ) -> &mut S::Value
    where
        K: 'static + Clone + Eq + Hash + Send + Sync,
        S: Slot,
        F: FnOnce() -> S::Value,
    {
        self.type_map
            .entry::<StorageWrapper<K, S>>()
            .or_insert_with(HashMap::new)
            .entry(storage_key.storable_key.clone())
            .or_insert_with(S::new)
            .get_or_insert_with(&storage_key.slot_key, f)
    }

    pub fn get<K, S>(
        &self,
        storage_key: &StorageKey<K, <S::SlotKeyGenerator as SlotKeyGenerator>::SlotKey>,
    ) -> Option<&S::Value>
    where
        K: 'static + Clone + Eq + Hash + Send + Sync,
        S: Slot,
    {
        self.type_map
            .get::<StorageWrapper<K, S>>()
            .unwrap()
            .get(&storage_key.storable_key)
            .unwrap()
            .get(&storage_key.slot_key)
    }

    pub fn expire(&mut self) {
        self.type_map = std::mem::replace(&mut self.type_map, typemap_rev::TypeMap::custom())
            .into_iter()
            .map(|(type_id, mut presentation_storage)| {
                presentation_storage.expire();
                (type_id, presentation_storage)
            })
            .collect();
    }
}

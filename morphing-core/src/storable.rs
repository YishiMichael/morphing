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
    fn update_or_insert<D, F, FO>(
        &mut self,
        slot_key: &<Self::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
        default: D,
        f: F,
    ) -> Option<FO>
    where
        D: FnOnce() -> Self::Value,
        F: FnOnce(&mut Self::Value) -> FO;
    fn get_and_unwrap(
        &self,
        slot_key: &<Self::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
    ) -> &Self::Value;
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

    fn update_or_insert<D, F, FO>(
        &mut self,
        _slot_key: &<Self::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
        default: D,
        f: F,
    ) -> Option<FO>
    where
        D: FnOnce() -> Self::Value,
        F: FnOnce(&mut Self::Value) -> FO,
    {
        if let Some(value) = self.0.as_mut() {
            Some(f(value))
        } else {
            self.0.insert(default());
            None
        }
    }

    fn get_and_unwrap(
        &self,
        _slot_key: &<Self::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
    ) -> &Self::Value {
        self.0.as_ref().unwrap()
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

    fn update_or_insert<D, F, FO>(
        &mut self,
        slot_key: &<Self::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
        default: D,
        f: F,
    ) -> Option<FO>
    where
        D: FnOnce() -> Self::Value,
        F: FnOnce(&mut Self::Value) -> FO,
    {
        if self.0.len() <= *slot_key {
            self.0.resize_with(slot_key + 1, || None);
        }
        let option_mut = self.0.get_mut(*slot_key).unwrap();
        if let Some(value) = option_mut.as_mut() {
            Some(f(value))
        } else {
            option_mut.insert(default());
            None
        }
    }

    fn get_and_unwrap(
        &self,
        slot_key: &<Self::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
    ) -> &Self::Value {
        self.0.get(*slot_key).as_ref().unwrap().as_ref().unwrap()
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

    fn update_or_insert<D, F, FO>(
        &mut self,
        slot_key: &<Self::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
        default: D,
        f: F,
    ) -> Option<FO>
    where
        D: FnOnce() -> Self::Value,
        F: FnOnce(&mut Self::Value) -> FO,
    {
        self.active.update_or_insert(slot_key, default, f)
    }

    fn get_and_unwrap(
        &self,
        slot_key: &<Self::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
    ) -> &Self::Value {
        self.active.get_and_unwrap(slot_key)
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
    type KeyInput<'s>: serde::Serialize;
    type Slot: Slot;

    fn key_input<'s>(
        &'s self,
        // storable_key_fn: &fn(&dyn serde_traitobject::Serialize) -> Box<dyn DynKey>,
    ) -> Self::KeyInput<'s>;
}

#[derive(Clone)]
pub struct StorageKey<S>
where
    S: Storable,
{
    storable_key: Box<dyn DynKey>,
    slot_key: <<S::Slot as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
}

struct SlotKeyGeneratorWrapper<S>(S);

impl<S> typemap_rev::TypeMapKey for SlotKeyGeneratorWrapper<S>
where
    S: Storable,
{
    type Value = HashMap<Box<dyn DynKey>, <S::Slot as Slot>::SlotKeyGenerator>;
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

    pub fn allocate<S>(&mut self, storable: &S) -> StorageKey<S>
    where
        S: Storable,
    {
        let storable_key = (self.storable_key_fn)(&storable.key_input());
        StorageKey {
            storable_key: storable_key.clone(),
            slot_key: self
                .type_map
                .entry::<SlotKeyGeneratorWrapper<S>>()
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

struct StorageWrapper<S>(S);

impl<S> typemap_rev::TypeMapKey for StorageWrapper<S>
where
    S: Storable,
{
    type Value = HashMap<Box<dyn DynKey>, S::Slot>;
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

    pub fn update_or_insert<S, D, F, FO>(
        &mut self,
        storage_key: &StorageKey<S>,
        default: D,
        f: F,
    ) -> Option<FO>
    where
        S: Storable,
        D: FnOnce() -> <S::Slot as Slot>::Value,
        F: FnOnce(&mut <S::Slot as Slot>::Value) -> FO,
    {
        self.type_map
            .entry::<StorageWrapper<S>>()
            .or_insert_with(HashMap::new)
            .entry(storage_key.storable_key.clone())
            .or_insert_with(S::Slot::new)
            .update_or_insert(&storage_key.slot_key, default, f)
    }

    pub fn get_and_unwrap<S>(&self, storage_key: &StorageKey<S>) -> &<S::Slot as Slot>::Value
    where
        S: Storable,
    {
        self.type_map
            .get::<StorageWrapper<S>>()
            .unwrap()
            .get(&storage_key.storable_key)
            .unwrap()
            .get_and_unwrap(&storage_key.slot_key)
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

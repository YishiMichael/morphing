use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use dyn_clone::DynClone;
use dyn_eq::DynEq;
use dyn_hash::DynHash;

pub type ResourceReuseResult = Result<(), ()>;

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
    fn update_or_insert<I>(
        &mut self,
        slot_key: &<Self::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
        input: I,
        result: &mut ResourceReuseResult,
        default: fn(I) -> Self::Value,
        f: fn(I, &mut Self::Value, &mut ResourceReuseResult),
    ) -> &Self::Value;
    fn get_and_unwrap(
        &self,
        slot_key: &<Self::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
    ) -> &Self::Value;
    fn expire(&mut self);
}

pub struct ArcSlot<V>(Option<Arc<V>>);

impl<V> Slot for ArcSlot<V>
where
    V: 'static + Send + Sync,
{
    type Value = Arc<V>;
    type SlotKeyGenerator = SharableSlotKeyGenerator;

    fn new() -> Self {
        Self(None)
    }

    fn update_or_insert<I>(
        &mut self,
        _slot_key: &<Self::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
        input: I,
        result: &mut ResourceReuseResult,
        default: fn(I) -> Self::Value,
        f: fn(I, &mut Self::Value, &mut ResourceReuseResult),
    ) -> &Self::Value {
        let value = match self.0.take() {
            Some(mut value) => {
                f(input, &mut value, result);
                value
            }
            None => {
                *result = Err(());
                default(input)
            }
        };
        self.0.insert(value)
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

    fn update_or_insert<I>(
        &mut self,
        slot_key: &<Self::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
        input: I,
        result: &mut ResourceReuseResult,
        default: fn(I) -> Self::Value,
        f: fn(I, &mut Self::Value, &mut ResourceReuseResult),
    ) -> &Self::Value {
        if self.0.len() <= *slot_key {
            self.0.resize_with(slot_key + 1, || None);
        }
        let option_mut = self.0.get_mut(*slot_key).unwrap();
        let value = match option_mut.take() {
            Some(mut value) => {
                f(input, &mut value, result);
                value
            }
            None => {
                *result = Err(());
                default(input)
            }
        };
        option_mut.insert(value)
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

    fn update_or_insert<I>(
        &mut self,
        slot_key: &<Self::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
        input: I,
        result: &mut ResourceReuseResult,
        default: fn(I) -> Self::Value,
        f: fn(I, &mut Self::Value, &mut ResourceReuseResult),
    ) -> &Self::Value {
        self.active
            .update_or_insert(slot_key, input, result, default, f)
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

pub trait StoreType: 'static + Send + Sync {
    type KeyInput: serde::Serialize;
    type Slot: Slot;

    // fn key_input<'s>(
    //     &'s self,
    //     // storable_key_fn: &fn(&dyn serde_traitobject::Serialize) -> Box<dyn DynKey>,
    // ) -> Self::KeyInput<'s>;
}

#[derive(Clone)]
pub struct StorageKey<ST>
where
    ST: StoreType,
{
    storable_key: Box<dyn DynKey>,
    slot_key: <<ST::Slot as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
}

struct SlotKeyGeneratorWrapper<KI, S>(KI, S);

impl<KI, S> typemap_rev::TypeMapKey for SlotKeyGeneratorWrapper<KI, S>
where
    KI: 'static,
    S: Slot,
{
    type Value = HashMap<Box<dyn DynKey>, S::SlotKeyGenerator>;
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

    pub fn allocate<ST>(&mut self, key_input: &ST::KeyInput) -> StorageKey<ST>
    where
        ST: StoreType,
    {
        let storable_key = (self.storable_key_fn)(key_input);
        StorageKey {
            storable_key: storable_key.clone(),
            slot_key: self
                .type_map
                .entry::<SlotKeyGeneratorWrapper<ST::KeyInput, ST::Slot>>()
                .or_insert_with(HashMap::new)
                .entry(storable_key)
                .or_insert_with(<ST::Slot as Slot>::SlotKeyGenerator::new)
                .generate_slot_key(),
        }
    }

    pub fn expire(&mut self) {
        self.type_map.clear();
    }
}

struct StorageWrapper<KI, S>(KI, S);

impl<KI, S> typemap_rev::TypeMapKey for StorageWrapper<KI, S>
where
    KI: 'static,
    S: Slot,
{
    type Value = HashMap<Box<dyn DynKey>, S>;
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

    pub fn update_or_insert<ST, I>(
        &mut self,
        storage_key: &StorageKey<ST>,
        input: I,
        result: &mut ResourceReuseResult,
        default: fn(I) -> <ST::Slot as Slot>::Value,
        f: fn(I, &mut <ST::Slot as Slot>::Value, &mut ResourceReuseResult),
    ) -> &<ST::Slot as Slot>::Value
    where
        ST: StoreType,
    {
        self.type_map
            .entry::<StorageWrapper<ST::KeyInput, ST::Slot>>()
            .or_insert_with(HashMap::new)
            .entry(storage_key.storable_key.clone())
            .or_insert_with(ST::Slot::new)
            .update_or_insert(&storage_key.slot_key, input, result, default, f)
    }

    pub fn get_and_unwrap<ST>(&self, storage_key: &StorageKey<ST>) -> &<ST::Slot as Slot>::Value
    where
        ST: StoreType,
    {
        self.type_map
            .get::<StorageWrapper<ST::KeyInput, ST::Slot>>()
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

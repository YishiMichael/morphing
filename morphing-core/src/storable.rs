use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

// trait SerdeKey: 'static + Clone + Eq + Hash + Send + Sync {}

// impl<T> SerdeKey for T where T: 'static + Clone + Eq + Hash + Send + Sync {}

// pub(crate) trait KeyFn: 'static + Default + Send + Sync {
//     type Input: 'static;
//     type Output: 'static;

//     fn eval_key(&self, input: &Self::Input) -> Self::Output;
// }

pub(crate) trait SlotKeyGenerator: 'static + Send + Sync {
    type SlotKey: 'static + Clone;

    fn new() -> Self;
    fn generate_slot_key(&mut self) -> Self::SlotKey;
}

pub(crate) struct SharableSlotKeyGenerator;

impl SlotKeyGenerator for SharableSlotKeyGenerator {
    type SlotKey = ();

    fn new() -> Self {
        Self
    }

    fn generate_slot_key(&mut self) -> Self::SlotKey {
        ()
    }
}

pub(crate) struct VecSlotKeyGenerator(usize);

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

pub(crate) trait Slot: 'static + Send + Sync {
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

pub(crate) struct SharableSlot<V>(Option<Arc<V>>);

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

pub(crate) struct VecSlot<V>(Vec<Option<V>>);

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

pub(crate) struct SwapSlot<S> {
    active: S,
    inactive: S,
}

impl<S> Slot for SwapSlot<S>
where
    S: Slot,
{
    // type Presentation = PS::Presentation;
    // type StorageIdInput = PS::StorageIdInput;
    // type KeyGenerator = PS::KeyGenerator;
    // type Value = PS::Value;
    // type PrepareRef<'s> = PS::PrepareRef<'s>;
    // type RenderRef<'s> = PS::RenderRef<'s>;

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

// #[derive(Default)]
// pub(crate) struct FnKeyGenerator<KF, KG> {
//     key_fn: KF,
//     key_generator: KG,
// }

// impl<KF, KG> KeyGenerator for FnKeyGenerator<KF, KG>
// where
//     KF: KeyFn,
//     KG: KeyGenerator<Input = ()>,
// {
//     type Input = KF::Input;
//     type Output = (KF::Output, KG::Output);

//     fn generate_key(&mut self, input: &Self::Input) -> Self::Output {
//         (
//             self.key_fn.eval_key(input),
//             self.key_generator.generate_key(()),
//         )
//     }
// }

// pub trait StorableMap: 'static + Send + Sync {
//     // type Presentation: Storable;
//     // type StorageIdInput;
//     // type KeyGenerator: KeyGenerator;
//     // type StorageId;
//     // type Value;
//     // type PrepareRef<'s>;
//     // type RenderRef<'s>;
//     type Key: 'static + Clone + Send + Sync;
//     type Value: 'static + Send + Sync;

//     fn new() -> Self;
//     fn get(&self, key: &Self::Key) -> Option<&Self::Value>;
//     fn entry(&mut self, key: Self::Key) -> Entry<Self::Key, Self::Value>;
//     // where
//     //     F: FnOnce() -> Self::Presentation;
//     // fn get(&self, storage_id: &Self::StorageId) -> Option<&Self::Target>;
//     // fn allocate(&mut self, key: K, presentation: Self::Target) -> Self::StorageId;
//     fn expire(&mut self);
// }

// pub(crate) struct MapStorage<K, V>(HashMap<K, V>);

// impl<K, V> StorableMap for MapStorage<K, V>
// where
//     K: 'static + Clone + Eq + Hash + Send + Sync,
//     V: 'static + Send + Sync,
// {
//     // type Presentation = P;
//     // type StorageIdInput = ();
//     // type KeyGenerator = ReadKeyGenerator;
//     // type Value = V;
//     // type PrepareRef<'s> = &'s mut Arc<P>;
//     // type RenderRef<'s> = &'s Arc<P>;

//     type Key = K;
//     type Value = V;

//     fn new() -> Self {
//         Self(HashMap::new())
//     }

//     fn get(&self, key: &Self::Key) -> Option<&Self::Value> {
//         self.0.get(key)
//     }

//     fn entry(&mut self, key: Self::Key) -> Entry<Self::Key, Self::Value> {
//         self.0.entry(key)
//     }

//     // fn allocate(&mut self, _key: (), presentation: Self::Target) -> Self::StorageId {
//     //     self.0.insert(presentation);
//     //     ()
//     // }

//     fn expire(&mut self) {
//         self.0.clear();
//         // std::mem::replace(&mut self.inactive, std::mem::take(&mut self.active));
//     }
// }

// pub(crate) struct ReadWriteStorage<K, P>(HashMap<K, Arc<RwLock<P>>>);

// impl<K, P> Storage<K> for ReadWriteStorage<K, P>
// where
//     P: Storable,
// {
//     // type Presentation = P;
//     // type StorageIdInput = ();
//     type KeyGenerator = ReadWriteKeyGenerator;
//     type Value = Arc<RwLock<P>>;
//     // type PrepareRef<'s> = &'s mut P;
//     // type RenderRef<'s> = &'s P;

//     fn new() -> Self {
//         Self(Vec::new())
//     }

//     fn get_mut(&mut self, key: &Self::StorageId) -> &mut Option<Self::Value> {
//         self.0.get_mut(*key)
//     }

//     fn get_ref(&self, key: &Self::StorageId) -> &Option<Self::Value> {
//         self.0.get(*key)
//     }

//     // fn get(&self, storage_id: &Self::StorageId) -> Option<&Self::Target> {
//     //     self.0.get(storage_id).unwrap()
//     // }

//     // fn allocate(&mut self, _input: (), presentation: Self::Target) -> Self::StorageId {
//     //     let storage_id = self.0.len();
//     //     self.0.push(presentation);
//     //     storage_id
//     // }

//     fn expire(&mut self) {
//         self.0.clear();
//         // std::mem::replace(&mut self.inactive, std::mem::take(&mut self.active));
//     }
// }

// pub(crate) struct MapStorage<K, PS>(HashMap<K, PS>);

// impl<K, VK, PS> PresentationStorage<(K, VK)> for MapStorage<K, PS>
// where
//     K: 'static + Clone + Eq + Hash + Send + Sync,
//     PS: PresentationStorage<VK>,
// {
//     // type Presentation = PS::Presentation;
//     // type StorageIdInput = (K, PS::StorageIdInput);
//     type StorageId = (K, PS::StorageId);
//     type Target = PS::Target;
//     // type PrepareRef<'s> = PS::PrepareRef<'s>;
//     // type RenderRef<'s> = PS::RenderRef<'s>;

//     fn new() -> Self {
//         Self(HashMap::new())
//     }

//     fn get_mut(&mut self, storage_id: &Self::StorageId) -> &mut Option<Self::Target> {
//         self.0
//             .get_mut(&storage_id.0)
//             .unwrap()
//             .get_mut(&storage_id.1)
//     }

//     fn get_ref(&self, storage_id: &Self::StorageId) -> &Option<Self::Target> {
//         self.0.get(&storage_id.0).unwrap().get_ref(&storage_id.1)
//     }

//     fn allocate(&mut self, key: (K, VK), presentation: Self::Target) -> Self::StorageId {
//         (
//             key.0.clone(),
//             self.0
//                 .entry(key.0)
//                 .or_insert_with(PS::new)
//                 .allocate(key.1, presentation),
//         )
//     }

//     fn expire(&mut self) {
//         self.0.values_mut().for_each(PS::expire);
//     }
// }

pub trait Storable: 'static + Send + Sync {
    type StorableKey: Clone + Eq + Hash + Send + Sync;
    type Slot: Slot;

    fn key(&self) -> Self::StorableKey;
    // fn store(&self) -> <Self::Slot as Slot>::Value;
}

// impl<PS> PresentationStoragePrimitive for Box<dyn PresentationStoragePrimitive<PresentationStorage = PS>>
// where
//     PS: PresentationStorage,
// {
//     type PresentationStorage = PS;

//     fn storage_id_input(&self) -> PS::StorageIdInput {
//         self.as_ref().storage_id_input()
//     }
// }

#[derive(Clone)]
pub struct StorageKey<K, SK> {
    storable_key: K,
    slot_key: SK,
}

// pub struct PresentationStorageEntry<PSP>
// where
//     PSP: PresentationStoragePrimitive,
// {
//     storage_id: Option<<PSP::PresentationStorage as PresentationStorage<PSP::Key>>::StorageId>,
//     primitive: PSP,
// }

// impl<S> Allocated<S>
// where
//     S: Storable,
// {
//     pub(crate) fn storable(&self) -> &S {
//         &self.storable
//     }
// }

struct SlotKeyGeneratorWrapper<K, S>(K, S);

impl<K, S> typemap_rev::TypeMapKey for SlotKeyGeneratorWrapper<K, S>
where
    K: 'static + Send + Sync,
    S: Slot,
{
    type Value = HashMap<K, S::SlotKeyGenerator>;
}

pub struct SlotKeyGeneratorTypeMap(typemap_rev::TypeMap);

impl SlotKeyGeneratorTypeMap {
    pub fn new() -> Self {
        Self(typemap_rev::TypeMap::new())
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
        let storable_key = storable.key();
        StorageKey {
            storable_key: storable_key.clone(),
            slot_key: self
                .0
                .entry::<SlotKeyGeneratorWrapper<S::StorableKey, S::Slot>>()
                .or_insert_with(HashMap::new)
                .entry(storable_key)
                .or_insert_with(<S::Slot as Slot>::SlotKeyGenerator::new)
                .generate_slot_key(),
        }
    }

    pub fn expire(&mut self) {
        self.0.clear();
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
        self.values_mut().map(Slot::expire);
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

pub struct StorageTypeMap(typemap_rev::TypeMap<dyn Expire>);

impl StorageTypeMap {
    pub fn new() -> Self {
        Self(typemap_rev::TypeMap::custom())
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
        self.0
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
        self.0
            .get::<StorageWrapper<K, S>>()
            .unwrap()
            .get(&storage_key.storable_key)
            .unwrap()
            .get(&storage_key.slot_key)
    }

    pub fn expire(&mut self) {
        self.0 = std::mem::replace(&mut self.0, typemap_rev::TypeMap::custom())
            .into_iter()
            .map(|(type_id, mut presentation_storage)| {
                presentation_storage.expire();
                (type_id, presentation_storage)
            })
            .collect();
    }
}

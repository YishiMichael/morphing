use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;

// trait SerdeKey: 'static + Clone + Eq + Hash + Send + Sync {}

// impl<T> SerdeKey for T where T: 'static + Clone + Eq + Hash + Send + Sync {}

pub trait KeyFn: 'static + Default + Send + Sync {
    type Input: 'static;
    type Output: 'static;

    fn eval(&self, input: &Self::Input) -> Self::Output;
}

trait KeyGenerator: 'static + Default + Send + Sync {
    type Input: 'static;
    type Output: 'static;

    fn generate_key(&mut self, input: &Self::Input) -> Self::Output;
}

#[derive(Default)]
struct UnitKeyGenerator;

impl KeyGenerator for UnitKeyGenerator {
    type Input = ();
    type Output = ();

    fn generate_key(&mut self, _input: &Self::Input) -> Self::Output {
        ()
    }
}

#[derive(Default)]
struct UniqueKeyGenerator(usize);

impl KeyGenerator for UniqueKeyGenerator {
    type Input = ();
    type Output = usize;

    fn generate_key(&mut self, _input: &Self::Input) -> Self::Output {
        let key = self.0;
        self.0 += 1;
        key
    }
}

#[derive(Default)]
struct FnKeyGenerator<KF, KG> {
    key_fn: KF,
    key_generator: KG,
}

impl<KF, KG> KeyGenerator for FnKeyGenerator<KF, KG>
where
    KF: KeyFn,
    KG: KeyGenerator<Input = ()>,
{
    type Input = KF::Input;
    type Output = (KF::Output, KG::Output);

    fn generate_key(&mut self, input: &Self::Input) -> Self::Output {
        (self.key_fn.eval(input), self.key_generator.generate_key(()))
    }
}

pub trait StorableMap: 'static + Send + Sync {
    // type Presentation: Storable;
    // type StorageIdInput;
    // type KeyGenerator: KeyGenerator;
    // type StorageId;
    // type Value;
    // type PrepareRef<'s>;
    // type RenderRef<'s>;
    type Key: 'static + Clone + Send + Sync;
    type Value: 'static + Send + Sync;

    fn new() -> Self;
    fn get(&self, key: &Self::Key) -> Option<&Self::Value>;
    fn entry(&mut self, key: Self::Key) -> Entry<Self::Key, Self::Value>;
    // where
    //     F: FnOnce() -> Self::Presentation;
    // fn get(&self, storage_id: &Self::StorageId) -> Option<&Self::Target>;
    // fn allocate(&mut self, key: K, presentation: Self::Target) -> Self::StorageId;
    fn expire(&mut self);
}

pub(crate) struct MapStorage<K, V>(HashMap<K, V>);

impl<K, V> StorableMap for MapStorage<K, V>
where
    K: 'static + Clone + Eq + Hash + Send + Sync,
    V: 'static + Send + Sync,
{
    // type Presentation = P;
    // type StorageIdInput = ();
    // type KeyGenerator = ReadKeyGenerator;
    // type Value = V;
    // type PrepareRef<'s> = &'s mut Arc<P>;
    // type RenderRef<'s> = &'s Arc<P>;

    type Key = K;
    type Value = V;

    fn new() -> Self {
        Self(HashMap::new())
    }

    fn get(&self, key: &Self::Key) -> Option<&Self::Value> {
        self.0.get(key)
    }

    fn entry(&mut self, key: Self::Key) -> Entry<Self::Key, Self::Value> {
        self.0.entry(key)
    }

    // fn allocate(&mut self, _key: (), presentation: Self::Target) -> Self::StorageId {
    //     self.0.insert(presentation);
    //     ()
    // }

    fn expire(&mut self) {
        self.0.clear();
        // std::mem::replace(&mut self.inactive, std::mem::take(&mut self.active));
    }
}

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

pub(crate) struct SwapStorage<SM> {
    active: SM,
    inactive: SM,
}

impl<SM> StorableMap for SwapStorage<SM>
where
    SM: StorableMap,
{
    // type Presentation = PS::Presentation;
    // type StorageIdInput = PS::StorageIdInput;
    // type KeyGenerator = PS::KeyGenerator;
    // type Value = PS::Value;
    // type PrepareRef<'s> = PS::PrepareRef<'s>;
    // type RenderRef<'s> = PS::RenderRef<'s>;

    type Key = SM::Key;
    type Value = SM::Value;

    fn new() -> Self {
        Self {
            active: SM::new(),
            inactive: SM::new(),
        }
    }

    fn get(&self, key: &Self::Key) -> Option<&Self::Value> {
        self.active.get(key)
    }

    fn entry(&mut self, key: Self::Key) -> Entry<Self::Key, Self::Value> {
        self.active.entry(key)
    }

    // fn allocate(&mut self, key: K, presentation: Self::Target) -> Self::StorageId {
    //     self.active.allocate(key, presentation)
    // }

    fn expire(&mut self) {
        let _ = std::mem::replace(
            &mut self.inactive,
            std::mem::replace(&mut self.active, SM::new()),
        );
    }
}

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
    type KeyGenerator: KeyGenerator<Input = Self, Output = <Self::StorableMap as StorableMap>::Key>;
    type StorableMap: StorableMap;

    // fn storage_id_input(&self) -> Self::Key;
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

pub struct Allocated<S>
where
    S: Storable,
{
    key: <S::StorableMap as StorableMap>::Key,
    storable: S,
}

// pub struct PresentationStorageEntry<PSP>
// where
//     PSP: PresentationStoragePrimitive,
// {
//     storage_id: Option<<PSP::PresentationStorage as PresentationStorage<PSP::Key>>::StorageId>,
//     primitive: PSP,
// }

impl<S> Allocated<S>
where
    S: Storable,
{
    pub(crate) fn storable(&self) -> &S {
        &self.storable
    }
}

struct KeyGeneratorWrapper<KG>(KG)
where
    KG: KeyGenerator;

struct StorableMapWrapper<SM>(SM)
where
    SM: StorableMap;

impl<KG> typemap_rev::TypeMapKey for KeyGeneratorWrapper<KG>
where
    KG: KeyGenerator,
{
    type Value = Self;
}

impl<SM> typemap_rev::TypeMapKey for StorableMapWrapper<SM>
where
    SM: StorableMap,
{
    type Value = Self;
}

trait Expire: Send + Sync {
    fn expire(&mut self);
}

impl<SM> Expire for StorableMapWrapper<SM>
where
    SM: StorableMap,
{
    fn expire(&mut self) {
        self.expire();
    }
}

// pub struct PresentationStorageWrapper<K, PS>(PS)
// where
//     PS: PresentationStorage<K>;

// impl<K, PS> Deref for PresentationStorageWrapper<K, PS>
// where
//     PS: PresentationStorage<K>,
// {
//     type Target = PS;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl<K, PS> DerefMut for PresentationStorageWrapper<K, PS>
// where
//     PS: PresentationStorage<K>,
// {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

impl<S> typemap_rev::IntoBox<dyn Expire> for Allocated<S>
where
    S: Storable,
{
    fn into_box(self) -> Box<dyn Expire> {
        Box::new(self.0)
    }
}

pub struct StorageTypeMap(typemap_rev::TypeMap<dyn Expire>);

impl StorageTypeMap {
    pub fn allocate<S>(&mut self, storable: S) -> Allocated<S>
    where
        S: Storable,
    {
        Allocated {
            key: self
                .0
                .entry::<KeyGeneratorWrapper<S::KeyGenerator>>()
                .or_insert_with(|| KeyGeneratorWrapper(S::KeyGenerator::new()))
                .0
                .generate_key(&storable),
            storable,
        }
    }

    pub fn get_or_insert_with<S, F>(
        &mut self,
        allocated: &Allocated<S>,
        f: F,
    ) -> &<S::StorableMap as StorableMap>::Value
    where
        S: Storable,
        F: FnOnce(&S) -> <S::StorableMap as StorableMap>::Value,
    {
        self.0
            .entry::<StorableMapWrapper<S::StorableMap>>()
            .or_insert_with(|| StorableMapWrapper(S::StorableMap::new()))
            .0
            .entry(allocated.key.clone())
            .get_or_insert(f(&allocated.storable))
    }

    pub fn get<S>(&self, allocated: &Allocated<S>) -> &<S::StorableMap as StorableMap>::Value
    where
        S: Storable,
    {
        self.0
            .entry::<StorableMapWrapper<S::StorableMap>>()
            .or_insert_with(|| StorableMapWrapper(S::StorableMap::new()))
            .0
            .get(&allocated.key)
            .unwrap()
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

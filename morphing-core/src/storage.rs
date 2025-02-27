use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub(crate) trait Storable: 'static + Send + Sync {}

impl<T> Storable for T where T: 'static + Send + Sync {}

// trait SerdeKey: 'static + Clone + Eq + Hash + Send + Sync {}

// impl<T> SerdeKey for T where T: 'static + Clone + Eq + Hash + Send + Sync {}

pub(crate) trait PresentationStorage: Storable {
    type Presentation: Storable;
    type StorageIdInput;
    type StorageId;
    type Target;
    // type PrepareRef<'s>;
    // type RenderRef<'s>;

    fn new() -> Self;
    fn allocate(&mut self, input: Self::StorageIdInput) -> Self::StorageId;
    fn get_mut(&mut self, storage_id: &Self::StorageId) -> &mut Option<Self::Target>;
    // where
    //     F: FnOnce() -> Self::Presentation;
    fn get_ref(&self, storage_id: &Self::StorageId) -> &Option<Self::Target>;
    fn expire(&mut self);
}

pub(crate) struct ReadStorage<P>(Option<Arc<P>>);

impl<P> PresentationStorage for ReadStorage<P>
where
    P: Storable,
{
    type Presentation = P;
    type StorageIdInput = ();
    type StorageId = ();
    type Target = Arc<P>;
    // type PrepareRef<'s> = &'s mut Arc<P>;
    // type RenderRef<'s> = &'s Arc<P>;

    fn new() -> Self {
        Self(None)
    }

    fn allocate(&mut self, _input: Self::StorageIdInput) -> Self::StorageId {
        ()
    }

    fn get_mut(&mut self, _storage_id: &Self::StorageId) -> &mut Option<Self::Target> {
        &mut self.0
    }

    fn get_ref(&self, _storage_id: &Self::StorageId) -> &Option<Self::Target> {
        &self.0
    }

    fn expire(&mut self) {
        self.0.take();
        // std::mem::replace(&mut self.inactive, std::mem::take(&mut self.active));
    }
}

pub(crate) struct ReadWriteStorage<P>(Vec<Option<P>>);

impl<P> PresentationStorage for ReadWriteStorage<P>
where
    P: Storable,
{
    type Presentation = P;
    type StorageIdInput = ();
    type StorageId = usize;
    type Target = P;
    // type PrepareRef<'s> = &'s mut P;
    // type RenderRef<'s> = &'s P;

    fn new() -> Self {
        Self(Vec::new())
    }

    fn allocate(&mut self, _input: Self::StorageIdInput) -> Self::StorageId {
        let storage_id = self.0.len();
        self.0.push(None);
        storage_id
    }

    fn get_mut(&mut self, storage_id: &Self::StorageId) -> &mut Option<Self::Target> {
        self.0.get_mut(*storage_id).unwrap()
    }

    fn get_ref(&self, storage_id: &Self::StorageId) -> &Option<Self::Target> {
        self.0.get(*storage_id).unwrap()
    }

    fn expire(&mut self) {
        self.0.clear();
        // std::mem::replace(&mut self.inactive, std::mem::take(&mut self.active));
    }
}

pub(crate) struct SwapStorage<PS> {
    active: PS,
    inactive: PS,
}

impl<PS> PresentationStorage for SwapStorage<PS>
where
    PS: PresentationStorage,
{
    type Presentation = PS::Presentation;
    type StorageIdInput = PS::StorageIdInput;
    type StorageId = PS::StorageId;
    type Target = PS::Target;
    // type PrepareRef<'s> = PS::PrepareRef<'s>;
    // type RenderRef<'s> = PS::RenderRef<'s>;

    fn new() -> Self {
        Self {
            active: PS::new(),
            inactive: PS::new(),
        }
    }

    fn allocate(&mut self, input: Self::StorageIdInput) -> Self::StorageId {
        self.active.allocate(input)
    }

    fn get_mut(&mut self, storage_id: &Self::StorageId) -> &mut Option<Self::Target> {
        self.active.get_mut(storage_id)
    }

    fn get_ref(&self, storage_id: &Self::StorageId) -> &Option<Self::Target> {
        self.active.get_ref(storage_id)
    }

    fn expire(&mut self) {
        std::mem::replace(
            &mut self.inactive,
            std::mem::replace(&mut self.active, PS::new()),
        );
    }
}

pub(crate) struct MapStorage<K, PS>(HashMap<K, PS>);

impl<K, PS> PresentationStorage for MapStorage<K, PS>
where
    K: 'static + Clone + Eq + Hash + Send + Sync,
    PS: PresentationStorage,
{
    type Presentation = PS::Presentation;
    type StorageIdInput = (K, PS::StorageIdInput);
    type StorageId = (K, PS::StorageId);
    type Target = PS::Target;
    // type PrepareRef<'s> = PS::PrepareRef<'s>;
    // type RenderRef<'s> = PS::RenderRef<'s>;

    fn new() -> Self {
        Self(HashMap::new())
    }

    fn allocate(&mut self, input: Self::StorageIdInput) -> Self::StorageId {
        (
            input.0.clone(),
            self.0
                .entry(input.0)
                .or_insert_with(PS::new)
                .allocate(input.1),
        )
    }

    fn get_mut(&mut self, storage_id: &Self::StorageId) -> &mut Option<Self::Target> {
        self.0
            .get_mut(&storage_id.0)
            .unwrap()
            .get_mut(&storage_id.1)
    }

    fn get_ref(&self, storage_id: &Self::StorageId) -> &Option<Self::Target> {
        self.0.get(&storage_id.0).unwrap().get_ref(&storage_id.1)
    }

    fn expire(&mut self) {
        self.0.values_mut().for_each(PS::expire);
    }
}

pub(crate) trait StorablePrimitive: 'static {
    type PresentationStorage: PresentationStorage;

    fn storage_id_input(
        &self,
    ) -> <Self::PresentationStorage as PresentationStorage>::StorageIdInput;
}

pub(crate) struct Allocated<SP>
where
    SP: StorablePrimitive,
{
    storage_id: <SP::PresentationStorage as PresentationStorage>::StorageId,
    storable_primitive: SP,
}

impl<SP> Allocated<SP>
where
    SP: StorablePrimitive,
{
    pub(crate) fn storable_primitive(&self) -> &SP {
        &self.storable_primitive
    }
}

impl<SP> typemap_rev::TypeMapKey for Allocated<SP>
where
    SP: StorablePrimitive,
{
    type Value = PresentationStorageWrapper<SP::PresentationStorage>;
}

trait Expire: Send + Sync {
    fn expire(&mut self);
}

impl<PS> Expire for PS
where
    PS: PresentationStorage,
{
    fn expire(&mut self) {
        self.expire();
    }
}

struct PresentationStorageWrapper<PS>(PS);

impl<PS> Deref for PresentationStorageWrapper<PS> {
    type Target = PS;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<PS> DerefMut for PresentationStorageWrapper<PS> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<PS> typemap_rev::IntoBox<dyn Expire> for PresentationStorageWrapper<PS>
where
    PS: PresentationStorage,
{
    fn into_box(self) -> Box<dyn Expire> {
        Box::new(self.0)
    }
}

pub(crate) struct StorageTypeMap(typemap_rev::TypeMap<dyn Expire>);

impl StorageTypeMap {
    pub(crate) fn allocate<SP>(&mut self, storable_primitive: SP) -> Allocated<SP>
    where
        SP: StorablePrimitive,
    {
        let presentation_storage = self
            .0
            .entry::<Allocated<SP>>()
            .or_insert_with(|| PresentationStorageWrapper(SP::PresentationStorage::new()));
        Allocated {
            storage_id: presentation_storage.allocate(storable_primitive.storage_id_input()),
            storable_primitive,
        }
    }

    pub(crate) fn get_mut<SP>(
        &mut self,
        allocated: &Allocated<SP>,
    ) -> &mut Option<<SP::PresentationStorage as PresentationStorage>::Target>
    where
        SP: StorablePrimitive,
    {
        self.0
            .get_mut::<Allocated<SP>>()
            .unwrap()
            .get_mut(&allocated.storage_id)
    }

    pub(crate) fn get_ref<SP>(
        &self,
        allocated: &Allocated<SP>,
    ) -> &Option<<SP::PresentationStorage as PresentationStorage>::Target>
    where
        SP: StorablePrimitive,
    {
        self.0
            .get::<Allocated<SP>>()
            .unwrap()
            .get_ref(&allocated.storage_id)
    }

    pub(crate) fn expire(&mut self) {
        self.0 = std::mem::replace(&mut self.0, typemap_rev::TypeMap::custom())
            .into_iter()
            .map(|(type_id, mut presentation_storage)| {
                presentation_storage.expire();
                (type_id, presentation_storage)
            })
            .collect();
    }
}

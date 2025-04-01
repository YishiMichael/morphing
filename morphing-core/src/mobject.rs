use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

use super::storable::SlotKeyGeneratorTypeMap;
use super::storable::StorageTypeMap;
use super::timer::Clock;
use super::timer::ClockSpan;
use super::timer::Rate;
use super::timer::Time;
use super::timer::TimeMetric;

pub trait Mobject:
    'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize
{
    type Structure: Structure;
    type Entity;
}

pub trait Structure:
    'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize
{
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Entity<M, S> {
    mobject: M,
    structure: S,
}

// pub trait EntityTrait:
//     'static
//     + Debug
//     + Send
//     + Sync
//     + for<'de> serde::Deserialize<'de>
//     + serde::Serialize
//     + Deref<Target = Self::Mobject>
// {
//     type Mobject;
//     type Structure;

//     fn mobject(entity: &Self) -> &Self::Mobject;
//     fn structure(entity: &Self) -> &Self::Structure;
// }

impl<M, S> Deref for Entity<M, S> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.mobject
    }
}

// trait Refresh<E>: Send + Sync {
//     fn refresh(&self, clock: Clock, clock_span: ClockSpan, entity: &mut E);
// }

// struct ContinuousUpdateRefresh<TM, R, CU> {
//     time_metric: TM,
//     rate: R,
//     continuous_update: CU,
// }

// impl<E, TM, R, CU> Refresh<E> for ContinuousUpdateRefresh<TM, R, CU>
// where
//     TM: TimeMetric,
//     R: Rate<TM>,
//     CU: ContinuousUpdate<TM, E>,
// {
//     fn refresh(&self, clock: Clock, clock_span: ClockSpan, entity: &mut E) {
//         self.continuous_update.continuous_update(
//             self.rate
//                 .eval(self.time_metric.localize_from_clock(clock, clock_span)),
//             &mut entity,
//         );
//     }
// }

// impl<M, S> EntityTrait for Entity<M, S>
// where
//     M: Mobject,
//     S: Structure,
// {
//     type Mobject = M;
//     type Structure = S;

//     fn mobject(entity: &Self) -> &Self::Mobject {
//         &entity.mobject
//     }

//     fn structure(entity: &Self) -> &Self::Structure {
//         &entity.structure
//     }
// }

// pub trait SpatialMobject: Mobject {}

// pub trait TemporalMobject: Mobject {}

// pub trait Timeline<TM>: Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize
// where
//     TM: TimeMetric,
// {
//     type Observatory;

//     // fn init_presentation(
//     //     &self,
//     //     mobject: M,
//     //     device: &wgpu::Device,
//     //     queue: &wgpu::Queue,
//     //     format: wgpu::TextureFormat,
//     // ) -> Self::Presentation;
//     fn observe(&self, time: Time) -> Self::Observatory;
// }

// pub enum MaybeArc<T> {
//     Borrowed(Arc<T>),
//     Owned(T),
// }

// impl<T> Deref for MaybeArc<T> {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         match self {
//             Self::Borrowed(value) => &value,
//             Self::Owned(value) => &value,
//         }
//     }
// }

// pub trait MaybeArc<T> {
//     fn borrow(&self) -> &T;
//     fn into_owned(self) -> T;
// }

// impl<T> MaybeArc<T> for T {
//     fn borrow(&self) -> &T {
//         self
//     }

//     fn into_owned(self) -> T {
//         self
//     }
// }

// impl<T> MaybeArc<T> for Arc<T>
// where
//     T: Clone,
// {
//     fn borrow(&self) -> &T {
//         self
//     }

//     fn into_owned(self) -> T {
//         Arc::unwrap_or_clone(self)
//     }
// }

// pub trait MobjectObservatory {
//     type Mobject;

//     fn mobject(&self) -> &Self::Mobject;
// }

// pub trait StructureObservatory {
//     type Structure;

//     fn structure(&self) -> &Self::Structure;
// }

// pub trait ObservatoryOwned: Observatory {
//     fn own(&self) -> Self::Entity;
// }

// pub trait Observatory:
//     'static + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize
// {
//     // type Initial;
//     type Target;

//     // fn initial(&self) -> Self::Initial;
//     // fn update(self, initial: Self::Initial) -> &Self::Target;
//     fn borrow(&self) -> &Self::Target;
// }

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// pub struct StaticObservatory<O>(Arc<O>);

// impl<O> Clone for StaticObservatory<O> {
//     fn clone(&self) -> Self {
//         Self(self.0.clone())
//     }
// }

// impl<O> Observatory for StaticObservatory<O>
// where
//     O: 'static + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
// {
//     type Target = O;

//     fn borrow(&self) -> &Self::Target {
//         &self.0
//     }
// }

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// pub struct DynamicObservatory<O>(O);

// impl<O> Observatory for DynamicObservatory<O>
// where
//     O: 'static + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
// {
//     type Target = O;

//     fn borrow(&self) -> &Self::Target {
//         &self.0
//     }
// }

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// pub struct GenericObservatory<O>(O);

// impl<E> MobjectObservatory for StaticObservatory<E>
// where
//     E: Entity,
// {
//     type Mobject = E::Mobject;

//     fn mobject(&self) -> &Self::Mobject {
//         self.0.mobject()
//     }
// }

// impl<E> StructureObservatory for StaticObservatory<E>
// where
//     E: Entity,
// {
//     type Structure = E::Structure;

//     fn structure(&self) -> &Self::Structure {
//         self.0.structure()
//     }
// }

// impl<E> MobjectObservatory for DynamicObservatory<E>
// where
//     E: Entity,
// {
//     type Mobject = E::Mobject;

//     fn mobject(&self) -> &Self::Mobject {
//         self.0.mobject()
//     }
// }

// impl<E> StructureObservatory for DynamicObservatory<E>
// where
//     E: Entity,
// {
//     type Structure = E::Structure;

//     fn structure(&self) -> &Self::Structure {
//         self.0.structure()
//     }
// }

// impl<M, S> MobjectObservatory for Entity<Arc<M>, S> {
//     type Mobject = M;

//     fn mobject(&self) -> &Self::Mobject {
//         self.0.mobject
//     }
// }

// impl<M, S> StructureObservatory for Entity<Arc<M>, S>
// where
//     S: StructureObservatory,
// {
//     type Structure = S::Structure;

//     fn structure(&self) -> &Self::Structure {
//         self.0.structure
//     }
// }

pub trait Worldline: Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize {
    type Observatory;

    fn observe(&self, clock: Clock, clock_span: ClockSpan) -> Self::Observatory;
}

// pub trait SerdeWorldline:
//     Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize + Worldline
// {
// }

// pub trait StructureWorldline:
//     Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize
// {
//     type Observatory;

//     fn observe(&self, clock: Clock, clock_span: ClockSpan) -> Self::Observatory;
// }

// pub trait Prepare<E>: Sized {
//     // type Presentation;

//     fn prepare_new(
//         entity: &E,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) -> Self;
//     fn prepare(
//         &mut self,
//         entity: &E,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) {
//         *self = Self::prepare_new(entity, device, queue, format);
//     } // -> &Self::Output; // input presentation write-only
// }

pub trait Render {
    fn render(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView);
}

// impl<W> SerdeWorldline for W where W: for<'de> serde::Deserialize<'de> + serde::Serialize + Worldline
// {}

// pub trait PreparableTimeline<M>: Timeline<M> {
//     type Presentation: Prepare<M>;

//     fn initialize(
//         &self,
//         mobject: &M,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) -> Self::Presentation;
//     fn prepare(
//         &self,
//         presentation: &mut Self::Presentation,
//         mobject: &M,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     );
// }

// pub trait RenderableTimeline<M>: PreparableTimeline<M>
// where
//     Self::Observant: Prepare<M> + Render,
// {
// }

// pub trait BufferSliceAllocatedTimeline {
//     fn prepare(
//         &self,
//         time: Time,
//         time_interval: Range<f32>,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) -> PresentationKey<Self::Presentation>;
//     fn prepare(
//         &self,
//         time: Time,
//         time_interval: Range<f32>,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) -> PresentationKey<Self::Presentation>;
// }

// #[derive(serde::Deserialize, serde::Serialize)]
// pub struct Schedule<MI>
// where
//     MI: 'static,
// {
//     time_interval: std::ops::Range<Time>,
//     Timeline: serde_traitobject::Box<dyn Timeline<Observatory = MI>>,
// }

// impl<M> Schedule<M> {
//     fn observe(&self, time: Time) -> Option<M> {
//         self.time_interval.contains(&time).then(|| {
//             self.Timeline
//                 .observe(time, self.time_interval.clone().into())
//         })
//     }
// }

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct StaticWorldline<M, S> {
    // time_rate: TR,
    // entity: Arc<M::Entity>,
    entity: Entity<Arc<M>, Arc<S>>,
}

impl<M, S> Worldline for StaticWorldline<M, S>
where
    M: Mobject<Entity = Entity<M, S>>,
    S: Structure,
{
    // type Transformed<TR0> = StaticWorldline<RateChain<TR0, TR::TimeMetric, TR>, M>
    // where
    //     TR0: TimeRate;
    type Observatory = Entity<M, S>;

    // fn transform<TR0>(
    //     self,
    //     rate_transform: RateTransform<TR0, TR::TimeMetric>,
    // ) -> Self::Transformed<TR0>
    // where
    //     TR0: TimeRate,
    // {
    //     StaticWorldline {
    //         time_rate: RateChain {
    //             rate_transform,
    //             time_rate: self.time_rate,
    //         },
    //         entity: self.entity,
    //     }
    // }

    fn observe(&self, _clock: Clock, _clock_span: ClockSpan) -> Self::Observatory {
        Entity {
            mobject: self.entity.mobject.as_ref().clone(),
            structure: self.entity.structure.as_ref().clone(),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct DynamicWorldline<M, S, TM, R, CU> {
    entity: Entity<M, S>,
    time_metric: TM,
    rate: R,
    continuous_update: CU,
}

impl<M, S, TM, R, CU> Worldline for DynamicWorldline<M, S, TM, R, CU>
where
    M: Mobject<Entity = Entity<M, S>>,
    S: Structure,
    TM: TimeMetric,
    R: Rate<TM>,
    CU: ContinuousUpdate<TM, M::Entity>,
{
    // type Transformed<TR0> = DynamicWorldline<RateChain<TR0, TR::TimeMetric, TR>, M, CU>
    // where
    //     TR0: TimeRate;
    type Observatory = Entity<M, S>;

    // fn transform<TR0>(
    //     self,
    //     rate_transform: RateTransform<TR0, TR::TimeMetric>,
    // ) -> Self::Transformed<TR0>
    // where
    //     TR0: TimeRate,
    // {
    //     DynamicWorldline {
    //         time_rate: RateChain {
    //             rate_transform,
    //             time_rate: self.time_rate,
    //         },
    //         entity: self.entity,
    //         continuous_update: self.continuous_update,
    //     }
    // }

    fn observe(&self, clock: Clock, clock_span: ClockSpan) -> Self::Observatory {
        let mut entity = Entity {
            mobject: self.entity.mobject.clone(),
            structure: self.entity.structure.clone(),
        };
        self.continuous_update.continuous_update(
            self.rate
                .eval(self.time_metric.localize_from_clock(clock, clock_span)),
            &mut entity,
        );
        entity
        // self.time_rate.eval_time(time).map(|metric_time| {
        //     self.continuous_update
        //         .continuous_update(*metric_time, self.entity.clone())
        // })
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct GenericWorldline<M, S> {
    entity: Entity<Arc<M>, S>,
}

impl<M, S> GenericWorldline<M, S> {
    pub fn map_structure_ref<F, FO>(&self, f: F) -> GenericWorldline<M, FO>
    where
        F: FnOnce(&S) -> FO,
    {
        GenericWorldline {
            entity: Entity {
                mobject: self.entity.mobject.clone(),
                structure: f(&self.entity.structure),
            },
        }
    }

    pub fn map_structure<F, FO>(self, f: F) -> GenericWorldline<M, FO>
    where
        F: FnOnce(S) -> FO,
    {
        GenericWorldline {
            entity: Entity {
                mobject: self.entity.mobject,
                structure: f(self.entity.structure),
            },
        }
    }
}

// impl<M, S> Worldline for GenericWorldline<M, S>
// where
//     M: Mobject<Entity = Entity<M, S::Observatory>>,
//     S: StructureWorldline,
// {
//     type Observatory = Entity<M, S::Observatory>;

//     fn observe(&self, clock: Clock, clock_span: ClockSpan) -> Self::Observatory {
//         Entity {
//             mobject: self.entity.mobject.as_ref().clone(),
//             structure: self.entity.structure.observe(clock, clock_span),
//         }
//     }
// }

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// pub struct GroupWorldline(Vec<Vec<(ClockSpan, serde_traitobject::Arc<dyn Worldline>)>>);

// pub enum PresentationKey<MP>
// where
//     MP: 'static + Send + Sync,
// {
//     Static(
//         Arc<StorageKey<
//             (TypeId, Box<dyn DynKey>),
//             <<SwapSlot<SharableSlot<MP>> as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
//         >>,
//     ),
//     Dynamic(
//         Arc<StorageKey<
//             (TypeId, Box<dyn DynKey>, Box<dyn DynKey>),
//             <<SwapSlot<VecSlot<MP>> as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
//         >>,
//     ),
// }

// impl<MP> PresentationKey<MP>
// where
//     MP: 'static + Send + Sync,
// {
//     pub fn read<'mp>(&self, storage_type_map: &'mp StorageTypeMap) -> &'mp MP {
//         match self {
//             Self::Static(key) => storage_type_map
//                 .get::<_, SwapSlot<SharableSlot<MP>>>(key)
//                 .as_ref()
//                 .unwrap(),
//             Self::Dynamic(key) => storage_type_map
//                 .get::<_, SwapSlot<VecSlot<MP>>>(key)
//                 .as_ref()
//                 .unwrap(),
//         }
//     }
// }

// demo code

// atom

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject0Structure {}

impl Structure for MyMobject0Structure {}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject0 {
    demo0: f32,
}

impl Mobject for MyMobject0 {
    type Structure = MyMobject0Structure;
    type Entity = Entity<MyMobject0, MyMobject0Structure>;
}

pub trait MyMobject0Trait {}

impl MyMobject0Trait for Entity<MyMobject0, MyMobject0Structure> {}

impl Worldline for GenericWorldline<MyMobject0, MyMobject0Structure> {
    type Observatory = GenericWorldline<MyMobject0, MyMobject0Structure>;

    #[allow(unused_variables)]
    fn observe(&self, clock: Clock, clock_span: ClockSpan) -> Self::Observatory {
        self.map_structure_ref(|structure| MyMobject0Structure {})
    }
}

// atom structured

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject1Structure<
    MA = Entity<MyMobject0, MyMobject0Structure>,
    MB = Entity<MyMobject0, MyMobject0Structure>,
> {
    ma: MA,
    mb: MB,
}

impl Structure for MyMobject1Structure {}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject1 {
    demo1: f32,
}

impl Mobject for MyMobject1 {
    type Structure = MyMobject1Structure;
    type Entity = Entity<MyMobject1, MyMobject1Structure>;
}

pub trait MyMobject1Trait {
    type MA;
    type MB;

    fn ma(&self) -> &Self::MA;
    fn mb(&self) -> &Self::MB;
}

impl<MA, MB> MyMobject1Trait for Entity<MyMobject1, MyMobject1Structure<MA, MB>> {
    type MA = MA;
    type MB = MB;

    fn ma(&self) -> &Self::MA {
        &self.structure.ma
    }

    fn mb(&self) -> &Self::MB {
        &self.structure.mb
    }
}

impl<MA, MB> Worldline for GenericWorldline<MyMobject1, MyMobject1Structure<MA, MB>>
where
    MA: Worldline<Observatory = Entity<MyMobject0, MyMobject0Structure>>,
    MB: Worldline<Observatory = Entity<MyMobject0, MyMobject0Structure>>,
{
    type Observatory = GenericWorldline<MyMobject1, MyMobject1Structure>;

    fn observe(&self, clock: Clock, clock_span: ClockSpan) -> Self::Observatory {
        self.map_structure_ref(|structure| MyMobject1Structure {
            ma: structure.ma.observe(clock, clock_span),
            mb: structure.mb.observe(clock, clock_span),
        })
    }
}

// prepare buffer slice

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject2Structure<
    MA = Entity<MyMobject1, MyMobject1Structure>,
    MB = Entity<MyMobject1, MyMobject1Structure>,
> {
    ma: MA,
    mb: MB,
}

impl Structure for MyMobject2Structure {}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject2 {
    demo2: f32,
}

impl Mobject for MyMobject2 {
    type Structure = MyMobject2Structure;
    type Entity = Entity<MyMobject2, MyMobject2Structure>;
}

pub trait MyMobject2Trait {
    type MA;
    type MB;

    fn ma(&self) -> &Self::MA;
    fn mb(&self) -> &Self::MB;
}

impl<MA, MB> MyMobject2Trait for Entity<MyMobject2, MyMobject2Structure<MA, MB>> {
    type MA = MA;
    type MB = MB;

    fn ma(&self) -> &Self::MA {
        &self.structure.ma
    }

    fn mb(&self) -> &Self::MB {
        &self.structure.mb
    }
}

impl<MA, MB> Worldline for GenericWorldline<MyMobject2, MyMobject2Structure<MA, MB>>
where
    MA: Worldline<Observatory = Entity<MyMobject1, MyMobject1Structure>>,
    MB: Worldline<Observatory = Entity<MyMobject1, MyMobject1Structure>>,
{
    type Observatory = GenericWorldline<MyMobject2, MyMobject2Structure>;

    fn observe(&self, clock: Clock, clock_span: ClockSpan) -> Self::Observatory {
        self.map_structure_ref(|structure| MyMobject2Structure {
            ma: structure.ma.observe(clock, clock_span),
            mb: structure.mb.observe(clock, clock_span),
        })
    }
}

pub trait BufferSliceAllocate {
    type BufferSliceAllocated;

    fn buffer_slice_allocate(self, offset: wgpu::BufferAddress) -> Self::BufferSliceAllocated;
}

pub trait BufferSlicePrepare {
    fn buffer_slice_prepare(
        &self,
        clock: Clock,
        clock_span: ClockSpan,
        buffer: &wgpu::Buffer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    );
}

impl<W> BufferSliceAllocate for W
where
    W: Worldline<Observatory = Entity<MyMobject2, MyMobject2Structure>>,
{
    type BufferSliceAllocated = (W, wgpu::BufferAddress);

    fn buffer_slice_allocate(self, offset: wgpu::BufferAddress) -> Self::BufferSliceAllocated {
        (self, offset)
    }
}

// impl BufferSliceAllocate for StaticWorldline<MyMobject2, MyMobject2Structure> {
//     type BufferSliceAllocated = StaticWorldline<MyMobject2, MyMobject2Structure>;

//     fn buffer_slice_allocate(self, _offset: wgpu::BufferAddress) -> Self::BufferSliceAllocated {
//         self
//     }
// }

// impl BufferSliceAllocate for DynamicWorldline<MyMobject2, MyMobject2Structure> {
//     type BufferSliceAllocated = (
//         DynamicWorldline<MyMobject2, MyMobject2Structure>,
//         wgpu::BufferAddress,
//     );

//     fn buffer_slice_allocate(self, offset: wgpu::BufferAddress) -> Self::BufferSliceAllocated {
//         (self, offset)
//     }
// }

// impl<MA, MB> BufferSliceAllocate for GenericWorldline<MyMobject2, MyMobject2Structure<MA, MB>> {
//     type BufferSliceAllocated = (
//         GenericWorldline<MyMobject2, MyMobject2Structure<MA, MB>>,
//         wgpu::BufferAddress,
//     );

//     fn buffer_slice_allocate(self, offset: wgpu::BufferAddress) -> Self::BufferSliceAllocated {
//         (self, offset)
//     }
// }

impl BufferSlicePrepare
    for <StaticWorldline<MyMobject2, MyMobject2Structure> as BufferSliceAllocate>::BufferSliceAllocated
{
    fn buffer_slice_prepare(
        &self,
        _clock: Clock,
        _clock_span: ClockSpan,
        _buffer: &wgpu::Buffer,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) {}
}

impl BufferSlicePrepare
    for <DynamicWorldline<MyMobject2, MyMobject2Structure> as BufferSliceAllocate>::BufferSliceAllocated
{
    fn buffer_slice_prepare(
        &self,
        clock: Clock,
        clock_span: ClockSpan,
        _buffer: &wgpu::Buffer,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) {
        let (worldline, _offset) = self;
        let entity = worldline.observe(clock, clock_span);
        let _bytes = [
            entity.demo2,
            entity.ma().demo1 + entity.ma().ma().demo0 + entity.ma().mb().demo0,
            entity.mb().demo1 + entity.mb().ma().demo0 + entity.mb().mb().demo0,
        ];
        // Pretend we write bytes into `buffer` at `offset`
    }
}

// TODO: WorldlinePrepare

// impl Prepare<Entity<MyMobject2, MyMobject2Structure>> for [f32; 3] {
//     fn prepare_new(
//         entity: &Entity<MyMobject2, MyMobject2Structure>,
//         _device: &wgpu::Device,
//         _queue: &wgpu::Queue,
//         _format: wgpu::TextureFormat,
//     ) -> Self {
//     }
// }

// prepare buffer slice structured

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject3Structure<
    MA = Entity<MyMobject2, MyMobject2Structure>,
    MB = Entity<MyMobject2, MyMobject2Structure>,
> {
    ma: MA,
    mb: MB,
}

impl Structure for MyMobject3Structure {}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject3 {
    demo3: f32,
}

impl Mobject for MyMobject3 {
    type Structure = MyMobject3Structure;
    type Entity = Entity<MyMobject3, MyMobject3Structure>;
}

pub trait MyMobject3Trait {
    type MA;
    type MB;

    fn ma(&self) -> &Self::MA;
    fn mb(&self) -> &Self::MB;
}

impl<MA, MB> MyMobject3Trait for Entity<MyMobject3, MyMobject3Structure<MA, MB>> {
    type MA = MA;
    type MB = MB;

    fn ma(&self) -> &Self::MA {
        &self.structure.ma
    }

    fn mb(&self) -> &Self::MB {
        &self.structure.mb
    }
}

impl<MA, MB> Worldline for GenericWorldline<MyMobject3, MyMobject3Structure<MA, MB>>
where
    MA: Worldline<Observatory = Entity<MyMobject2, MyMobject2Structure>>,
    MB: Worldline<Observatory = Entity<MyMobject2, MyMobject2Structure>>,
{
    type Observatory = GenericWorldline<MyMobject3, MyMobject3Structure>;

    fn observe(&self, clock: Clock, clock_span: ClockSpan) -> Self::Observatory {
        self.map_structure_ref(|structure| MyMobject3Structure {
            ma: structure.ma.observe(clock, clock_span),
            mb: structure.mb.observe(clock, clock_span),
        })
    }
}

impl BufferSliceAllocate for Entity<MyMobject3, MyMobject3Structure> {
    type BufferSliceAllocated = Entity<
        MyMobject3,
        MyMobject3Structure<
            <Entity<MyMobject2, MyMobject2Structure> as BufferSliceAllocate>::BufferSliceAllocated,
            <Entity<MyMobject2, MyMobject2Structure> as BufferSliceAllocate>::BufferSliceAllocated,
        >,
    >;

    fn buffer_slice_allocate(self, offset: wgpu::BufferAddress) -> Self::BufferSliceAllocated {
        Entity {
            mobject: self.mobject,
            structure: MyMobject3Structure {
                ma: self.structure.ma.buffer_slice_allocate(offset + 0),
                mb: self.structure.mb.buffer_slice_allocate(offset + 4),
            },
        }
    }
}

impl BufferSlicePrepare
    for <Entity<MyMobject3, MyMobject3Structure> as BufferSliceAllocate>::BufferSliceAllocated
{
    fn buffer_slice_prepare(
        &self,
        buffer: &wgpu::Buffer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) {
        self.structure
            .ma
            .buffer_slice_prepare(buffer, device, queue, format);
        self.structure
            .mb
            .buffer_slice_prepare(buffer, device, queue, format);
    }
}

// buffer

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject4Structure<
    MA = Entity<MyMobject3, MyMobject3Structure>,
    MB = Entity<MyMobject3, MyMobject3Structure>,
> {
    ma: MA,
    mb: MB,
}

impl Structure for MyMobject4Structure {}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject4 {
    demo4: f32,
}

impl Mobject for MyMobject4 {
    type Structure = MyMobject4Structure;
    type Entity = Entity<MyMobject4, MyMobject4Structure>;
}

pub trait MyMobject4Trait {
    type MA;
    type MB;

    fn ma(&self) -> &Self::MA;
    fn mb(&self) -> &Self::MB;
}

impl<MA, MB> MyMobject4Trait for Entity<MyMobject4, MyMobject4Structure<MA, MB>> {
    type MA = MA;
    type MB = MB;

    fn ma(&self) -> &Self::MA {
        &self.structure.ma
    }

    fn mb(&self) -> &Self::MB {
        &self.structure.mb
    }
}

impl<MA, MB> Worldline for GenericWorldline<MyMobject4, MyMobject4Structure<MA, MB>>
where
    MA: Worldline<Observatory = Entity<MyMobject3, MyMobject3Structure>>,
    MB: Worldline<Observatory = Entity<MyMobject3, MyMobject3Structure>>,
{
    type Observatory = GenericWorldline<MyMobject4, MyMobject4Structure>;

    fn observe(&self, clock: Clock, clock_span: ClockSpan) -> Self::Observatory {
        self.map_structure_ref(|structure| MyMobject4Structure {
            ma: structure.ma.observe(clock, clock_span),
            mb: structure.mb.observe(clock, clock_span),
        })
    }
}

impl BufferSliceAllocate for Entity<MyMobject4, MyMobject4Structure> {
    type BufferSliceAllocated = Entity<
        MyMobject4,
        MyMobject4Structure<
            <Entity<MyMobject3, MyMobject3Structure> as BufferSliceAllocate>::BufferSliceAllocated,
            <Entity<MyMobject3, MyMobject3Structure> as BufferSliceAllocate>::BufferSliceAllocated,
        >,
    >;

    fn buffer_slice_allocate(self, offset: wgpu::BufferAddress) -> Self::BufferSliceAllocated {
        Entity {
            mobject: self.mobject,
            structure: MyMobject4Structure {
                ma: self.structure.ma.buffer_slice_allocate(offset + 0),
                mb: self.structure.mb.buffer_slice_allocate(offset + 8),
            },
        }
    }
}

impl BufferSlicePrepare
    for <Entity<MyMobject4, MyMobject4Structure> as BufferSliceAllocate>::BufferSliceAllocated
{
    fn buffer_slice_prepare(
        &self,
        buffer: &wgpu::Buffer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) {
        self.structure
            .ma
            .buffer_slice_prepare(buffer, device, queue, format);
        self.structure
            .mb
            .buffer_slice_prepare(buffer, device, queue, format);
    }
}

pub trait BufferAllocate {
    type BufferAllocated;

    fn buffer_allocate(
        self,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::BufferAllocated;
}

impl BufferAllocate for Entity<MyMobject4, MyMobject4Structure> {
    type BufferAllocated = (
        <Entity<MyMobject4, MyMobject4Structure> as BufferSliceAllocate>::BufferSliceAllocated,
        BufferAllocationKey,
    );

    fn buffer_allocate(
        self,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::BufferAllocated {
        // allocate a key in `slot_key_generator_type_map`
        // let key = slot_key_generator_type_map.allocate(storable)
        let entity = self.buffer_slice_allocate(0);
        (entity, key)
    }
}

pub trait BufferPrepare {
    fn buffer_prepare(
        &self,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    );
}

impl BufferPrepare
    for <Entity<MyMobject4, MyMobject4Structure> as BufferAllocate>::BufferAllocated
{
    fn buffer_prepare(
        &self,
        _storage_type_map: &mut StorageTypeMap,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) {
        let (_entity, _allocation_key) = self;
        // Pretend we create an buffer at `storage_type_map[allocation_key]`, initialized using `entity`
        // Then, prepare the buffer via
        // entity.buffer_slice_prepare(buffer, device, queue, format);
    }
}

// buffer structured

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject5Structure<
    MA = Entity<MyMobject4, MyMobject4Structure>,
    MB = Entity<MyMobject4, MyMobject4Structure>,
> {
    ma: MA,
    mb: MB,
}

impl Structure for MyMobject5Structure {}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject5 {
    demo5: f32,
}

impl Mobject for MyMobject5 {
    type Structure = MyMobject5Structure;
    type Entity = Entity<MyMobject5, MyMobject5Structure>;
}

pub trait MyMobject5Trait {
    type MA;
    type MB;

    fn ma(&self) -> &Self::MA;
    fn mb(&self) -> &Self::MB;
}

impl<MA, MB> MyMobject5Trait for Entity<MyMobject5, MyMobject5Structure<MA, MB>> {
    type MA = MA;
    type MB = MB;

    fn ma(&self) -> &Self::MA {
        &self.structure.ma
    }

    fn mb(&self) -> &Self::MB {
        &self.structure.mb
    }
}

impl<MA, MB> Worldline for GenericWorldline<MyMobject5, MyMobject5Structure<MA, MB>>
where
    MA: Worldline<Observatory = Entity<MyMobject4, MyMobject4Structure>>,
    MB: Worldline<Observatory = Entity<MyMobject4, MyMobject4Structure>>,
{
    type Observatory = GenericWorldline<MyMobject5, MyMobject5Structure>;

    fn observe(&self, clock: Clock, clock_span: ClockSpan) -> Self::Observatory {
        self.map_structure_ref(|structure| MyMobject5Structure {
            ma: structure.ma.observe(clock, clock_span),
            mb: structure.mb.observe(clock, clock_span),
        })
    }
}

impl BufferAllocate for Entity<MyMobject5, MyMobject5Structure> {
    type BufferAllocated = Entity<
        MyMobject5,
        MyMobject5Structure<
            <Entity<MyMobject4, MyMobject4Structure> as BufferAllocate>::BufferAllocated,
            <Entity<MyMobject4, MyMobject4Structure> as BufferAllocate>::BufferAllocated,
        >,
    >;

    fn buffer_allocate(
        self,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::BufferAllocated {
        Entity {
            mobject: self.mobject,
            structure: MyMobject5Structure {
                ma: self
                    .structure
                    .ma
                    .buffer_allocate(slot_key_generator_type_map),
                mb: self
                    .structure
                    .mb
                    .buffer_allocate(slot_key_generator_type_map),
            },
        }
    }
}

impl BufferPrepare
    for <Entity<MyMobject5, MyMobject5Structure> as BufferAllocate>::BufferAllocated
{
    fn buffer_prepare(
        &self,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) {
        self.structure
            .ma
            .buffer_prepare(storage_type_map, device, queue, format);
        self.structure
            .mb
            .buffer_prepare(storage_type_map, device, queue, format);
    }
}

// we still have one more level of bind group, but similar to buffer and is omitted here

// render

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject6Structure<
    MA = Entity<MyMobject5, MyMobject5Structure>,
    MB = Entity<MyMobject5, MyMobject5Structure>,
> {
    ma: MA,
    mb: MB,
}

impl Structure for MyMobject6Structure {}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject6 {
    demo6: f32,
}

impl Mobject for MyMobject6 {
    type Structure = MyMobject6Structure;
    type Entity = Entity<MyMobject6, MyMobject6Structure>;
}

pub trait MyMobject6Trait {
    type MA;
    type MB;

    fn ma(&self) -> &Self::MA;
    fn mb(&self) -> &Self::MB;
}

impl<MA, MB> MyMobject6Trait for Entity<MyMobject6, MyMobject6Structure<MA, MB>> {
    type MA = MA;
    type MB = MB;

    fn ma(&self) -> &Self::MA {
        &self.structure.ma
    }

    fn mb(&self) -> &Self::MB {
        &self.structure.mb
    }
}

impl<MA, MB> Worldline for GenericWorldline<MyMobject6, MyMobject6Structure<MA, MB>>
where
    MA: Worldline<Observatory = Entity<MyMobject5, MyMobject5Structure>>,
    MB: Worldline<Observatory = Entity<MyMobject5, MyMobject5Structure>>,
{
    type Observatory = GenericWorldline<MyMobject6, MyMobject6Structure>;

    fn observe(&self, clock: Clock, clock_span: ClockSpan) -> Self::Observatory {
        self.map_structure_ref(|structure| MyMobject6Structure {
            ma: structure.ma.observe(clock, clock_span),
            mb: structure.mb.observe(clock, clock_span),
        })
    }
}

// render structured

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject7Structure<
    MA = Entity<MyMobject6, MyMobject6Structure>,
    MB = Entity<MyMobject6, MyMobject6Structure>,
> {
    ma: MA,
    mb: MB,
}

impl Structure for MyMobject7Structure {}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject7 {
    demo7: f32,
}

impl Mobject for MyMobject7 {
    type Structure = MyMobject7Structure;
    type Entity = Entity<MyMobject7, MyMobject7Structure>;
}

pub trait MyMobject7Trait {
    type MA;
    type MB;

    fn ma(&self) -> &Self::MA;
    fn mb(&self) -> &Self::MB;
}

impl<MA, MB> MyMobject7Trait for Entity<MyMobject7, MyMobject7Structure<MA, MB>> {
    type MA = MA;
    type MB = MB;

    fn ma(&self) -> &Self::MA {
        &self.structure.ma
    }

    fn mb(&self) -> &Self::MB {
        &self.structure.mb
    }
}

impl<MA, MB> Worldline for GenericWorldline<MyMobject7, MyMobject7Structure<MA, MB>>
where
    MA: Worldline<Observatory = Entity<MyMobject6, MyMobject6Structure>>,
    MB: Worldline<Observatory = Entity<MyMobject6, MyMobject6Structure>>,
{
    type Observatory = GenericWorldline<MyMobject7, MyMobject7Structure>;

    fn observe(&self, clock: Clock, clock_span: ClockSpan) -> Self::Observatory {
        self.map_structure_ref(|structure| MyMobject7Structure {
            ma: structure.ma.observe(clock, clock_span),
            mb: structure.mb.observe(clock, clock_span),
        })
    }
}

// impl<TE, CU> Timeline for MySpatialMobjectDynamicWorldline<TE, CU>
// where
//     TE: TimeEval,
//     CU: ContinuousUpdate<TE::OutputTimeMetric, MySpatialMobject>,
// {
//     type Mobject = MySpatialMobject;

//     fn observe(
//         &self,
//         time: Time,
//         time_interval: Range<Time>,
//         mobject: Self::Mobject,
//     ) -> Self::Mobject {
//         self.continuous_update
//             .continuous_update(self.time_eval.time_eval(time, time_interval), mobject)
//     }
// }

// pub trait PreparableMobject: Mobject {
//     type Output: 'static + Send + Sync;

//     fn prepare(
//         &self,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) -> Self::Output;
//     // fn present_in_place(
//     //     &self,
//     //     device: &wgpu::Device,
//     //     queue: &wgpu::Queue,
//     //     format: wgpu::TextureFormat,
//     //     presentation: &mut Self::Presentation,
//     // ); // presentation write-only
// }

// pub trait UpdateOnce<PM>
// where
//     PM: PreparableMobject,
// {
//     fn update_once(&self, mobject: &mut PM);
// }

pub trait ContinuousUpdate<TM, E>:
    'static + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize
{
    fn continuous_update(&self, time: Time, entity: &mut E);

    // {
    //     // let mut mobject = mobject.clone();
    //     // self.update(&mut mobject, time_metric);
    //     std::mem::replace(presentation, mobject.present(device, queue, format));
    // }
    // {
    //     mobject.present_in_place(device, queue, format, presentation);
    // }
}

// pub trait DiscreteUpdate {
//     fn discrete_update(self, timer: &Timer, alive: &Alive) -> &Alive;
// }

// pub struct Alive<'t, TQ, TS>
// where
//     TQ: TypeQuery,
//     TS: TimelineState<TQ>,
// {
//     parent: P,
//     spawn_time: Rc<Time>,
//     // config: &'t Config,
//     // timer: &'t Timer,

//     // attached_mobject: AttachedMobject<'t, 'a, TQ>,
//     mobject: Arc<M>,
// }

// impl<U, PM> UpdateOnce<PM> for U
// where
//     U: Update<PM, NormalizedTimeMetric>,
//     PM: PreparableMobject,
// {
//     fn update_once(&self, mobject: &mut PM) {
//         self.update(mobject, NormalizedTimeMetric(1.0));
//     }
// }

// pub trait Prepare {
//     type Output;

//     fn prepare(
//         &self,
//         time: Time,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) -> Self::Output;
// }

// pub trait Render {
//     fn render(
//         &self,
//         storage_type_map: &StorageTypeMap,
//         encoder: &mut wgpu::CommandEncoder,
//         target: &wgpu::TextureView,
//     );
// }

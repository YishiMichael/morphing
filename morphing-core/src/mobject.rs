use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

use super::storable::StorageTypeMap;
use super::timer::Clock;
use super::timer::ClockSpan;
use super::timer::Rate;
use super::timer::Time;
use super::timer::TimeMetric;

pub trait Mobject:
    'static + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize
{
    type Entity;
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Entity<M, S> {
    mobject: M,
    structure: S,
}

pub trait EntityTrait:
    'static
    + Debug
    + Send
    + Sync
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
    + Deref<Target = Self::Mobject>
{
    type Mobject;
    type Structure;

    // fn mobject(entity: &Self) -> &Self::Mobject;
    fn structure(entity: &Self) -> &Self::Structure;
}

impl<M, S> Deref for Entity<M, S> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.mobject
    }
}

impl<M, S> EntityTrait for Entity<M, S>
where
    M: 'static + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
    S: 'static + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    type Mobject = M;
    type Structure = S;

    // fn mobject(entity: &Self) -> &Self::Mobject {
    //     &entity.mobject
    // }

    fn structure(entity: &Self) -> &Self::Structure {
        &entity.structure
    }
}

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

pub trait Observatory:
    'static + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize
{
    // type Initial;
    type Target;

    // fn initial(&self) -> Self::Initial;
    // fn update(self, initial: Self::Initial) -> &Self::Target;
    fn borrow(&self) -> &Self::Target;
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct StaticObservatory<O>(Arc<O>);

impl<O> Clone for StaticObservatory<O> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<O> Observatory for StaticObservatory<O>
where
    O: 'static + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    type Target = O;

    fn borrow(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct DynamicObservatory<O>(O);

impl<O> Observatory for DynamicObservatory<O>
where
    O: 'static + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    type Target = O;

    fn borrow(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct GenericObservatory<O>(O);

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

pub trait Worldline:
    Send + Sync + serde_traitobject::Deserialize + serde_traitobject::Serialize
{
    type Observatory;

    fn observe(&self, clock: Clock, clock_span: ClockSpan) -> Self::Observatory;
}

pub trait SerdeWorldline:
    Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize + Worldline
{
}

pub trait Prepare<E>: Sized {
    // type Presentation;

    fn present(
        entity: &E,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self;
    fn prepare(
        &mut self,
        entity: &E,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) {
        *self = Self::present(entity, device, queue, format);
    } // -> &Self::Output; // input presentation write-only
}

pub trait Render {
    fn render(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView);
}

impl<W> SerdeWorldline for W where W: for<'de> serde::Deserialize<'de> + serde::Serialize + Worldline
{}

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

// pub trait AllocatedTimeline {
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
    entity: Entity<StaticObservatory<M>, StaticObservatory<S>>,
}

impl<M, S> Worldline for StaticWorldline<M, S>
where
    M: Mobject<Entity = Entity<M, S>>,
    S: Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    // type Transformed<TR0> = StaticWorldline<RateChain<TR0, TR::TimeMetric, TR>, M>
    // where
    //     TR0: TimeRate;
    type Observatory = Entity<StaticObservatory<M>, StaticObservatory<S>>;

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
            mobject: self.entity.mobject.clone(),
            structure: self.entity.structure.clone(),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct DynamicWorldline<M, S, TM, R, CU> {
    entity: Entity<DynamicObservatory<M>, DynamicObservatory<S>>,
    time_metric: TM,
    rate: R,
    continuous_update: CU,
}

impl<M, S, TM, R, CU> Worldline for DynamicWorldline<M, S, TM, R, CU>
where
    M: Clone + Mobject<Entity = Entity<M, S>>,
    S: Clone + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize,
    TM: TimeMetric,
    R: Rate<TM>,
    CU: ContinuousUpdate<TM, M::Entity>,
{
    // type Transformed<TR0> = DynamicWorldline<RateChain<TR0, TR::TimeMetric, TR>, M, CU>
    // where
    //     TR0: TimeRate;
    type Observatory = Entity<DynamicObservatory<M>, DynamicObservatory<S>>;

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
            mobject: self.entity.mobject.0.clone(),
            structure: self.entity.structure.0.clone(),
        };
        self.continuous_update.continuous_update(
            self.rate
                .eval(self.time_metric.localize_from_clock(clock, clock_span)),
            &mut entity,
        );
        Entity {
            mobject: DynamicObservatory(entity.mobject),
            structure: DynamicObservatory(entity.structure),
        }
        // self.time_rate.eval_time(time).map(|metric_time| {
        //     self.continuous_update
        //         .continuous_update(*metric_time, self.entity.clone())
        // })
    }
}

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// pub struct GroupWorldline(Vec<Vec<(ClockSpan, serde_traitobject::Arc<dyn Worldline>)>>);

// atom

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject0Structure {}

impl Observatory for GenericObservatory<MyMobject0Structure> {
    type Target = MyMobject0Structure;

    fn borrow(&self) -> &Self::Target {
        &MyMobject0Structure {}
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject0 {
    demo0: f32,
}

impl Mobject for MyMobject0 {
    type Entity = Entity<MyMobject0, MyMobject0Structure>;
}

pub trait MyMobject0Trait {}

impl MyMobject0Trait for Entity<MyMobject0, MyMobject0Structure> {}

pub trait MyMobject0ObservatoryTrait:
    EntityTrait<
    Mobject: Observatory<Target = MyMobject0>,
    Structure: Observatory<Target = MyMobject0Structure>,
>
{
}

impl<M, S> MyMobject0ObservatoryTrait for Entity<M, S>
where
    M: Observatory<Target = MyMobject0>,
    S: Observatory<Target = MyMobject0Structure>,
{
}

impl Worldline for Entity<StaticObservatory<MyMobject0>, GenericObservatory<MyMobject0Structure>> {
    type Observatory =
        Entity<StaticObservatory<MyMobject0>, GenericObservatory<MyMobject0Structure>>;

    #[allow(unused_variables)]
    fn observe(&self, clock: Clock, clock_span: ClockSpan) -> Self::Observatory {
        Entity {
            mobject: self.mobject.clone(),
            structure: GenericObservatory(MyMobject0Structure {}),
        }
    }
}

// atom structured

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject1Structure<
    MA = Entity<MyMobject0, MyMobject0Structure>,
    MB = Entity<MyMobject0, MyMobject0Structure>,
> {
    ma: MA,
    mb: MB,
}

impl<MA, MB> Observatory for GenericObservatory<MyMobject1Structure<MA, MB>>
where
    MA: Observatory<Target: MyMobject0ObservatoryTrait>,
    MB: Observatory<Target: MyMobject0ObservatoryTrait>,
{
    type Target = MyMobject1Structure<MA::Target, MB::Target>;

    fn borrow(&self) -> &Self::Target {
        &self.0
    }

    fn update(self, target: Self::Target) -> Self::Target {
        MyMobject1Structure {
            ma: self.0.ma.update(target.ma),
            mb: self.0.mb.update(target.mb),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject1 {
    demo1: f32,
}

impl Mobject for MyMobject1 {
    type Entity = Entity<MyMobject1, MyMobject1Structure>;
}

pub trait MyMobject1Trait {
    type MA;
    type MB;

    fn ma(&self) -> &Self::MA;
    fn mb(&self) -> &Self::MB;
}

impl MyMobject1Trait for Entity<MyMobject1, MyMobject1Structure> {
    type MA = Entity<MyMobject0, MyMobject0Structure>;
    type MB = Entity<MyMobject0, MyMobject0Structure>;

    fn ma(&self) -> &Self::MA {
        &self.structure.ma
    }

    fn mb(&self) -> &Self::MB {
        &self.structure.mb
    }
}

pub trait MyMobject1ObservatoryTrait:
    EntityTrait<
    Mobject: Observatory<Target = MyMobject1>,
    Structure: Observatory<Target = MyMobject1Structure<Self::MA, Self::MB>>,
>
{
    type MA: Observatory<Target: MyMobject0ObservatoryTrait>;
    type MB: Observatory<Target: MyMobject0ObservatoryTrait>;

    fn ma(&self) -> &Self::MA;
    fn mb(&self) -> &Self::MB;
}

impl<M, S, MA, MB> MyMobject1ObservatoryTrait for Entity<M, S>
where
    M: Observatory<Target = MyMobject1>,
    S: Observatory<Target = MyMobject1Structure<MA, MB>>,
    MA: Observatory<Target: MyMobject0ObservatoryTrait>,
    MB: Observatory<Target: MyMobject0ObservatoryTrait>,
{
    type MA = MA;
    type MB = MB;

    fn ma(&self) -> &Self::MA {
        &self.structure.ma
    }

    fn mb(&self) -> &Self::MB {
        &self.structure.mb
    }
}

impl<MA, MB> Worldline
    for Entity<StaticObservatory<MyMobject1>, GenericObservatory<MyMobject1Structure<MA, MB>>>
where
    MA: SerdeWorldline,
    MB: SerdeWorldline,
{
    type Observatory = Entity<
        StaticObservatory<MyMobject1>,
        GenericObservatory<MyMobject1Structure<MA::Observatory, MB::Observatory>>,
    >;

    fn observe(&self, clock: Clock, clock_span: ClockSpan) -> Self::Observatory {
        Entity {
            mobject: self.mobject.clone(),
            structure: GenericObservatory(MyMobject1Structure {
                ma: self.structure.ma.observe(clock, clock_span),
                mb: self.structure.mb.observe(clock, clock_span),
            }),
        }
    }
}

// prepare

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject2Structure<
    MA = Entity<MyMobject1, MyMobject1Structure>,
    MB = Entity<MyMobject1, MyMobject1Structure>,
> {
    ma: MA,
    mb: MB,
}

impl<MA, MB> Observatory for GenericObservatory<MyMobject2Structure<MA, MB>>
where
    MA: Observatory<Target: MyMobject1ObservatoryTrait>,
    MB: Observatory<Target: MyMobject1ObservatoryTrait>,
{
    type Target = MyMobject2Structure<MA, MB>;

    fn borrow(&self) -> &Self::Target {
        &self.0
    }

    fn update(self, target: &mut Self::Target) {
        self.ma.update(&mut target.ma);
        self.mb.update(&mut target.mb);
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject2 {
    demo2: f32,
}

impl Mobject for MyMobject2 {
    type Entity = Entity<MyMobject2, MyMobject2Structure>;
}

pub trait MyMobject2Trait {
    type MA;
    type MB;

    fn ma(&self) -> &Self::MA;
    fn mb(&self) -> &Self::MB;
}

impl MyMobject2Trait for Entity<MyMobject2, MyMobject2Structure> {
    type MA = Entity<MyMobject1, MyMobject1Structure>;
    type MB = Entity<MyMobject1, MyMobject1Structure>;

    fn ma(&self) -> &Self::MA {
        &self.structure.ma
    }

    fn mb(&self) -> &Self::MB {
        &self.structure.mb
    }
}

pub trait MyMobject2ObservatoryTrait:
    EntityTrait<
    Mobject: Observatory<Target = MyMobject2>,
    Structure: Observatory<Target = MyMobject2Structure<Self::MA, Self::MB>>,
>
{
    type MA: Observatory<Target: MyMobject1ObservatoryTrait>;
    type MB: Observatory<Target: MyMobject1ObservatoryTrait>;

    fn ma(&self) -> &Self::MA;
    fn mb(&self) -> &Self::MB;
}

impl<M, S, MA, MB> MyMobject2ObservatoryTrait for Entity<M, S>
where
    M: Observatory<Target = MyMobject2>,
    S: Observatory<Target = MyMobject2Structure<MA, MB>>,
    MA: Observatory<Target: MyMobject1ObservatoryTrait>,
    MB: Observatory<Target: MyMobject1ObservatoryTrait>,
{
    type MA = MA;
    type MB = MB;

    fn ma(&self) -> &Self::MA {
        &self.structure.ma
    }

    fn mb(&self) -> &Self::MB {
        &self.structure.mb
    }
}

impl<MA, MB> Worldline
    for Entity<StaticObservatory<MyMobject2>, GenericObservatory<MyMobject2Structure<MA, MB>>>
where
    MA: SerdeWorldline,
    MB: SerdeWorldline,
{
    type Observatory = Entity<
        StaticObservatory<MyMobject2>,
        GenericObservatory<MyMobject2Structure<MA::Observatory, MB::Observatory>>,
    >;

    fn observe(&self, clock: Clock, clock_span: ClockSpan) -> Self::Observatory {
        Entity {
            mobject: self.mobject.clone(),
            structure: GenericObservatory(MyMobject2Structure {
                ma: self.structure.ma.observe(clock, clock_span),
                mb: self.structure.mb.observe(clock, clock_span),
            }),
        }
    }
}

impl Prepare<Entity<MyMobject2, MyMobject2Structure>>
    for (f32, ((f32, (f32, f32)), (f32, (f32, f32))))
{
    fn present(
        entity: &Entity<MyMobject2, MyMobject2Structure>,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) -> Self {
        (
            entity.demo2,
            (
                (
                    entity.ma().demo1,
                    (entity.ma().ma().demo0, entity.ma().mb().demo0),
                ),
                (
                    entity.mb().demo1,
                    (entity.mb().ma().demo0, entity.mb().mb().demo0),
                ),
            ),
        )
    }
}

// prepare structured

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject3Structure<
    MA = Entity<MyMobject2, MyMobject2Structure>,
    MB = Entity<MyMobject2, MyMobject2Structure>,
> {
    ma: MA,
    mb: MB,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject3 {
    demo3: f32,
}

impl Mobject for MyMobject3 {
    type Entity = Entity<MyMobject3, MyMobject3Structure>;
}

impl<MA, MB> Worldline for MyMobject3Structure<MA, MB>
where
    MA: SerdeWorldline,
    MB: SerdeWorldline,
{
    type Observatory = MyMobject3Structure<MA::Observatory, MB::Observatory>;

    fn observe(&self, clock: Clock, clock_span: ClockSpan) -> Self::Observatory {
        MyMobject3Structure {
            ma: self.ma.observe(clock, clock_span),
            mb: self.mb.observe(clock, clock_span),
        }
    }
}

// render

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject4Structure<
    MA = Entity<MyMobject3, MyMobject3Structure>,
    MB = Entity<MyMobject3, MyMobject3Structure>,
> {
    ma: MA,
    mb: MB,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject4 {
    demo4: f32,
}

impl Mobject for MyMobject4 {
    type Entity = Entity<MyMobject4, MyMobject4Structure>;
}

impl<MA, MB> Worldline for MyMobject4Structure<MA, MB>
where
    MA: SerdeWorldline,
    MB: SerdeWorldline,
{
    type Observatory = MyMobject4Structure<MA::Observatory, MB::Observatory>;

    fn observe(&self, clock: Clock, clock_span: ClockSpan) -> Self::Observatory {
        MyMobject4Structure {
            ma: self.ma.observe(clock, clock_span),
            mb: self.mb.observe(clock, clock_span),
        }
    }
}

// render structured

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject5Structure<
    MA = Entity<MyMobject4, MyMobject4Structure>,
    MB = Entity<MyMobject4, MyMobject4Structure>,
> {
    ma: MA,
    mb: MB,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MyMobject5 {
    demo5: f32,
}

impl Mobject for MyMobject5 {
    type Entity = Entity<MyMobject5, MyMobject5Structure>;
}

impl<MA, MB> Worldline for MyMobject5Structure<MA, MB>
where
    MA: SerdeWorldline,
    MB: SerdeWorldline,
{
    type Observatory = MyMobject5Structure<MA::Observatory, MB::Observatory>;

    fn observe(&self, clock: Clock, clock_span: ClockSpan) -> Self::Observatory {
        MyMobject5Structure {
            ma: self.ma.observe(clock, clock_span),
            mb: self.mb.observe(clock, clock_span),
        }
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

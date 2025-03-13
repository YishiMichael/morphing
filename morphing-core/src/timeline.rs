use core::range::Range;
use std::any::TypeId;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

use super::stage::Archive;
use super::stage::Channel;
use super::stage::ChannelAttachment;
use super::stage::ChannelIndex;
use super::stage::Layer;
use super::stage::LayerAttachment;
use super::stage::LayerIndex;
use super::stage::World;
use super::storable::DynKey;
use super::storable::SharableSlot;
use super::storable::Slot;
use super::storable::SlotKeyGenerator;
use super::storable::SlotKeyGeneratorTypeMap;
use super::storable::Storable;
use super::storable::StorageKey;
use super::storable::StorageTypeMap;
use super::storable::SwapSlot;
use super::storable::VecSlot;
use super::timer::DenormalizedTimeEval;
use super::timer::IncreasingTimeEval;
use super::timer::NormalizedTimeEval;
use super::timer::NormalizedTimeMetric;
use super::timer::RateComposeTimeEval;
use super::timer::Time;
use super::timer::TimeEval;
use super::timer::TimeMetric;
use super::timer::Timer;
use super::traits::Construct;
use super::traits::Mobject;
use super::traits::MobjectBuilder;
use super::traits::MobjectPresentation;
use super::traits::Rate;
use super::traits::Update;

pub enum PresentationKey<MP>
where
    MP: 'static + Send + Sync,
{
    Static(
        Arc<StorageKey<
            (TypeId, Box<dyn DynKey>),
            <<SwapSlot<SharableSlot<MP>> as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
        >>,
    ),
    Dynamic(
        Arc<StorageKey<
            (TypeId, Box<dyn DynKey>, Box<dyn DynKey>),
            <<SwapSlot<VecSlot<MP>> as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
        >>,
    ),
}

impl<MP> PresentationKey<MP>
where
    MP: 'static + Send + Sync,
{
    pub fn read<'mp>(&self, storage_type_map: &'mp StorageTypeMap) -> &'mp MP {
        match self {
            Self::Static(key) => storage_type_map
                .get::<_, SwapSlot<SharableSlot<MP>>>(key)
                .as_ref()
                .unwrap(),
            Self::Dynamic(key) => storage_type_map
                .get::<_, SwapSlot<VecSlot<MP>>>(key)
                .as_ref()
                .unwrap(),
        }
    }
}

pub trait Timeline:
    'static + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize + Storable
{
    type MobjectPresentation: Send + Sync;

    fn init_presentation(&self, device: &wgpu::Device) -> <Self::Slot as Slot>::Value;
    fn erase_presentation_key(
        &self,
        mobject_presentation_key: Arc<
            StorageKey<
                Self::StorableKey,
                <<Self::Slot as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
            >,
        >,
    ) -> PresentationKey<Self::MobjectPresentation>;
    fn prepare_presentation(
        &self,
        time: Time,
        time_interval: Range<Time>,
        mobject_presentation: &mut <Self::Slot as Slot>::Value,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    );
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct StaticTimeline<TQ>
where
    TQ: TypeQuery,
{
    mobject: Arc<TQ::Mobject>,
}

impl<TQ> Storable for StaticTimeline<TQ>
where
    TQ: TypeQuery,
{
    type StorableKey = (TypeId, Box<dyn DynKey>);
    type Slot = SwapSlot<SharableSlot<TQ::MobjectPresentation>>;

    fn key(
        &self,
        storable_key_fn: &fn(&dyn serde_traitobject::Serialize) -> Box<dyn DynKey>,
    ) -> Self::StorableKey {
        (
            TypeId::of::<(TQ::Layer, TQ::Mobject)>(),
            storable_key_fn(self.mobject.as_ref()),
        )
    }
}

impl<TQ> Timeline for StaticTimeline<TQ>
where
    TQ: TypeQuery,
{
    type MobjectPresentation = TQ::MobjectPresentation;

    fn init_presentation(&self, device: &wgpu::Device) -> <Self::Slot as Slot>::Value {
        Arc::new(TQ::MobjectPresentation::presentation(
            self.mobject.as_ref(),
            device,
        ))
    }

    fn erase_presentation_key(
        &self,
        mobject_presentation_key: Arc<
            StorageKey<
                Self::StorableKey,
                <<Self::Slot as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
            >,
        >,
    ) -> PresentationKey<Self::MobjectPresentation> {
        PresentationKey::Static(mobject_presentation_key)
    }

    fn prepare_presentation(
        &self,
        _time: Time,
        _time_interval: Range<Time>,
        _mobject_presentation: &mut <Self::Slot as Slot>::Value,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
    ) {
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct DynamicTimeline<TQ, TE, U>
where
    TQ: TypeQuery,
{
    mobject: Arc<TQ::Mobject>,
    time_eval: TE,
    update: U,
}

impl<TQ, TE, U> Storable for DynamicTimeline<TQ, TE, U>
where
    TQ: TypeQuery,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, TQ>,
{
    type StorableKey = (TypeId, Box<dyn DynKey>, Box<dyn DynKey>);
    type Slot = SwapSlot<VecSlot<TQ::MobjectPresentation>>;

    fn key(
        &self,
        storable_key_fn: &fn(&dyn serde_traitobject::Serialize) -> Box<dyn DynKey>,
    ) -> Self::StorableKey {
        (
            TypeId::of::<(TQ::Layer, TQ::Mobject, U)>(),
            storable_key_fn(self.mobject.as_ref()),
            storable_key_fn(&self.update),
        )
    }
}

impl<TQ, TE, U> Timeline for DynamicTimeline<TQ, TE, U>
where
    TQ: TypeQuery,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, TQ>,
{
    type MobjectPresentation = TQ::MobjectPresentation;

    fn init_presentation(&self, device: &wgpu::Device) -> <Self::Slot as Slot>::Value {
        TQ::MobjectPresentation::presentation(self.mobject.as_ref(), device)
    }

    fn erase_presentation_key(
        &self,
        mobject_presentation_key: Arc<
            StorageKey<
                Self::StorableKey,
                <<Self::Slot as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
            >,
        >,
    ) -> PresentationKey<Self::MobjectPresentation> {
        PresentationKey::Dynamic(mobject_presentation_key)
    }

    fn prepare_presentation(
        &self,
        time: Time,
        time_interval: Range<Time>,
        mobject_presentation: &mut <Self::Slot as Slot>::Value,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) {
        self.update.prepare_presentation(
            self.time_eval.time_eval(time, time_interval),
            self.mobject.as_ref(),
            mobject_presentation,
            device,
            queue,
            format,
        );
    }
}

pub trait TimelineErasure:
    'static + serde_traitobject::Deserialize + serde_traitobject::Serialize
{
    type MobjectPresentation;

    fn allocate(
        self: Box<Self>,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Box<dyn AllocatedTimelineErasure<MobjectPresentation = Self::MobjectPresentation>>;
}

pub trait AllocatedTimelineErasure {
    type MobjectPresentation: Send + Sync;

    fn prepare(
        &self,
        time: Time,
        time_interval: Range<f32>,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> PresentationKey<Self::MobjectPresentation>;
}

struct AllocatedTimeline<T>
where
    T: Timeline,
{
    storage_key: Arc<
        StorageKey<
            T::StorableKey,
            <<T::Slot as Slot>::SlotKeyGenerator as SlotKeyGenerator>::SlotKey,
        >,
    >,
    timeline: Box<T>,
}

impl<T> TimelineErasure for T
where
    T: Timeline,
{
    type MobjectPresentation = T::MobjectPresentation;

    fn allocate(
        self: Box<Self>,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Box<dyn AllocatedTimelineErasure<MobjectPresentation = Self::MobjectPresentation>> {
        Box::new(AllocatedTimeline {
            storage_key: Arc::new(slot_key_generator_type_map.allocate(self.as_ref())),
            timeline: self,
        })
    }
}

impl<T> AllocatedTimelineErasure for AllocatedTimeline<T>
where
    T: Timeline,
{
    type MobjectPresentation = T::MobjectPresentation;

    fn prepare(
        &self,
        time: Time,
        time_interval: Range<f32>,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> PresentationKey<Self::MobjectPresentation> {
        let mobject_presentation = storage_type_map
            .get_or_insert_with::<_, T::Slot, _>(&self.storage_key, || {
                self.timeline.init_presentation(device)
            });
        self.timeline.prepare_presentation(
            time,
            time_interval,
            mobject_presentation,
            device,
            queue,
            format,
        );
        self.timeline
            .erase_presentation_key(self.storage_key.clone())
    }
}

pub trait TypeQuery: 'static + Debug + Send + Sync {
    type World: World;
    type LayerIndex: LayerIndex<Self::World, Layer = Self::Layer>;
    type Layer: Layer;
    type ChannelIndex: ChannelIndex<Self::Layer, Channel = Self::Channel>;
    type Channel: Channel<MobjectPresentation = Self::MobjectPresentation>;
    type Mobject: Mobject;
    type MobjectPresentation: MobjectPresentation<Self::Mobject>;
}

pub struct TypeQueried<W, LI, L, CI, C, M, MP>(PhantomData<fn() -> (W, LI, L, CI, C, M, MP)>);

impl<W, LI, L, CI, C, M, MP> Debug for TypeQueried<W, LI, L, CI, C, M, MP> {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl<W, LI, L, CI, C, M, MP> TypeQuery for TypeQueried<W, LI, L, CI, C, M, MP>
where
    W: World,
    LI: LayerIndex<W, Layer = L>,
    L: Layer,
    CI: ChannelIndex<L, Channel = C>,
    C: Channel<MobjectPresentation = MP>,
    M: Mobject,
    MP: MobjectPresentation<M>,
{
    type World = W;
    type LayerIndex = LI;
    type Layer = L;
    type ChannelIndex = CI;
    type Channel = C;
    type Mobject = M;
    type MobjectPresentation = MP;
}

pub struct AttachedMobject<'t, 'a, TQ>
where
    TQ: TypeQuery,
{
    mobject: Arc<TQ::Mobject>,
    attachment: &'a ChannelAttachment<
        't,
        TQ::World,
        TQ::LayerIndex,
        TQ::Layer,
        TQ::ChannelIndex,
        TQ::Channel,
        TQ::MobjectPresentation,
    >,
}

impl<'t, 'a, TQ> AttachedMobject<'t, 'a, TQ>
where
    TQ: TypeQuery,
{
    fn launch<TS>(self, timeline_state: TS) -> Alive<'t, 'a, TQ, TS>
    where
        TS: TimelineState<TQ>,
    {
        Alive(Some(AliveInner {
            alive_id: self.attachment.timer.generate_alive_id(),
            spawn_time: self.attachment.timer.time(),
            attached_mobject: self,
            timeline_state,
        }))
    }

    fn update<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut TQ::Mobject),
    {
        let mut mobject = Arc::unwrap_or_clone(self.mobject);
        f(&mut mobject);
        AttachedMobject {
            mobject: Arc::new(mobject),
            attachment: self.attachment,
        }
    }

    fn update_cloned<F>(&self, f: F) -> Self
    where
        F: FnOnce(&mut TQ::Mobject),
    {
        Self {
            mobject: self.mobject.clone(),
            attachment: self.attachment,
        }
        .update(f)
    }

    fn transit_simple<F, T>(self, alive_id: usize, time_interval: Range<Rc<Time>>, f: F) -> Self
    where
        F: FnOnce(Arc<TQ::Mobject>) -> T,
        T: TimelineErasure<MobjectPresentation = TQ::MobjectPresentation>,
    {
        if !Rc::ptr_eq(&time_interval.start, &time_interval.end) {
            self.attachment.channel.push(
                alive_id,
                Range {
                    start: *time_interval.start,
                    end: *time_interval.end,
                },
                f(self.mobject.clone()),
            );
        }
        self
    }

    fn transit_complex<TE, C>(
        self,
        alive_id: usize,
        time_interval: Range<Rc<Time>>,
        time_eval: &TE,
        construct: C,
    ) -> Self
    where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
        C: Construct<TQ>,
    {
        let timer = Timer::new();
        let world = TQ::World::new();
        let attached_mobject = {
            let world_attachment = World::attachment(&world, self.attachment.config, &timer);
            let (attached_mobject, CollapsedTimelineState) = construct
                .construct(
                    &world_attachment,
                    self.attachment.config,
                    &timer,
                    AttachedMobject {
                        mobject: self.mobject,
                        attachment: TQ::ChannelIndex::index_attachment(
                            TQ::LayerIndex::index_attachment(&world_attachment),
                        ),
                    }
                    .launch(CollapsedTimelineState),
                )
                .terminate();
            AttachedMobject {
                mobject: attached_mobject.mobject,
                attachment: self.attachment,
            }
        };
        if !Rc::ptr_eq(&time_interval.start, &time_interval.end) {
            self.attachment.world.merge(
                world.archive(),
                alive_id,
                time_eval,
                Range {
                    start: *time_interval.start,
                    end: *time_interval.end,
                },
                Range {
                    start: 0.0,
                    end: *timer.time(),
                },
            );
        }
        attached_mobject
    }
}

struct AliveInner<'t, 'a, TQ, TS>
where
    TQ: TypeQuery,
    TS: TimelineState<TQ>,
{
    alive_id: usize,
    spawn_time: Rc<Time>,
    attached_mobject: AttachedMobject<'t, 'a, TQ>,
    timeline_state: TS,
}

pub struct Alive<'t, 'a, TQ, TS>(Option<AliveInner<'t, 'a, TQ, TS>>)
where
    TQ: TypeQuery,
    TS: TimelineState<TQ>;

impl<'t, 'a, TQ, TS> Alive<'t, 'a, TQ, TS>
where
    TQ: TypeQuery,
    TS: TimelineState<TQ>,
{
    fn terminate(&mut self) -> (AttachedMobject<'t, 'a, TQ>, TS::ResidueTimelineState) {
        let inner = self.0.take().unwrap();
        inner.timeline_state.transit_with_residue(
            inner.alive_id,
            Range {
                start: inner.spawn_time.clone(),
                end: inner.attached_mobject.attachment.timer.time(),
            },
            inner.attached_mobject,
        )
    }
}

impl<TQ, TS> Drop for Alive<'_, '_, TQ, TS>
where
    TQ: TypeQuery,
    TS: TimelineState<TQ>,
{
    fn drop(&mut self) {
        if self.0.is_some() {
            self.terminate();
        }
    }
}

pub trait TimelineState<TQ>: 'static
where
    TQ: TypeQuery,
{
    type ResidueTimelineState: TimelineState<TQ>;

    fn transit_with_residue<'t, 'a>(
        self,
        alive_id: usize,
        time_interval: Range<Rc<Time>>,
        attached_mobject: AttachedMobject<'t, 'a, TQ>,
    ) -> (AttachedMobject<'t, 'a, TQ>, Self::ResidueTimelineState);
}

pub struct CollapsedTimelineState;

impl<TQ> TimelineState<TQ> for CollapsedTimelineState
where
    TQ: TypeQuery,
{
    type ResidueTimelineState = CollapsedTimelineState;

    fn transit_with_residue<'t, 'a>(
        self,
        alive_id: usize,
        time_interval: Range<Rc<Time>>,
        attached_mobject: AttachedMobject<'t, 'a, TQ>,
    ) -> (AttachedMobject<'t, 'a, TQ>, Self::ResidueTimelineState) {
        (
            attached_mobject.transit_simple(alive_id, time_interval, |mobject| StaticTimeline::<
                TQ,
            > {
                mobject,
            }),
            CollapsedTimelineState,
        )
    }
}

pub struct IndeterminedTimelineState<TE> {
    time_eval: TE,
}

impl<TQ, TE> TimelineState<TQ> for IndeterminedTimelineState<TE>
where
    TQ: TypeQuery,
    TE: TimeEval,
{
    type ResidueTimelineState = IndeterminedTimelineState<TE>;

    fn transit_with_residue<'t, 'a>(
        self,
        alive_id: usize,
        time_interval: Range<Rc<Time>>,
        attached_mobject: AttachedMobject<'t, 'a, TQ>,
    ) -> (AttachedMobject<'t, 'a, TQ>, Self::ResidueTimelineState) {
        (
            attached_mobject.transit_simple(alive_id, time_interval, |mobject| StaticTimeline::<
                TQ,
            > {
                mobject,
            }),
            IndeterminedTimelineState {
                time_eval: self.time_eval,
            },
        )
    }
}

pub struct UpdateTimelineState<TE, U> {
    time_eval: TE,
    update: U,
}

impl<TQ, TE, U> TimelineState<TQ> for UpdateTimelineState<TE, U>
where
    TQ: TypeQuery,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, TQ>,
{
    type ResidueTimelineState = CollapsedTimelineState;

    fn transit_with_residue<'t, 'a>(
        self,
        alive_id: usize,
        time_interval: Range<Rc<Time>>,
        attached_mobject: AttachedMobject<'t, 'a, TQ>,
    ) -> (AttachedMobject<'t, 'a, TQ>, Self::ResidueTimelineState) {
        let attached_mobject_updated = attached_mobject.update_cloned(|mobject| {
            self.update.update(
                self.time_eval.time_eval(
                    *time_interval.end,
                    Range {
                        start: *time_interval.start,
                        end: *time_interval.end,
                    },
                ),
                mobject,
            )
        });
        attached_mobject.transit_simple(alive_id, time_interval, |mobject| DynamicTimeline {
            mobject,
            time_eval: self.time_eval,
            update: self.update,
        });
        (attached_mobject_updated, CollapsedTimelineState)
    }
}

pub struct ConstructTimelineState<TE, C> {
    time_eval: TE,
    construct: C,
}

impl<TQ, TE, C> TimelineState<TQ> for ConstructTimelineState<TE, C>
where
    TQ: TypeQuery,
    TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    C: Construct<TQ>,
{
    type ResidueTimelineState = CollapsedTimelineState;

    fn transit_with_residue<'t, 'a>(
        self,
        alive_id: usize,
        time_interval: Range<Rc<Time>>,
        attached_mobject: AttachedMobject<'t, 'a, TQ>,
    ) -> (AttachedMobject<'t, 'a, TQ>, Self::ResidueTimelineState) {
        (
            attached_mobject.transit_complex(
                alive_id,
                time_interval,
                &self.time_eval,
                self.construct,
            ),
            CollapsedTimelineState,
        )
    }
}

pub trait Spawn<'t, 'a, TQ, I>
where
    TQ: TypeQuery,
{
    fn spawn(&'a self, input: I) -> Alive<'t, 'a, TQ, CollapsedTimelineState>;
}

pub trait Quantize: Sized {
    type Output<TE>
    where
        TE: TimeEval;

    fn quantize<TE>(self, time_metric: TE) -> Self::Output<TE>
    where
        TE: TimeEval;
}

pub trait Collapse: Sized {
    type Output;

    fn collapse(self) -> Self::Output;
}

pub trait ApplyRate<TM>: Sized
where
    TM: TimeMetric,
{
    type Output<R>
    where
        R: Rate<TM>;

    fn apply_rate<R>(self, rate: R) -> Self::Output<R>
    where
        R: Rate<TM>;
}

pub trait ApplyUpdate<TM, TQ>: Sized
where
    TM: TimeMetric,
    TQ: TypeQuery,
{
    type Output<U>
    where
        U: Update<TM, TQ>;

    fn apply_update<U>(self, update: U) -> Self::Output<U>
    where
        U: Update<TM, TQ>;
}

pub trait ApplyConstruct<TQ>: Sized
where
    TQ: TypeQuery,
{
    type Output<C>
    where
        C: Construct<TQ>;

    fn apply_construct<C>(self, construct: C) -> Self::Output<C>
    where
        C: Construct<TQ>;
}

impl<'t, 'a, TQ> Spawn<'t, 'a, TQ, TQ::Mobject>
    for ChannelAttachment<
        't,
        TQ::World,
        TQ::LayerIndex,
        TQ::Layer,
        TQ::ChannelIndex,
        TQ::Channel,
        TQ::MobjectPresentation,
    >
where
    TQ: TypeQuery,
{
    fn spawn(&'a self, mobject: TQ::Mobject) -> Alive<'t, 'a, TQ, CollapsedTimelineState> {
        AttachedMobject {
            mobject: Arc::new(mobject),
            attachment: self,
        }
        .launch(CollapsedTimelineState)
    }
}

impl<'t, 'a, TQ, MB> Spawn<'t, 'a, TQ, MB>
    for LayerAttachment<
        't,
        TQ::World,
        TQ::LayerIndex,
        TQ::Layer,
        <TQ::Layer as Layer>::Residue<'t, TQ::World, TQ::LayerIndex>,
    >
where
    TQ: TypeQuery,
    MB: MobjectBuilder<TQ::Layer, OutputTypeQuery<TQ::World, TQ::LayerIndex> = TQ>,
{
    fn spawn(&'a self, mobject_builder: MB) -> Alive<'t, 'a, TQ, CollapsedTimelineState> {
        mobject_builder.instantiate(self, self.config)
    }
}

impl<'t, 'a, TQ> Quantize for Alive<'t, 'a, TQ, CollapsedTimelineState>
where
    TQ: TypeQuery,
{
    type Output<TE> =
        Alive<'t, 'a, TQ, IndeterminedTimelineState<TE>>
    where
        TE: TimeEval;

    #[must_use]
    fn quantize<TE>(mut self, time_eval: TE) -> Self::Output<TE>
    where
        TE: TimeEval,
    {
        let (attached_mobject, CollapsedTimelineState) = self.terminate();
        attached_mobject.launch(IndeterminedTimelineState { time_eval })
    }
}

impl<'t, 'a, TQ, TE, U> Collapse for Alive<'t, 'a, TQ, UpdateTimelineState<TE, U>>
where
    TQ: TypeQuery,
    TE: TimeEval,
    U: Update<TE::OutputTimeMetric, TQ>,
{
    type Output = Alive<'t, 'a, TQ, CollapsedTimelineState>;

    #[must_use]
    fn collapse(mut self) -> Self::Output {
        let (attached_mobject, CollapsedTimelineState) = self.terminate();
        attached_mobject.launch(CollapsedTimelineState)
    }
}

impl<'t, 'a, TQ, TE, C> Collapse for Alive<'t, 'a, TQ, ConstructTimelineState<TE, C>>
where
    TQ: TypeQuery,
    TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    C: Construct<TQ>,
{
    type Output = Alive<'t, 'a, TQ, CollapsedTimelineState>;

    #[must_use]
    fn collapse(mut self) -> Self::Output {
        let (attached_mobject, CollapsedTimelineState) = self.terminate();
        attached_mobject.launch(CollapsedTimelineState)
    }
}

impl<'t, 'a, TQ, TE> ApplyRate<TE::OutputTimeMetric>
    for Alive<'t, 'a, TQ, IndeterminedTimelineState<TE>>
where
    TQ: TypeQuery,
    TE: TimeEval,
{
    type Output<R> =
        Alive<'t, 'a, TQ, IndeterminedTimelineState<RateComposeTimeEval<R, TE>>>
    where
        R: Rate<TE::OutputTimeMetric>;

    #[must_use]
    fn apply_rate<R>(mut self, rate: R) -> Self::Output<R>
    where
        R: Rate<TE::OutputTimeMetric>,
    {
        let (attached_mobject, IndeterminedTimelineState { time_eval }) = self.terminate();
        attached_mobject.launch(IndeterminedTimelineState {
            time_eval: RateComposeTimeEval { rate, time_eval },
        })
    }
}

impl<'t, 'a, TQ, TE> ApplyUpdate<TE::OutputTimeMetric, TQ>
    for Alive<'t, 'a, TQ, IndeterminedTimelineState<TE>>
where
    TQ: TypeQuery,
    TE: TimeEval,
{
    type Output<U> =
        Alive<'t, 'a, TQ, UpdateTimelineState<TE, U>>
    where
        U: Update<TE::OutputTimeMetric, TQ>;

    #[must_use]
    fn apply_update<U>(mut self, update: U) -> Self::Output<U>
    where
        U: Update<TE::OutputTimeMetric, TQ>,
    {
        let (attached_mobject, IndeterminedTimelineState { time_eval }) = self.terminate();
        attached_mobject.launch(UpdateTimelineState { time_eval, update })
    }
}

impl<'t, 'a, TQ> ApplyUpdate<NormalizedTimeMetric, TQ> for Alive<'t, 'a, TQ, CollapsedTimelineState>
where
    TQ: TypeQuery,
{
    type Output<U> =
        Alive<'t, 'a, TQ, CollapsedTimelineState>
    where
        U: Update<NormalizedTimeMetric, TQ>;

    #[must_use]
    fn apply_update<U>(mut self, update: U) -> Self::Output<U>
    where
        U: Update<NormalizedTimeMetric, TQ>,
    {
        let (attached_mobject, CollapsedTimelineState) = self.terminate();
        attached_mobject
            .update(|mobject| {
                update.update(NormalizedTimeMetric(1.0), mobject);
            })
            .launch(CollapsedTimelineState)
    }
}

impl<'t, 'a, TQ, TE> ApplyConstruct<TQ> for Alive<'t, 'a, TQ, IndeterminedTimelineState<TE>>
where
    TQ: TypeQuery,
    TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
{
    type Output<C> =
        Alive<'t, 'a, TQ, ConstructTimelineState<TE, C>>
    where
        C: Construct<TQ>;

    #[must_use]
    fn apply_construct<C>(mut self, construct: C) -> Self::Output<C>
    where
        C: Construct<TQ>,
    {
        let (attached_mobject, IndeterminedTimelineState { time_eval }) = self.terminate();
        attached_mobject.launch(ConstructTimelineState {
            time_eval,
            construct,
        })
    }
}

pub trait QuantizeExt: Quantize {
    fn animate(self) -> Self::Output<NormalizedTimeEval>;
    fn animating(self) -> Self::Output<DenormalizedTimeEval>;
}

impl<TS> QuantizeExt for TS
where
    TS: Quantize,
{
    #[must_use]
    fn animate(self) -> Self::Output<NormalizedTimeEval> {
        self.quantize(NormalizedTimeEval)
    }

    #[must_use]
    fn animating(self) -> Self::Output<DenormalizedTimeEval> {
        self.quantize(DenormalizedTimeEval)
    }
}

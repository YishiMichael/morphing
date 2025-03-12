use core::range::Range;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::timeline::Alive;
use crate::timeline::AttachedMobject;
use crate::timeline::CollapsedTimelineState;
use crate::timeline::TypeQueried;
use crate::timeline::TypeQuery;
use crate::traits::Mobject;
use crate::traits::MobjectBuilder;
use crate::traits::MobjectPresentation;

use super::config::Config;
use super::storable::SlotKeyGeneratorTypeMap;
use super::storable::StorageTypeMap;
use super::timeline::IncreasingTimeEval;
use super::timeline::NormalizedTimeMetric;
use super::timeline::PresentationKey;
use super::timeline::Time;
use super::timeline::TimelineAllocationErasure;
use super::timeline::TimelineErasure;
use super::timeline::Timer;

pub trait Channel: 'static + Sized {
    type MobjectPresentation: Send + Sync;

    type Architecture: ChannelArchitecture<Self::MobjectPresentation>;
    type Archive;
    type Allocation;
    type Prepare;
    type Attachment<'c, W, LI, L, CI>: ChannelAttachment<
        W,
        LI,
        L,
        CI,
        Self,
        Self::MobjectPresentation,
    >
    where
        W: WorldIndexed<LI, Layer = L>,
        LI: LayerIndex,
        L: LayerIndexed<CI, Channel = Self>,
        CI: ChannelIndex;

    fn architecture() -> Self::Architecture;
    fn merge<TE>(
        architecture: &Self::Architecture,
        archive: Self::Archive,
        alive_id: usize,
        time_eval: &TE,
        parent_time_interval: Range<Time>,
        child_time_interval: Range<Time>,
    ) where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>;
    fn archive(architecture: Self::Architecture) -> Self::Archive;
    fn allocation(
        archive: Self::Archive,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Allocation;
    fn prepare(
        allocation: &Self::Allocation,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Prepare;
    fn attachment<'c, W, LI, L, CI>(
        architecture: &'c Self::Architecture,
        config: &'c Config,
        timer: &'c Timer,
        world_architecture: &'c W::Architecture,
        layer_architecture: &'c L::Architecture,
    ) -> Self::Attachment<'c, W, LI, L, CI>
    where
        W: WorldIndexed<LI, Layer = L>,
        LI: LayerIndex,
        L: LayerIndexed<CI, Channel = Self>,
        CI: ChannelIndex;
}

pub trait Layer: 'static + Sized {
    type Architecture;
    type Archive;
    type Allocation;
    type Prepare;
    type Attachment<'l, W, LI>: LayerAttachment<W, LI, Self>
    where
        W: WorldIndexed<LI, Layer = Self>,
        LI: LayerIndex;

    fn architecture() -> Self::Architecture;
    fn merge<TE>(
        architecture: &Self::Architecture,
        archive: Self::Archive,
        alive_id: usize,
        time_eval: &TE,
        parent_time_interval: Range<Time>,
        child_time_interval: Range<Time>,
    ) where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>;
    fn archive(architecture: Self::Architecture) -> Self::Archive;
    fn allocation(
        archive: Self::Archive,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Allocation;
    fn prepare(
        allocation: &Self::Allocation,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Prepare;
    fn render(
        prepare: &Self::Prepare,
        storage_type_map: &StorageTypeMap,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    );
    fn attachment<'l, W, LI>(
        architecture: &'l Self::Architecture,
        config: &'l Config,
        timer: &'l Timer,
        world_architecture: &'l W::Architecture,
    ) -> Self::Attachment<'l, W, LI>
    where
        W: WorldIndexed<LI, Layer = Self>,
        LI: LayerIndex;
}

pub trait World: 'static {
    type Architecture;
    type Archive;
    type Allocation;
    type Prepare;
    type Attachment<'w>;

    fn architecture() -> Self::Architecture;
    fn merge<TE>(
        architecture: &Self::Architecture,
        archive: Self::Archive,
        alive_id: usize,
        time_eval: &TE,
        parent_time_interval: Range<Time>,
        child_time_interval: Range<Time>,
    ) where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>;
    fn archive(architecture: Self::Architecture) -> Self::Archive;
    fn allocation(
        archive: Self::Archive,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Allocation;
    fn prepare(
        allocation: &Self::Allocation,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Prepare;
    fn render(
        prepare: &Self::Prepare,
        storage_type_map: &StorageTypeMap,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    );
    fn attachment<'w>(
        architecture: &'w Self::Architecture,
        config: &'w Config,
        timer: &'w Timer,
    ) -> Self::Attachment<'w>;
}

pub trait ChannelIndex: 'static {}

pub trait LayerIndex: 'static {}

pub struct Idx<const IDX: usize>([(); IDX]); // TODO

impl<const IDX: usize> ChannelIndex for Idx<IDX> {}

impl<const IDX: usize> LayerIndex for Idx<IDX> {}

// pub trait Channel<MP>: Channel {}

pub trait LayerIndexed<CI>: Layer
where
    CI: ChannelIndex,
{
    type Channel: Channel;

    fn index(this: &Self::Architecture) -> &<Self::Channel as Channel>::Architecture;
}

pub trait WorldIndexed<LI>: World
where
    LI: LayerIndex,
{
    type Layer: Layer;

    fn index(this: &Self::Architecture) -> &<Self::Layer as Layer>::Architecture;
}

pub(crate) trait ChannelAttachment<W, LI, L, CI, C, MP>: 'static
where
    W: WorldIndexed<LI, Layer = L>,
    LI: LayerIndex,
    L: LayerIndexed<CI, Channel = C>,
    CI: ChannelIndex,
    C: Channel<MobjectPresentation = MP>,
{
    fn config(&self) -> &Config;
    fn timer(&self) -> &Timer;
    fn world_architecture(&self) -> &W::Architecture;
    fn channel_architecture(&self) -> &C::Architecture;
    // fn spawn<M>(
    //     &self,
    //     mobject: M,
    // ) -> Alive<TypeQueried<W, LI, L, CI, C, M, MP, SKF, Self>, CollapsedTimelineState>
    // where
    //     M: Mobject,
    //     MP: MobjectPresentation<M>;
}

pub(crate) trait LayerAttachment<W, LI, L>: 'static
where
    W: WorldIndexed<LI, Layer = L>,
    LI: LayerIndex,
    L: Layer,
{
    fn config(&self) -> &Config;
    // fn spawn<M>(
    //     &self,
    //     mobject: M,
    // ) -> Alive<TypeQueried<W, LI, L, CI, C, M, MP, SKF, Self>, CollapsedTimelineState>
    // where
    //     M: Mobject,
    //     MP: MobjectPresentation<M>;
}

pub trait Spawn<I> {
    type TypeQuery: TypeQuery;

    fn spawn(&self, input: I) -> Alive<Self::TypeQuery, CollapsedTimelineState>;
}

pub struct ChannelAttachmentImpl<'c, W, LI, L, CI, C, MP>
where
    W: WorldIndexed<LI, Layer = L>,
    LI: LayerIndex,
    L: LayerIndexed<CI, Channel = C>,
    CI: ChannelIndex,
    C: Channel<MobjectPresentation = MP>,
{
    config: &'c Config,
    timer: &'c Timer,
    world_architecture: &'c W::Architecture,
    layer_index: PhantomData<LI>,
    layer_architecture: &'c L::Architecture,
    channel_index: PhantomData<CI>,
    channel_architecture: &'c C::Architecture,
    residue: PhantomData<MP>,
}

pub struct LayerAttachmentImpl<'l, W, LI, L, R>
where
    W: WorldIndexed<LI, Layer = L>,
    LI: LayerIndex,
    L: Layer,
{
    config: &'l Config,
    timer: &'l Timer,
    world_architecture: &'l W::Architecture,
    layer_index: PhantomData<LI>,
    layer_architecture: &'l L::Architecture,
    residue: R,
}

pub struct WorldAttachmentImpl<'w, W, R>
where
    W: World,
{
    config: &'w Config,
    timer: &'w Timer,
    world_architecture: &'w W::Architecture,
    residue: R,
}

impl<W, LI, L, CI, C, MP> ChannelAttachment<W, LI, L, CI, C, MP>
    for ChannelAttachmentImpl<'_, W, LI, L, CI, C, MP>
where
    W: WorldIndexed<LI, Layer = L>,
    LI: LayerIndex,
    L: LayerIndexed<CI, Channel = C>,
    CI: ChannelIndex,
    C: Channel<MobjectPresentation = MP>,
    MP: 'static + Send + Sync,
{
    fn config(&self) -> &Config {
        self.config
    }

    fn timer(&self) -> &Timer {
        self.timer
    }

    fn world_architecture(&self) -> &W::Architecture {
        self.world_architecture
    }

    fn channel_architecture(&self) -> &C::Architecture {
        self.channel_architecture
    }
}

impl<W, LI, L, CI, C, M, MP> Spawn<M> for ChannelAttachmentImpl<'_, W, LI, L, CI, C, MP>
where
    W: WorldIndexed<LI, Layer = L>,
    LI: LayerIndex,
    L: LayerIndexed<CI, Channel = C>,
    CI: ChannelIndex,
    C: Channel<MobjectPresentation = MP>,
    M: Mobject,
    MP: MobjectPresentation<M>,
{
    type TypeQuery = TypeQueried<W, LI, L, CI, C, M, MP, Self>;

    fn spawn(&self, mobject: M) -> Alive<Self::TypeQuery, CollapsedTimelineState> {
        AttachedMobject::new(Arc::new(mobject), self).launch(CollapsedTimelineState)
    }
}

impl<W, LI, L, R> LayerAttachment<W, LI, L> for LayerAttachmentImpl<'_, W, LI, L, R>
where
    W: WorldIndexed<LI, Layer = L>,
    LI: LayerIndex,
    L: Layer,
{
    fn config(&self) -> &Config {
        self.config
    }
}

impl<'l, W, LI, L, R, MB> Spawn<MB> for LayerAttachmentImpl<'_, W, LI, L, R>
where
    W: WorldIndexed<LI, Layer = L>,
    LI: LayerIndex,
    L: Layer<Attachment<'l, W, LI> = Self>,
    MB: MobjectBuilder<L>,
    // M: Mobject,
    // MP: MobjectPresentation<M>,
{
    type TypeQuery = MB::OutputTypeQuery<W, LI>;

    fn spawn(&self, mobject_builder: MB) -> Alive<Self::TypeQuery, CollapsedTimelineState> {
        mobject_builder.instantiate(self, self.config())
    }
}

pub(crate) enum Node<V> {
    Singleton(V),
    Multiton(Vec<V>),
}

impl<V> IntoIterator for Node<V> {
    type Item = V;
    type IntoIter = <Vec<V> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Singleton(v) => vec![v],
            Self::Multiton(vs) => vs,
        }
        .into_iter()
    }
}

pub(crate) trait ChannelArchitecture<MP>
where
    MP: 'static + Send + Sync,
{
    fn new() -> Self;
    fn push(
        &self,
        alive_id: usize,
        node: Node<(
            Range<Time>,
            Box<dyn TimelineErasure<MobjectPresentation = MP>>,
        )>,
    );
    fn archive(
        self,
    ) -> Vec<(
        Range<Time>,
        Box<dyn TimelineErasure<MobjectPresentation = MP>>,
    )>;
}

struct ChannelArchitectureImpl<MP>(
    RefCell<
        Vec<(
            usize,
            Node<(
                Range<Time>,
                Box<dyn TimelineErasure<MobjectPresentation = MP>>,
            )>,
        )>,
    >,
)
where
    MP: 'static + Send + Sync;

impl<MP> ChannelArchitecture<MP> for ChannelArchitectureImpl<MP>
where
    MP: 'static + Send + Sync,
{
    fn new() -> Self {
        Self(RefCell::new(Vec::new()))
    }

    fn push(
        &self,
        alive_id: usize,
        node: Node<(
            Range<Time>,
            Box<dyn TimelineErasure<MobjectPresentation = MP>>,
        )>,
    ) {
        self.0.borrow_mut().push((alive_id, node));
    }

    fn archive(
        self,
    ) -> Vec<(
        Range<Time>,
        Box<dyn TimelineErasure<MobjectPresentation = MP>>,
    )> {
        let mut nodes = self.0.into_inner();
        nodes.sort_by_key(|(alive_id, _)| *alive_id);
        nodes
            .into_iter()
            .flat_map(|(_, timeline)| timeline)
            .collect()
    }
}

pub struct ChannelImpl<MP>(MP);

impl<MP> Channel for ChannelImpl<MP>
where
    MP: 'static + Send + Sync,
{
    type MobjectPresentation = MP;

    type Architecture = ChannelArchitectureImpl<MP>;
    type Archive = Vec<(
        Range<Time>,
        Box<dyn TimelineErasure<MobjectPresentation = MP>>,
    )>;
    type Allocation = Vec<(
        Range<Time>,
        Box<dyn TimelineAllocationErasure<MobjectPresentation = MP>>,
    )>;
    type Prepare = Vec<PresentationKey<MP>>;
    type Attachment<'c, W, LI, L, CI> = ChannelAttachmentImpl<'c, W, LI, L, CI, Self, MP>
    where
        W: WorldIndexed<LI, Layer = L>,
        LI: LayerIndex,
        L: LayerIndexed<CI, Channel = Self>,
        CI: ChannelIndex;

    fn architecture() -> Self::Architecture {
        ChannelArchitectureImpl::new()
    }

    fn merge<TE>(
        architecture: &Self::Architecture,
        archive: Self::Archive,
        alive_id: usize,
        time_eval: &TE,
        parent_time_interval: Range<Time>,
        child_time_interval: Range<Time>,
    ) where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    {
        architecture.push(
            alive_id,
            Node::Multiton(
                archive
                    .into_iter()
                    .map(|(time_interval, animation)| {
                        (
                            Range {
                                start: parent_time_interval.start
                                    + (parent_time_interval.end - parent_time_interval.start)
                                        * *time_eval.time_eval(
                                            time_interval.start,
                                            child_time_interval.clone(),
                                        ),
                                end: parent_time_interval.start
                                    + (parent_time_interval.end - parent_time_interval.start)
                                        * *time_eval.time_eval(
                                            time_interval.end,
                                            child_time_interval.clone(),
                                        ),
                            },
                            animation,
                        )
                    })
                    .collect(),
            ),
        );
    }

    fn archive(architecture: Self::Architecture) -> Self::Archive {
        architecture.archive()
    }

    fn allocation(
        archive: Self::Archive,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Allocation {
        archive
            .into_iter()
            .map(|(time_interval, timeline)| {
                (
                    time_interval,
                    timeline.allocation(slot_key_generator_type_map),
                )
            })
            .collect()
    }

    fn prepare(
        allocation: &Self::Allocation,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Prepare {
        allocation
            .into_iter()
            .filter_map(|(time_interval, timeline)| {
                time_interval.contains(&time).then(|| {
                    timeline.prepare(
                        time,
                        *time_interval,
                        storage_type_map,
                        device,
                        queue,
                        format,
                    )
                })
            })
            .collect()
    }

    fn attachment<'c, W, LI, L, CI>(
        architecture: &'c Self::Architecture,
        config: &'c Config,
        timer: &'c Timer,
        world_architecture: &'c W::Architecture,
        layer_architecture: &'c L::Architecture,
    ) -> Self::Attachment<'c, W, LI, L, CI>
    where
        W: WorldIndexed<LI, Layer = L>,
        LI: LayerIndex,
        L: LayerIndexed<CI, Channel = Self>,
        CI: ChannelIndex,
    {
        ChannelAttachmentImpl {
            config,
            timer,
            world_architecture,
            layer_index: PhantomData,
            layer_architecture,
            channel_index: PhantomData,
            channel_architecture: architecture,
            residue: PhantomData,
        }
    }
}

// impl<MP> Channel<MP> for ChannelImpl<MP> where MP: 'static + Send + Sync {}

// test code
struct MyMobjectPresentation0;
struct MyMobjectPresentation1;

/*
#[derive(Layer)]
pub struct MyLayer {
    pub channel_0: MyMobjectPresentation0,
    pub channel_1: MyMobjectPresentation1,
}
// `render_my_layer` shall be in scope
*/

#[allow(non_camel_case_types)]
pub struct MyLayer<channel_0 = (), channel_1 = ()> {
    pub channel_0: channel_0,
    pub channel_1: channel_1,
}

impl Layer for MyLayer {
    type Architecture = MyLayer<
        <ChannelImpl<MyMobjectPresentation0> as Channel>::Architecture,
        <ChannelImpl<MyMobjectPresentation1> as Channel>::Architecture,
    >;
    type Archive = MyLayer<
        <ChannelImpl<MyMobjectPresentation0> as Channel>::Archive,
        <ChannelImpl<MyMobjectPresentation1> as Channel>::Archive,
    >;
    type Allocation = MyLayer<
        <ChannelImpl<MyMobjectPresentation0> as Channel>::Allocation,
        <ChannelImpl<MyMobjectPresentation1> as Channel>::Allocation,
    >;
    type Prepare = MyLayer<
        <ChannelImpl<MyMobjectPresentation0> as Channel>::Prepare,
        <ChannelImpl<MyMobjectPresentation1> as Channel>::Prepare,
    >;
    type Attachment<'l, W, LI> = LayerAttachmentImpl<'l, W, LI, Self, MyLayer<
        <ChannelImpl<MyMobjectPresentation0> as Channel>::Attachment<'l, W, LI, Self, Idx<0>>,
        <ChannelImpl<MyMobjectPresentation1> as Channel>::Attachment<'l, W, LI, Self, Idx<1>>,
    >>
    where
        W: WorldIndexed<LI, Layer = Self>,
        LI: LayerIndex;

    fn architecture() -> Self::Architecture {
        MyLayer {
            channel_0: <ChannelImpl<MyMobjectPresentation0> as Channel>::architecture(),
            channel_1: <ChannelImpl<MyMobjectPresentation1> as Channel>::architecture(),
        }
    }

    fn merge<TE>(
        architecture: &Self::Architecture,
        archive: Self::Archive,
        alive_id: usize,
        time_eval: &TE,
        parent_time_interval: Range<Time>,
        child_time_interval: Range<Time>,
    ) where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    {
        <ChannelImpl<MyMobjectPresentation0> as Channel>::merge(
            &architecture.channel_0,
            archive.channel_0,
            alive_id,
            time_eval,
            parent_time_interval,
            child_time_interval,
        );
        <ChannelImpl<MyMobjectPresentation1> as Channel>::merge(
            &architecture.channel_1,
            archive.channel_1,
            alive_id,
            time_eval,
            parent_time_interval,
            child_time_interval,
        );
    }

    fn archive(architecture: Self::Architecture) -> Self::Archive {
        MyLayer {
            channel_0: <ChannelImpl<MyMobjectPresentation0> as Channel>::archive(
                architecture.channel_0,
            ),
            channel_1: <ChannelImpl<MyMobjectPresentation1> as Channel>::archive(
                architecture.channel_1,
            ),
        }
    }

    fn allocation(
        archive: Self::Archive,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Allocation {
        MyLayer {
            channel_0: <ChannelImpl<MyMobjectPresentation0> as Channel>::allocation(
                archive.channel_0,
                slot_key_generator_type_map,
            ),
            channel_1: <ChannelImpl<MyMobjectPresentation1> as Channel>::allocation(
                archive.channel_1,
                slot_key_generator_type_map,
            ),
        }
    }

    fn prepare(
        allocation: &Self::Allocation,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Prepare {
        MyLayer {
            channel_0: <ChannelImpl<MyMobjectPresentation0> as Channel>::prepare(
                &allocation.channel_0,
                time,
                storage_type_map,
                device,
                queue,
                format,
            ),
            channel_1: <ChannelImpl<MyMobjectPresentation1> as Channel>::prepare(
                &allocation.channel_1,
                time,
                storage_type_map,
                device,
                queue,
                format,
            ),
        }
    }

    fn render(
        prepare: &Self::Prepare,
        storage_type_map: &StorageTypeMap,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        render_my_layer(prepare, storage_type_map, encoder, target);
    }

    fn attachment<'l, W, LI>(
        architecture: &'l Self::Architecture,
        config: &'l Config,
        timer: &'l Timer,
        world_architecture: &'l W::Architecture,
    ) -> Self::Attachment<'l, W, LI>
    where
        W: WorldIndexed<LI, Layer = Self>,
        LI: LayerIndex,
    {
        LayerAttachmentImpl {
            config,
            timer,
            world_architecture,
            layer_index: PhantomData,
            layer_architecture: architecture,
            residue: MyLayer {
                channel_0: <ChannelImpl<MyMobjectPresentation0> as Channel>::attachment(
                    &architecture.channel_0,
                    config,
                    timer,
                    world_architecture,
                    architecture,
                ),
                channel_1: <ChannelImpl<MyMobjectPresentation1> as Channel>::attachment(
                    &architecture.channel_1,
                    config,
                    timer,
                    world_architecture,
                    architecture,
                ),
            },
        }
    }
}

impl LayerIndexed<Idx<0>> for MyLayer {
    type Channel = ChannelImpl<MyMobjectPresentation0>;

    fn index(this: &Self::Architecture) -> &<Self::Channel as Channel>::Architecture {
        &this.channel_0
    }
}

impl LayerIndexed<Idx<1>> for MyLayer {
    type Channel = ChannelImpl<MyMobjectPresentation1>;

    fn index(this: &Self::Architecture) -> &<Self::Channel as Channel>::Architecture {
        &this.channel_1
    }
}

// pub trait WorldIndexed<const LI: usize>: World {
//     type Layer: Layer;
// }

// impl<SKF, W> MyLayerAttachmentImpl<'_, SKF, W>
// where
//     ,
//     W: WorldErasure,
// {
//     #[must_use]
//     fn spawn<L, MB>(
//         &self,
//         mobject_builder: MB,
//     ) -> Alive<SKF, W, MB::Instantiation, CollapsedAnimationState>
//     where
//         L: LayerErasure<SKF, Attachment = Self>,
//         MB: MobjectBuilder<SKF, L>,
//     {
//         mobject_builder.instantiate(self)
//     }
// }

// hand-written
fn render_my_layer(
    prepared_layer: &<MyLayer as Layer>::Prepare,
    storage_type_map: &StorageTypeMap,
    encoder: &mut wgpu::CommandEncoder,
    target: &wgpu::TextureView,
) {
}

/*
#[derive(World)]
pub struct MyWorld {
    pub layer_0: MyLayer,
    pub layer_1: MyLayer,
}
*/

#[allow(non_camel_case_types)]
pub struct MyWorld<layer_0 = (), layer_1 = ()> {
    pub layer_0: layer_0,
    pub layer_1: layer_1,
}

impl World for MyWorld {
    type Architecture = MyWorld<<MyLayer as Layer>::Architecture, <MyLayer as Layer>::Architecture>;
    type Archive = MyWorld<<MyLayer as Layer>::Archive, <MyLayer as Layer>::Archive>;
    type Allocation = MyWorld<<MyLayer as Layer>::Allocation, <MyLayer as Layer>::Allocation>;
    type Prepare = MyWorld<<MyLayer as Layer>::Prepare, <MyLayer as Layer>::Prepare>;
    type Attachment<'w> = WorldAttachmentImpl<
        'w,
        Self,
        MyWorld<
            <MyLayer as Layer>::Attachment<'w, Self, Idx<0>>,
            <MyLayer as Layer>::Attachment<'w, Self, Idx<1>>,
        >,
    >;

    fn architecture() -> Self::Architecture {
        MyWorld {
            layer_0: <MyLayer as Layer>::architecture(),
            layer_1: <MyLayer as Layer>::architecture(),
        }
    }

    fn merge<TE>(
        architecture: &Self::Architecture,
        archive: Self::Archive,
        alive_id: usize,
        time_eval: &TE,
        parent_time_interval: Range<Time>,
        child_time_interval: Range<Time>,
    ) where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    {
        <MyLayer as Layer>::merge(
            &architecture.layer_0,
            archive.layer_0,
            alive_id,
            time_eval,
            parent_time_interval,
            child_time_interval,
        );
        <MyLayer as Layer>::merge(
            &architecture.layer_1,
            archive.layer_1,
            alive_id,
            time_eval,
            parent_time_interval,
            child_time_interval,
        );
    }

    fn archive(architecture: Self::Architecture) -> Self::Archive {
        MyWorld {
            layer_0: <MyLayer as Layer>::archive(architecture.layer_0),
            layer_1: <MyLayer as Layer>::archive(architecture.layer_1),
        }
    }

    fn allocation(
        archive: Self::Archive,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Allocation {
        MyWorld {
            layer_0: <MyLayer as Layer>::allocation(archive.layer_0, slot_key_generator_type_map),
            layer_1: <MyLayer as Layer>::allocation(archive.layer_1, slot_key_generator_type_map),
        }
    }

    fn prepare(
        allocation: &Self::Allocation,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Prepare {
        MyWorld {
            layer_0: <MyLayer as Layer>::prepare(
                &allocation.layer_0,
                time,
                storage_type_map,
                device,
                queue,
                format,
            ),
            layer_1: <MyLayer as Layer>::prepare(
                &allocation.layer_1,
                time,
                storage_type_map,
                device,
                queue,
                format,
            ),
        }
    }

    fn render(
        prepare: &Self::Prepare,
        storage_type_map: &StorageTypeMap,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        <MyLayer as Layer>::render(&prepare.layer_0, storage_type_map, encoder, target);
        <MyLayer as Layer>::render(&prepare.layer_1, storage_type_map, encoder, target);
    }

    fn attachment<'w>(
        architecture: &'w Self::Architecture,
        config: &'w Config,
        timer: &'w Timer,
    ) -> Self::Attachment<'w> {
        WorldAttachmentImpl {
            config,
            timer,
            world_architecture: architecture,
            residue: MyWorld {
                layer_0: <MyLayer as Layer>::attachment(
                    &architecture.layer_0,
                    config,
                    timer,
                    architecture,
                ),
                layer_1: <MyLayer as Layer>::attachment(
                    &architecture.layer_1,
                    config,
                    timer,
                    architecture,
                ),
            },
        }
    }
}

impl WorldIndexed<Idx<0>> for MyWorld {
    type Layer = MyLayer;

    fn index(this: &Self::Architecture) -> &<Self::Layer as Layer>::Architecture {
        &this.layer_0
    }
}

impl WorldIndexed<Idx<1>> for MyWorld {
    type Layer = MyLayer;

    fn index(this: &Self::Architecture) -> &<Self::Layer as Layer>::Architecture {
        &this.layer_0
    }
}

// #[derive(Debug)]
// struct MySerializableKeyFn;

// impl StorableKeyFn for MySerializableKeyFn {
//     type Output = ();

//     fn eval_key<S>(_serializable: &S) -> Self::Output
//     where
//         S: serde::Serialize,
//     {
//         ()
//     }
// }

// impl World for MyWorld<'_> {
//     type SerializableKeyFn = MySerializableKeyFn;

//     fn architecture(config: &Config, timer_stack: &TimerStack) -> Rc<Self> {
//         Rc::new_cyclic(|world| Self {
//             layer_0: MyWorld::architecture(config, timer_stack, world.clone()),
//             layer_1: MyWorld::architecture(config, timer_stack, world.clone()),
//         })
//     }

//     fn grow_stack(&self) {
//         self.layer_0.grow_stack();
//         self.layer_1.grow_stack();
//     }
//     fn shrink_stack<TE>(&self, time_interval: Range<Time>, time_eval: &TE)
//     where
//         TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
//     {
//         self.layer_0.shrink_stack(time_interval.clone(), time_eval);
//         self.layer_1.shrink_stack(time_interval.clone(), time_eval);
//     }

//     fn collect(self) -> Vec<Box<dyn LayerPreallocated>> {
//         Vec::from([
//             Box::architecture(self.layer_0.collect()) as Box<dyn LayerPreallocated>,
//             Box::architecture(self.layer_1.collect()) as Box<dyn LayerPreallocated>,
//         ])
//     }
// }
// end test code

// trait StageMapper<O>
// where
//     O: Object,
// {
//     type Target;
// }

// struct AttachedStage;
// struct ArchivedStage;
// struct AllocatedStage;
// struct PreparedStage;
// struct AttachmentStage<'c, C>(&'c C);

// impl<O> StageMapper<O> for AttachedStage
// where
//     O: Object,
// {
//     type Target = O::Attached;
// }

// impl<O> StageMapper<O> for ArchivedStage
// where
//     O: Object,
// {
//     type Target = O::Archived;
// }

// impl<O> StageMapper<O> for AllocatedStage
// where
//     O: Object,
// {
//     type Target = O::Allocated;
// }

// impl<O> StageMapper<O> for PreparedStage
// where
//     O: Object,
// {
//     type Target = O::Prepared;
// }

// impl<'c, O, C> StageMapper<O> for AttachmentStage<'c, C>
// where
//     O: 'c + Object,
// {
//     type Target = O::Attachment<'c, C>;
// }

// struct At<O, SM>(SM::Target)
// where
//     O: Object,
//     SM: StageMapper<O>;

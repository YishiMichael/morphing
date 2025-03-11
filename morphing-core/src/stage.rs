use core::range::Range;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use super::config::Config;
use super::storable::SlotKeyGeneratorTypeMap;
use super::storable::StorableKeyFn;
use super::storable::StorageTypeMap;
use super::timeline::Alive;
use super::timeline::IncreasingTimeEval;
use super::timeline::MobjectLocate;
use super::timeline::NormalizedTimeMetric;
use super::timeline::PresentationKey;
use super::timeline::Time;
use super::timeline::TimelineAllocationErasure;
use super::timeline::TimelineErasure;
use super::timeline::TimelineState;
use super::timeline::Timer;
use super::traits::Mobject;

pub trait Channel: 'static {
    type Architecture<SKF>
    where
        SKF: StorableKeyFn;
    type Archive<SKF>
    where
        SKF: StorableKeyFn;
    type Allocation<SKF>
    where
        SKF: StorableKeyFn;
    type Prepare<SKF>
    where
        SKF: StorableKeyFn;
    type Attachment<'c, W, LI, L, CI, SKF>
    where
        W: WorldIndexed<LI, Layer = L>,
        L: LayerIndexed<CI, Channel = Self>,
        SKF: StorableKeyFn;

    fn architecture<SKF>() -> Self::Architecture<SKF>
    where
        SKF: StorableKeyFn;
    fn merge<TE, SKF>(
        architecture: &Self::Architecture<SKF>,
        archive: Self::Archive<SKF>,
        alive_id: usize,
        time_eval: &TE,
        parent_time_interval: Range<Time>,
        child_time_interval: Range<Time>,
    ) where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
        SKF: StorableKeyFn;
    fn archive<SKF>(architecture: Self::Architecture<SKF>) -> Self::Archive<SKF>
    where
        SKF: StorableKeyFn;
    fn allocation<SKF>(
        archive: Self::Archive<SKF>,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Allocation<SKF>
    where
        SKF: StorableKeyFn;
    fn prepare<SKF>(
        allocation: &Self::Allocation<SKF>,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Prepare<SKF>
    where
        SKF: StorableKeyFn;
    fn attachment<'c, W, LI, L, CI, SKF>(
        architecture: &'c Self::Architecture<SKF>,
        config: &'c Config,
        timer: &'c Timer,
        world_architecture: &'c W::Architecture<SKF>,
        layer_architecture: &'c L::Architecture<SKF>,
    ) -> Self::Attachment<'c, W, LI, L, CI, SKF>
    where
        W: WorldIndexed<LI, Layer = L>,
        L: LayerIndexed<CI, Channel = Self>,
        SKF: StorableKeyFn;
}

pub trait Layer: 'static {
    type Architecture<SKF>
    where
        SKF: StorableKeyFn;
    type Archive<SKF>
    where
        SKF: StorableKeyFn;
    type Allocation<SKF>
    where
        SKF: StorableKeyFn;
    type Prepare<SKF>
    where
        SKF: StorableKeyFn;
    type Attachment<'l, W, LI, SKF>
    where
        W: WorldIndexed<LI, Layer = Self>,
        SKF: StorableKeyFn;

    fn architecture<SKF>() -> Self::Architecture<SKF>
    where
        SKF: StorableKeyFn;
    fn merge<TE, SKF>(
        architecture: &Self::Architecture<SKF>,
        archive: Self::Archive<SKF>,
        alive_id: usize,
        time_eval: &TE,
        parent_time_interval: Range<Time>,
        child_time_interval: Range<Time>,
    ) where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
        SKF: StorableKeyFn;
    fn archive<SKF>(architecture: Self::Architecture<SKF>) -> Self::Archive<SKF>
    where
        SKF: StorableKeyFn;
    fn allocation<SKF>(
        archive: Self::Archive<SKF>,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Allocation<SKF>
    where
        SKF: StorableKeyFn;
    fn prepare<SKF>(
        allocation: &Self::Allocation<SKF>,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Prepare<SKF>
    where
        SKF: StorableKeyFn;
    fn render<SKF>(
        prepare: &Self::Prepare<SKF>,
        storage_type_map: &StorageTypeMap,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) where
        SKF: StorableKeyFn;
    fn attachment<'l, W, LI, SKF>(
        architecture: &'l Self::Architecture<SKF>,
        config: &'l Config,
        timer: &'l Timer,
        world_architecture: &'l W::Architecture<SKF>,
    ) -> Self::Attachment<'l, W, LI, SKF>
    where
        W: WorldIndexed<LI, Layer = Self>,
        SKF: StorableKeyFn;
}

pub trait World: 'static {
    type Architecture<SKF>
    where
        SKF: StorableKeyFn;
    type Archive<SKF>
    where
        SKF: StorableKeyFn;
    type Allocation<SKF>
    where
        SKF: StorableKeyFn;
    type Prepare<SKF>
    where
        SKF: StorableKeyFn;
    type Attachment<'w, SKF>
    where
        SKF: StorableKeyFn;

    fn architecture<SKF>() -> Self::Architecture<SKF>
    where
        SKF: StorableKeyFn;
    fn merge<TE, SKF>(
        architecture: &Self::Architecture<SKF>,
        archive: Self::Archive<SKF>,
        alive_id: usize,
        time_eval: &TE,
        parent_time_interval: Range<Time>,
        child_time_interval: Range<Time>,
    ) where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
        SKF: StorableKeyFn;
    fn archive<SKF>(architecture: Self::Architecture<SKF>) -> Self::Archive<SKF>
    where
        SKF: StorableKeyFn;
    fn allocation<SKF>(
        archive: Self::Archive<SKF>,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Allocation<SKF>
    where
        SKF: StorableKeyFn;
    fn prepare<SKF>(
        allocation: &Self::Allocation<SKF>,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Prepare<SKF>
    where
        SKF: StorableKeyFn;
    fn render<SKF>(
        prepare: &Self::Prepare<SKF>,
        storage_type_map: &StorageTypeMap,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) where
        SKF: StorableKeyFn;
    fn attachment<'w, SKF>(
        architecture: &'w Self::Architecture<SKF>,
        config: &'w Config,
        timer: &'w Timer,
    ) -> Self::Attachment<'w, SKF>
    where
        SKF: StorableKeyFn;
}

pub trait LayerIndexed<CI>: Layer {
    type Channel: Channel;

    fn index<SKF>(this: &Self::Architecture<SKF>) -> &<Self::Channel as Channel>::Architecture<SKF>
    where
        SKF: StorableKeyFn;
}

pub trait WorldIndexed<LI>: World {
    type Layer: Layer;

    fn index<SKF>(this: &Self::Architecture<SKF>) -> &<Self::Layer as Layer>::Architecture<SKF>
    where
        SKF: StorableKeyFn;
}

pub struct ChannelAttachment<'c, W, LI, L, CI, C, SKF>
where
    W: WorldIndexed<LI, Layer = L>,
    L: LayerIndexed<CI, Channel = C>,
    C: Channel,
    SKF: StorableKeyFn,
{
    config: &'c Config,
    timer: &'c Timer,
    world_architecture: &'c W::Architecture<SKF>,
    layer_index: PhantomData<LI>,
    layer_architecture: &'c L::Architecture<SKF>,
    channel_index: PhantomData<CI>,
    channel_architecture: &'c C::Architecture<SKF>,
}

pub struct LayerAttachment<'l, W, LI, L, R, SKF>
where
    W: WorldIndexed<LI, Layer = L>,
    L: Layer,
    SKF: StorableKeyFn,
{
    config: &'l Config,
    timer: &'l Timer,
    world_architecture: &'l W::Architecture<SKF>,
    layer_index: PhantomData<LI>,
    layer_architecture: &'l L::Architecture<SKF>,
    residue: R,
}

pub struct WorldAttachment<'w, W, R, SKF>
where
    W: World,
    SKF: StorableKeyFn,
{
    config: &'w Config,
    timer: &'w Timer,
    world_architecture: &'w W::Architecture<SKF>,
    residue: R,
}

enum Node<V> {
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

struct ChannelArchitecture<MP, SKF>(
    RefCell<
        Vec<(
            usize,
            Node<(
                Range<Time>,
                Box<dyn TimelineErasure<SKF, MobjectPresentation = MP>>,
            )>,
        )>,
    >,
)
where
    MP: 'static + Send + Sync,
    SKF: StorableKeyFn;

impl<MP, SKF> ChannelArchitecture<MP, SKF>
where
    MP: 'static + Send + Sync,
    SKF: StorableKeyFn,
{
    fn new() -> Self {
        Self(RefCell::new(Vec::new()))
    }

    fn push(
        &self,
        alive_id: usize,
        node: Node<(
            Range<Time>,
            Box<dyn TimelineErasure<SKF, MobjectPresentation = MP>>,
        )>,
    ) {
        self.0.borrow_mut().push((alive_id, node));
    }

    fn archive(
        self,
    ) -> Vec<(
        Range<Time>,
        Box<dyn TimelineErasure<SKF, MobjectPresentation = MP>>,
    )> {
        let mut nodes = self.0.into_inner();
        nodes.sort_by_key(|(alive_id, _)| *alive_id);
        nodes
            .into_iter()
            .flat_map(|(_, timeline)| timeline)
            .collect()
    }
}

impl<W, LI, L, CI, MP, SKF> ChannelAttachment<'_, W, LI, L, CI, PresentationChannel<MP>, SKF>
where
    W: WorldIndexed<LI, Layer = L>,
    L: LayerIndexed<CI, Channel = PresentationChannel<MP>>,
    MP: 'static + Send + Sync,
    SKF: StorableKeyFn,
{
    pub(crate) fn config(&self) -> &Config {
        self.config
    }

    pub(crate) fn timer(&self) -> &Timer {
        &self.timer
    }

    pub(crate) fn push(
        &self,
        alive_id: usize,
        time_interval: Range<Rc<Time>>,
        timeline: Box<dyn TimelineErasure<SKF, MobjectPresentation = MP>>,
    ) {
        if !Rc::ptr_eq(&time_interval.start, &time_interval.end) {
            self.channel_architecture.push(
                alive_id,
                Node::Singleton((
                    Range {
                        start: *time_interval.start,
                        end: *time_interval.end,
                    },
                    timeline,
                )),
            );
        }
    }

    pub(crate) fn extend<TE>(
        &self,
        alive_id: usize,
        time_interval: Range<Rc<Time>>,
        time_eval: &TE,
        world_time: Time,
        world_architecture: W::Architecture<SKF>,
    ) where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    {
        if !Rc::ptr_eq(&time_interval.start, &time_interval.end) {
            W::merge(
                self.world_architecture,
                W::archive(world_architecture),
                alive_id,
                time_eval,
                Range {
                    start: *time_interval.start,
                    end: *time_interval.end,
                },
                Range {
                    start: 0.0,
                    end: world_time,
                },
            );
        }
    }

    pub(crate) fn start<ML, TS>(&self, mobject_locate: ML, timeline_state: TS) -> Alive<ML, TS, SKF>
    where
        ML: MobjectLocate<
            World = W,
            LayerIndex = LI,
            Layer = L,
            ChannelIndex = CI,
            Channel = PresentationChannel<MP>,
        >,
        <ML as MobjectLocate>::Mobject: Mobject<L, MobjectPresentation = MP>,
        TS: TimelineState<ML>,
    {
        Alive {
            alive_id: self.timer().generate_alive_id(),
            spawn_time: self.timer().time(),
            channel_attachment: self,
            mobject_locate,
            timeline_state: Some(timeline_state),
        }
    }

    // pub fn spawn_mobject<M>(&self, mobject: Box<M>) -> Alive<MobjectLocate<>>
}

pub struct PresentationChannel<MP>(MP);

impl<MP> Channel for PresentationChannel<MP>
where
    MP: 'static + Send + Sync,
{
    type Architecture<SKF> = ChannelArchitecture<MP, SKF>
    where
        SKF: StorableKeyFn;
    type Archive<SKF> = Vec<(Range<Time>, Box<dyn TimelineErasure<SKF, MobjectPresentation = MP>>)>
    where
        SKF: StorableKeyFn;
    type Allocation<SKF> = Vec<(Range<Time>, Box<dyn TimelineAllocationErasure<SKF, MobjectPresentation = MP>>)>
    where
        SKF: StorableKeyFn;
    type Prepare<SKF> = Vec<PresentationKey<MP, SKF>>
    where
        SKF: StorableKeyFn;
    type Attachment<'c, W, LI, L, CI, SKF> = ChannelAttachment<'c, W, LI, L, CI, PresentationChannel<MP>, SKF>
    where
        W: WorldIndexed<LI, Layer = L>,
        L: LayerIndexed<CI, Channel = Self>,
        SKF: StorableKeyFn;

    fn architecture<SKF>() -> Self::Architecture<SKF>
    where
        SKF: StorableKeyFn,
    {
        ChannelArchitecture::new()
    }

    fn merge<TE, SKF>(
        architecture: &Self::Architecture<SKF>,
        archive: Self::Archive<SKF>,
        alive_id: usize,
        time_eval: &TE,
        parent_time_interval: Range<Time>,
        child_time_interval: Range<Time>,
    ) where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
        SKF: StorableKeyFn,
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

    fn archive<SKF>(architecture: Self::Architecture<SKF>) -> Self::Archive<SKF>
    where
        SKF: StorableKeyFn,
    {
        architecture.archive()
    }

    fn allocation<SKF>(
        archive: Self::Archive<SKF>,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Allocation<SKF>
    where
        SKF: StorableKeyFn,
    {
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

    fn prepare<SKF>(
        allocation: &Self::Allocation<SKF>,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Prepare<SKF>
    where
        SKF: StorableKeyFn,
    {
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

    fn attachment<'c, W, LI, L, CI, SKF>(
        architecture: &'c Self::Architecture<SKF>,
        config: &'c Config,
        timer: &'c Timer,
        world_architecture: &'c W::Architecture<SKF>,
        layer_architecture: &'c L::Architecture<SKF>,
    ) -> Self::Attachment<'c, W, LI, L, CI, SKF>
    where
        W: WorldIndexed<LI, Layer = L>,
        L: LayerIndexed<CI, Channel = Self>,
        SKF: StorableKeyFn,
    {
        ChannelAttachment {
            config,
            timer,
            world_architecture,
            layer_index: PhantomData,
            layer_architecture,
            channel_index: PhantomData,
            channel_architecture: architecture,
        }
    }
}

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
    type Architecture<SKF> = MyLayer<
        <PresentationChannel<MyMobjectPresentation0> as Channel>::Architecture<SKF>,
        <PresentationChannel<MyMobjectPresentation1> as Channel>::Architecture<SKF>,
    >
    where
        SKF: StorableKeyFn;
    type Archive<SKF> = MyLayer<
        <PresentationChannel<MyMobjectPresentation0> as Channel>::Archive<SKF>,
        <PresentationChannel<MyMobjectPresentation1> as Channel>::Archive<SKF>,
    >
    where
        SKF: StorableKeyFn;
    type Allocation<SKF> = MyLayer<
        <PresentationChannel<MyMobjectPresentation0> as Channel>::Allocation<SKF>,
        <PresentationChannel<MyMobjectPresentation1> as Channel>::Allocation<SKF>,
    >
    where
        SKF: StorableKeyFn;
    type Prepare<SKF> = MyLayer<
        <PresentationChannel<MyMobjectPresentation0> as Channel>::Prepare<SKF>,
        <PresentationChannel<MyMobjectPresentation1> as Channel>::Prepare<SKF>,
    >
    where
        SKF: StorableKeyFn;
    type Attachment<'l, W, LI, SKF> = LayerAttachment<'l, W, LI, MyLayer, MyLayer<
        <PresentationChannel<MyMobjectPresentation0> as Channel>::Attachment<'l, W, LI, MyLayer, [(); 0], SKF>,
        <PresentationChannel<MyMobjectPresentation1> as Channel>::Attachment<'l, W, LI, MyLayer, [(); 1], SKF>,
    >, SKF>
    where
        W: WorldIndexed<LI, Layer = Self>,
        SKF: StorableKeyFn;

    fn architecture<SKF>() -> Self::Architecture<SKF>
    where
        SKF: StorableKeyFn,
    {
        MyLayer {
            channel_0: <PresentationChannel<MyMobjectPresentation0> as Channel>::architecture(),
            channel_1: <PresentationChannel<MyMobjectPresentation1> as Channel>::architecture(),
        }
    }

    fn merge<TE, SKF>(
        architecture: &Self::Architecture<SKF>,
        archive: Self::Archive<SKF>,
        alive_id: usize,
        time_eval: &TE,
        parent_time_interval: Range<Time>,
        child_time_interval: Range<Time>,
    ) where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
        SKF: StorableKeyFn,
    {
        <PresentationChannel<MyMobjectPresentation0> as Channel>::merge(
            &architecture.channel_0,
            archive.channel_0,
            alive_id,
            time_eval,
            parent_time_interval,
            child_time_interval,
        );
        <PresentationChannel<MyMobjectPresentation1> as Channel>::merge(
            &architecture.channel_1,
            archive.channel_1,
            alive_id,
            time_eval,
            parent_time_interval,
            child_time_interval,
        );
    }

    fn archive<SKF>(architecture: Self::Architecture<SKF>) -> Self::Archive<SKF>
    where
        SKF: StorableKeyFn,
    {
        MyLayer {
            channel_0: <PresentationChannel<MyMobjectPresentation0> as Channel>::archive(
                architecture.channel_0,
            ),
            channel_1: <PresentationChannel<MyMobjectPresentation1> as Channel>::archive(
                architecture.channel_1,
            ),
        }
    }

    fn allocation<SKF>(
        archive: Self::Archive<SKF>,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Allocation<SKF>
    where
        SKF: StorableKeyFn,
    {
        MyLayer {
            channel_0: <PresentationChannel<MyMobjectPresentation0> as Channel>::allocation(
                archive.channel_0,
                slot_key_generator_type_map,
            ),
            channel_1: <PresentationChannel<MyMobjectPresentation1> as Channel>::allocation(
                archive.channel_1,
                slot_key_generator_type_map,
            ),
        }
    }

    fn prepare<SKF>(
        allocation: &Self::Allocation<SKF>,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Prepare<SKF>
    where
        SKF: StorableKeyFn,
    {
        MyLayer {
            channel_0: <PresentationChannel<MyMobjectPresentation0> as Channel>::prepare(
                &allocation.channel_0,
                time,
                storage_type_map,
                device,
                queue,
                format,
            ),
            channel_1: <PresentationChannel<MyMobjectPresentation1> as Channel>::prepare(
                &allocation.channel_1,
                time,
                storage_type_map,
                device,
                queue,
                format,
            ),
        }
    }

    fn render<SKF>(
        prepare: &Self::Prepare<SKF>,
        storage_type_map: &StorageTypeMap,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) where
        SKF: StorableKeyFn,
    {
        render_my_layer(prepare, storage_type_map, encoder, target);
    }

    fn attachment<'l, W, LI, SKF>(
        architecture: &'l Self::Architecture<SKF>,
        config: &'l Config,
        timer: &'l Timer,
        world_architecture: &'l W::Architecture<SKF>,
    ) -> Self::Attachment<'l, W, LI, SKF>
    where
        W: WorldIndexed<LI, Layer = Self>,
        SKF: StorableKeyFn,
    {
        LayerAttachment {
            config,
            timer,
            world_architecture,
            layer_index: PhantomData,
            layer_architecture: architecture,
            residue: MyLayer {
                channel_0: <PresentationChannel<MyMobjectPresentation0> as Channel>::attachment(
                    &architecture.channel_0,
                    config,
                    timer,
                    world_architecture,
                    architecture,
                ),
                channel_1: <PresentationChannel<MyMobjectPresentation1> as Channel>::attachment(
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

impl LayerIndexed<[(); 0]> for MyLayer {
    type Channel = PresentationChannel<MyMobjectPresentation0>;

    fn index<SKF>(this: &Self::Architecture<SKF>) -> &<Self::Channel as Channel>::Architecture<SKF>
    where
        SKF: StorableKeyFn,
    {
        &this.channel_0
    }
}

impl LayerIndexed<[(); 1]> for MyLayer {
    type Channel = PresentationChannel<MyMobjectPresentation1>;

    fn index<SKF>(this: &Self::Architecture<SKF>) -> &<Self::Channel as Channel>::Architecture<SKF>
    where
        SKF: StorableKeyFn,
    {
        &this.channel_1
    }
}

// pub trait WorldIndexed<const LI: usize>: World {
//     type Layer: Layer;
// }

// impl<SKF, W> MyLayerAttachment<'_, SKF, W>
// where
//     SKF: StorableKeyFn,
//     W: WorldErasure<SKF>,
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

// hand-written, privide name
fn render_my_layer<SKF>(
    prepared_layer: &<MyLayer as Layer>::Prepare<SKF>,
    storage_type_map: &StorageTypeMap,
    encoder: &mut wgpu::CommandEncoder,
    target: &wgpu::TextureView,
) where
    SKF: StorableKeyFn,
{
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
    type Architecture<SKF> = MyWorld<
        <MyLayer as Layer>::Architecture<SKF>,
        <MyLayer as Layer>::Architecture<SKF>,
    >
    where
        SKF: StorableKeyFn;
    type Archive<SKF> = MyWorld<
        <MyLayer as Layer>::Archive<SKF>,
        <MyLayer as Layer>::Archive<SKF>,
    >
    where
        SKF: StorableKeyFn;
    type Allocation<SKF> = MyWorld<
        <MyLayer as Layer>::Allocation<SKF>,
        <MyLayer as Layer>::Allocation<SKF>,
    >
    where
        SKF: StorableKeyFn;
    type Prepare<SKF> = MyWorld<
        <MyLayer as Layer>::Prepare<SKF>,
        <MyLayer as Layer>::Prepare<SKF>,
    >
    where
        SKF: StorableKeyFn;
    type Attachment<'w, SKF> = WorldAttachment<'w, MyWorld, MyWorld<
        <MyLayer as Layer>::Attachment<'w, MyWorld, [(); 0], SKF>,
        <MyLayer as Layer>::Attachment<'w, MyWorld, [(); 1], SKF>,
    >, SKF>
    where
        SKF: StorableKeyFn;

    fn architecture<SKF>() -> Self::Architecture<SKF>
    where
        SKF: StorableKeyFn,
    {
        MyWorld {
            layer_0: <MyLayer as Layer>::architecture(),
            layer_1: <MyLayer as Layer>::architecture(),
        }
    }

    fn merge<TE, SKF>(
        architecture: &Self::Architecture<SKF>,
        archive: Self::Archive<SKF>,
        alive_id: usize,
        time_eval: &TE,
        parent_time_interval: Range<Time>,
        child_time_interval: Range<Time>,
    ) where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
        SKF: StorableKeyFn,
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

    fn archive<SKF>(architecture: Self::Architecture<SKF>) -> Self::Archive<SKF>
    where
        SKF: StorableKeyFn,
    {
        MyWorld {
            layer_0: <MyLayer as Layer>::archive(architecture.layer_0),
            layer_1: <MyLayer as Layer>::archive(architecture.layer_1),
        }
    }

    fn allocation<SKF>(
        archive: Self::Archive<SKF>,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Allocation<SKF>
    where
        SKF: StorableKeyFn,
    {
        MyWorld {
            layer_0: <MyLayer as Layer>::allocation(archive.layer_0, slot_key_generator_type_map),
            layer_1: <MyLayer as Layer>::allocation(archive.layer_1, slot_key_generator_type_map),
        }
    }

    fn prepare<SKF>(
        allocation: &Self::Allocation<SKF>,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Prepare<SKF>
    where
        SKF: StorableKeyFn,
    {
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

    fn render<SKF>(
        prepare: &Self::Prepare<SKF>,
        storage_type_map: &StorageTypeMap,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) where
        SKF: StorableKeyFn,
    {
        <MyLayer as Layer>::render(&prepare.layer_0, storage_type_map, encoder, target);
        <MyLayer as Layer>::render(&prepare.layer_1, storage_type_map, encoder, target);
    }

    fn attachment<'w, SKF>(
        architecture: &'w Self::Architecture<SKF>,
        config: &'w Config,
        timer: &'w Timer,
    ) -> Self::Attachment<'w, SKF>
    where
        SKF: StorableKeyFn,
    {
        WorldAttachment {
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

impl WorldIndexed<[(); 0]> for MyWorld {
    type Layer = MyLayer;

    fn index<SKF>(this: &Self::Architecture<SKF>) -> &<Self::Layer as Layer>::Architecture<SKF>
    where
        SKF: StorableKeyFn,
    {
        &this.layer_0
    }
}

impl WorldIndexed<[(); 1]> for MyWorld {
    type Layer = MyLayer;

    fn index<SKF>(this: &Self::Architecture<SKF>) -> &<Self::Layer as Layer>::Architecture<SKF>
    where
        SKF: StorableKeyFn,
    {
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

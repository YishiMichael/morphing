use core::range::Range;
use std::cell::RefCell;
use std::rc::Rc;

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
use super::traits::StorableKeyFn;

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
    type Attachment<'c, WA, LA, CA>
    where
        Self: 'c,
        WA: 'c,
        LA: 'c,
        CA: 'c;

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
    fn attachment<'c, WA, LA, SKF>(
        architecture: &'c Self::Architecture<SKF>,
        config: &Config,
        timer: &Timer,
        world_architecture: &WA,
        layer_architecture: &LA,
    ) -> Self::Attachment<'c, WA, LA, Self::Architecture<SKF>>
    where
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
    type Attachment<'l, WA, LA>
    where
        Self: 'l,
        WA: 'l,
        LA: 'l;

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
    fn attachment<'l, WA, SKF>(
        architecture: &'l Self::Architecture<SKF>,
        config: &Config,
        timer: &Timer,
        world_architecture: &WA,
    ) -> Self::Attachment<'l, WA, Self::Architecture<SKF>>
    where
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
    type Attachment<'w, WA>
    where
        Self: 'w,
        WA: 'w;

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
        config: &Config,
        timer: &Timer,
    ) -> Self::Attachment<'w, Self::Architecture<SKF>>
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

pub struct ChannelAttachment<'c, WA, LA, CA> {
    config: &'c Config,
    timer: &'c Timer,
    world_architecture: &'c WA,
    layer_architecture: &'c LA,
    channel_architecture: &'c CA,
}

pub struct LayerAttachment<'c, WA, LA, R> {
    config: &'c Config,
    timer: &'c Timer,
    world_architecture: &'c WA,
    layer_architecture: &'c LA,
    residue: R,
}

pub struct WorldAttachment<'c, WA, R> {
    config: &'c Config,
    timer: &'c Timer,
    world_architecture: &'c WA,
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
        nodes.sort_by_key(|(alive_id, _)| alive_id);
        nodes
            .into_iter()
            .flat_map(|(_, timeline)| timeline)
            .collect()
    }
}

impl<WA, LA, MP, SKF> ChannelAttachment<'_, WA, LA, ChannelArchitecture<MP, SKF>>
where
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
                Node::Singleton((*time_interval.start..*time_interval.end, timeline)),
            );
        }
    }

    pub(crate) fn extend<W, TE>(
        &self,
        alive_id: usize,
        time_interval: Range<Rc<Time>>,
        time_eval: &TE,
        world_time: Time,
        world_architecture: WA,
    ) where
        W: World<Architecture<SKF> = WA>,
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    {
        if !Rc::ptr_eq(&time_interval.start, &time_interval.end) {
            World::merge(
                self.world_architecture,
                World::archive(world_architecture),
                alive_id,
                time_eval,
                *time_interval.start..*time_interval.end,
                0.0..world_time,
            );
        }
    }
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
    type Attachment<'c, WA, LA, CA> = ChannelAttachment<'c, WA, LA, CA>
    where
        Self: 'c,
        WA: 'c,
        LA: 'c,
        CA: 'c;

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
                            parent_time_interval.start
                                + (parent_time_interval.end - parent_time_interval.start)
                                    * *time_eval
                                        .time_eval(time_interval.start, child_time_interval.clone())
                                ..parent_time_interval.start
                                    + (parent_time_interval.end - parent_time_interval.start)
                                        * *time_eval.time_eval(
                                            time_interval.end,
                                            child_time_interval.clone(),
                                        ),
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
        archive.into_iter().map(|(time_interval, timeline)| {
            (
                time_interval,
                timeline.allocation(slot_key_generator_type_map),
            )
        })
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
            .flatten()
            .collect()
    }

    fn attachment<'c, WA, LA, SKF>(
        architecture: &'c Self::Architecture<SKF>,
        config: &Config,
        timer: &Timer,
        world_architecture: &WA,
        layer_architecture: &LA,
    ) -> Self::Attachment<'c, WA, LA, Self::Architecture<SKF>>
    where
        SKF: StorableKeyFn,
    {
        ChannelAttachment {
            config,
            timer,
            world_architecture,
            layer_architecture,
            channel_architecture: architecture,
        }
    }
}

// test code
struct MyMobjectPresentation0;
struct MyMobjectPresentation1;

/*
#[derive(Layer)]
struct MyLayer {
    channel_0: MyMobjectPresentation0,
    channel_1: MyMobjectPresentation1,
}
// `render_my_layer` shall be in scope
*/

#[allow(non_camel_case_types)]
struct MyLayer<channel_0 = (), channel_1 = ()> {
    channel_0: channel_0,
    channel_1: channel_1,
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
    type Attachment<'l, WA, LA> = LayerAttachment<'l, WA, LA, MyLayer<
        <PresentationChannel<MyMobjectPresentation0> as Channel>::Attachment<'l, WA, LA, PresentationChannel<MyMobjectPresentation0>>,
        <PresentationChannel<MyMobjectPresentation1> as Channel>::Attachment<'l, WA, LA, PresentationChannel<MyMobjectPresentation1>>,
    >>
    where
        Self: 'l,
        WA: 'l,
        LA: 'l;

    fn architecture<SKF>() -> Self::Architecture<SKF>
    where
        SKF: StorableKeyFn,
    {
        MyLayer {
            channel_0: Channel::architecture(),
            channel_1: Channel::architecture(),
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
        Channel::merge(
            architecture.channel_0,
            archive.channel_0,
            alive_id,
            time_eval,
            parent_time_interval,
            child_time_interval,
        );
        Channel::merge(
            architecture.channel_1,
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
            channel_0: Channel::archive(architecture.channel_0),
            channel_1: Channel::archive(architecture.channel_1),
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
            channel_0: Channel::allocation(archive.channel_0, slot_key_generator_type_map),
            channel_1: Channel::allocation(archive.channel_1, slot_key_generator_type_map),
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
            channel_0: Channel::prepare(
                allocation.channel_0,
                time,
                storage_type_map,
                device,
                queue,
                format,
            ),
            channel_1: Channel::prepare(
                allocation.channel_1,
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

    fn attachment<'l, WA, SKF>(
        architecture: &'l Self::Architecture<SKF>,
        config: &Config,
        timer: &Timer,
        world_architecture: &WA,
    ) -> Self::Attachment<'l, WA, Self::Architecture<SKF>>
    where
        SKF: StorableKeyFn,
    {
        LayerAttachment {
            config,
            timer,
            world_architecture,
            layer: architecture,
            residue: MyLayer {
                channel_0: Channel::attachment(
                    architecture.channel_0,
                    config,
                    timer,
                    world_architecture,
                    architecture,
                ),
                channel_1: Channel::attachment(
                    architecture.channel_1,
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
        this.channel_0
    }
}

impl LayerIndexed<[(); 1]> for MyLayer {
    type Channel = PresentationChannel<MyMobjectPresentation1>;

    fn index<SKF>(this: &Self::Architecture<SKF>) -> &<Self::Channel as Channel>::Architecture<SKF>
    where
        SKF: StorableKeyFn,
    {
        this.channel_1
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
struct MyWorld {
    layer_0: MyLayer,
    layer_1: MyLayer,
}
*/

#[allow(non_camel_case_types)]
struct MyWorld<layer_0 = (), layer_1 = ()> {
    layer_0: layer_0,
    layer_1: layer_1,
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
    type Attachment<'w, WA> = WorldAttachment<'w, WA, MyWorld<
        <MyLayer as Layer>::Attachment<'w, WA, MyLayer>,
        <MyLayer as Layer>::Attachment<'w, WA, MyLayer>,
    >>
    where
        Self: 'w,
        WA: 'w;

    fn architecture<SKF>() -> Self::Architecture<SKF>
    where
        SKF: StorableKeyFn,
    {
        MyWorld {
            layer_0: Layer::architecture(),
            layer_1: Layer::architecture(),
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
        Layer::merge(
            architecture.layer_0,
            archive.layer_0,
            alive_id,
            time_eval,
            parent_time_interval,
            child_time_interval,
        );
        Layer::merge(
            architecture.layer_1,
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
            layer_0: Layer::archive(architecture.layer_0),
            layer_1: Layer::archive(architecture.layer_1),
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
            layer_0: Layer::allocation(archive.layer_0, slot_key_generator_type_map),
            layer_1: Layer::allocation(archive.layer_1, slot_key_generator_type_map),
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
            layer_0: Layer::prepare(
                allocation.layer_0,
                time,
                storage_type_map,
                device,
                queue,
                format,
            ),
            layer_1: Layer::prepare(
                allocation.layer_1,
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

    fn attachment<'w, SKF>(
        architecture: &'w Self::Architecture<SKF>,
        config: &Config,
        timer: &Timer,
    ) -> Self::Attachment<'w, Self::Architecture<SKF>>
    where
        SKF: StorableKeyFn,
    {
        WorldAttachment {
            config,
            timer,
            layer: architecture,
            residue: MyWorld {
                layer_0: Layer::attachment(architecture.layer_0, config, timer, architecture),
                layer_1: Layer::attachment(architecture.layer_1, config, timer, architecture),
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
        this.layer_0
    }
}

impl WorldIndexed<[(); 1]> for MyWorld {
    type Layer = MyLayer;

    fn index<SKF>(this: &Self::Architecture<SKF>) -> &<Self::Layer as Layer>::Architecture<SKF>
    where
        SKF: StorableKeyFn,
    {
        this.layer_0
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

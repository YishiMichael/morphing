use core::range::Range;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::Arc;

use super::config::Config;
use super::storable::SlotKeyGeneratorTypeMap;
use super::storable::StorageTypeMap;
use super::timeline::Alive;
use super::timeline::AllocatedTimelineErasure;
use super::timeline::AttachedMobject;
use super::timeline::CollapsedTimelineState;
use super::timeline::IncreasingTimeEval;
use super::timeline::NormalizedTimeMetric;
use super::timeline::PresentationKey;
use super::timeline::Time;
use super::timeline::TimelineErasure;
use super::timeline::Timer;
use super::timeline::TypeQueried;
use super::timeline::TypeQuery;
use super::traits::Mobject;
use super::traits::MobjectBuilder;
use super::traits::MobjectPresentation;

pub trait Archive {
    type Output;

    fn new() -> Self;
    fn merge<TE>(
        &self,
        output: Self::Output,
        alive_id: usize,
        time_eval: &TE,
        parent_time_interval: Range<Time>,
        child_time_interval: Range<Time>,
    ) where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>;
    fn archive(self) -> Self::Output;
}

pub trait Allocate {
    type Output;

    fn allocate(self, slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap) -> Self::Output;
}

pub trait Prepare {
    type Output;

    fn prepare(
        &self,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Output;
}

pub trait Render {
    fn render(
        &self,
        storage_type_map: &StorageTypeMap,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    );
}

pub trait Spawn<'a, I> {
    type TypeQuery: TypeQuery;

    fn spawn(&'a self, input: I) -> Alive<'a, Self::TypeQuery, CollapsedTimelineState>;
}

pub trait Channel: 'static + Sized {
    type MobjectPresentation;

    fn push<T>(&self, alive_id: usize, time_interval: Range<Time>, timeline: T)
    where
        T: TimelineErasure<MobjectPresentation = Self::MobjectPresentation>;
    fn attachment<'c, W, LI, L, CI>(
        &'c self,
        config: &'c Config,
        timer: &'c Timer,
        world: &'c W,
        layer: &'c L,
    ) -> ChannelAttachment<'c, W, LI, L, CI, Self, Self::MobjectPresentation>
    where
        W: WorldIndexed<LI, Layer = L>,
        LI: LayerIndex,
        L: LayerIndexed<CI, Channel = Self>,
        CI: ChannelIndex;
}

pub trait Layer: 'static + Sized {
    type Residue<'l, W, LI>
    where
        W: 'l + WorldIndexed<LI, Layer = Self>,
        LI: LayerIndex,
        Self: 'l;

    fn attachment<'l, W, LI>(
        &'l self,
        config: &'l Config,
        timer: &'l Timer,
        world: &'l W,
    ) -> LayerAttachment<'l, W, LI, Self, Self::Residue<'l, W, LI>>
    where
        W: WorldIndexed<LI, Layer = Self>,
        LI: LayerIndex;
}

pub trait World: 'static + Sized {
    type Residue<'w>
    where
        Self: 'w;

    fn attachment<'w>(
        &'w self,
        config: &'w Config,
        timer: &'w Timer,
    ) -> WorldAttachment<'w, Self, Self::Residue<'w>>;
}

pub struct ChannelAttachment<'c, W, LI, L, CI, C, MP>
where
    W: WorldIndexed<LI, Layer = L>,
    LI: LayerIndex,
    L: LayerIndexed<CI, Channel = C>,
    CI: ChannelIndex,
    C: Channel<MobjectPresentation = MP>,
{
    config: &'c Config,
    timer: &'c Timer,
    world: &'c W,
    layer_index: PhantomData<LI>,
    layer: &'c L,
    channel_index: PhantomData<CI>,
    channel: &'c C,
    mobject_presentation: PhantomData<MP>,
}

pub struct LayerAttachment<'l, W, LI, L, R>
where
    W: WorldIndexed<LI, Layer = L>,
    LI: LayerIndex,
    L: Layer,
{
    config: &'l Config,
    timer: &'l Timer,
    world: &'l W,
    layer_index: PhantomData<LI>,
    layer: &'l L,
    residue: R,
}

pub struct WorldAttachment<'w, W, R>
where
    W: World,
{
    config: &'w Config,
    timer: &'w Timer,
    world: &'w W,
    residue: R,
}

// pub(crate) trait ChannelAttachment<W, LI, L, CI, C, MP>: 'static
// where
//     W: WorldIndexed<LI, Layer = L>,
//     LI: LayerIndex,
//     L: LayerIndexed<CI, Channel = C>,
//     CI: ChannelIndex,
//     C: Channel<MobjectPresentation = MP>,
// {
//     fn config(&self) -> &Config;
//     fn timer(&self) -> &Timer;
//     fn world_architecture(&self) -> &W::Architecture;
//     fn channel_architecture(&self) -> &C::Architecture;
//     // fn spawn<M>(
//     //     &self,
//     //     mobject: M,
//     // ) -> Alive<TypeQueried<W, LI, L, CI, C, M, MP, SKF, Self>, CollapsedTimelineState>
//     // where
//     //     M: Mobject,
//     //     MP: MobjectPresentation<M>;
// }

// pub trait Channel: 'static {
//     type MobjectPresentation: Send + Sync;

//     type Architecture: Channel<Self::MobjectPresentation>;
//     type Archive;
//     type Allocation;
//     type Prepare;
//     type Attachment<'c, W, LI, L, CI>: ChannelAttachment<
//         W,
//         LI,
//         L,
//         CI,
//         Self,
//         Self::MobjectPresentation,
//     >
//     where
//         W: WorldIndexed<LI, Layer = L>,
//         LI: LayerIndex,
//         L: LayerIndexed<CI, Channel = Self>,
//         CI: ChannelIndex;

//     fn architecture() -> Self::Architecture;

// }

// pub trait Layer: 'static {
//     type Architecture;
//     type Archive;
//     type Allocation;
//     type Prepare;
//     type Attachment<'l, W, LI>: LayerAttachment<W, LI, Self>
//     where
//         W: WorldIndexed<LI, Layer = Self>,
//         LI: LayerIndex;

//     fn architecture() -> Self::Architecture;
//     fn merge<TE>(
//         architecture: &Self::Architecture,
//         output: Self::Output,
//         alive_id: usize,
//         time_eval: &TE,
//         parent_time_interval: Range<Time>,
//         child_time_interval: Range<Time>,
//     ) where
//         TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>;
//     fn archive(architecture: Self::Architecture) -> Self::Archive;
//     fn allocation(
//         output: Self::Output,
//         slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
//     ) -> Self::Allocation;
//     fn prepare(
//         allocation: &Self::Allocation,
//         time: Time,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) -> Self::Prepare;
//     fn render(
//         prepare: &Self::Prepare,
//         storage_type_map: &StorageTypeMap,
//         encoder: &mut wgpu::CommandEncoder,
//         target: &wgpu::TextureView,
//     );
//     fn attachment<'l, W, LI>(
//         architecture: &'l Self::Architecture,
//         config: &'l Config,
//         timer: &'l Timer,
//         world_architecture: &'l W::Architecture,
//     ) -> Self::Attachment<'l, W, LI>
//     where
//         W: WorldIndexed<LI, Layer = Self>,
//         LI: LayerIndex;
// }

// pub trait World: 'static {
//     type Architecture;
//     type Archive;
//     type Allocation;
//     type Prepare;
//     type Attachment<'w>;

//     fn architecture() -> Self::Architecture;
//     fn merge<TE>(
//         architecture: &Self::Architecture,
//         output: Self::Output,
//         alive_id: usize,
//         time_eval: &TE,
//         parent_time_interval: Range<Time>,
//         child_time_interval: Range<Time>,
//     ) where
//         TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>;
//     fn archive(architecture: Self::Architecture) -> Self::Archive;
//     fn allocation(
//         output: Self::Output,
//         slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
//     ) -> Self::Allocation;
//     fn prepare(
//         allocation: &Self::Allocation,
//         time: Time,
//         storage_type_map: &mut StorageTypeMap,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         format: wgpu::TextureFormat,
//     ) -> Self::Prepare;
//     fn render(
//         prepare: &Self::Prepare,
//         storage_type_map: &StorageTypeMap,
//         encoder: &mut wgpu::CommandEncoder,
//         target: &wgpu::TextureView,
//     );
//     fn attachment<'w>(
//         architecture: &'w Self::Architecture,
//         config: &'w Config,
//         timer: &'w Timer,
//     ) -> Self::Attachment<'w>;
// }

pub trait ChannelIndex: 'static {}

pub trait LayerIndex: 'static {}

pub struct Idx<const IDX: usize>([(); IDX]);

impl<const IDX: usize> ChannelIndex for Idx<IDX> {}

impl<const IDX: usize> LayerIndex for Idx<IDX> {}

// pub trait Channel<MP>: Channel {}

pub trait LayerIndexed<CI>: Layer
where
    CI: ChannelIndex,
{
    type Channel: Channel;

    fn index(&self) -> &Self::Channel;
}

pub trait WorldIndexed<LI>: World
where
    LI: LayerIndex,
{
    type Layer: Layer;

    fn index(&self) -> &Self::Layer;
}

// pub(crate) trait LayerAttachment<W, LI, L>: 'static
// where
//     W: WorldIndexed<LI, Layer = L>,
//     LI: LayerIndex,
//     L: Layer,
// {
//     fn config(&self) -> &Config;
//     // fn spawn<M>(
//     //     &self,
//     //     mobject: M,
//     // ) -> Alive<TypeQueried<W, LI, L, CI, C, M, MP, SKF, Self>, CollapsedTimelineState>
//     // where
//     //     M: Mobject,
//     //     MP: MobjectPresentation<M>;
// }

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

pub type ChannelType<MP> = RefCell<
    Vec<(
        usize,
        Node<(
            Range<Time>,
            Box<dyn TimelineErasure<MobjectPresentation = MP>>,
        )>,
    )>,
>;

// pub struct ChannelType<MP>(
//     ,
// )
// where
//     MP: 'static + Send + Sync;

impl<MP> Archive for ChannelType<MP>
where
    MP: 'static + Send + Sync,
{
    type Output = Vec<(
        Range<Time>,
        Box<dyn TimelineErasure<MobjectPresentation = MP>>,
    )>;

    fn new() -> Self {
        RefCell::new(Vec::new())
    }

    fn archive(self) -> Self::Output {
        let mut nodes = self.into_inner();
        nodes.sort_by_key(|(alive_id, _)| *alive_id);
        nodes
            .into_iter()
            .flat_map(|(_, timeline)| timeline)
            .collect()
    }

    fn merge<TE>(
        &self,
        output: Self::Output,
        alive_id: usize,
        time_eval: &TE,
        parent_time_interval: Range<Time>,
        child_time_interval: Range<Time>,
    ) where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    {
        self.borrow_mut().push((
            alive_id,
            Node::Multiton(
                output
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
        ));
    }
}

impl<MP> Channel for ChannelType<MP>
where
    MP: 'static + Send + Sync,
{
    type MobjectPresentation = MP;

    fn push<T>(&self, alive_id: usize, time_interval: Range<Time>, timeline: T)
    where
        T: TimelineErasure<MobjectPresentation = Self::MobjectPresentation>,
    {
        self.borrow_mut().push((
            alive_id,
            Node::Singleton((time_interval, Box::new(timeline))),
        ));
    }

    fn attachment<'c, W, LI, L, CI>(
        &'c self,
        config: &'c Config,
        timer: &'c Timer,
        world: &'c W,
        layer: &'c L,
    ) -> ChannelAttachment<'c, W, LI, L, CI, Self, Self::MobjectPresentation>
    where
        W: WorldIndexed<LI, Layer = L>,
        LI: LayerIndex,
        L: LayerIndexed<CI, Channel = Self>,
        CI: ChannelIndex,
    {
        ChannelAttachment {
            config,
            timer,
            world,
            layer_index: PhantomData,
            layer,
            channel_index: PhantomData,
            channel: self,
            mobject_presentation: PhantomData,
        }
    }
}

impl<MP> Allocate
    for Vec<(
        Range<Time>,
        Box<dyn TimelineErasure<MobjectPresentation = MP>>,
    )>
where
    MP: 'static,
{
    type Output = Vec<(
        Range<Time>,
        Box<dyn AllocatedTimelineErasure<MobjectPresentation = MP>>,
    )>;

    fn allocate(self, slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap) -> Self::Output {
        self.into_iter()
            .map(|(time_interval, timeline)| {
                (
                    time_interval,
                    timeline.allocate(slot_key_generator_type_map),
                )
            })
            .collect()
    }
}

impl<MP> Prepare
    for Vec<(
        Range<Time>,
        Box<dyn AllocatedTimelineErasure<MobjectPresentation = MP>>,
    )>
where
    MP: 'static + Send + Sync,
{
    type Output = Vec<PresentationKey<MP>>;

    fn prepare(
        &self,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Output {
        self.into_iter()
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
}

// impl<W, LI, L, CI, C, MP> ChannelAttachment<W, LI, L, CI, C, MP>
//     for ChannelAttachmentImpl<'_, W, LI, L, CI, C, MP>
// where
//     W: WorldIndexed<LI, Layer = L>,
//     LI: LayerIndex,
//     L: LayerIndexed<CI, Channel = C>,
//     CI: ChannelIndex,
//     C: Channel<MobjectPresentation = MP>,
//     MP: 'static + Send + Sync,
// {
//     fn config(&self) -> &Config {
//         self.config
//     }

//     fn timer(&self) -> &Timer {
//         self.timer
//     }

//     fn world_architecture(&self) -> &W::Architecture {
//         self.world_architecture
//     }

//     fn channel_architecture(&self) -> &C::Architecture {
//         self.channel_architecture
//     }
// }

impl<'c, W, LI, L, CI, C, M, MP> Spawn<'c, M> for ChannelAttachment<'c, W, LI, L, CI, C, MP>
where
    W: WorldIndexed<LI, Layer = L>,
    LI: LayerIndex,
    L: LayerIndexed<CI, Channel = C>,
    CI: ChannelIndex,
    C: Channel<MobjectPresentation = MP>,
    M: Mobject,
    MP: MobjectPresentation<M>,
{
    type TypeQuery = TypeQueried<W, LI, L, CI, C, M, MP>;

    fn spawn(&'c self, mobject: M) -> Alive<'c, Self::TypeQuery, CollapsedTimelineState> {
        AttachedMobject::new(Arc::new(mobject), self).launch(CollapsedTimelineState)
    }
}

impl<'l, W, LI, L, R, MB> Spawn<'l, MB> for LayerAttachment<'l, W, LI, L, R>
where
    W: WorldIndexed<LI, Layer = L>,
    LI: LayerIndex,
    L: Layer<Residue<'l, W, LI> = R>,
    MB: MobjectBuilder<L>,
{
    type TypeQuery = MB::OutputTypeQuery<W, LI>;

    fn spawn(&'l self, mobject_builder: MB) -> Alive<'l, Self::TypeQuery, CollapsedTimelineState> {
        mobject_builder.instantiate(self, self.config)
    }
}

// impl<MP> Channel<MP> for ChannelType<MP> where MP: 'static + Send + Sync {}

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
pub struct MyLayer<
    channel_0 = ChannelType<MyMobjectPresentation0>,
    channel_1 = ChannelType<MyMobjectPresentation1>,
> {
    pub channel_0: channel_0,
    pub channel_1: channel_1,
}

impl Archive for MyLayer {
    type Output = MyLayer<
        <ChannelType<MyMobjectPresentation0> as Archive>::Output,
        <ChannelType<MyMobjectPresentation1> as Archive>::Output,
    >;

    fn new() -> Self {
        MyLayer {
            channel_0: Archive::new(),
            channel_1: Archive::new(),
        }
    }

    fn merge<TE>(
        &self,
        output: Self::Output,
        alive_id: usize,
        time_eval: &TE,
        parent_time_interval: Range<Time>,
        child_time_interval: Range<Time>,
    ) where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    {
        self.channel_0.merge(
            output.channel_0,
            alive_id,
            time_eval,
            parent_time_interval,
            child_time_interval,
        );
        self.channel_1.merge(
            output.channel_1,
            alive_id,
            time_eval,
            parent_time_interval,
            child_time_interval,
        );
    }

    fn archive(self) -> Self::Output {
        MyLayer {
            channel_0: self.channel_0.archive(),
            channel_1: self.channel_1.archive(),
        }
    }
}

impl Layer for MyLayer {
    type Residue<'l, W, LI> = MyLayer<
        ChannelAttachment<'l, W, LI, Self, Idx<0>, ChannelType<MyMobjectPresentation0>, MyMobjectPresentation0>,
        ChannelAttachment<'l, W, LI, Self, Idx<1>, ChannelType<MyMobjectPresentation1>, MyMobjectPresentation1>,
    > where
        W: 'l + WorldIndexed<LI, Layer = Self>,
        LI: LayerIndex;

    fn attachment<'l, W, LI>(
        &'l self,
        config: &'l Config,
        timer: &'l Timer,
        world: &'l W,
    ) -> LayerAttachment<'l, W, LI, Self, Self::Residue<'l, W, LI>>
    where
        W: WorldIndexed<LI, Layer = Self>,
        LI: LayerIndex,
    {
        LayerAttachment {
            config,
            timer,
            world,
            layer_index: PhantomData,
            layer: self,
            residue: MyLayer {
                channel_0: self.channel_0.attachment(config, timer, world, self),
                channel_1: self.channel_1.attachment(config, timer, world, self),
            },
        }
    }
}

impl Allocate
    for MyLayer<
        <ChannelType<MyMobjectPresentation0> as Archive>::Output,
        <ChannelType<MyMobjectPresentation1> as Archive>::Output,
    >
{
    type Output = MyLayer<
        <<ChannelType<MyMobjectPresentation0> as Archive>::Output as Allocate>::Output,
        <<ChannelType<MyMobjectPresentation1> as Archive>::Output as Allocate>::Output,
    >;

    fn allocate(self, slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap) -> Self::Output {
        MyLayer {
            channel_0: self.channel_0.allocate(slot_key_generator_type_map),
            channel_1: self.channel_1.allocate(slot_key_generator_type_map),
        }
    }
}

impl Prepare
    for MyLayer<
        <<ChannelType<MyMobjectPresentation0> as Archive>::Output as Allocate>::Output,
        <<ChannelType<MyMobjectPresentation1> as Archive>::Output as Allocate>::Output,
    >
{
    type Output = MyLayer<
        <<<ChannelType<MyMobjectPresentation0> as Archive>::Output as Allocate>::Output as Prepare>::Output,
        <<<ChannelType<MyMobjectPresentation1> as Archive>::Output as Allocate>::Output as Prepare>::Output,
    >;

    fn prepare(
        &self,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Output {
        MyLayer {
            channel_0: self
                .channel_0
                .prepare(time, storage_type_map, device, queue, format),
            channel_1: self
                .channel_1
                .prepare(time, storage_type_map, device, queue, format),
        }
    }
}

impl LayerIndexed<Idx<0>> for MyLayer {
    type Channel = ChannelType<MyMobjectPresentation0>;

    fn index(&self) -> &Self::Channel {
        &self.channel_0
    }
}

impl LayerIndexed<Idx<1>> for MyLayer {
    type Channel = ChannelType<MyMobjectPresentation1>;

    fn index(&self) -> &Self::Channel {
        &self.channel_1
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
impl Render
    for MyLayer<
        Vec<PresentationKey<MyMobjectPresentation0>>,
        Vec<PresentationKey<MyMobjectPresentation1>>,
    >
{
    fn render(
        &self,
        storage_type_map: &StorageTypeMap,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
    }
}

/*
#[derive(World)]
pub struct MyWorld {
    pub layer_0: MyLayer,
    pub layer_1: MyLayer,
}
*/

#[allow(non_camel_case_types)]
pub struct MyWorld<layer_0 = MyLayer, layer_1 = MyLayer> {
    pub layer_0: layer_0,
    pub layer_1: layer_1,
}

impl Archive for MyWorld {
    type Output = MyWorld<<MyLayer as Archive>::Output, <MyLayer as Archive>::Output>;

    fn new() -> Self {
        MyWorld {
            layer_0: Archive::new(),
            layer_1: Archive::new(),
        }
    }

    fn merge<TE>(
        &self,
        output: Self::Output,
        alive_id: usize,
        time_eval: &TE,
        parent_time_interval: Range<Time>,
        child_time_interval: Range<Time>,
    ) where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    {
        self.layer_0.merge(
            output.layer_0,
            alive_id,
            time_eval,
            parent_time_interval,
            child_time_interval,
        );
        self.layer_1.merge(
            output.layer_1,
            alive_id,
            time_eval,
            parent_time_interval,
            child_time_interval,
        );
    }

    fn archive(self) -> Self::Output {
        MyWorld {
            layer_0: self.layer_0.archive(),
            layer_1: self.layer_1.archive(),
        }
    }
}

impl World for MyWorld {
    type Residue<'w> = MyWorld<
        LayerAttachment<'w, Self, Idx<0>, MyLayer, <MyLayer as Layer>::Residue<'w, Self, Idx<0>>>,
        LayerAttachment<'w, Self, Idx<1>, MyLayer, <MyLayer as Layer>::Residue<'w, Self, Idx<1>>>,
    >;

    fn attachment<'w>(
        &'w self,
        config: &'w Config,
        timer: &'w Timer,
    ) -> WorldAttachment<'w, Self, Self::Residue<'w>> {
        WorldAttachment {
            config,
            timer,
            world: self,
            residue: MyWorld {
                layer_0: self.layer_0.attachment(config, timer, self),
                layer_1: self.layer_1.attachment(config, timer, self),
            },
        }
    }
}

impl Allocate for MyWorld<<MyLayer as Archive>::Output, <MyLayer as Archive>::Output> {
    type Output = MyWorld<
        <<MyLayer as Archive>::Output as Allocate>::Output,
        <<MyLayer as Archive>::Output as Allocate>::Output,
    >;

    fn allocate(self, slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap) -> Self::Output {
        MyWorld {
            layer_0: self.layer_0.allocate(slot_key_generator_type_map),
            layer_1: self.layer_1.allocate(slot_key_generator_type_map),
        }
    }
}

impl Prepare
    for MyWorld<
        <<MyLayer as Archive>::Output as Allocate>::Output,
        <<MyLayer as Archive>::Output as Allocate>::Output,
    >
{
    type Output = MyWorld<
        <<<MyLayer as Archive>::Output as Allocate>::Output as Prepare>::Output,
        <<<MyLayer as Archive>::Output as Allocate>::Output as Prepare>::Output,
    >;

    fn prepare(
        &self,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Output {
        MyWorld {
            layer_0: self
                .layer_0
                .prepare(time, storage_type_map, device, queue, format),
            layer_1: self
                .layer_1
                .prepare(time, storage_type_map, device, queue, format),
        }
    }
}

impl WorldIndexed<Idx<0>> for MyWorld {
    type Layer = MyLayer;

    fn index(&self) -> &Self::Layer {
        &self.layer_0
    }
}

impl WorldIndexed<Idx<1>> for MyWorld {
    type Layer = MyLayer;

    fn index(&self) -> &Self::Layer {
        &self.layer_1
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

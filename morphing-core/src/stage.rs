use core::range::Range;
use std::cell::RefCell;
use std::marker::PhantomData;

use super::config::Config;
use super::storable::SlotKeyGeneratorTypeMap;
use super::storable::StorageTypeMap;
use super::timeline::AllocatedTimelineErasure;
use super::timeline::PresentationKey;
use super::timeline::TimelineErasure;
use super::timer::IncreasingTimeEval;
use super::timer::NormalizedTimeMetric;
use super::timer::Time;
use super::timer::Timer;

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

pub trait Channel: 'static + Sized + Archive {
    type MobjectPresentation;

    fn push<T>(&self, alive_id: usize, time_interval: Range<Time>, timeline: T)
    where
        T: TimelineErasure<MobjectPresentation = Self::MobjectPresentation>;
    fn attachment<'c, 't, W, LI, L, CI>(
        &'t self,
        config: &'c Config,
        timer: &'t Timer,
        world: &'t W,
        layer: &'t L,
    ) -> ChannelAttachment<'c, 't, W, LI, L, CI, Self, Self::MobjectPresentation>
    where
        W: WorldIndexed<LI, Layer = L>,
        LI: LayerIndex,
        L: LayerIndexed<CI, Channel = Self>,
        CI: ChannelIndex;
}

pub trait Layer: 'static + Sized + Archive {
    type Residue<'c, 't, W, LI>
    where
        W: WorldIndexed<LI, Layer = Self>,
        LI: LayerIndex,
        Self: 't;

    fn attachment<'c, 't, W, LI>(
        &'t self,
        config: &'c Config,
        timer: &'t Timer,
        world: &'t W,
    ) -> LayerAttachment<'c, 't, W, LI, Self, Self::Residue<'c, 't, W, LI>>
    where
        W: WorldIndexed<LI, Layer = Self>,
        LI: LayerIndex;
}

pub trait World: 'static + Sized + Archive {
    type Residue<'c, 't>
    where
        Self: 't;

    fn attachment<'c, 't>(
        &'t self,
        config: &'c Config,
        timer: &'t Timer,
    ) -> WorldAttachment<'c, 't, Self, Self::Residue<'c, 't>>;
}

pub struct ChannelAttachment<'c, 't, W, LI, L, CI, C, MP>
where
    W: WorldIndexed<LI, Layer = L>,
    LI: LayerIndex,
    L: LayerIndexed<CI, Channel = C>,
    CI: ChannelIndex,
    C: Channel<MobjectPresentation = MP>,
{
    pub config: &'c Config,
    pub timer: &'t Timer,
    pub world: &'t W,
    pub layer_index: PhantomData<LI>,
    pub layer: &'t L,
    pub channel_index: PhantomData<CI>,
    pub channel: &'t C,
    pub mobject_presentation: PhantomData<MP>,
}

pub struct LayerAttachment<'c, 't, W, LI, L, R>
where
    W: WorldIndexed<LI, Layer = L>,
    LI: LayerIndex,
    L: Layer,
{
    pub config: &'c Config,
    pub timer: &'t Timer,
    pub world: &'t W,
    pub layer_index: PhantomData<LI>,
    pub layer: &'t L,
    pub residue: R,
}

pub struct WorldAttachment<'c, 't, W, R>
where
    W: World,
{
    pub config: &'c Config,
    pub timer: &'t Timer,
    pub world: &'t W,
    pub residue: R,
}

pub trait ChannelIndex: 'static {}

pub trait LayerIndex: 'static {}

pub struct Idx<const IDX: usize>([(); IDX]);

impl<const IDX: usize> ChannelIndex for Idx<IDX> {}

impl<const IDX: usize> LayerIndex for Idx<IDX> {}

pub trait LayerIndexed<CI>: Layer
where
    CI: ChannelIndex,
{
    type Channel: Channel;

    fn index_attachment<'c, 't, 'a, W, LI>(
        attachment: &'a LayerAttachment<'c, 't, W, LI, Self, Self::Residue<'c, 't, W, LI>>,
    ) -> &'a ChannelAttachment<
        'c,
        't,
        W,
        LI,
        Self,
        CI,
        Self::Channel,
        <Self::Channel as Channel>::MobjectPresentation,
    >
    where
        W: WorldIndexed<LI, Layer = Self>,
        LI: LayerIndex;
}

pub trait WorldIndexed<LI>: World
where
    LI: LayerIndex,
{
    type Layer: Layer;

    fn index_attachment<'c, 't, 'a>(
        attachment: &'a WorldAttachment<'c, 't, Self, Self::Residue<'c, 't>>,
    ) -> &'a LayerAttachment<
        'c,
        't,
        Self,
        LI,
        Self::Layer,
        <Self::Layer as Layer>::Residue<'c, 't, Self, LI>,
    >;
}

pub enum Node<V> {
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

    fn attachment<'c, 't, W, LI, L, CI>(
        &'t self,
        config: &'c Config,
        timer: &'t Timer,
        world: &'t W,
        layer: &'t L,
    ) -> ChannelAttachment<'c, 't, W, LI, L, CI, Self, Self::MobjectPresentation>
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

// test code
pub struct MyMobjectPresentation0;
pub struct MyMobjectPresentation1;

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
    type Residue<'c, 't, W, LI> = MyLayer<
        ChannelAttachment<'c, 't, W, LI, Self, Idx<0>, ChannelType<MyMobjectPresentation0>, MyMobjectPresentation0>,
        ChannelAttachment<'c, 't, W, LI, Self, Idx<1>, ChannelType<MyMobjectPresentation1>, MyMobjectPresentation1>,
    > where
        W: WorldIndexed<LI, Layer = Self>,
        LI: LayerIndex;

    fn attachment<'c, 't, W, LI>(
        &'t self,
        config: &'c Config,
        timer: &'t Timer,
        world: &'t W,
    ) -> LayerAttachment<'c, 't, W, LI, Self, Self::Residue<'c, 't, W, LI>>
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

    fn index_attachment<'c, 't, 'a, W, LI>(
        attachment: &'a LayerAttachment<'c, 't, W, LI, Self, Self::Residue<'c, 't, W, LI>>,
    ) -> &'a ChannelAttachment<
        'c,
        't,
        W,
        LI,
        Self,
        Idx<0>,
        Self::Channel,
        <Self::Channel as Channel>::MobjectPresentation,
    >
    where
        W: WorldIndexed<LI, Layer = Self>,
        LI: LayerIndex,
    {
        &attachment.residue.channel_0
    }
}

impl LayerIndexed<Idx<1>> for MyLayer {
    type Channel = ChannelType<MyMobjectPresentation1>;

    fn index_attachment<'c, 't, 'a, W, LI>(
        attachment: &'a LayerAttachment<'c, 't, W, LI, Self, Self::Residue<'c, 't, W, LI>>,
    ) -> &'a ChannelAttachment<
        'c,
        't,
        W,
        LI,
        Self,
        Idx<1>,
        Self::Channel,
        <Self::Channel as Channel>::MobjectPresentation,
    >
    where
        W: WorldIndexed<LI, Layer = Self>,
        LI: LayerIndex,
    {
        &attachment.residue.channel_1
    }
}

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
    type Residue<'c, 't> = MyWorld<
        LayerAttachment<
            'c,
            't,
            Self,
            Idx<0>,
            MyLayer,
            <MyLayer as Layer>::Residue<'c, 't, Self, Idx<0>>,
        >,
        LayerAttachment<
            'c,
            't,
            Self,
            Idx<1>,
            MyLayer,
            <MyLayer as Layer>::Residue<'c, 't, Self, Idx<1>>,
        >,
    >;

    fn attachment<'c, 't>(
        &'t self,
        config: &'c Config,
        timer: &'t Timer,
    ) -> WorldAttachment<'c, 't, Self, Self::Residue<'c, 't>> {
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

    fn index_attachment<'c, 't, 'a>(
        attachment: &'a WorldAttachment<'c, 't, Self, Self::Residue<'c, 't>>,
    ) -> &'a LayerAttachment<
        'c,
        't,
        Self,
        Idx<0>,
        Self::Layer,
        <Self::Layer as Layer>::Residue<'c, 't, Self, Idx<0>>,
    > {
        &attachment.residue.layer_0
    }
}

impl WorldIndexed<Idx<1>> for MyWorld {
    type Layer = MyLayer;

    fn index_attachment<'c, 't, 'a>(
        attachment: &'a WorldAttachment<'c, 't, Self, Self::Residue<'c, 't>>,
    ) -> &'a LayerAttachment<
        'c,
        't,
        Self,
        Idx<1>,
        Self::Layer,
        <Self::Layer as Layer>::Residue<'c, 't, Self, Idx<1>>,
    > {
        &attachment.residue.layer_1
    }
}

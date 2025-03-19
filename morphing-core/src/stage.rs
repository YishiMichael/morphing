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

pub trait World: 'static + Sized + Archive {
    type Residue<'t>
    where
        Self: 't;

    fn attachment<'t>(
        &'t self,
        config: &'t Config,
        timer: &'t Timer,
    ) -> WorldAttachment<'t, Self, Self::Residue<'t>>;
}

pub trait LayerIndex<W>: 'static + Sized
where
    W: World,
{
    type Layer: Layer;

    fn index_attachment<'t, 'a>(
        attachment: &'a WorldAttachment<'t, W, W::Residue<'t>>,
    ) -> &'a LayerAttachment<'t, W, Self, Self::Layer, <Self::Layer as Layer>::Residue<'t, W, Self>>;
}

pub trait Layer: 'static + Sized + Archive {
    type Residue<'t, W, LI>
    where
        W: World,
        LI: LayerIndex<W, Layer = Self>,
        Self: 't;

    fn attachment<'t, W, LI>(
        &'t self,
        config: &'t Config,
        timer: &'t Timer,
        world: &'t W,
    ) -> LayerAttachment<'t, W, LI, Self, Self::Residue<'t, W, LI>>
    where
        W: World,
        LI: LayerIndex<W, Layer = Self>;
}

pub trait ChannelIndex<L>: 'static + Sized
where
    L: Layer,
{
    type Channel: Channel;

    fn index_attachment<'t, 'a, W, LI>(
        attachment: &'a LayerAttachment<'t, W, LI, L, L::Residue<'t, W, LI>>,
    ) -> &'a ChannelAttachment<
        't,
        W,
        LI,
        L,
        Self,
        Self::Channel,
        <Self::Channel as Channel>::Presentation,
    >
    where
        W: World,
        LI: LayerIndex<W, Layer = L>;
}

pub trait Channel: 'static + Sized + Archive {
    type Presentation;

    fn push<T>(&self, alive_id: usize, time_interval: Range<Time>, timeline: T)
    where
        T: TimelineErasure<Presentation = Self::Presentation>;
    fn attachment<'t, W, LI, L, CI>(
        &'t self,
        config: &'t Config,
        timer: &'t Timer,
        world: &'t W,
        layer: &'t L,
    ) -> ChannelAttachment<'t, W, LI, L, CI, Self, Self::Presentation>
    where
        W: World,
        LI: LayerIndex<W, Layer = L>,
        L: Layer,
        CI: ChannelIndex<L, Channel = Self>;
}

pub struct WorldAttachment<'t, W, R>
where
    W: World,
{
    pub config: &'t Config,
    pub timer: &'t Timer,
    pub world: &'t W,
    pub residue: R,
}

pub struct LayerAttachment<'t, W, LI, L, R>
where
    W: World,
    LI: LayerIndex<W, Layer = L>,
    L: Layer,
{
    pub config: &'t Config,
    pub timer: &'t Timer,
    pub world: &'t W,
    pub layer_index: PhantomData<LI>,
    pub layer: &'t L,
    pub residue: R,
}

pub struct ChannelAttachment<'t, W, LI, L, CI, C, MP>
where
    W: World,
    LI: LayerIndex<W, Layer = L>,
    L: Layer,
    CI: ChannelIndex<L, Channel = C>,
    C: Channel<Presentation = MP>,
{
    pub config: &'t Config,
    pub timer: &'t Timer,
    pub world: &'t W,
    pub layer_index: PhantomData<LI>,
    pub layer: &'t L,
    pub channel_index: PhantomData<CI>,
    pub channel: &'t C,
    pub presentation: PhantomData<MP>,
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
        Node<(Range<Time>, Box<dyn TimelineErasure<Presentation = MP>>)>,
    )>,
>;

impl<MP> Archive for ChannelType<MP>
where
    MP: 'static + Send + Sync,
{
    type Output = Vec<(Range<Time>, Box<dyn TimelineErasure<Presentation = MP>>)>;

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
    type Presentation = MP;

    fn push<T>(&self, alive_id: usize, time_interval: Range<Time>, timeline: T)
    where
        T: TimelineErasure<Presentation = Self::Presentation>,
    {
        self.borrow_mut().push((
            alive_id,
            Node::Singleton((time_interval, Box::new(timeline))),
        ));
    }

    fn attachment<'t, W, LI, L, CI>(
        &'t self,
        config: &'t Config,
        timer: &'t Timer,
        world: &'t W,
        layer: &'t L,
    ) -> ChannelAttachment<'t, W, LI, L, CI, Self, Self::Presentation>
    where
        W: World,
        LI: LayerIndex<W, Layer = L>,
        L: Layer,
        CI: ChannelIndex<L, Channel = Self>,
    {
        ChannelAttachment {
            config,
            timer,
            world,
            layer_index: PhantomData,
            layer,
            channel_index: PhantomData,
            channel: self,
            presentation: PhantomData,
        }
    }
}

impl<MP> Allocate for Vec<(Range<Time>, Box<dyn TimelineErasure<Presentation = MP>>)>
where
    MP: 'static,
{
    type Output = Vec<(
        Range<Time>,
        Box<dyn AllocatedTimelineErasure<Presentation = MP>>,
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
        Box<dyn AllocatedTimelineErasure<Presentation = MP>>,
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

use core::range::Range;
use std::cell::RefCell;
use std::marker::PhantomData;

use super::config::Config;
use super::storable::SlotKeyGeneratorTypeMap;
use super::storable::StorageTypeMap;
use super::timeline::IncreasingTimeEval;
use super::timeline::Node;
use super::timeline::NormalizedTimeMetric;
use super::timeline::PresentationKey;
use super::timeline::Time;
use super::timeline::TimelineAllocationErasure;
use super::timeline::TimelineErasure;
use super::timeline::Timer;
use super::traits::StorableKeyFn;

pub trait Channel {
    type Architecture<SKF, const LI: usize, const CI: usize>
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
    type Attachment<'c, W, L, C>
    where
        Self: 'c,
        W: 'c,
        L: 'c,
        C: 'c;

    fn architecture<SKF, const LI: usize, const CI: usize>() -> Self::Architecture<SKF, LI, CI>
    where
        SKF: StorableKeyFn;
    fn merge<TE, SKF, const LI: usize, const CI: usize>(
        architecture: &mut Self::Architecture<SKF, LI, CI>,
        archive: Self::Archive<SKF>,
        alive_id: usize,
        time_eval: &TE,
        parent_time_interval: Range<Time>,
        child_time_interval: Range<Time>,
    ) where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
        SKF: StorableKeyFn;
    fn archive<SKF, const LI: usize, const CI: usize>(
        architecture: Self::Architecture<SKF, LI, CI>,
    ) -> Self::Archive<SKF>
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
    fn attachment<'c, W, L, SKF, const LI: usize, const CI: usize>(
        architecture: &'c Self::Architecture<SKF, LI, CI>,
        config: &Config,
        timer: &Timer,
        world: &W,
        layer: &L,
    ) -> Self::Attachment<'c, W, L, Self::Architecture<SKF, LI, CI>>
    where
        SKF: StorableKeyFn;
}

pub trait Layer {
    type Architecture<SKF, const LI: usize>
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
    type Attachment<'l, W, L>
    where
        Self: 'l,
        W: 'l,
        L: 'l;

    fn architecture<SKF, const LI: usize>() -> Self::Architecture<SKF, LI>
    where
        SKF: StorableKeyFn;
    fn merge<TE, SKF, const LI: usize>(
        architecture: &mut Self::Architecture<SKF, LI>,
        archive: Self::Archive<SKF>,
        alive_id: usize,
        time_eval: &TE,
        parent_time_interval: Range<Time>,
        child_time_interval: Range<Time>,
    ) where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
        SKF: StorableKeyFn;
    fn archive<SKF, const LI: usize>(
        architecture: Self::Architecture<SKF, LI>,
    ) -> Self::Archive<SKF>
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
    fn attachment<'l, W, SKF, const LI: usize>(
        architecture: &'l Self::Architecture<SKF, LI>,
        config: &Config,
        timer: &Timer,
        world: &W,
    ) -> Self::Attachment<'l, W, Self::Architecture<SKF, LI>>
    where
        SKF: StorableKeyFn;
}

pub trait World {
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
    type Attachment<'w, W>
    where
        Self: 'w,
        W: 'w;

    fn architecture<SKF>() -> Self::Architecture<SKF>
    where
        SKF: StorableKeyFn;
    fn merge<TE, SKF>(
        architecture: &mut Self::Architecture<SKF>,
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

pub struct ChannelAttachment<'c, W, L, C, R> {
    config: &'c Config,
    timer: &'c Timer,
    world: &'c W,
    layer: &'c L,
    channel: &'c C,
    residue: R,
}

pub struct LayerAttachment<'c, W, L, R> {
    config: &'c Config,
    timer: &'c Timer,
    world: &'c W,
    layer: &'c L,
    residue: R,
}

pub struct WorldAttachment<'c, W, R> {
    config: &'c Config,
    timer: &'c Timer,
    world: &'c W,
    residue: R,
}

pub struct PresentationChannel<MP>(MP);

impl<MP> Channel for PresentationChannel<MP>
where
    MP: 'static + Send + Sync,
{
    type Architecture<SKF, const LI: usize, const CI: usize> = (
        Vec<(
            usize,
            Node<(
                Range<Time>,
                Box<dyn TimelineErasure<SKF, MobjectPresentation = MP>>,
            )>,
        )>,
        PhantomData<([(); LI], [(); CI])>,
    )
    where
        SKF: StorableKeyFn;
    type Archive<SKF> = Vec<(Range<Time>, Box<dyn TimelineErasure<SKF, MobjectPresentation = MP>>)>
    where
        SKF: StorableKeyFn;
    type Allocation<SKF> = Vec<(Range<Time>, Box<dyn TimelineAllocationErasure<SKF, MobjectPresentation = MP>>)>
    where
        SKF: StorableKeyFn;
    type Prepare<SKF> = Vec<PresentationKey<SKF, MP>>
    where
        SKF: StorableKeyFn;
    type Attachment<'c, W, L, C> = ChannelAttachment<'c, W, L, C, ()>
    where
        Self: 'c,
        W: 'c,
        L: 'c,
        C: 'c;

    fn architecture<SKF, const LI: usize, const CI: usize>() -> Self::Architecture<SKF, LI, CI>
    where
        SKF: StorableKeyFn,
    {
        (Vec::new(), PhantomData)
    }

    fn merge<TE, SKF, const LI: usize, const CI: usize>(
        architecture: &mut Self::Architecture<SKF, LI, CI>,
        archive: Self::Archive<SKF>,
        alive_id: usize,
        time_eval: &TE,
        parent_time_interval: Range<Time>,
        child_time_interval: Range<Time>,
    ) where
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
        SKF: StorableKeyFn,
    {
        architecture.0.push((
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
        ));
    }

    fn archive<SKF, const LI: usize, const CI: usize>(
        architecture: Self::Architecture<SKF, LI, CI>,
    ) -> Self::Archive<SKF>
    where
        SKF: StorableKeyFn,
    {
        architecture.0.sort_by_key(|(alive_id, _)| alive_id);
        architecture
            .0
            .into_iter()
            .flat_map(|(_, timeline)| timeline)
            .collect()
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

    fn attachment<'c, W, L, SKF, const LI: usize, const CI: usize>(
        architecture: &'c Self::Architecture<SKF, LI, CI>,
        config: &Config,
        timer: &Timer,
        world: &W,
        layer: &L,
    ) -> Self::Attachment<'c, W, L, Self::Architecture<SKF, LI, CI>>
    where
        SKF: StorableKeyFn,
    {
        ChannelAttachment {
            config,
            timer,
            world,
            layer,
            channel: architecture,
            residue: (),
        }
    }
}

// pub struct Alive<'a, SKF, W, M, TS>
// where
//     SKF: StorableKeyFn,
//     W: WorldErasure<SKF>,
//     M: Mobject,
//     TS: AnimationState<SKF, W, M>,
// {
//     channel_attachment: &'a ChannelAttachment<'a, SKF, W, M::MobjectPresentation>,
//     spawn_time: Rc<Time>,
//     mobject: Arc<M>,
//     animation_state: Option<TS>,
// }

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
    type Architecture<SKF, const LI: usize> = MyLayer<
        <PresentationChannel<MyMobjectPresentation0> as Channel>::Architecture<SKF, LI, 0>,
        <PresentationChannel<MyMobjectPresentation1> as Channel>::Architecture<SKF, LI, 1>,
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
    type Attachment<'l, W, L> = LayerAttachment<'l, W, L, MyLayer<
        <PresentationChannel<MyMobjectPresentation0> as Channel>::Attachment<'l, W, L, PresentationChannel<MyMobjectPresentation0>>,
        <PresentationChannel<MyMobjectPresentation1> as Channel>::Attachment<'l, W, L, PresentationChannel<MyMobjectPresentation1>>,
    >>
    where
        Self: 'l,
        W: 'l,
        L: 'l;

    fn architecture<SKF, const LI: usize>() -> Self::Architecture<SKF, LI>
    where
        SKF: StorableKeyFn,
    {
        MyLayer {
            channel_0: Channel::architecture(),
            channel_1: Channel::architecture(),
        }
    }

    fn merge<TE, SKF, const LI: usize>(
        architecture: &mut Self::Architecture<SKF, LI>,
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

    fn archive<SKF, const LI: usize>(
        architecture: Self::Architecture<SKF, LI>,
    ) -> Self::Archive<SKF>
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

    fn attachment<'l, W, SKF, const LI: usize>(
        architecture: &'l Self::Architecture<SKF, LI>,
        config: &Config,
        timer: &Timer,
        world: &W,
    ) -> Self::Attachment<'l, W, Self::Architecture<SKF, LI>>
    where
        SKF: StorableKeyFn,
    {
        LayerAttachment {
            config,
            timer,
            world,
            layer: architecture,
            residue: MyLayer {
                channel_0: Channel::attachment(
                    architecture.channel_0,
                    config,
                    timer,
                    world,
                    architecture,
                ),
                channel_1: Channel::attachment(
                    architecture.channel_1,
                    config,
                    timer,
                    world,
                    architecture,
                ),
            },
        }
    }
}

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
        <MyLayer as Layer>::Architecture<SKF, 0>,
        <MyLayer as Layer>::Architecture<SKF, 1>,
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
    type Attachment<'w, W> = WorldAttachment<'w, W, MyWorld<
        <MyLayer as Layer>::Attachment<'w, W, MyLayer>,
        <MyLayer as Layer>::Attachment<'w, W, MyLayer>,
    >>
    where
        Self: 'w,
        W: 'w;

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
        architecture: &mut Self::Architecture<SKF>,
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

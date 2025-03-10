use std::cell::RefCell;
use std::ops::Range;

use super::animation::AnimationAllocatedErasure;
use super::animation::AnimationErasure;
use super::animation::IncreasingTimeEval;
use super::animation::Node;
use super::animation::NormalizedTimeMetric;
use super::animation::PresentationKey;
use super::animation::Time;
use super::storable::SlotKeyGeneratorTypeMap;
use super::storable::StorageTypeMap;
use super::traits::StorableKeyFn;

pub trait Cell {
    type Attached<SKF>
    where
        SKF: StorableKeyFn;
    type Archived<SKF>
    where
        SKF: StorableKeyFn;
    type Allocated<SKF>
    where
        SKF: StorableKeyFn;
    type Attachment<'c, C, SKF>
    where
        Self: 'c,
        C: 'c,
        SKF: StorableKeyFn;

    fn new<SKF>() -> Self::Attached<SKF>
    where
        SKF: StorableKeyFn;
    fn attachment<'c, C, SKF>(
        this: &'c Self::Attached<SKF>,
        context: &'c C,
    ) -> Self::Attachment<'c, C, SKF>
    where
        SKF: StorableKeyFn;
    fn archive<SKF>(this: Self::Attached<SKF>) -> Self::Archived<SKF>
    where
        SKF: StorableKeyFn;
    fn allocate<SKF>(
        this: Self::Archived<SKF>,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Allocated<SKF>
    where
        SKF: StorableKeyFn;
}

pub trait Merge: Cell {
    fn merge<SKF, TE>(
        this: &mut Self::Attached<SKF>,
        child: Self::Attached<SKF>,
        time_interval: &Range<Time>,
        time_eval: &TE,
        child_time_interval: &Range<Time>,
    ) where
        SKF: StorableKeyFn,
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>;
}

pub trait Prepare: Cell {
    type Prepared<SKF>
    where
        SKF: StorableKeyFn;

    fn prepare<SKF>(
        this: &Self::Allocated<SKF>,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Prepared<SKF>
    where
        SKF: StorableKeyFn;
}

pub trait Render: Prepare {
    fn render<SKF>(
        this: &Self::Prepared<SKF>,
        storage_type_map: &StorageTypeMap,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) where
        SKF: StorableKeyFn;
}

pub trait Layer: Render {}

pub trait World: Render {}

struct Timeline<MP>(MP);

impl<MP> Cell for Timeline<MP>
where
    MP: 'static + Send + Sync,
{
    type Attached<SKF> = RefCell<
        Option<(
            Range<Time>,
            Node<Box<dyn AnimationErasure<SKF, MobjectPresentation = MP>>>,
        )>,
    > where SKF: StorableKeyFn;
    type Archived<SKF> = (
        Range<Time>,
        Node<Box<dyn AnimationErasure<SKF, MobjectPresentation = MP>>>,
    ) where SKF: StorableKeyFn;
    type Allocated<SKF> = (
        Range<Time>,
        Node<Box<dyn AnimationAllocatedErasure<SKF, MobjectPresentation = MP>>>,
    ) where SKF: StorableKeyFn;
    type Attachment<'c, C, SKF> = TimelineAttachment<'c, MP, C, SKF>
    where
        Self: 'c,
        C: 'c,
        SKF: StorableKeyFn;

    fn new<SKF>() -> Self::Attached<SKF>
    where
        SKF: StorableKeyFn,
    {
        RefCell::new(None)
    }

    fn attachment<'c, C, SKF>(
        this: &'c Self::Attached<SKF>,
        context: &'c C,
    ) -> Self::Attachment<'c, C, SKF>
    where
        SKF: StorableKeyFn,
    {
        TimelineAttachment {
            animation_entry: this,
            context,
        }
    }

    fn archive<SKF>(this: Self::Attached<SKF>) -> Self::Archived<SKF>
    where
        SKF: StorableKeyFn,
    {
        this.into_inner().unwrap()
    }

    fn allocate<SKF>(
        this: Self::Archived<SKF>,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Allocated<SKF>
    where
        SKF: StorableKeyFn,
    {
        let (time_interval, animation_node) = this;
        (
            time_interval,
            animation_node.map(|animation| animation.allocate(slot_key_generator_type_map)),
        )
    }
}

impl<MP> Prepare for Timeline<MP>
where
    MP: 'static + Send + Sync,
{
    type Prepared<SKF> = Option<Node<PresentationKey<SKF, MP>>> where SKF: StorableKeyFn;

    fn prepare<SKF>(
        this: &Self::Allocated<SKF>,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Prepared<SKF>
    where
        SKF: StorableKeyFn,
    {
        let (time_interval, animation_node) = this;
        time_interval.contains(&time).then(|| {
            animation_node.map_ref(|animation| {
                animation.prepare(
                    time,
                    time_interval.clone(),
                    storage_type_map,
                    device,
                    queue,
                    format,
                )
            })
        })
    }
}

struct TimelineAttachment<'c, MP, C, SKF>
where
    MP: 'static + Send + Sync,
    SKF: StorableKeyFn,
{
    animation_entry: &'c <Timeline<MP> as Cell>::Attached<SKF>,
    context: &'c C,
}

struct Channel<MP>(MP);

impl<MP> Cell for Channel<MP>
where
    MP: 'static + Send + Sync,
{
    type Attached<SKF> = Vec<<Timeline<MP> as Cell>::Attached<SKF>> where SKF: StorableKeyFn;
    type Archived<SKF> = Vec<<Timeline<MP> as Cell>::Archived<SKF>> where SKF: StorableKeyFn;
    type Allocated<SKF> = Vec<<Timeline<MP> as Cell>::Allocated<SKF>> where SKF: StorableKeyFn;
    type Attachment<'c, C, SKF> = ChannelAttachment<'c, MP, C, SKF>
    where
        Self: 'c,
        C: 'c,
        SKF: StorableKeyFn;

    fn new<SKF>() -> Self::Attached<SKF>
    where
        SKF: StorableKeyFn,
    {
        Vec::new()
    }

    fn attachment<'c, C, SKF>(
        this: &'c Self::Attached<SKF>,
        context: &'c C,
    ) -> Self::Attachment<'c, C, SKF>
    where
        SKF: StorableKeyFn,
    {
        this.into_iter()
            .map(|timeline| Cell::attachment(context))
            .collect()
    }

    fn archive<SKF>(this: Self::Attached<SKF>) -> Self::Archived<SKF>
    where
        SKF: StorableKeyFn,
    {
        this.into_iter()
            .map(|timeline| Cell::archive(timeline))
            .collect()
    }

    fn allocate<SKF>(
        this: Self::Archived<SKF>,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Allocated<SKF>
    where
        SKF: StorableKeyFn,
    {
        this.into_iter()
            .map(|timeline| Cell::allocate(timeline, slot_key_generator_type_map))
            .collect()
    }
}

impl<MP> Merge for Channel<MP>
where
    MP: 'static + Send + Sync,
{
    fn merge<SKF, TE>(
        this: &mut Self::Attached<SKF>,
        child: Self::Attached<SKF>,
        time_interval: &Range<Time>,
        time_eval: &TE,
        child_time_interval: &Range<Time>,
    ) where
        SKF: StorableKeyFn,
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    {
        this.0
            .extend(child.0.into_iter().map(|(entry_time_interval, animation)| {
                (
                    time_interval.start
                        + (time_interval.end - time_interval.start)
                            * *time_eval
                                .time_eval(entry_time_interval.start, child_time_interval.clone())
                        ..time_interval.start
                            + (time_interval.end - time_interval.start)
                                * *time_eval.time_eval(
                                    entry_time_interval.end,
                                    child_time_interval.clone(),
                                ),
                    animation,
                )
            }));
    }
}

impl<MP> Prepare for Channel<MP>
where
    MP: 'static + Send + Sync,
{
    type Prepared<SKF> = Vec<PresentationKey<SKF, MP>> where SKF: StorableKeyFn;

    fn prepare<SKF>(
        this: &Self::Allocated<SKF>,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Prepared<SKF>
    where
        SKF: StorableKeyFn,
    {
        this.into_iter()
            .filter_map(|timeline| {
                Prepare::prepare(timeline, time, storage_type_map, device, queue, format)
            })
            .flatten()
            .collect()
    }
}

struct ChannelAttachment<'c, MP, C, SKF>
where
    MP: 'static + Send + Sync,
    SKF: StorableKeyFn,
{
    animation_entries: &'c <Channel<MP> as Cell>::Attached<SKF>,
    context: &'c C,
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
struct MyLayer<CHANNEL_0 = (), CHANNEL_1 = ()> {
    channel_0: CHANNEL_0,
    channel_1: CHANNEL_1,
}

impl Cell for MyLayer {
    type Attached<SKF> = MyLayer<
        <Channel<MyMobjectPresentation0> as Cell>::Attached<SKF>,
        <Channel<MyMobjectPresentation1> as Cell>::Attached<SKF>,
    > where SKF: StorableKeyFn;
    type Archived<SKF> = MyLayer<
        <Channel<MyMobjectPresentation0> as Cell>::Archived<SKF>,
        <Channel<MyMobjectPresentation1> as Cell>::Archived<SKF>,
    > where SKF: StorableKeyFn;
    type Allocated<SKF> = MyLayer<
        <Channel<MyMobjectPresentation0> as Cell>::Allocated<SKF>,
        <Channel<MyMobjectPresentation1> as Cell>::Allocated<SKF>,
    > where SKF: StorableKeyFn;
    type Attachment<'c, C, SKF> = MyLayer<
        <Channel<MyMobjectPresentation0> as Cell>::Attachment<'c, C, SKF>,
        <Channel<MyMobjectPresentation1> as Cell>::Attachment<'c, C, SKF>
    >
    where
        Self: 'c,
        C: 'c,
        SKF: StorableKeyFn; // wrapper needed to add `spawn` method

    fn new<SKF>() -> Self::Attached<SKF>
    where
        SKF: StorableKeyFn,
    {
        MyLayer {
            channel_0: Cell::new(),
            channel_1: Cell::new(),
        }
    }

    fn attachment<'c, C, SKF>(
        this: &'c Self::Attached<SKF>,
        context: &'c C,
    ) -> Self::Attachment<'c, C, SKF>
    where
        SKF: StorableKeyFn,
    {
        MyLayer {
            channel_0: Cell::attachment(this.channel_0, context),
            channel_1: Cell::attachment(this.channel_1, context),
        }
    }

    fn archive<SKF>(this: Self::Attached<SKF>) -> Self::Archived<SKF>
    where
        SKF: StorableKeyFn,
    {
        MyLayer {
            channel_0: Cell::archive(this.channel_0),
            channel_1: Cell::archive(this.channel_1),
        }
    }

    fn allocate<SKF>(
        this: Self::Archived<SKF>,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Allocated<SKF>
    where
        SKF: StorableKeyFn,
    {
        MyLayer {
            channel_0: Cell::allocate(this.channel_0, slot_key_generator_type_map),
            channel_1: Cell::allocate(this.channel_1, slot_key_generator_type_map),
        }
    }
}

impl Merge for MyLayer {
    fn merge<SKF, TE>(
        this: &mut Self::Attached<SKF>,
        child: Self::Attached<SKF>,
        time_interval: &Range<Time>,
        time_eval: &TE,
        child_time_interval: &Range<Time>,
    ) where
        SKF: StorableKeyFn,
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    {
        Merge::merge(
            this.channel_0,
            child.channel_0,
            time_interval,
            time_eval,
            child_time_interval,
        );
        Merge::merge(
            this.channel_1,
            child.channel_1,
            time_interval,
            time_eval,
            child_time_interval,
        );
    }
}

impl Prepare for MyLayer {
    type Prepared<SKF> = MyLayer<
        <Channel<MyMobjectPresentation0> as Prepare>::Prepared<SKF>,
        <Channel<MyMobjectPresentation1> as Prepare>::Prepared<SKF>,
    > where SKF: StorableKeyFn;

    fn prepare<SKF>(
        this: &Self::Allocated<SKF>,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Prepared<SKF>
    where
        SKF: StorableKeyFn,
    {
        MyLayer {
            channel_0: Prepare::prepare(
                this.channel_0,
                time,
                storage_type_map,
                device,
                queue,
                format,
            ),
            channel_1: Prepare::prepare(
                this.channel_1,
                time,
                storage_type_map,
                device,
                queue,
                format,
            ),
        }
    }
}

impl Render for MyLayer {
    fn render<SKF>(
        this: &Self::Prepared<SKF>,
        storage_type_map: &StorageTypeMap,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) where
        SKF: StorableKeyFn,
    {
        render_my_layer(this, storage_type_map, encoder, target)
    }
}

impl Layer for MyLayer {}

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
    prepared_layer: &<MyLayer as Prepare>::Prepared<SKF>,
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
struct MyWorld<LAYER_0 = (), LAYER_1 = ()> {
    layer_0: LAYER_0,
    layer_1: LAYER_1,
}

impl Cell for MyWorld {
    type Attached<SKF> = MyWorld<
        <MyLayer as Cell>::Attached<SKF>,
        <MyLayer as Cell>::Attached<SKF>,
    > where SKF: StorableKeyFn;
    type Archived<SKF> = MyWorld<
        <MyLayer as Cell>::Archived<SKF>,
        <MyLayer as Cell>::Archived<SKF>,
    > where SKF: StorableKeyFn;
    type Allocated<SKF> = MyWorld<
        <MyLayer as Cell>::Allocated<SKF>,
        <MyLayer as Cell>::Allocated<SKF>,
    > where SKF: StorableKeyFn;
    type Attachment<'c, C, SKF> = MyWorld<
        <MyLayer as Cell>::Attachment<'c, C, SKF>,
        <MyLayer as Cell>::Attachment<'c, C, SKF>
    >
    where
        Self: 'c,
        C: 'c,
        SKF: StorableKeyFn; // wrapper needed to add `spawn` method

    fn new<SKF>() -> Self::Attached<SKF>
    where
        SKF: StorableKeyFn,
    {
        MyWorld {
            layer_0: Cell::new(),
            layer_1: Cell::new(),
        }
    }

    fn attachment<'c, C, SKF>(
        this: &'c Self::Attached<SKF>,
        context: &'c C,
    ) -> Self::Attachment<'c, C, SKF>
    where
        SKF: StorableKeyFn,
    {
        MyWorld {
            layer_0: Cell::attachment(this.layer_0, context),
            layer_1: Cell::attachment(this.layer_1, context),
        }
    }

    fn archive<SKF>(this: Self::Attached<SKF>) -> Self::Archived<SKF>
    where
        SKF: StorableKeyFn,
    {
        MyWorld {
            layer_0: Cell::archive(this.layer_0),
            layer_1: Cell::archive(this.layer_1),
        }
    }

    fn allocate<SKF>(
        this: Self::Archived<SKF>,
        slot_key_generator_type_map: &mut SlotKeyGeneratorTypeMap,
    ) -> Self::Allocated<SKF>
    where
        SKF: StorableKeyFn,
    {
        MyWorld {
            layer_0: Cell::allocate(this.layer_0, slot_key_generator_type_map),
            layer_1: Cell::allocate(this.layer_1, slot_key_generator_type_map),
        }
    }
}

impl Merge for MyWorld {
    fn merge<SKF, TE>(
        this: &mut Self::Attached<SKF>,
        child: Self::Attached<SKF>,
        time_interval: &Range<Time>,
        time_eval: &TE,
        child_time_interval: &Range<Time>,
    ) where
        SKF: StorableKeyFn,
        TE: IncreasingTimeEval<OutputTimeMetric = NormalizedTimeMetric>,
    {
        Merge::merge(
            this.layer_0,
            child.layer_0,
            time_interval,
            time_eval,
            child_time_interval,
        );
        Merge::merge(
            this.layer_1,
            child.layer_1,
            time_interval,
            time_eval,
            child_time_interval,
        );
    }
}

impl Prepare for MyWorld {
    type Prepared<SKF> = MyWorld<
        <MyLayer as Prepare>::Prepared<SKF>,
        <MyLayer as Prepare>::Prepared<SKF>,
    > where SKF: StorableKeyFn;

    fn prepare<SKF>(
        this: &Self::Allocated<SKF>,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Prepared<SKF>
    where
        SKF: StorableKeyFn,
    {
        MyWorld {
            layer_0: Prepare::prepare(this.layer_0, time, storage_type_map, device, queue, format),
            layer_1: Prepare::prepare(this.layer_1, time, storage_type_map, device, queue, format),
        }
    }
}

impl Render for MyWorld {
    fn render<SKF>(
        this: &Self::Prepared<SKF>,
        storage_type_map: &StorageTypeMap,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) where
        SKF: StorableKeyFn,
    {
        Render::render(this.layer_0, storage_type_map, encoder, target);
        Render::render(this.layer_1, storage_type_map, encoder, target);
    }
}

impl World for MyWorld {}

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

//     fn new(config: &Config, timer_stack: &TimerStack) -> Rc<Self> {
//         Rc::new_cyclic(|world| Self {
//             layer_0: MyWorld::new(config, timer_stack, world.clone()),
//             layer_1: MyWorld::new(config, timer_stack, world.clone()),
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
//             Box::new(self.layer_0.collect()) as Box<dyn LayerPreallocated>,
//             Box::new(self.layer_1.collect()) as Box<dyn LayerPreallocated>,
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

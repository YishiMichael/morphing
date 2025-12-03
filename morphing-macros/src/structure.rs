// use super::LayerArgs;
// use super::WorldArgs;

// External crate dependencies:
// - serde
// - morphing_core
// pub(crate) fn world(_args: WorldArgs, item_struct: syn::ItemStruct) -> proc_macro2::TokenStream {}

// External crate dependencies:
// - serde
// - morphing_core
// pub(crate) fn layer(_args: LayerArgs, item_struct: syn::ItemStruct) -> proc_macro2::TokenStream {}

// test code
/*
pub struct MyMobjectPresentation0;
pub struct MyMobjectPresentation1;

/*
#[layer]
pub struct MyLayer {
    pub channel_0: MyMobjectPresentation0,
    pub channel_1: MyMobjectPresentation1,
}
*/

#[derive(::serde::Deserialize, ::serde::Serialize)]
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
    type Residue<'t, W, LI> = MyLayer<
        ChannelAttachment<'t, W, LI, Self, MyLayerChannel0, ChannelType<MyMobjectPresentation0>, MyMobjectPresentation0>,
        ChannelAttachment<'t, W, LI, Self, MyLayerChannel1, ChannelType<MyMobjectPresentation1>, MyMobjectPresentation1>,
    > where
        W: World,
        LI: LayerIndex<W, Layer = Self>;

    fn attachment<'t, W, LI>(
        &'t self,
        config: &'t Config,
        timer: &'t Timer,
        world: &'t W,
    ) -> LayerAttachment<'t, W, LI, Self, Self::Residue<'t, W, LI>>
    where
        W: World,
        LI: LayerIndex<W, Layer = Self>,
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

pub struct MyLayerChannel0;

impl ChannelIndex<MyLayer> for MyLayerChannel0 {
    type Channel = ChannelType<MyMobjectPresentation0>;

    fn index_attachment<'t, 'a, W, LI>(
        attachment: &'a LayerAttachment<'t, W, LI, MyLayer, <MyLayer as Layer>::Residue<'t, W, LI>>,
    ) -> &'a ChannelAttachment<
        't,
        W,
        LI,
        MyLayer,
        Self,
        Self::Channel,
        <Self::Channel as Channel>::MobjectPresentation,
    >
    where
        W: World,
        LI: LayerIndex<W, Layer = MyLayer>,
    {
        &attachment.residue.channel_0
    }
}

pub struct MyLayerChannel1;

impl ChannelIndex<MyLayer> for MyLayerChannel1 {
    type Channel = ChannelType<MyMobjectPresentation1>;

    fn index_attachment<'t, 'a, W, LI>(
        attachment: &'a LayerAttachment<'t, W, LI, MyLayer, <MyLayer as Layer>::Residue<'t, W, LI>>,
    ) -> &'a ChannelAttachment<
        't,
        W,
        LI,
        MyLayer,
        Self,
        Self::Channel,
        <Self::Channel as Channel>::MobjectPresentation,
    >
    where
        W: World,
        LI: LayerIndex<W, Layer = MyLayer>,
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
#[world]
pub struct MyWorld {
    pub layer_0: MyLayer,
    pub layer_1: MyLayer,
}
*/

#[derive(::serde::Deserialize, ::serde::Serialize)]
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
    type Residue<'t> = MyWorld<
        LayerAttachment<
            't,
            Self,
            MyWorldLayer0,
            MyLayer,
            <MyLayer as Layer>::Residue<'t, Self, MyWorldLayer0>,
        >,
        LayerAttachment<
            't,
            Self,
            MyWorldLayer1,
            MyLayer,
            <MyLayer as Layer>::Residue<'t, Self, MyWorldLayer1>,
        >,
    >;

    fn attachment<'t>(
        &'t self,
        config: &'t Config,
        timer: &'t Timer,
    ) -> WorldAttachment<'t, Self, Self::Residue<'t>> {
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

pub struct MyWorldLayer0;

impl LayerIndex<MyWorld> for MyWorldLayer0 {
    type Layer = MyLayer;

    fn index_attachment<'t, 'a>(
        attachment: &'a WorldAttachment<'t, MyWorld, <MyWorld as World>::Residue<'t>>,
    ) -> &'a LayerAttachment<
        't,
        MyWorld,
        Self,
        Self::Layer,
        <Self::Layer as Layer>::Residue<'t, MyWorld, Self>,
    > {
        &attachment.residue.layer_0
    }
}

pub struct MyWorldLayer1;

impl LayerIndex<MyWorld> for MyWorldLayer1 {
    type Layer = MyLayer;

    fn index_attachment<'t, 'a>(
        attachment: &'a WorldAttachment<'t, MyWorld, <MyWorld as World>::Residue<'t>>,
    ) -> &'a LayerAttachment<
        't,
        MyWorld,
        Self,
        Self::Layer,
        <Self::Layer as Layer>::Residue<'t, MyWorld, Self>,
    > {
        &attachment.residue.layer_1
    }
}
*/

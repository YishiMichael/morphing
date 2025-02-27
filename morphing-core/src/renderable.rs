pub(crate) struct Supervisor<'c> {
    config: &'c Config,
    time: RefCell<Rc<Time>>,
    // layers: RefCell<Vec<Rc<dyn LayerErased>>>,
}

impl<'c> Supervisor<'c> {
    pub(crate) fn new(config: &'c Config) -> Self {
        Self {
            config,
            time: RefCell::new(Rc::new(0.0)),
            // layers: RefCell::new(Vec::new()),
        }
    }

    pub fn wait(&self, delta_time: Time) {
        assert!(
            delta_time.is_sign_positive(),
            "`Supervisor::wait` expects a positive-signed `delta_time`, got {delta_time}",
        );
        let mut time = self.time.borrow_mut();
        *time = Rc::new(**time + delta_time);
    }
}

impl TimeContext for Supervisor<'_> {
    fn time(&self) -> Rc<Time> {
        self.time.borrow().clone()
    }

    fn time_interval(&self) -> Range<Time> {
        0.0..**self.time.borrow()
    }
}

trait Renderable:
    'static + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize + StorablePrimitive
{
    fn prepare(
        &self,
        time: Time,
        prepare_ref: &mut Option<<Self::PresentationStorage as PresentationStorage>::Target>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    );
    fn render(
        &self,
        render_ref: &Option<<Self::PresentationStorage as PresentationStorage>::Target>,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    );
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct LayerRenderable<L> {
    layer: Arc<L>,
}

impl<L> StorablePrimitive for LayerRenderable<L>
where
    L: Layer,
{
    type PresentationStorage = ReadWriteStorage<L::LayerPresentation>;

    fn storage_id_input(
        &self,
    ) -> <Self::PresentationStorage as PresentationStorage>::StorageIdInput {
        ()
    }
}

impl<L> Renderable for LayerRenderable<L>
where
    L: Layer,
{
    fn prepare(
        &self,
        time: Time,
        prepare_ref: &mut Option<<Self::PresentationStorage as PresentationStorage>::Target>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) {
        prepare_ref.insert(self.layer.prepare(time, device, queue, format));
    }

    fn render(
        &self,
        render_ref: &Option<<Self::PresentationStorage as PresentationStorage>::Target>,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        self.layer
            .render(render_ref.as_ref().unwrap(), encoder, target);
    }
}

trait RenderableErased {
    fn allocate(self, storage_type_map: &mut StorageTypeMap) -> Box<dyn AllocatedRenderableErased>;
}

impl<R> RenderableErased for R
where
    R: Renderable,
{
    fn allocate(self, storage_type_map: &mut StorageTypeMap) -> Box<dyn AllocatedRenderableErased> {
        Box::new(storage_type_map.allocate(self))
    }
}

trait AllocatedRenderableErased {
    fn prepare(
        &self,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    );
    fn render(
        &self,
        storage_type_map: &StorageTypeMap,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    );
}

impl<R> AllocatedRenderableErased for Allocated<R>
where
    R: Renderable,
{
    fn prepare(
        &self,
        time: Time,
        storage_type_map: &mut StorageTypeMap,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) {
        self.storable_primitive().prepare(
            time,
            storage_type_map.get_mut(self),
            device,
            queue,
            format,
        );
    }

    fn render(
        &self,
        storage_type_map: &StorageTypeMap,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        self.storable_primitive()
            .render(storage_type_map.get_ref(self), encoder, target);
    }
}

impl<L> Archive for LayerRenderable<L>
where
    L: Layer,
{
    type Archived = (Range<Time>, Box<dyn RenderableErased>);

    fn archive(&mut self, time_interval: Range<Time>) -> Self::Archived {
        (
            time_interval,
            Box::new(LayerRenderable {
                layer: self.layer.clone(),
            }),
        )
    }
}

pub type RenderableAliveCollector<'c, 'w> =
    AliveCollector<'c, 'w, (Range<Time>, Box<dyn RenderableErased>)>;

impl RenderableAliveCollector<'_, '_> {
    // fn new_static_timeline_entry<M>(
    //     &self,
    //     time_interval: Range<Time>,
    //     mobject: Arc<M>,
    // ) -> TimelineEntry
    // where
    //     M: Mobject,
    // {
    //     TimelineEntry {
    //         time_interval,
    //         timeline: serde_traitobject::Box::new(StaticTimeline {
    //             id: self.storage.static_allocate(&mobject),
    //             mobject,
    //         }),
    //     }
    // }

    // fn new_dynamic_timeline_entry<M, TE, U>(
    //     &self,
    //     time_interval: Range<Time>,
    //     mobject: Arc<M>,
    //     time_eval: Arc<TE>,
    //     update: Arc<U>,
    // ) -> TimelineEntry
    // where
    //     M: Mobject,
    //     TE: TimeEval,
    //     U: Update<TE::OutputTimeMetric, M>,
    // {
    //     TimelineEntry {
    //         time_interval,
    //         timeline: serde_traitobject::Box::new(DynamicTimeline {
    //             id: self.storage.dynamic_allocate(&mobject, &update),
    //             mobject,
    //             time_eval,
    //             update,
    //         }),
    //     }
    // }

    // fn iter_timeline_entries(self) -> impl Iterator<Item = Box<dyn AllocatedTimelineErased>> {
    //     self.timeline_slots
    //         .into_inner()
    //         .into_iter()
    //         .flat_map(|slot| slot.unwrap())
    // }

    // fn arc_time(&self) -> Arc<Time> {
    //     self.time.borrow().clone()
    // }

    // fn push<T>(&self, time_interval: Range<f32>, timeline: Arc<T>)
    // where
    //     T: 'static + Timeline,
    // {
    //     // Hash `Arc<T>` instead of `T`.
    //     // Presentation maps inside `storage` are identified only by `T::Presentation` type, without `T`.
    //     let timeline = serde_traitobject::Arc::from(timeline as Arc<dyn Timeline>);
    //     let hash = seahash::hash(&ron::ser::to_string(&timeline).unwrap().into_bytes());
    //     self.timeline_entries.borrow_mut().push(TimelineEntry {
    //         hash,
    //         time_interval,
    //         timeline,
    //     });
    // }

    // fn start<'sv, TS>(&'sv self, alive_content: AliveContent<TS>) -> Alive<'am, TC, TS>
    // where
    //     TS: TimelineState,
    // {
    //     let alive_content = Arc::new(alive_content);
    //     // let weak_timeline_state = Arc::downgrade(&timeline_state);
    //     let weak = Arc::downgrade(&alive_content);
    //     self.timeline_slots.borrow_mut().push(Err(alive_content));
    //     Alive {
    //         supervisor: self,
    //         weak,
    //     }
    // }

    // fn end<'sv, TS>(&'sv self, alive: &Alive<'am, TC, TS>) -> AliveContent<TS::OutputTimelineState>
    // where
    //     TS: TimelineState,
    // {
    //     let alive_content = alive.weak.upgrade().unwrap();
    //     let mut timeline_slots_ref = self.timeline_slots.borrow_mut();
    //     let slot = timeline_slots_ref
    //         .iter_mut()
    //         .rfind(|slot| {
    //             slot.as_ref().is_err_and(|alive_content_ref| {
    //                 Arc::ptr_eq(alive_content_ref, &(alive_content.clone() as Arc<dyn Any>))
    //             })
    //         })
    //         .unwrap();
    //     *slot = Ok(Vec::new());

    //     let AliveContent {
    //         spawn_time,
    //         timeline_state,
    //     } = match Arc::try_unwrap(alive_content) {
    //         Ok(alive_content) => alive_content,
    //         Err(_) => unreachable!(),
    //     };
    //     let archive_time = self.time.borrow().clone();
    //     let timeline_entries_sink = TimelineEntriesSink(
    //         (!Arc::ptr_eq(&spawn_time, &archive_time)).then(|| slot.as_mut().unwrap()),
    //     );
    //     let output_timeline_state =
    //         timeline_state.into_next(self, *spawn_time..*archive_time, timeline_entries_sink);
    //     AliveContent {
    //         spawn_time: archive_time,
    //         timeline_state: output_timeline_state,
    //     }

    //     // let (any_timeline_state, timeline_entries) =
    //     //     &mut supervisor.timeline_slots.borrow_mut()[self.index];

    //     // assert!(any_timeline_state.take().is_some());
    //     // let archive_time = supervisor.arc_time();
    //     // let spawn_time = std::mem::replace(&mut self.spawn_time, archive_time.clone());
    //     // let timeline_entries_sink = TimelineEntriesSink(
    //     //     (!Arc::ptr_eq(&spawn_time, &archive_time)).then_some(timeline_entries),
    //     // );
    //     // let output_timeline_state = timeline_state.into_next(
    //     //     *spawn_time..*archive_time,
    //     //     supervisor,
    //     //     timeline_entries_sink,
    //     // );
    // }

    #[must_use]
    pub fn spawn<LB>(
        &self,
        layer_builder: LB,
    ) -> Alive<'_, World, LayerRenderable<LB::Instantiation>>
    where
        LB: LayerBuilder,
        // 'sv: 'c,
    {
        self.start(LayerRenderable {
            layer: Arc::new(layer_builder.instantiate(&self.time_context().config)),
        })
    }

    // #[must_use]
    // pub fn spawn<'sv, MB>(
    //     &'sv self,
    //     mobject_builder: MB,
    // ) -> Alive<'am, TC, CollapsedTimelineState<MB::Instantiation>>
    // where
    //     MB: MobjectBuilder,
    //     'sv: 'c,
    // {
    //     self.start(AliveContent {
    //         spawn_time: self.time.borrow().clone(),
    //         timeline_state: CollapsedTimelineState {
    //             mobject: Arc::new(mobject_builder.instantiate(&self.config)),
    //         },
    //     })
    // }
}

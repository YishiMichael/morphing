use std::cell::RefCell;
use std::ops::Range;

use super::config::Config;
use super::renderable::RenderableErased;
use super::timeline::TimelineErased;

// pub(crate) trait AliveContent<C>: 'static {
//     type Next;
//     // type Input;
//     type Output;

//     // fn new(input: Self::Input, context: &C) -> Self;
//     fn iterate(self, context: &C) -> (Self::Output, Self::Next);
// }

pub type Time = f32;

// pub struct WithSpawnTime<C> {
//     spawn_time: Rc<Time>,
//     content: C,
// }

// pub(crate) struct WithTimeInterval<C> {
//     time_interval: Range<Time>,
//     content: C,
// }

// pub(crate) struct Supervisor<'cf> {
//     config: &'cf Config,
//     time: Rc<Time>,
//     // layers: RefCell<Vec<Rc<dyn LayerErased>>>,
// }

// impl<'cf> Supervisor<'cf> {
//     pub(crate) fn new(config: &'cf Config) -> Self {
//         Self {
//             config,
//             time: Rc::new(0.0),
//             // layers: RefCell::new(Vec::new()),
//         }
//     }

//     fn time(&self) -> Rc<Time> {
//         self.time.clone()
//     }

//     fn time_interval(&self) -> Range<Time> {
//         0.0..*self.time
//     }
// }

// pub(crate) struct AliveRecorder<V> {
//     // context: &'ac AC,
//     time: Time,
//     range_map: RefCell<rangemap::RangeMap<Time, V>>,
//     // recorder: RefCell<Vec<Result<Option<AA>, Rc<Time>>>>,
// }

// impl<A> AliveRecorder<'_, 'w, A>
// where
//     A: Archive,
// {

// }

// impl<V> AliveRecorder<V> {
//     // fn new_content(&self, input: AC::Input) -> AC {
//     //     AC::new(input, &self.context)
//     // }

//     pub(crate) fn new() -> Self {
//         AliveRecorder {
//             // context,
//             time: 0.0,
//             range_map: RefCell::new(rangemap::RangeMap::new()),
//         }
//     }

//     // pub fn inherit(&self) -> Self {
//     //     Self::new(&self.supervisor)
//     // }

//     // pub fn config(&self) -> &'cf Config {
//     //     self.supervisor.config
//     // }

//     // pub fn spawn<I>(&self, instantiator: I) -> Alive<'_, '_, '_, A>
//     // where
//     //     I: Instantiator<A>,
//     // {
//     //     self.start(instantiator.instantiate(&self.world))
//     // }

//     // pub(crate) fn supervisor(&self) -> &'sv Supervisor<'cf> {
//     //     &self.supervisor
//     // }

//     pub(crate) fn start<A>(&self, archive: A) -> Alive<'_, V, A>
//     where
//         A: Archive<V>,
//     {
//         // let content = Rc::new(content);
//         // let weak_content = Rc::downgrade(&content);
//         // let mut recorder = self.recorder.borrow_mut();
//         // let index = recorder.len();
//         // recorder.push(Err(self.supervisor.time()));
//         Alive {
//             alive_recorder: self,
//             spawn_time: self.time,
//             archive: Some(archive),
//         }
//     }

//     // pub(crate) fn world(&self) -> &World<'_> {
//     //     &self.world
//     // }

//     pub(crate) fn collect(self) -> (Range<Time>, rangemap::RangeMap<Time, V>) {
//         (0.0..self.time, self.range_map.into_inner())
//     }
// }

pub trait ArchiveState {
    type LocalArchive: Default;
    type GlobalArchive;

    fn archive(
        &mut self,
        time_interval: Range<Time>,
        local_archive: Self::LocalArchive,
        global_archive: &Self::GlobalArchive,
    );
}

pub trait IntoArchiveState<AC>
where
    AC: AliveContext,
{
    type ArchiveState: ArchiveState;

    fn into_archive_state(self, alive_context: &AC) -> Self::ArchiveState;
}

pub trait AliveContext: Sized {
    type Archive;

    fn time(&self) -> Time;
    fn archive_ref(&self) -> &Self::Archive;

    fn start<AS>(&self, archive_state: AS) -> Alive<'_, Self, AS>
    where
        AS: ArchiveState<GlobalArchive = Self::Archive>,
    {
        Alive {
            alive_context: self,
            // index: usize,
            spawn_time: self.time(),
            archive_state: Some(archive_state),
            local_archive: AS::LocalArchive::default(),
        }
    }

    fn end<AS>(&self, alive: &mut Alive<'_, Self, AS>) -> AS
    where
        AS: ArchiveState<GlobalArchive = Self::Archive>,
    {
        // let mut recorder = self.alive_recorder.recorder.borrow_mut();
        // let entry = recorder.get_mut(self.index).unwrap();
        let spawn_time = alive.spawn_time;
        let archive_time = self.time();
        let mut archive_state = alive.archive_state.take().unwrap();
        if spawn_time < archive_time {
            archive_state.archive(
                spawn_time..archive_time,
                std::mem::replace(&mut alive.local_archive, AS::LocalArchive::default()),
                self.archive_ref(),
            );
        }
        // *entry = Ok((!Rc::ptr_eq(spawn_time, archive_time))
        //     .then(|| archive.archive(*spawn_time..*archive_time, self.alive_recorder.recorder.borrow_mut())));
        archive_state
        // let content = self.weak_content.upgrade().unwrap();
        // let mut slots_ref = self.manager.slots.borrow_mut();
        // let (archive, content_ref) = slots_ref
        //     .iter_mut()
        //     .rfind(|(_, content_ref)| {
        //         content_ref.as_ref().is_some_and(|content_ref| {
        //             Rc::ptr_eq(content_ref, &(content.clone() as Rc<dyn Any>))
        //         })
        //     })
        //     .unwrap();
        // content_ref.take();
        // let content = match Rc::try_unwrap(content) {
        //     Ok(content) => content,
        //     Err(_) => unreachable!(),
        // };
        // f(archive, content, &self.manager.context)
    }
}

pub trait Spawn: AliveContext {
    fn spawn<IAS>(&self, into_archive_state: IAS) -> Alive<'_, Self, IAS::ArchiveState>
    where
        IAS: IntoArchiveState<Self>,
        IAS::ArchiveState: ArchiveState<GlobalArchive = Self::Archive>;
}

impl<AC> Spawn for AC
where
    AC: AliveContext,
{
    #[must_use]
    fn spawn<IAS>(&self, into_archive_state: IAS) -> Alive<'_, Self, IAS::ArchiveState>
    where
        IAS: IntoArchiveState<Self>,
        IAS::ArchiveState: ArchiveState<GlobalArchive = Self::Archive>,
    {
        self.start(into_archive_state.into_archive_state(self))
    }
}

pub struct AliveRoot<'c> {
    config: &'c Config,
    time: RefCell<Time>,
    archive: RefCell<
        Vec<(
            Range<Time>,
            Box<dyn RenderableErased>,
            Vec<(Range<Time>, Box<dyn TimelineErased>)>,
        )>,
    >,
}

impl<'c> AliveRoot<'c> {
    pub(crate) fn new(config: &'c Config) -> Self {
        Self {
            config,
            time: RefCell::new(0.0),
            archive: RefCell::default(),
        }
    }

    pub(crate) fn config(&self) -> &'c Config {
        &self.config
    }

    pub(crate) fn into_archive(
        self,
    ) -> (
        Range<Time>,
        Vec<(
            Range<Time>,
            Box<dyn RenderableErased>,
            Vec<(Range<Time>, Box<dyn TimelineErased>)>,
        )>,
    ) {
        (0.0..self.time.into_inner(), self.archive.into_inner())
    }

    pub fn wait(&self, delta_time: Time) {
        assert!(
            delta_time.is_sign_positive(),
            "`AliveRoot::wait` expects a positive-signed `delta_time`, got {delta_time}",
        );
        self.time.replace_with(|&mut time| time + delta_time);
    }
}

impl AliveContext for AliveRoot<'_> {
    type Archive = RefCell<
        Vec<(
            Range<Time>,
            Box<dyn RenderableErased>,
            Vec<(Range<Time>, Box<dyn TimelineErased>)>,
        )>,
    >;

    fn time(&self) -> Time {
        *self.time.borrow()
    }

    fn archive_ref(&self) -> &Self::Archive {
        &self.archive
    }
}

pub struct Alive<'ac, AC, AS>
where
    AC: AliveContext<Archive = AS::GlobalArchive>,
    AS: ArchiveState,
{
    alive_context: &'ac AC,
    spawn_time: Time,
    archive_state: Option<AS>,
    local_archive: AS::LocalArchive,
}

impl<'ac, AC, AS> Alive<'ac, AC, AS>
where
    AC: AliveContext<Archive = AS::GlobalArchive>,
    AS: ArchiveState,
{
    pub(crate) fn alive_context(&self) -> &'ac AC {
        &self.alive_context
    }

    pub(crate) fn archive_state(&self) -> &AS {
        self.archive_state.as_ref().unwrap()
    }

    pub(crate) fn map<F, FO>(&mut self, f: F) -> Alive<'ac, AC, FO>
    where
        F: FnOnce(&AC, AS) -> FO,
        FO: ArchiveState<GlobalArchive = AC::Archive>,
    {
        self.alive_context
            .start(f(self.alive_context, self.alive_context.end(self)))
    }
}

impl<AC, AS> Drop for Alive<'_, AC, AS>
where
    AC: AliveContext<Archive = AS::GlobalArchive>,
    AS: ArchiveState,
{
    fn drop(&mut self) {
        if self.archive_state.is_some() {
            self.alive_context.end(self);
        }
    }
}

impl<AC, AS> AliveContext for Alive<'_, AC, AS>
where
    AC: AliveContext<Archive = AS::GlobalArchive>,
    AS: ArchiveState,
{
    type Archive = AS::LocalArchive;

    fn time(&self) -> Time {
        self.alive_context.time()
    }

    fn archive_ref(&self) -> &Self::Archive {
        &self.local_archive
    }
}

pub fn execute(
    scene_fn: fn(&AliveRoot),
    config: &Config,
) -> (
    Range<Time>,
    Vec<(
        Range<Time>,
        Box<dyn RenderableErased>,
        Vec<(Range<Time>, Box<dyn TimelineErased>)>,
    )>,
) {
    let alive_root = AliveRoot::new(config);
    scene_fn(&alive_root);
    alive_root.into_archive()
}

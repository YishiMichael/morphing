use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;

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

pub(crate) trait TimeContext {
    fn time(&self) -> Rc<Time>;
    fn time_interval(&self) -> Range<Time>;
}

pub(crate) trait Archive {
    type Archived;

    fn archive(&mut self, time_interval: Range<f32>) -> Self::Archived;
}

pub(crate) struct AliveRecorder<'tc, TC, AA> {
    time_context: &'tc TC,
    collector: RefCell<Vec<Result<Option<AA>, Rc<Time>>>>,
}

// impl<A> AliveRecorder<'_, 'w, A>
// where
//     A: Archive,
// {

// }

impl<TC, AA> AliveRecorder<'_, TC, AA>
where
    TC: TimeContext,
{
    // fn new_content(&self, input: AC::Input) -> AC {
    //     AC::new(input, &self.context)
    // }

    pub fn new(time_context: &TC) -> Self {
        AliveRecorder {
            time_context,
            collector: RefCell::new(Vec::new()),
        }
    }

    // pub fn inherit(&self) -> Self {
    //     Self::new(&self.world)
    // }

    // pub fn spawn<I>(&self, instantiator: I) -> Alive<'_, '_, '_, A>
    // where
    //     I: Instantiator<A>,
    // {
    //     self.start(instantiator.instantiate(&self.world))
    // }

    pub(crate) fn start<A>(&self, archive: A) -> Alive<'_, '_, TC, A>
    where
        A: Archive<Archived = AA>,
    {
        // let content = Rc::new(content);
        // let weak_content = Rc::downgrade(&content);
        let mut collector = self.collector.borrow_mut();
        let index = collector.len();
        collector.push(Err(self.time_context.time()));
        Alive {
            alive_recorder: self,
            index,
            archive: Some(archive),
        }
    }

    // pub(crate) fn world(&self) -> &World<'_> {
    //     &self.world
    // }

    pub(crate) fn collect(self) -> (Range<Time>, Vec<AA>) {
        (
            self.time_context.time_interval(),
            self.collector
                .into_inner()
                .into_iter()
                .filter_map(|item| item.unwrap())
                .collect(),
        )
    }
}

pub(crate) struct Alive<'tc, 'ar, TC, A>
where
    TC: TimeContext,
    A: Archive,
{
    alive_recorder: &'ar AliveRecorder<'tc, TC, A::Archived>,
    index: usize,
    // spawn_time: Rc<Time>,
    archive: Option<A>,
}

impl<'tc, TC, A> Alive<'tc, '_, TC, A>
where
    TC: TimeContext,
    A: Archive,
{
    pub(crate) fn time_context(&self) -> &'tc TC {
        &self.alive_recorder.time_context
    }

    pub(crate) fn archive(&self) -> &A {
        self.archive.as_ref().unwrap()
    }

    pub(crate) fn end(&mut self) -> A {
        let mut collector = self.alive_recorder.collector.borrow_mut();
        let entry = collector.get_mut(self.index).unwrap();
        let spawn_time = match entry.as_mut() {
            Ok(_) => unimplemented!(),
            Err(spawn_time) => spawn_time.clone(),
        };
        let archive_time = self.alive_recorder.time_context.time();
        let mut archive = self.archive.take().unwrap();
        *entry = Ok((!Rc::ptr_eq(&spawn_time, &archive_time))
            .then(|| archive.archive(*spawn_time..*archive_time)));
        archive
        // let content = self.weak_content.upgrade().unwrap();
        // let mut slots_ref = self.manager.slots.borrow_mut();
        // let (archived, content_ref) = slots_ref
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
        // f(archived, content, &self.manager.context)
    }
}

impl<TC, A> Alive<'_, '_, TC, A>
where
    TC: TimeContext,
    A: Archive,
{
    fn drop(&mut self) {
        if self.archive.is_some() {
            self.end();
        }
    }
}

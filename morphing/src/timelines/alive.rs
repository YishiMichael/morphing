use std::cell::RefCell;
use std::ops::Range;
use std::sync::Arc;

use super::super::toplevel::world::World;
use super::timeline::Timeline;
use super::timeline::TimelineEntries;

pub struct Supervisor<'w> {
    world: &'w World,
    time: RefCell<Arc<f32>>,
    timeline_entries: RefCell<TimelineEntries>,
}

impl<'w> Supervisor<'w> {
    pub(crate) fn new(world: &'w World) -> Self {
        Self {
            world,
            time: RefCell::new(Arc::new(0.0)),
            timeline_entries: RefCell::new(TimelineEntries::new()),
        }
    }

    pub(crate) fn get_time(&self) -> Arc<f32> {
        self.time.borrow().clone()
    }

    pub(crate) fn into_timeline_entries(self) -> TimelineEntries {
        self.timeline_entries.into_inner()
    }

    pub fn wait(&self, delta_time: f32) {
        assert!(
            delta_time.is_sign_positive(),
            "`Supervisor::wait` expects a non-negative argument `delta_time`, got {delta_time}",
        );
        let mut time = self.time.borrow_mut();
        *time = Arc::new(**time + delta_time);
    }
}

pub struct Alive<'s, T> {
    supervisor: &'s Supervisor<'s>,
    spawn_time: Arc<f32>,
    timeline: T,
}

impl<'w, T> Alive<'w, T> {
    fn new(supervisor: &'w Supervisor, timeline: T) -> Self {
        Self {
            supervisor,
            spawn_time: supervisor.get_time(),
            timeline,
        }
    }

    fn archive<F, O>(self, f: F) -> O
    where
        T: Clone + Timeline,
        F: FnOnce(&'w Supervisor, Range<f32>, T) -> O,
    {
        let Alive {
            supervisor,
            spawn_time,
            timeline,
        } = self;
        let arc_time_interval = spawn_time..supervisor.get_time();
        let time_interval = *arc_time_interval.start..*arc_time_interval.end;
        if Arc::ptr_eq(&arc_time_interval.start, &arc_time_interval.end) {
            f(supervisor, time_interval, timeline)
        } else {
            let output = f(supervisor, time_interval.clone(), timeline.clone());
            supervisor
                .timeline_entries
                .borrow_mut()
                .push(time_interval, timeline);
            output
        }
    }
}

pub mod traits {
    use super::super::super::mobjects::mobject::Mobject;
    use super::super::act::Act;
    use super::super::construct::Construct;
    use super::super::rates::Rate;
    use super::super::timeline::dynamic::AbsoluteTimelineMetric;
    use super::super::timeline::dynamic::DynamicTimelineMetric;
    use super::super::timeline::dynamic::RelativeTimelineMetric;
    use super::super::update::Update;
    use super::Supervisor;

    pub trait Spawn {
        type Output<'s>;

        fn spawn<'s>(self, supervisor: &'s Supervisor) -> Self::Output<'s>;
    }

    pub trait Destroy {
        fn destroy(self);
    }

    pub trait Quantize: Sized {
        type Output<ME>
        where
            ME: DynamicTimelineMetric;

        fn quantize<ME>(self, metric: ME) -> Self::Output<ME>
        where
            ME: DynamicTimelineMetric;

        // TODO: Add and check if #[must_use] takes effect.
        fn animate(self) -> Self::Output<RelativeTimelineMetric> {
            self.quantize(RelativeTimelineMetric)
        }

        fn animating(self) -> Self::Output<AbsoluteTimelineMetric> {
            self.quantize(AbsoluteTimelineMetric)
        }
    }

    pub trait Collapse {
        type Output;

        fn collapse(self) -> Self::Output;
    }

    pub trait ApplyRate: Sized {
        type Output<R>
        where
            R: Rate;

        fn apply_rate<R>(self, rate: R) -> Self::Output<R>
        where
            R: Rate;
    }

    pub trait ApplyAct<M>
    where
        M: Mobject,
    {
        type Output<A>
        where
            A: Act<M>;

        fn apply_act<A>(self, act: A) -> Self::Output<A>
        where
            A: Act<M>;
    }

    pub trait ApplyUpdate<M>
    where
        M: Mobject,
    {
        type Output<U>
        where
            U: Update<M>;

        fn apply_update<U>(self, update: U) -> Self::Output<U>
        where
            U: Update<M>;
    }

    pub trait ApplyConstruct<M>
    where
        M: Mobject,
    {
        type Output<C>
        where
            C: Construct<M>;

        fn apply_construct<C>(self, construct: C) -> Self::Output<C>
        where
            C: Construct<M>;
    }

    impl<'w> Supervisor<'w> {
        #[must_use]
        pub fn spawn<SP>(&'w self, spawn: SP) -> SP::Output<'w>
        where
            SP: Spawn,
        {
            spawn.spawn(self)
        }
    }
}

pub mod base_impl {
    use std::sync::Arc;

    use super::super::super::mobjects::mobject::Mobject;
    use super::super::super::mobjects::mobject::MobjectBuilder;
    use super::super::act::Act;
    use super::super::act::MobjectDiff;
    use super::super::construct::Construct;
    use super::super::rates::Rate;
    use super::super::timeline::action::ActionTimelineContent;
    use super::super::timeline::continuous::ContinuousTimelineContent;
    use super::super::timeline::discrete::DiscreteTimelineContent;
    use super::super::timeline::dynamic::DynamicTimeline;
    use super::super::timeline::dynamic::DynamicTimelineContent;
    use super::super::timeline::dynamic::DynamicTimelineMetric;
    use super::super::timeline::dynamic::IndeterminedTimelineContent;
    use super::super::timeline::steady::SteadyTimeline;
    use super::super::update::Update;
    use super::traits::ApplyAct;
    use super::traits::ApplyConstruct;
    use super::traits::ApplyRate;
    use super::traits::ApplyUpdate;
    use super::traits::Collapse;
    use super::traits::Destroy;
    use super::traits::Quantize;
    use super::traits::Spawn;
    use super::Alive;
    use super::Supervisor;

    impl<MB> Spawn for MB
    where
        MB: MobjectBuilder,
    {
        type Output<'s> = Alive<'s, SteadyTimeline<MB::Instantiation>>;

        #[must_use]
        fn spawn<'s>(self, supervisor: &'s Supervisor) -> Self::Output<'s> {
            Alive::new(
                supervisor,
                SteadyTimeline {
                    mobject: self.instantiate(supervisor.world),
                },
            )
        }
    }

    impl<M> Destroy for Alive<'_, SteadyTimeline<M>>
    where
        M: Mobject,
    {
        fn destroy(self) {
            self.archive(|_, _, _| ());
        }
    }

    impl<'s, M> Quantize for Alive<'s, SteadyTimeline<M>>
    where
        M: Mobject,
    {
        type Output<ME> = Alive<'s, DynamicTimeline<IndeterminedTimelineContent<M>, ME, IdentityRate>>
        where
            ME: DynamicTimelineMetric;

        #[must_use]
        fn quantize<ME>(self, metric: ME) -> Self::Output<ME>
        where
            ME: DynamicTimelineMetric,
        {
            self.archive(|supervisor, _, timeline| {
                Alive::new(
                    supervisor,
                    DynamicTimeline {
                        content: IndeterminedTimelineContent {
                            mobject: timeline.mobject,
                        },
                        metric,
                        rate: IdentityRate,
                    },
                )
            })
        }
    }

    impl<'s, CO, ME, R> Collapse for Alive<'s, DynamicTimeline<CO, ME, R>>
    where
        CO: DynamicTimelineContent,
        ME: DynamicTimelineMetric,
        R: Rate,
    {
        type Output = Alive<'s, SteadyTimeline<CO::CollapseOutput>>;

        #[must_use]
        fn collapse(self) -> Self::Output {
            self.archive(|supervisor, time_interval, timeline| {
                Alive::new(
                    supervisor,
                    SteadyTimeline {
                        mobject: timeline.content.content_collapse(
                            timeline
                                .rate
                                .eval(timeline.metric.eval(time_interval.end, time_interval)),
                        ),
                    },
                )
            })
        }
    }

    impl<'s, M, ME, R> ApplyRate for Alive<'s, DynamicTimeline<IndeterminedTimelineContent<M>, ME, R>>
    where
        M: Mobject,
        ME: DynamicTimelineMetric,
        R: Rate,
    {
        type Output<RA> = Alive<'s, DynamicTimeline<IndeterminedTimelineContent<M>, ME, ComposeRate<RA, R>>>
        where
            RA: Rate;

        #[must_use]
        fn apply_rate<RA>(self, rate: RA) -> Self::Output<RA>
        where
            RA: Rate,
        {
            self.archive(|supervisor, _, timeline| {
                Alive::new(
                    supervisor,
                    DynamicTimeline {
                        content: IndeterminedTimelineContent {
                            mobject: timeline.content.mobject,
                        },
                        metric: timeline.metric,
                        rate: ComposeRate(rate, timeline.rate),
                    },
                )
            })
        }
    }

    impl<M> ApplyAct<M> for Alive<'_, SteadyTimeline<M>>
    where
        M: Mobject,
    {
        type Output<A> = Self where A: Act<M>;

        #[must_use]
        fn apply_act<A>(self, act: A) -> Self::Output<A>
        where
            A: Act<M>,
        {
            self.archive(|supervisor, _, timeline| {
                let mut mobject = timeline.mobject;
                act.act(&mobject).apply(&mut mobject, 1.0);
                Alive::new(supervisor, SteadyTimeline { mobject })
            })
        }
    }

    impl<'s, M, ME, R> ApplyAct<M> for Alive<'s, DynamicTimeline<IndeterminedTimelineContent<M>, ME, R>>
    where
        M: Mobject,
        ME: DynamicTimelineMetric,
        R: Rate,
    {
        type Output<A> = Alive<'s, DynamicTimeline<ActionTimelineContent<M, A::Diff>, ME, R>>
        where
            A: Act<M>;

        #[must_use]
        fn apply_act<A>(self, act: A) -> Self::Output<A>
        where
            A: Act<M>,
        {
            self.archive(|supervisor, _, timeline| {
                let mobject = timeline.content.mobject;
                let diff = act.act(&mobject);
                Alive::new(
                    supervisor,
                    DynamicTimeline {
                        content: ActionTimelineContent { mobject, diff },
                        metric: timeline.metric,
                        rate: timeline.rate,
                    },
                )
            })
        }
    }

    impl<'s, M, ME, R, MD> ApplyAct<M>
        for Alive<'s, DynamicTimeline<ActionTimelineContent<M, MD>, ME, R>>
    where
        M: Mobject,
        ME: DynamicTimelineMetric,
        R: Rate,
        MD: MobjectDiff<M>,
    {
        type Output<A> = Alive<'s, DynamicTimeline<ActionTimelineContent<M, ComposeMobjectDiff<A::Diff, MD>>, ME, R>>
        where
            A: Act<M>;

        #[must_use]
        fn apply_act<A>(self, act: A) -> Self::Output<A>
        where
            A: Act<M>,
        {
            self.archive(|supervisor, _, timeline| {
                let mobject = timeline.content.mobject;
                let diff = ComposeMobjectDiff(act.act(&mobject), timeline.content.diff);
                Alive::new(
                    supervisor,
                    DynamicTimeline {
                        content: ActionTimelineContent { mobject, diff },
                        metric: timeline.metric,
                        rate: timeline.rate,
                    },
                )
            })
        }
    }

    impl<'s, M, ME, R> ApplyUpdate<M>
        for Alive<'s, DynamicTimeline<IndeterminedTimelineContent<M>, ME, R>>
    where
        M: Mobject,
        ME: DynamicTimelineMetric,
        R: Rate,
    {
        type Output<U> = Alive<'s, DynamicTimeline<ContinuousTimelineContent<M, U>, ME, R>>
        where
            U: Update<M>;

        #[must_use]
        fn apply_update<U>(self, update: U) -> Self::Output<U>
        where
            U: Update<M>,
        {
            self.archive(|supervisor, _, timeline| {
                Alive::new(
                    supervisor,
                    DynamicTimeline {
                        content: ContinuousTimelineContent {
                            mobject: timeline.content.mobject,
                            update,
                        },
                        metric: timeline.metric,
                        rate: timeline.rate,
                    },
                )
            })
        }
    }

    impl<'s, M, ME, R> ApplyConstruct<M>
        for Alive<'s, DynamicTimeline<IndeterminedTimelineContent<M>, ME, R>>
    where
        M: Mobject,
        ME: DynamicTimelineMetric,
        R: Rate,
    {
        type Output<C> = Alive<'s, DynamicTimeline<DiscreteTimelineContent<C::Output>, ME, R>>
        where
            C: Construct<M>;

        #[must_use]
        fn apply_construct<C>(self, construct: C) -> Self::Output<C>
        where
            C: Construct<M>,
        {
            self.archive(|supervisor, _, timeline| {
                let child_supervisor = Supervisor::new(supervisor.world);
                let input_mobject = timeline.content.mobject;
                let output_mobject = construct
                    .construct(
                        Alive::new(
                            &child_supervisor,
                            SteadyTimeline {
                                mobject: input_mobject,
                            },
                        ),
                        &child_supervisor,
                    )
                    .archive(|_, _, steady_timeline| steady_timeline.mobject);
                Alive::new(
                    supervisor,
                    DynamicTimeline {
                        content: DiscreteTimelineContent {
                            mobject: output_mobject,
                            timeline_entries: child_supervisor.into_timeline_entries(),
                        },
                        metric: timeline.metric,
                        rate: timeline.rate,
                    },
                )
            })
        }
    }

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct IdentityRate;

    impl Rate for IdentityRate {
        fn eval(&self, t: f32) -> f32 {
            t
        }
    }

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct ComposeRate<R0, R1>(R0, R1);

    impl<R0, R1> Rate for ComposeRate<R0, R1>
    where
        R0: Rate,
        R1: Rate,
    {
        fn eval(&self, t: f32) -> f32 {
            self.0.eval(self.1.eval(t))
        }
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct ComposeMobjectDiff<MD0, MD1>(MD0, MD1);

    impl<M, MD0, MD1> MobjectDiff<M> for ComposeMobjectDiff<MD0, MD1>
    where
        M: Mobject,
        MD0: MobjectDiff<M>,
        MD1: MobjectDiff<M>,
    {
        fn apply(&self, mobject: &mut M, alpha: f32) {
            self.1.apply(mobject, alpha);
            self.0.apply(mobject, alpha);
        }

        fn apply_presentation(
            &self,
            mobject_presentation: &mut M::MobjectPresentation,
            reference_mobject: &M,
            alpha: f32,
            queue: &iced::widget::shader::wgpu::Queue,
        ) {
            self.1
                .apply_presentation(mobject_presentation, reference_mobject, alpha, queue);
            self.0
                .apply_presentation(mobject_presentation, reference_mobject, alpha, queue);
        }
    }
}

pub mod derived_impl {
    use super::super::super::mobjects::mobject::Mobject;
    use super::super::act::Act;
    use super::super::construct::Construct;
    use super::super::rates::Rate;
    use super::super::timeline::dynamic::DynamicTimelineMetric;
    use super::super::update::Update;
    use super::traits::ApplyAct;
    use super::traits::ApplyConstruct;
    use super::traits::ApplyRate;
    use super::traits::ApplyUpdate;
    use super::traits::Collapse;
    use super::traits::Destroy;
    use super::traits::Quantize;
    use super::traits::Spawn;
    use super::Supervisor;

    macro_rules! polymorphism {
        (
            $container:ident<
                $($lifetime:lifetime,)*
                $($ty_param:ident,)*
                $(($morphed_ty_param:ident),)*
                $(const $const_ty_param:ident: $const_ty_param_ty:ty,)*
            >
        ) => {
            impl<$($lifetime,)* $($ty_param,)* $($morphed_ty_param,)* $(const $const_ty_param: $const_ty_param_ty,)*> Spawn
            for $container!(<$($lifetime,)* $($ty_param,)* $($morphed_ty_param,)* $($const_ty_param,)*>)
            where
                $($morphed_ty_param: Spawn,)*
            {
                type Output<'s> = $container!(<$($lifetime,)* $($ty_param,)* $($morphed_ty_param::Output<'s>,)* $($const_ty_param,)*>);

                #[must_use]
                #[allow(unused_variables)]
                fn spawn<'s>(self, supervisor: &'s Supervisor) -> Self::Output<'s> {
                    $container!(spawn, self, supervisor,)
                }
            }

            impl<$($lifetime,)* $($ty_param,)* $($morphed_ty_param,)* $(const $const_ty_param: $const_ty_param_ty,)*> Destroy
            for $container!(<$($lifetime,)* $($ty_param,)* $($morphed_ty_param,)* $($const_ty_param,)*>)
            where
                $($morphed_ty_param: Destroy,)*
            {
                #[allow(unused_variables)]
                fn destroy(self) {
                    let _: $container!(<$($lifetime,)* $($ty_param,)* $(${ignore($morphed_ty_param)} (),)* $($const_ty_param,)*>)
                        = $container!(destroy, self,);
                }
            }

            impl<$($lifetime,)* $($ty_param,)* $($morphed_ty_param,)* $(const $const_ty_param: $const_ty_param_ty,)*> Quantize
            for $container!(<$($lifetime,)* $($ty_param,)* $($morphed_ty_param,)* $($const_ty_param,)*>)
            where
                $($morphed_ty_param: Quantize,)*
            {
                type Output<ME> = $container!(<$($lifetime,)* $($ty_param,)* $($morphed_ty_param::Output<ME>,)* $($const_ty_param,)*>)
                where
                    ME: DynamicTimelineMetric;

                #[must_use]
                #[allow(unused_variables)]
                fn quantize<ME>(self, metric: ME) -> Self::Output<ME>
                where
                    ME: DynamicTimelineMetric,
                {
                    $container!(quantize, self, metric.clone(),)
                }
            }

            impl<$($lifetime,)* $($ty_param,)* $($morphed_ty_param,)* $(const $const_ty_param: $const_ty_param_ty,)*> Collapse
            for $container!(<$($lifetime,)* $($ty_param,)* $($morphed_ty_param,)* $($const_ty_param,)*>)
            where
                $($morphed_ty_param: Collapse,)*
            {
                type Output = $container!(<$($lifetime,)* $($ty_param,)* $($morphed_ty_param::Output,)* $($const_ty_param,)*>);

                #[must_use]
                #[allow(unused_variables)]
                fn collapse(self) -> Self::Output {
                    $container!(collapse, self,)
                }
            }

            impl<$($lifetime,)* $($ty_param,)* $($morphed_ty_param,)* $(const $const_ty_param: $const_ty_param_ty,)*> ApplyRate
            for $container!(<$($lifetime,)* $($ty_param,)* $($morphed_ty_param,)* $($const_ty_param,)*>)
            where
                $($morphed_ty_param: ApplyRate,)*
            {
                type Output<RA> = $container!(<$($lifetime,)* $($ty_param,)* $($morphed_ty_param::Output<RA>,)* $($const_ty_param,)*>)
                where
                    RA: Rate;

                #[must_use]
                #[allow(unused_variables)]
                fn apply_rate<RA>(self, rate: RA) -> Self::Output<RA>
                where
                    RA: Rate,
                {
                    $container!(apply_rate, self, rate.clone(),)
                }
            }

            impl<$($lifetime,)* M, $($ty_param,)* $($morphed_ty_param,)* $(const $const_ty_param: $const_ty_param_ty,)*> ApplyAct<M>
            for $container!(<$($lifetime,)* $($ty_param,)* $($morphed_ty_param,)* $($const_ty_param,)*>)
            where
                M: Mobject,
                $($morphed_ty_param: ApplyAct<M>,)*
            {
                type Output<A> = $container!(<$($lifetime,)* $($ty_param,)* $($morphed_ty_param::Output<A>,)* $($const_ty_param,)*>)
                where
                    A: Act<M>;

                #[must_use]
                #[allow(unused_variables)]
                fn apply_act<A>(self, act: A) -> Self::Output<A>
                where
                    A: Act<M>,
                {
                    $container!(apply_act, self, act.clone(),)
                }
            }

            impl<$($lifetime,)* M, $($ty_param,)* $($morphed_ty_param,)* $(const $const_ty_param: $const_ty_param_ty,)*> ApplyUpdate<M>
            for $container!(<$($lifetime,)* $($ty_param,)* $($morphed_ty_param,)* $($const_ty_param,)*>)
            where
                M: Mobject,
                $($morphed_ty_param: ApplyUpdate<M>,)*
            {
                type Output<U> = $container!(<$($lifetime,)* $($ty_param,)* $($morphed_ty_param::Output<U>,)* $($const_ty_param,)*>)
                where
                    U: Update<M>;

                #[must_use]
                #[allow(unused_variables)]
                fn apply_update<U>(self, update: U) -> Self::Output<U>
                where
                    U: Update<M>,
                {
                    $container!(apply_update, self, update.clone(),)
                }
            }

            impl<$($lifetime,)* M, $($ty_param,)* $($morphed_ty_param,)* $(const $const_ty_param: $const_ty_param_ty,)*> ApplyConstruct<M>
            for $container!(<$($lifetime,)* $($ty_param,)* $($morphed_ty_param,)* $($const_ty_param,)*>)
            where
                M: Mobject,
                $($morphed_ty_param: ApplyConstruct<M>,)*
            {
                type Output<C> = $container!(<$($lifetime,)* $($ty_param,)* $($morphed_ty_param::Output<C>,)* $($const_ty_param,)*>)
                where
                    C: Construct<M>;

                #[must_use]
                #[allow(unused_variables)]
                fn apply_construct<C>(self, construct: C) -> Self::Output<C>
                where
                    C: Construct<M>,
                {
                    $container!(apply_construct, self, construct.clone(),)
                }
            }
        };
    }

    macro_rules! polymorphism_tuple {
        ($($i:tt,)*) => {paste::paste!{
            macro_rules! [<tuple_${count($i)}_macro>] {
                (<$(${ignore($i)} $$[<T${index()}>]:ty,)*>) => {
                    ($(${ignore($i)} $$[<T${index()}>],)*)
                };
                ($$method:ident, $$self:expr, $$($$variable:expr,)*) => {
                    ($(${ignore($i)} $$self.${index()}.$$method($$($$variable),*),)*)
                };
            }
            polymorphism!([<tuple_${count($i)}_macro>]<$(${ignore($i)} ([<T${index()}>]),)*>);
        }}
    }
    polymorphism_tuple!();
    polymorphism_tuple!(_,);
    polymorphism_tuple!(_, _,);
    polymorphism_tuple!(_, _, _,);
    polymorphism_tuple!(_, _, _, _,);
    polymorphism_tuple!(_, _, _, _, _,);
    polymorphism_tuple!(_, _, _, _, _, _,);
    polymorphism_tuple!(_, _, _, _, _, _, _,);
    polymorphism_tuple!(_, _, _, _, _, _, _, _,);
    polymorphism_tuple!(_, _, _, _, _, _, _, _, _,);
    polymorphism_tuple!(_, _, _, _, _, _, _, _, _, _,);
    polymorphism_tuple!(_, _, _, _, _, _, _, _, _, _, _,);
    polymorphism_tuple!(_, _, _, _, _, _, _, _, _, _, _, _,);
    polymorphism_tuple!(_, _, _, _, _, _, _, _, _, _, _, _, _,);
    polymorphism_tuple!(_, _, _, _, _, _, _, _, _, _, _, _, _, _,);
    polymorphism_tuple!(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _,);

    macro_rules! array_macro {
        (<$T:ty, $N:ident,>) => {
            [$T; $N]
        };
        ($method:ident, $self:expr, $($variable:expr,)*) => {
            $self.map(|element| element.$method($($variable),*))
        };
    }
    polymorphism!(array_macro<(T), const N: usize,>);

    macro_rules! vec_macro {
        (<$T:ty,>) => {
            Vec<$T>
        };
        ($method:ident, $self:expr, $($variable:expr,)*) => {
            $self.into_iter().map(|element| element.$method($($variable),*)).collect()
        };
    }
    polymorphism!(vec_macro<(T),>);
}

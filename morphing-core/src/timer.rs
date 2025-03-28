use core::range::IterRangeFrom;
use core::range::Range;
use core::range::RangeFrom;
use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::Deref;
use std::rc::Rc;
use std::time::Duration;

pub type Time = f32;
pub type Clock = Duration;

pub struct Timer {
    alive_id_generator: RefCell<IterRangeFrom<usize>>,
    clock: RefCell<Rc<Clock>>,
}

impl Timer {
    pub(crate) fn new() -> Self {
        Self {
            alive_id_generator: RefCell::new(RangeFrom { start: 0 }.into_iter()),
            clock: RefCell::new(Rc::new(Clock::ZERO)),
        }
    }

    pub(crate) fn generate_alive_id(&self) -> usize {
        self.alive_id_generator.borrow_mut().next().unwrap()
    }

    pub(crate) fn clock(&self) -> Rc<Clock> {
        self.clock.borrow().clone()
    }

    pub fn wait(&self, secs: f32) {
        let clock = Clock::from_secs_f32(secs);
        if !clock.is_zero() {
            self.clock
                .replace_with(|rc_clock| Rc::new(**rc_clock + clock));
        }
    }
}

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
#[serde(from = "(Clock, Clock)", into = "(Clock, Clock)")]
pub struct ClockSpan(Range<Clock>);

impl Deref for ClockSpan {
    type Target = Range<Clock>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<(Clock, Clock)> for ClockSpan {
    fn from(value: (Clock, Clock)) -> Self {
        Self(Range {
            start: value.0,
            end: value.1,
        })
    }
}

impl From<ClockSpan> for (Clock, Clock) {
    fn from(value: ClockSpan) -> Self {
        (value.start, value.end)
    }
}

pub trait TimeMetric:
    'static + Copy + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize
{
    fn localize_from_clock(&self, clock: Clock, clock_span: ClockSpan) -> Time;
    fn globalize_into_clock(&self, time: Time, clock_span: ClockSpan) -> Clock;
}

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
pub struct NormalizedTimeMetric;

impl TimeMetric for NormalizedTimeMetric {
    fn localize_from_clock(&self, clock: Clock, clock_span: ClockSpan) -> Time {
        (clock - clock_span.start).as_secs_f32() / (clock_span.end - clock_span.start).as_secs_f32()
    }

    fn globalize_into_clock(&self, time: Time, clock_span: ClockSpan) -> Clock {
        Clock::from_secs_f32(time * (clock_span.end - clock_span.start).as_secs_f32())
            + clock_span.start
    }
}

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
pub struct DenormalizedTimeMetric;

impl TimeMetric for DenormalizedTimeMetric {
    fn localize_from_clock(&self, clock: Clock, clock_span: ClockSpan) -> Time {
        (clock - clock_span.start).as_secs_f32()
    }

    fn globalize_into_clock(&self, time: Time, clock_span: ClockSpan) -> Clock {
        Clock::from_secs_f32(time) + clock_span.start
    }
}

pub trait Rate<TM>:
    'static + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize
{
    fn eval(&self, time: Time) -> Time;
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct IdentityRate;

impl<TM> Rate<TM> for IdentityRate {
    fn eval(&self, time: Time) -> Time {
        time
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ComposeRate<R0, R1>(pub(crate) R0, pub(crate) R1);

impl<TM, R0, R1> Rate<TM> for ComposeRate<R0, R1>
where
    R0: Rate<TM>,
    R1: Rate<TM>,
{
    fn eval(&self, time: Time) -> Time {
        self.0.eval(self.1.eval(time))
    }
}

// mod serde_range {
//     pub fn serialize<Idx, S>(
//         value: &core::range::Range<Idx>,
//         serializer: S,
//     ) -> Result<S::Ok, S::Error>
//     where
//         Idx: Copy + serde::Serialize,
//         S: serde::Serializer,
//     {
//         let range: std::ops::Range<Idx> = (*value).into();
//         serde::Serialize::serialize(&range, serializer)
//     }

//     pub fn deserialize<'d, Idx, D>(deserializer: D) -> Result<core::range::Range<Idx>, D::Error>
//     where
//         Idx: serde::Deserialize<'d>,
//         D: serde::Deserializer<'d>,
//     {
//         let range: std::ops::Range<Idx> = serde::Deserialize::deserialize(deserializer)?;
//         Ok(range.into())
//     }
// }

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// pub struct LocalClockSpan<TM> {
//     pub(crate) time_metric: TM,
//     #[serde(with = "serde_range")]
//     pub(crate) clock_span: Range<Clock>,
// }

// impl<TM> LocalClockSpan<TM>
// where
//     TM: TimeMetric,
// {
//     pub(crate) fn eval(&self, clock: Clock) -> Option<Time> {
//         self.clock_span
//             .contains(&clock)
//             .then(|| self.time_metric.localize_from_clock(clock, self.clock_span))
//     }
// }

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// pub struct GlobalClockSpan<TM> {
//     pub(crate) time_metric: TM,
//     #[serde(with = "serde_range")]
//     pub(crate) clock_span: Range<Clock>,
// }

// impl<TM> GlobalClockSpan<TM>
// where
//     TM: TimeMetric,
// {
//     pub(crate) fn eval(&self, time: Time) -> Clock {
//         self.time_metric.globalize_into_clock(time, self.clock_span)
//     }
// }

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// pub struct ClockTransform<TM, TR> {
//     pub(crate) clock_span: GlobalClockSpan<TM>,
//     pub(crate) time_rate: TR,
// }

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// pub struct RateCompose<R, TR> {
//     pub(crate) rate: R,
//     pub(crate) time_rate: TR,
// }

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// pub struct RateChain<TR0, TM1, TR1> {
//     pub(crate) time_rate: TR0,
//     pub(crate) clock_transform: ClockTransform<TM1, TR1>,
// }

// pub struct FromPhysicalTime<TM> {
//     pub(crate) time_metric: TM,
//     pub(crate) time_interval: Range<Time>, // Local
// }

// pub struct ToPhysicalTime<TM> {
//     pub(crate) time_metric: TM,
//     pub(crate) time_interval: Range<Time>, // Global
// }

// pub trait TimeEval:
//     'static + Clone + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize
// {
//     type OutputTimeMetric: TimeMetric;

//     fn time_eval(&self, time: Time, time_interval: Range<Time>) -> Self::OutputTimeMetric;
// }

// pub trait IncreasingTimeEval: TimeEval {}

// #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
// pub struct NormalizedTimeEval;

// impl TimeEval for NormalizedTimeEval {
//     type OutputTimeMetric = NormalizedTimeMetric;

//     fn time_eval(&self, time: Time, time_interval: Range<Time>) -> Self::OutputTimeMetric {
//         NormalizedTimeMetric(
//             (time - time_interval.start) / (time_interval.end - time_interval.start),
//         )
//     }
// }

// impl IncreasingTimeEval for NormalizedTimeEval {}

// #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
// pub struct DenormalizedTimeEval;

// impl TimeEval for DenormalizedTimeEval {
//     type OutputTimeMetric = DenormalizedTimeMetric;

//     fn time_eval(&self, time: Time, time_interval: Range<Time>) -> Self::OutputTimeMetric {
//         DenormalizedTimeMetric(time - time_interval.start)
//     }
// }

// impl IncreasingTimeEval for DenormalizedTimeEval {}

// impl<R0, R1, TM> Rate<TM> for Compose<R0, R1>
// where
//     TM: TimeMetric,
//     R0: Rate<TM>,
//     R1: Rate<TM>,
// {
//     fn eval(&self, time_metric: TM) -> TM {
//         self.0.eval(self.1.eval(time_metric))
//     }
// }

// pub trait ClockSpan {
//     fn clock_span(&self) -> &Range<Clock>;
// }

// pub trait TimeRate:
//     'static + Debug + Send + Sync + for<'de> serde::Deserialize<'de> + serde::Serialize + ClockSpan
// {
//     type TimeMetric: TimeMetric;

//     fn eval_time(&self, clock: Clock) -> Time;
// }

// impl<TM> ClockSpan for LocalClockSpan<TM>
// where
//     TM: TimeMetric,
// {
//     fn clock_span(&self) -> &Range<Clock> {
//         &self.clock_span
//     }
// }

// impl<TM> TimeRate for LocalClockSpan<TM>
// where
//     TM: TimeMetric,
// {
//     type TimeMetric = TM;

//     fn eval_time(&self, clock: Clock) -> Time {
//         TM::localize_from_clock(clock, self.clock_span)
//     }
// }

// impl<R, TR> ClockSpan for RateCompose<R, TR>
// where
//     R: Rate<TR::TimeMetric>,
//     TR: TimeRate,
// {
//     fn clock_span(&self) -> &Range<Clock> {
//         self.time_rate.clock_span()
//     }
// }

// impl<R, TR> TimeRate for RateCompose<R, TR>
// where
//     R: Rate<TR::TimeMetric>,
//     TR: TimeRate,
// {
//     type TimeMetric = TR::TimeMetric;

//     fn eval_time(&self, clock: Clock) -> Time {
//         self.rate.eval(self.time_rate.eval_time(clock))
//     }
// }

// impl<TR0, TR1> ClockSpan for RateChain<TR0, TR1::TimeMetric, TR1>
// where
//     TR0: TimeRate,
//     TR1: TimeRate,
// {
//     fn clock_span(&self) -> &Range<Clock> {
//         self.clock_transform.time_rate.clock_span()
//     }
// }

// impl<TR0, TR1> TimeRate for RateChain<TR0, TR1::TimeMetric, TR1>
// where
//     TR0: TimeRate,
//     TR1: TimeRate,
// {
//     type TimeMetric = TR0::TimeMetric;

//     fn eval_time(&self, clock: Clock) -> Time {
//         self.time_rate.eval_time(TR1::TimeMetric::globalize_into_clock(
//             self.clock_transform.time_rate.eval_time(clock),
//             self.clock_transform.clock_span.clock_span,
//         ))
//     }
// }

// impl<R, TMC, TR> TimeRate for Compose<Compose<R, TMC>, TR>
// where
//     R: Rate<TMC::TimeMetric>,
//     TMC: TimeMetricConvert,
//     TR: TimeRate,
// {
//     type OutputTimeMetric = TMC::TimeMetric;

//     fn eval(&self, time: Time, time_interval: Range<Time>) -> Option<Self::OutputTimeMetric> {
//         self.1.eval(time, time_interval).and_then(|time_metric| self.0.1.metric_to_time(time_metric, time_interval))
//     }
// }

// impl<TR0, TR1> TimeRate for Compose<TR0, TR1>
// where
//     TR0: TimeRate,
//     TR1: TimeRate,
// {
//     type OutputTimeMetric = TR0::OutputTimeMetric;

//     fn eval(&self, time: Time, time_interval: Range<Time>) -> Option<Self::OutputTimeMetric> {
//         self.1.eval(time, time_interval).and_then(|time_metric| )
//     }
// }

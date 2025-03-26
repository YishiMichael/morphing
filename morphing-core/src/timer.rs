use core::range::IterRangeFrom;
use core::range::Range;
use core::range::RangeFrom;
use std::cell::RefCell;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

pub type Time = f32;

pub struct Timer {
    alive_id_generator: RefCell<IterRangeFrom<usize>>,
    time: RefCell<Rc<Time>>,
}

impl Timer {
    pub(crate) fn new() -> Self {
        Self {
            alive_id_generator: RefCell::new(RangeFrom { start: 0 }.into_iter()),
            time: RefCell::new(Rc::new(0.0)),
        }
    }

    pub(crate) fn generate_alive_id(&self) -> usize {
        self.alive_id_generator.borrow_mut().next().unwrap()
    }

    pub(crate) fn time(&self) -> Rc<Time> {
        self.time.borrow().clone()
    }

    pub fn wait(&self, time: Time) {
        self.time.replace_with(|rc_time| Rc::new(**rc_time + time));
    }
}

pub trait TimeMetric:
    'static + Copy + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
{
    type MetricTime: Deref<Target = Time>;
    fn to_metric(time: Time, time_interval: Range<Time>) -> Self::MetricTime;
    fn from_metric(metric_time: Self::MetricTime, time_interval: Range<Time>) -> Time;
    fn apply_rate<R>(rate: &R, metric_time: Self::MetricTime) -> Self::MetricTime
    where
        R: Rate<Self>;
}

pub struct NormalizedMetricTime(Time);

impl Deref for NormalizedMetricTime {
    type Target = Time;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
pub struct NormalizedTimeMetric;

impl TimeMetric for NormalizedTimeMetric {
    type MetricTime = NormalizedMetricTime;

    fn to_metric(time: Time, time_interval: Range<Time>) -> Self::MetricTime {
        NormalizedMetricTime(
            (time - time_interval.start) / (time_interval.end - time_interval.start),
        )
    }

    fn from_metric(metric_time: Self::MetricTime, time_interval: Range<Time>) -> Time {
        metric_time.0 * (time_interval.end - time_interval.start) + time_interval.start
    }

    fn apply_rate<R>(rate: &R, metric_time: Self::MetricTime) -> Self::MetricTime
    where
        R: Rate<Self>,
    {
        NormalizedMetricTime(rate.eval(metric_time.0))
    }
}

pub struct DenormalizedMetricTime(Time);

impl Deref for DenormalizedMetricTime {
    type Target = Time;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
pub struct DenormalizedTimeMetric;

impl TimeMetric for DenormalizedTimeMetric {
    type MetricTime = DenormalizedMetricTime;

    fn to_metric(time: Time, time_interval: Range<Time>) -> Self::MetricTime {
        DenormalizedMetricTime(time - time_interval.start)
    }

    fn from_metric(metric_time: Self::MetricTime, time_interval: Range<Time>) -> Time {
        metric_time.0 + time_interval.start
    }

    fn apply_rate<R>(rate: &R, metric_time: Self::MetricTime) -> Self::MetricTime
    where
        R: Rate<Self>,
    {
        DenormalizedMetricTime(rate.eval(metric_time.0))
    }
}

pub trait Rate<TM>:
    'static + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
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

mod serde_range {
    pub fn serialize<Idx, S>(
        value: &core::range::Range<Idx>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        Idx: Copy + serde::Serialize,
        S: serde::Serializer,
    {
        let range: std::ops::Range<Idx> = (*value).into();
        serde::Serialize::serialize(&range, serializer)
    }

    pub fn deserialize<'d, Idx, D>(deserializer: D) -> Result<core::range::Range<Idx>, D::Error>
    where
        Idx: serde::Deserialize<'d>,
        D: serde::Deserializer<'d>,
    {
        let range: std::ops::Range<Idx> = serde::Deserialize::deserialize(deserializer)?;
        Ok(range.into())
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LocalTimeInterval<TM> {
    #[serde(with = "serde_range")]
    pub(crate) time_interval: Range<Time>,
    pub(crate) time_metric: PhantomData<TM>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct GlobalTimeInterval<TM> {
    #[serde(with = "serde_range")]
    pub(crate) time_interval: Range<Time>,
    pub(crate) time_metric: PhantomData<TM>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RateTransform<TR, TM> {
    pub(crate) time_rate: TR,
    pub(crate) global_time_interval: GlobalTimeInterval<TM>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RateCompose<R, TR> {
    pub(crate) rate: R,
    pub(crate) time_rate: TR,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RateChain<TR0, TM1, TR1> {
    pub(crate) rate_transform: RateTransform<TR0, TM1>,
    pub(crate) time_rate: TR1,
}

// pub struct FromPhysicalTime<TM> {
//     pub(crate) time_metric: TM,
//     pub(crate) time_interval: Range<Time>, // Local
// }

// pub struct ToPhysicalTime<TM> {
//     pub(crate) time_metric: TM,
//     pub(crate) time_interval: Range<Time>, // Global
// }

// pub trait TimeEval:
//     'static + Clone + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
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

pub trait TimeRate:
    'static + Debug + Send + Sync + serde::de::DeserializeOwned + serde::Serialize
{
    type TimeMetric: TimeMetric;

    fn eval_time(&self, time: Time) -> <Self::TimeMetric as TimeMetric>::MetricTime;
}

impl<TM> TimeRate for LocalTimeInterval<TM>
where
    TM: TimeMetric,
{
    type TimeMetric = TM;

    fn eval_time(&self, time: Time) -> <Self::TimeMetric as TimeMetric>::MetricTime {
        TM::to_metric(time, self.time_interval)
    }
}

impl<R, TR> TimeRate for RateCompose<R, TR>
where
    R: Rate<TR::TimeMetric>,
    TR: TimeRate,
{
    type TimeMetric = TR::TimeMetric;

    fn eval_time(&self, time: Time) -> <Self::TimeMetric as TimeMetric>::MetricTime {
        TR::TimeMetric::apply_rate(&self.rate, self.time_rate.eval_time(time))
    }
}

impl<TR0, TR1> TimeRate for RateChain<TR0, TR1::TimeMetric, TR1>
where
    TR0: TimeRate,
    TR1: TimeRate,
{
    type TimeMetric = TR0::TimeMetric;

    fn eval_time(&self, time: Time) -> <Self::TimeMetric as TimeMetric>::MetricTime {
        self.rate_transform
            .time_rate
            .eval_time(TR1::TimeMetric::from_metric(
                self.time_rate.eval_time(time),
                self.rate_transform.global_time_interval.time_interval,
            ))
    }
}

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

use itertools::Itertools;

// #[derive(Clone)]
// pub struct Path(pub lyon::path::Path);

#[derive(Clone)]
pub struct Path(Vec<bezier_rs::Subpath<ManipulatorGroupId>>);

// TODO: port ctors from bezier_rs::Subpath

impl Path {
    pub fn concat<I: IntoIterator<Item = Self>>(iter: I) -> Self {
        Self::from_iter(iter.into_iter().flat_map(|path| path.0))
    }

    pub fn subpaths(&self) -> &[bezier_rs::Subpath<ManipulatorGroupId>] {
        &self.0
    }

    pub fn dash(&self, array: &[f64], phase: f64) -> Self {
        Self::from_iter(self.subpaths().into_iter().flat_map(|subpath| {
            let total_length = subpath.length(None);
            let phase = phase / total_length;
            let mut alphas = array
                .into_iter()
                .map(|length| length / total_length)
                .scan(0.0, |alpha_acc, alpha| {
                    *alpha_acc += alpha;
                    Some(*alpha_acc)
                })
                .collect_vec();
            alphas.rotate_right(1);
            let alpha_period = alphas
                .get_mut(0)
                .map(|alpha| std::mem::take(alpha))
                .unwrap_or_default();
            (-(phase / alpha_period).ceil() as i32..((1.0 - phase) / alpha_period).ceil() as i32)
                .map(move |i| i as f64 * alpha_period + phase)
                .flat_map(move |alpha_offset| {
                    alphas
                        .clone()
                        .into_iter()
                        .tuples()
                        .filter_map(move |(alpha_0, alpha_1)| {
                            let alpha_0 = (alpha_offset + alpha_0).max(0.0);
                            let alpha_1 = (alpha_offset + alpha_1).min(1.0);
                            (alpha_0 < alpha_1).then_some((alpha_0, alpha_1))
                        })
                })
                .map(|(alpha_0, alpha_1)| {
                    subpath.trim(
                        bezier_rs::SubpathTValue::GlobalEuclidean(alpha_0),
                        bezier_rs::SubpathTValue::GlobalEuclidean(alpha_1),
                    )
                })
        }))
    }

    pub fn from_lyon_path(path: &lyon::path::Path) -> Self {
        #[inline]
        fn convert_point(lyon::geom::Point { x, y, .. }: lyon::geom::Point<f32>) -> (f64, f64) {
            (x as f64, y as f64)
        }

        let mut event_iter = path.iter();
        let mut subpaths = Vec::new();
        while event_iter.next().is_some() {
            let beziers = event_iter
                .take_while_ref(|event| !matches!(event, &lyon::path::PathEvent::End { .. }))
                .map(|event| match event {
                    lyon::path::PathEvent::Line {
                        from: start,
                        to: end,
                    } => bezier_rs::Bezier {
                        start: convert_point(start).into(),
                        end: convert_point(end).into(),
                        handles: bezier_rs::BezierHandles::Linear,
                    },
                    lyon::path::PathEvent::Quadratic {
                        from: start,
                        ctrl: handle,
                        to: end,
                    } => bezier_rs::Bezier {
                        start: convert_point(start).into(),
                        end: convert_point(end).into(),
                        handles: bezier_rs::BezierHandles::Quadratic {
                            handle: convert_point(handle).into(),
                        },
                    },
                    lyon::path::PathEvent::Cubic {
                        from: start,
                        ctrl1: handle_start,
                        ctrl2: handle_end,
                        to: end,
                    } => bezier_rs::Bezier {
                        start: convert_point(start).into(),
                        end: convert_point(end).into(),
                        handles: bezier_rs::BezierHandles::Cubic {
                            handle_start: convert_point(handle_start).into(),
                            handle_end: convert_point(handle_end).into(),
                        },
                    },
                    _ => unreachable!(),
                })
                .collect_vec();
            let closed = match event_iter.next() {
                Some(lyon::path::PathEvent::End { close, .. }) => close,
                _ => unreachable!(),
            };
            subpaths.push(bezier_rs::Subpath::from_beziers(&beziers, closed));
        }
        Self(subpaths)
    }

    pub fn to_lyon_path(&self) -> lyon::path::Path {
        #[inline]
        fn convert_point((x, y): (f64, f64)) -> lyon::geom::Point<f32> {
            lyon::geom::point(x as f32, y as f32)
        }

        lyon::path::Path::from_iter(self.subpaths().iter().flat_map(|subpath| {
            let begin_point = convert_point(subpath[0].anchor.into());
            let end_point = convert_point(subpath[subpath.len() - 1].anchor.into());
            std::iter::once(lyon::path::PathEvent::Begin { at: begin_point })
                .chain(subpath.iter().map(
                    |bezier_rs::Bezier {
                         start: from,
                         end: to,
                         handles,
                     }| {
                        match handles {
                            bezier_rs::BezierHandles::Linear => lyon::path::PathEvent::Line {
                                from: convert_point(from.into()),
                                to: convert_point(to.into()),
                            },
                            bezier_rs::BezierHandles::Quadratic { handle: ctrl } => {
                                lyon::path::PathEvent::Quadratic {
                                    from: convert_point(from.into()),
                                    ctrl: convert_point(ctrl.into()),
                                    to: convert_point(to.into()),
                                }
                            }
                            bezier_rs::BezierHandles::Cubic {
                                handle_start: ctrl1,
                                handle_end: ctrl2,
                            } => lyon::path::PathEvent::Cubic {
                                from: convert_point(from.into()),
                                ctrl1: convert_point(ctrl1.into()),
                                ctrl2: convert_point(ctrl2.into()),
                                to: convert_point(to.into()),
                            },
                        }
                    },
                ))
                .chain(std::iter::once(lyon::path::PathEvent::End {
                    last: end_point,
                    first: begin_point,
                    close: subpath.closed,
                }))
                .collect_vec()
        }))
    }
}

impl FromIterator<bezier_rs::Subpath<ManipulatorGroupId>> for Path {
    fn from_iter<T: IntoIterator<Item = bezier_rs::Subpath<ManipulatorGroupId>>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

pub struct PathBuilder {
    builder: lyon::path::Builder,
    closed: bool,
}

impl PathBuilder {
    pub fn new() -> Self {
        Self {
            builder: lyon::path::Builder::new(),
            closed: true,
        }
    }

    pub fn build(mut self) -> Path {
        if !self.closed {
            self.builder.end(false);
        }
        Path::from_lyon_path(&self.builder.build())
    }
}

impl ttf_parser::OutlineBuilder for PathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        if !self.closed {
            self.builder.end(false);
        }
        self.closed = false;
        self.builder.begin(lyon::geom::point(x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.closed = false;
        self.builder.line_to(lyon::geom::point(x, y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.closed = false;
        self.builder
            .quadratic_bezier_to(lyon::geom::point(x1, y1), lyon::geom::point(x, y));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.closed = false;
        self.builder.cubic_bezier_to(
            lyon::geom::point(x1, y1),
            lyon::geom::point(x2, y2),
            lyon::geom::point(x, y),
        );
    }

    fn close(&mut self) {
        self.closed = true;
        self.builder.end(true);
    }
}

#[derive(Clone, Hash, PartialEq)]
pub struct ManipulatorGroupId(usize);

impl bezier_rs::Identifier for ManipulatorGroupId {
    fn new() -> Self {
        let mut counter_ref = MANIPULATOR_GROUP_ID_COUNTER.lock();
        *counter_ref += 1;
        Self(*counter_ref)
    }
}

static MANIPULATOR_GROUP_ID_COUNTER: parking_lot::Mutex<usize> = parking_lot::Mutex::new(0);

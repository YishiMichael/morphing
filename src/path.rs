use itertools::Itertools;

#[derive(Clone)]
pub struct Path(pub lyon::path::Path);

#[derive(Clone)]
pub struct Subpaths(pub Vec<bezier_rs::Subpath<ManipulatorGroupId>>);

impl Path {
    // fn from_bezier_subpaths(subpaths: Vec<bezier_rs::Subpath<ManipulatorGroupId>>) -> Self {
    //     subpaths.into_iter()
    // }

    // fn into_to_bezier_subpaths(self) -> Vec<bezier_rs::Subpath<ManipulatorGroupId>> {
    //     SubpathIter(self.0.into_iter()).collect()
    // }

    // fn to_bezier_subpaths(&self) -> Vec<bezier_rs::Subpath<ManipulatorGroupId>> {
    //     SubpathIter(self.0.iter()).collect()
    // }

    // fn fill_triangulation(
    //     &self,
    //     options: &lyon::tessellation::FillOptions,
    // ) -> lyon::tessellation::VertexBuffers<lyon::math::Point, u16> {
    //     let mut buffers = lyon::tessellation::VertexBuffers::new();
    //     lyon::tessellation::FillTessellator::new().tessellate(
    //         &self.0,
    //         options,
    //         &mut lyon::tessellation::geometry_builder::simple_builder(&mut buffers),
    //     ); // TODO: handle err
    //     buffers
    // }

    // fn stroke_triangulation(
    //     &self,
    //     options: &lyon::tessellation::StrokeOptions,
    // ) -> lyon::tessellation::VertexBuffers<lyon::math::Point, u16> {
    //     let mut buffers = lyon::tessellation::VertexBuffers::new();
    //     lyon::tessellation::StrokeTessellator::new().tessellate(
    //         &self.0,
    //         options,
    //         &mut lyon::tessellation::geometry_builder::simple_builder(&mut buffers),
    //     ); // TODO: handle err
    //     buffers
    // }
}

impl From<&Subpaths> for Path {
    fn from(subpaths: &Subpaths) -> Self {
        #[inline]
        fn convert_point((x, y): (f64, f64)) -> lyon::geom::Point<f32> {
            lyon::geom::Point::new(x as f32, y as f32)
        }

        Self(lyon::path::Path::from_iter(subpaths.0.iter().flat_map(
            |subpath| {
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
            },
        )))
    }
}

impl From<Subpaths> for Path {
    #[inline]
    fn from(subpaths: Subpaths) -> Self {
        Self::from(&subpaths)
    }
}

impl From<&Path> for Subpaths {
    fn from(path: &Path) -> Self {
        #[inline]
        fn convert_point(lyon::geom::Point { x, y, .. }: lyon::geom::Point<f32>) -> (f64, f64) {
            (x as f64, y as f64)
        }

        let mut event_iter = path.0.iter();
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
}

impl From<Path> for Subpaths {
    #[inline]
    fn from(path: Path) -> Self {
        Self::from(&path)
    }
}

// struct EventIter<I> {
//     subpath_iter: I,
//     remaining_event_iter: Option<?>,
// }

// impl<'a, I: 'a + Iterator<Item = bezier_rs::Subpath<ManipulatorGroupId>>> Iterator for EventIter<I> {
//     type Item = lyon::path::PathEvent;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.subpath_iter.next().map(||)
//     }
// }

// struct SubpathIter<I> {
//     event_iter: I,
// }

// impl<'a, I: 'a + Iterator<Item = lyon::path::PathEvent> + Clone> Iterator for SubpathIter<I> {
//     type Item = bezier_rs::Subpath<ManipulatorGroupId>;

//     fn next(&mut self) -> Option<Self::Item> {
//         fn convert_point(lyon::geom::Point { x, y, .. }: lyon::geom::Point<f32>) -> (f64, f64) {
//             (x as f64, y as f64)
//         }

//         self.event_iter.next().map(|_| {
//             let beziers = self
//                 .event_iter
//                 .take_while_ref(|event| !matches!(event, &lyon::path::PathEvent::End { .. }))
//                 .map(|event| match event {
//                     lyon::path::PathEvent::Line { from, to } => {
//                         bezier_rs::Bezier::from_linear_dvec2(
//                             convert_point(from).into(),
//                             convert_point(to).into(),
//                         )
//                     }
//                     lyon::path::PathEvent::Quadratic { from, ctrl, to } => {
//                         bezier_rs::Bezier::from_quadratic_dvec2(
//                             convert_point(from).into(),
//                             convert_point(ctrl).into(),
//                             convert_point(to).into(),
//                         )
//                     }
//                     lyon::path::PathEvent::Cubic {
//                         from,
//                         ctrl1,
//                         ctrl2,
//                         to,
//                     } => bezier_rs::Bezier::from_cubic_dvec2(
//                         convert_point(from).into(),
//                         convert_point(ctrl1).into(),
//                         convert_point(ctrl2).into(),
//                         convert_point(to).into(),
//                     ),
//                     _ => unreachable!(),
//                 })
//                 .collect_vec();
//             let closed = match self.0.next() {
//                 Some(lyon::path::PathEvent::End { close, .. }) => close,
//                 _ => unreachable!(),
//             };
//             bezier_rs::Subpath::from_beziers(&beziers, closed)
//         })
//     }
// }

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

pub mod fill;
pub mod mobject;
pub mod path;
pub mod stroke;
pub mod world;

use fill::Fill;
use itertools::Itertools;
use mobject::Mobject;
use path::ManipulatorGroupId;
use path::Path;
use path::Subpaths;
use stroke::Stroke;
use world::TypstWorld;

fn typst_path_to_subpath(path: typst::visualize::Path) -> bezier_rs::Subpath<ManipulatorGroupId> {
    #[inline]
    fn convert_point(typst::layout::Point { x, y }: typst::layout::Point) -> (f64, f64) {
        (x.to_raw(), y.to_raw())
    }

    let mut anchor = None;
    let mut closed = false;
    let beziers = path
        .0
        .into_iter()
        .filter_map(|path_item| match path_item {
            typst::visualize::PathItem::MoveTo(start) => {
                assert!(anchor.replace(convert_point(start)).is_none());
                None
            }
            typst::visualize::PathItem::LineTo(end) => {
                let end = convert_point(end);
                Some(bezier_rs::Bezier {
                    start: anchor.replace(end).unwrap().into(),
                    end: end.into(),
                    handles: bezier_rs::BezierHandles::Linear,
                })
            }
            typst::visualize::PathItem::CubicTo(handle_start, handle_end, end) => {
                let end = convert_point(end);
                Some(bezier_rs::Bezier {
                    start: anchor.replace(end).unwrap().into(),
                    end: end.into(),
                    handles: bezier_rs::BezierHandles::Cubic {
                        handle_start: convert_point(handle_start).into(),
                        handle_end: convert_point(handle_end).into(),
                    },
                })
            }
            typst::visualize::PathItem::ClosePath => {
                assert!(anchor.take().is_some());
                closed = true;
                None
            }
        })
        .collect_vec();
    bezier_rs::Subpath::from_beziers(&beziers, closed)
}

pub fn typst_shape_to_mobjects(shape: typst::visualize::Shape) -> Vec<Box<dyn Mobject>> {
    let subpath = typst_path_to_subpath(match shape.geometry {
        typst::visualize::Geometry::Line(point) => {
            let mut path = typst::visualize::Path::new();
            path.line_to(point);
            path
        }
        typst::visualize::Geometry::Rect(size) => typst::visualize::Path::rect(size),
        typst::visualize::Geometry::Path(path) => path,
    });

    let mut mobjects: Vec<Box<dyn Mobject>> = Vec::new();
    if let Some(fill) = shape.fill {
        if let typst::visualize::Paint::Solid(color) = fill {
            mobjects.push(Box::new(Fill {
                path: Path::from(Subpaths::from(subpath.clone())),
                color: rgb::Rgba::from(color.to_vec4()),
                options: match shape.fill_rule {
                    typst::visualize::FillRule::NonZero => {
                        lyon::tessellation::FillOptions::non_zero()
                    }
                    typst::visualize::FillRule::EvenOdd => {
                        lyon::tessellation::FillOptions::even_odd()
                    }
                },
            }));
        } else {
            panic!("Unsopported paint");
        }
    }
    if let Some(stroke) = shape.stroke {
        if let typst::visualize::Paint::Solid(color) = stroke.paint {
            let cap = match stroke.cap {
                typst::visualize::LineCap::Butt => lyon::tessellation::LineCap::Butt,
                typst::visualize::LineCap::Round => lyon::tessellation::LineCap::Round,
                typst::visualize::LineCap::Square => lyon::tessellation::LineCap::Square,
            };
            let join = match stroke.join {
                typst::visualize::LineJoin::Miter => lyon::tessellation::LineJoin::Miter,
                typst::visualize::LineJoin::Round => lyon::tessellation::LineJoin::Round,
                typst::visualize::LineJoin::Bevel => lyon::tessellation::LineJoin::Bevel,
            };
            mobjects.push(Box::new(Stroke {
                path: if let Some(dash_pattern) = stroke.dash {
                    let total_length = subpath.length(None);
                    let phase = dash_pattern.phase.to_raw() / total_length;
                    let mut alphas = dash_pattern
                        .array
                        .into_iter()
                        .map(|length| length.to_raw() / total_length)
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
                    Path::from(Subpaths(
                        (-(phase / alpha_period).ceil() as i32
                            ..((1.0 - phase) / alpha_period).ceil() as i32)
                            .flat_map(|i| {
                                alphas
                                    .iter()
                                    .tuples()
                                    .filter_map(move |(&alpha_0, &alpha_1)| {
                                        let alpha_0 = (i as f64 * alpha_period + alpha_0).max(0.0);
                                        let alpha_1 = (i as f64 * alpha_period + alpha_1).min(1.0);
                                        (alpha_0 < alpha_1).then_some((alpha_0, alpha_1))
                                    })
                            })
                            .map(|(alpha_0, alpha_1)| {
                                subpath.trim(
                                    bezier_rs::SubpathTValue::GlobalEuclidean(alpha_0),
                                    bezier_rs::SubpathTValue::GlobalEuclidean(alpha_1),
                                )
                            })
                            .collect(),
                    ))
                } else {
                    Path::from(Subpaths::from(subpath))
                },
                color: rgb::Rgba::from(color.to_vec4()),
                options: lyon::tessellation::StrokeOptions::default()
                    .with_line_width(stroke.thickness.to_raw() as f32)
                    .with_start_cap(cap)
                    .with_end_cap(cap)
                    .with_line_join(join)
                    .with_miter_limit(stroke.miter_limit.get() as f32),
            }));
        } else {
            panic!("Unsopported paint");
        }
    }

    mobjects
}

// fn visit_frame_items<F: FnMut(typst::layout::FrameItem, typst::layout::Transform)>(
//     f: F,
//     frame: typst::layout::Frame,
//     ts: typst::layout::Transform,
// ) {
//     for (pos, item) in frame.items() {
//         // File size optimization.
//         // TODO: SVGs could contain links, couldn't they?
//         if matches!(item, FrameItem::Link(_, _) | FrameItem::Tag(_)) {
//             continue;
//         }

//         let x = pos.x.to_pt();
//         let y = pos.y.to_pt();
//         self.xml.start_element("g");
//         self.xml
//             .write_attribute_fmt("transform", format_args!("translate({x} {y})"));

//         match item {
//             FrameItem::Group(group) => self.render_group(state.pre_translate(*pos), group),
//             FrameItem::Text(text) => self.render_text(state.pre_translate(*pos), text),
//             FrameItem::Shape(shape, _) => self.render_shape(state.pre_translate(*pos), shape),
//             FrameItem::Image(image, size, _) => self.render_image(image, size),
//             FrameItem::Link(_, _) => unreachable!(),
//             FrameItem::Tag(_) => unreachable!(),
//         };

//         self.xml.end_element();
//     }
// }

fn main() -> () {
    let world = TypstWorld::default();

    let text = "text".to_string();

    let document = world.document(text);

    let frame = document.pages.into_iter().exactly_one().unwrap().frame; // TODO: unwrap

    // frame.items().for_each(|a|);

    // println!("{content:?}");
    // println!("{document:?}");
    // let svg = typst_svg::svg_merged(&document, typst::layout::Abs::zero());
    // println!("{svg}");
}

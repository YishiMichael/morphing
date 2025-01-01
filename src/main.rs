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
    fn convert_point(point: typst::layout::Point) -> lyon::math::Point {
        lyon::math::point(point.x.to_raw() as f32, point.y.to_raw() as f32)
    }

    let (mut builder, closed) = path.0.into_iter().fold(
        (lyon::path::Path::builder(), false),
        |(mut builder, closed), path_item| match path_item {
            typst::visualize::PathItem::MoveTo(at) => {
                if !closed {
                    builder.end(false);
                }
                builder.begin(convert_point(at));
                (builder, false)
            }
            typst::visualize::PathItem::LineTo(to) => {
                builder.line_to(convert_point(to));
                (builder, false)
            }
            typst::visualize::PathItem::CubicTo(ctrl1, ctrl2, to) => {
                builder.cubic_bezier_to(
                    convert_point(ctrl1),
                    convert_point(ctrl2),
                    convert_point(to),
                );
                (builder, false)
            }
            typst::visualize::PathItem::ClosePath => {
                builder.end(true);
                (builder, true)
            }
        },
    );
    if !closed {
        builder.end(false);
    }
    Path(builder.build())
}

fn typst_shape_to_mobjects(shape: typst::visualize::Shape) -> Vec<Box<dyn Mobject>> {
    let path = typst_path_to_path(match shape.geometry {
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
                path: path.clone(),
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
                    Path::from(
                        Subpaths::from(path)
                            .0
                            .iter()
                            .map(|subpath| Subpaths::flatten(iter)),
                    )
                } else {
                    path
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

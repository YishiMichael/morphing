pub mod fill;
pub mod mobject;
pub mod path;
pub mod stroke;
pub mod world;

use std::borrow::Cow;

use itertools::Itertools;

use fill::Fill;
use mobject::Mobject;
use path::Path;
use path::PathBuilder;
use stroke::Stroke;
use ttf_parser::OutlineBuilder;
use world::TypstWorld;

fn typst_path_to_path(path: &typst::visualize::Path) -> Path {
    let mut builder = PathBuilder::new();
    path.0.iter().for_each(|path_item| match path_item {
        &typst::visualize::PathItem::MoveTo(start) => {
            builder.move_to(start.x.to_pt() as f32, start.y.to_pt() as f32)
        }
        &typst::visualize::PathItem::LineTo(end) => {
            builder.line_to(end.x.to_pt() as f32, end.y.to_pt() as f32)
        }
        &typst::visualize::PathItem::CubicTo(handle_start, handle_end, end) => builder.curve_to(
            handle_start.x.to_pt() as f32,
            handle_start.y.to_pt() as f32,
            handle_end.x.to_pt() as f32,
            handle_end.y.to_pt() as f32,
            end.x.to_pt() as f32,
            end.y.to_pt() as f32,
        ),
        &typst::visualize::PathItem::ClosePath => builder.close(),
    });
    builder.build()
}

fn outline_glyph_to_path(font: &typst::text::Font, id: ttf_parser::GlyphId) -> Option<Path> {
    let mut builder = PathBuilder::new();
    font.ttf().outline_glyph(id, &mut builder)?;
    Some(builder.build())
}

fn path_to_mobjects(
    path: Path,
    fill_rule: Option<typst::visualize::FillRule>,
    fill: Option<&typst::visualize::Paint>,
    stroke: Option<&typst::visualize::FixedStroke>,
) -> Vec<Box<dyn Mobject>> {
    #[inline]
    fn paint_to_color(paint: &typst::visualize::Paint) -> rgb::Rgba<f32> {
        if let typst::visualize::Paint::Solid(color) = paint {
            rgb::Rgba::from(color.to_vec4())
        } else {
            panic!("Unsopported paint");
        }
    }

    let stroke_mobject = stroke.map(|stroke| Stroke {
        path: if let Some(dash_pattern) = &stroke.dash {
            path.dash(
                &dash_pattern
                    .array
                    .iter()
                    .map(|length| length.to_pt())
                    .collect_vec(),
                dash_pattern.phase.to_pt(),
            )
        } else {
            path.clone()
        },
        color: paint_to_color(&stroke.paint),
        options: {
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
            lyon::tessellation::StrokeOptions::default()
                .with_line_width(stroke.thickness.to_pt() as f32)
                .with_start_cap(cap)
                .with_end_cap(cap)
                .with_line_join(join)
                .with_miter_limit(stroke.miter_limit.get() as f32)
        },
    });
    let fill_mobject = fill.map(|fill| Fill {
        path,
        color: paint_to_color(fill),
        options: match fill_rule.unwrap_or_default() {
            typst::visualize::FillRule::NonZero => lyon::tessellation::FillOptions::non_zero(),
            typst::visualize::FillRule::EvenOdd => lyon::tessellation::FillOptions::even_odd(),
        },
    });

    let mut mobjects: Vec<Box<dyn Mobject>> = Vec::new();
    if let Some(fill_mobject) = fill_mobject {
        mobjects.push(Box::new(fill_mobject));
    }
    if let Some(stroke_mobject) = stroke_mobject {
        mobjects.push(Box::new(stroke_mobject));
    }
    mobjects
}

fn typst_shape_to_mobjects(
    shape: &typst::visualize::Shape,
    span: typst::syntax::Span,
    transform: typst::layout::Transform,
) -> Vec<(
    typst::syntax::Span,
    Box<dyn Mobject>,
    typst::layout::Transform,
)> {
    let typst_path = match &shape.geometry {
        &typst::visualize::Geometry::Line(point) => {
            let mut path = typst::visualize::Path::new();
            path.line_to(point);
            Cow::Owned(path)
        }
        &typst::visualize::Geometry::Rect(size) => Cow::Owned(typst::visualize::Path::rect(size)),
        &typst::visualize::Geometry::Path(ref path) => Cow::Borrowed(path),
    };
    path_to_mobjects(
        typst_path_to_path(&typst_path),
        Some(shape.fill_rule),
        shape.fill.as_ref(),
        shape.stroke.as_ref(),
    )
    .into_iter()
    .map(|mobject| (span, mobject, transform))
    .collect()
}

fn typst_text_to_mobjects(
    text: &typst::text::TextItem,
    transform: typst::layout::Transform,
) -> Vec<(
    typst::syntax::Span,
    Box<dyn Mobject>,
    typst::layout::Transform,
)> {
    let scale = typst::layout::Ratio::new(text.size.to_pt() / text.font.units_per_em());
    text.glyphs
        .iter()
        .scan(typst::layout::Abs::pt(0.0), |x, glyph| {
            let offset = *x + glyph.x_offset.at(text.size);
            *x += glyph.x_advance.at(text.size);

            let transform = transform
                .pre_concat(typst::layout::Transform::scale(
                    typst::layout::Ratio::one(),
                    -typst::layout::Ratio::one(),
                ))
                .pre_concat(typst::layout::Transform::translate(
                    offset,
                    typst::layout::Abs::zero(),
                ))
                .pre_concat(typst::layout::Transform::scale(scale, scale));

            Some((transform, glyph))
        })
        .flat_map(|(transform, glyph)| {
            outline_glyph_to_path(&text.font, ttf_parser::GlyphId(glyph.id))
                .into_iter()
                .flat_map(move |path| {
                    path_to_mobjects(path, None, Some(&text.fill), text.stroke.as_ref())
                        .into_iter()
                        .map(
                            move |mobject| (glyph.span.0, mobject, transform), // TODO: option span?)
                        )
                })
        })
        .collect()
}

pub fn typst_frame_to_mobjects(
    frame: &typst::layout::Frame,
    transform: typst::layout::Transform,
) -> Vec<(
    typst::syntax::Span,
    Box<dyn Mobject>,
    typst::layout::Transform,
)> {
    // #[inline]
    // fn convert_point(typst::layout::Point { x, y }: typst::layout::Point) -> (f32, f32) {
    //     (x.to_pt() as f32, y.to_pt() as f32)
    // }

    // #[inline]
    // fn convert_transform(
    //     typst::layout::Transform {
    //         sx,
    //         ky,
    //         kx,
    //         sy,
    //         tx,
    //         ty,
    //     }: typst::layout::Transform,
    // ) -> ttf_parser::Transform {
    //     ttf_parser::Transform::new(
    //         sx.get() as f32,
    //         ky.get() as f32,
    //         kx.get() as f32,
    //         sy.get() as f32,
    //         tx.to_pt() as f32,
    //         ty.to_pt() as f32,
    //     )
    // }

    frame
        .items()
        .flat_map(|(position, item)| {
            // let (x, y) = convert_point(*position);
            let transform =
                transform.pre_concat(typst::layout::Transform::translate(position.x, position.y));
            match item {
                &typst::layout::FrameItem::Group(ref group) => {
                    if group.clip_path.is_some() {
                        panic!("Clip path not supported");
                    }
                    typst_frame_to_mobjects(&group.frame, transform.pre_concat(group.transform))
                }
                &typst::layout::FrameItem::Text(ref text) => {
                    typst_text_to_mobjects(text, transform)
                }
                &typst::layout::FrameItem::Shape(ref shape, span) => {
                    typst_shape_to_mobjects(shape, span, transform)
                }
                &typst::layout::FrameItem::Image(_, _, _) => {
                    panic!("Unsopported item: image")
                }
                &typst::layout::FrameItem::Link(..) => panic!("Unsopported item: link"),
                &typst::layout::FrameItem::Tag(..) => panic!("Unsopported item: tag"),
            }
        })
        .collect()
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

    dbg!(typst_frame_to_mobjects(&frame, typst::layout::Transform::identity()).len());

    // frame.items().for_each(|a|);

    // println!("{content:?}");
    // println!("{document:?}");
    // let svg = typst_svg::svg_merged(&document, typst::layout::Abs::zero());
    // println!("{svg}");
}

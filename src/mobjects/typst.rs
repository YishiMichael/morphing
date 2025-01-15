use std::borrow::Cow;
use std::ops::Range;

use itertools::Itertools;
use ttf_parser::OutlineBuilder;

use super::super::components::fill::Fill;
use super::super::components::paint::Gradient;
use super::super::components::paint::Paint;
use super::super::components::path::Path;
use super::super::components::path::PathBuilder;
use super::super::components::stroke::DashPattern;
use super::super::components::stroke::Stroke;
use super::super::components::transform::Transform;
use super::super::toplevel::renderer::Renderer;
use super::super::toplevel::world::World;
use super::mobject::Mobject;
use super::mobject::MobjectBuilder;
use super::shape::ShapeMobject;

pub struct Typst(String);

impl Typst {
    pub fn new<S>(text: S) -> Self
    where
        S: ToString,
    {
        Self(text.to_string())
    }
}

impl MobjectBuilder for Typst {
    type Instantiation = TypstMobject;

    fn instantiate(self, world: &World) -> Self::Instantiation {
        TypstMobject::instantiate(self.0, world)
    }
}

#[derive(Clone)]
struct TypstMobjectToken {
    span: Option<Range<usize>>,
    mobject: ShapeMobject,
}

#[derive(Clone)]
pub struct TypstMobject {
    // transform: Transform,
    text: String,
    tokens: Vec<TypstMobjectToken>,
}

impl TypstMobject {
    fn instantiate(text: String, world: &World) -> Self {
        let document = world.typst_world.document(text.clone());
        Self {
            text,
            tokens: TypstMobject::from_typst_document(&document, world),
        }
    }

    fn outline_glyph_to_path(font: &typst::text::Font, id: ttf_parser::GlyphId) -> Option<Path> {
        let mut builder = PathBuilder::new();
        font.ttf().outline_glyph(id, &mut builder)?;
        Some(builder.build())
    }

    fn typst_path_to_path(path: &typst::visualize::Path) -> Path {
        let mut builder = PathBuilder::new();
        path.0.iter().for_each(|path_item| match path_item {
            &typst::visualize::PathItem::MoveTo(start) => {
                builder.move_to(start.x.to_pt() as f32, start.y.to_pt() as f32)
            }
            &typst::visualize::PathItem::LineTo(end) => {
                builder.line_to(end.x.to_pt() as f32, end.y.to_pt() as f32)
            }
            &typst::visualize::PathItem::CubicTo(handle_start, handle_end, end) => builder
                .curve_to(
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

    fn typst_paint_to_paint(paint: &typst::visualize::Paint, path: &Path) -> Paint {
        match paint {
            typst::visualize::Paint::Solid(color) => Paint {
                solid: Some(palette::Srgba::from(color.to_vec4())),
                gradients: Vec::new(),
            },
            typst::visualize::Paint::Gradient(gradient) => {
                let gradient = match gradient {
                    typst::visualize::Gradient::Linear(linear_gradient) => {
                        let sample_points = match path.bounding_box() {
                            None => Vec::new(),
                            Some(
                                [glam::DVec2 { x: x_min, y: y_min }, glam::DVec2 { x: x_max, y: y_max }],
                            ) => Vec::from([
                                glam::DVec2 { x: x_min, y: y_min },
                                glam::DVec2 { x: x_min, y: y_max },
                                glam::DVec2 { x: x_max, y: y_min },
                                glam::DVec2 { x: x_max, y: y_max },
                            ]),
                        };
                        let direction = glam::DVec2 {
                            x: linear_gradient.angle.cos(),
                            y: linear_gradient.angle.sin(),
                        };
                        let (from, to) = sample_points
                            .iter()
                            .copied()
                            .minmax_by_key(|point| point.dot(direction))
                            .into_option()
                            .unwrap_or_default();
                        let to = from + (to - from).project_onto(direction);
                        Gradient {
                            from: from.as_vec2(),
                            to: to.as_vec2(),
                            radius_diff: (to - from).length() as f32,
                            radius_quotient: 1.0,
                            radial_stops: Some(
                                linear_gradient
                                    .stops
                                    .iter()
                                    .map(|(color, ratio)| {
                                        (ratio.get() as f32, palette::Srgba::from(color.to_vec4()))
                                    })
                                    .collect(),
                            ),
                            angular_stops: None,
                        }
                    }
                    typst::visualize::Gradient::Radial(radial_gradient) => {
                        let center = glam::DVec2 {
                            x: radial_gradient.center.x.get(),
                            y: radial_gradient.center.y.get(),
                        };
                        let focal_center = glam::DVec2 {
                            x: radial_gradient.focal_center.x.get(),
                            y: radial_gradient.focal_center.y.get(),
                        };
                        let radius = radial_gradient.radius.get();
                        let focal_radius = radial_gradient.focal_radius.get();
                        let direction = (center - focal_center)
                            .try_normalize()
                            .unwrap_or(glam::DVec2::new(1.0, 0.0));
                        Gradient {
                            from: (focal_center + focal_radius * direction).as_vec2(),
                            to: (center + radius * direction).as_vec2(),
                            radius_diff: (radius - focal_radius) as f32,
                            radius_quotient: (radius / focal_radius) as f32,
                            radial_stops: Some(
                                radial_gradient
                                    .stops
                                    .iter()
                                    .map(|(color, ratio)| {
                                        (ratio.get() as f32, palette::Srgba::from(color.to_vec4()))
                                    })
                                    .collect(),
                            ),
                            angular_stops: None,
                        }
                    }
                    typst::visualize::Gradient::Conic(conic_gradient) => {
                        let center = glam::DVec2 {
                            x: conic_gradient.center.x.get(),
                            y: conic_gradient.center.y.get(),
                        };
                        let direction = glam::DVec2 {
                            x: conic_gradient.angle.cos(),
                            y: conic_gradient.angle.sin(),
                        };
                        Gradient {
                            from: center.as_vec2(),
                            to: (center + direction).as_vec2(),
                            radius_diff: 1.0,
                            radius_quotient: 0.0,
                            radial_stops: None,
                            angular_stops: Some(
                                conic_gradient
                                    .stops
                                    .iter()
                                    .map(|(color, ratio)| {
                                        (ratio.get() as f32, palette::Srgba::from(color.to_vec4()))
                                    })
                                    .collect(),
                            ),
                        }
                    }
                };
                Paint {
                    solid: None,
                    gradients: vec![gradient],
                }
            }
            typst::visualize::Paint::Pattern(..) => unimplemented!(),
        }
        // if let typst::visualize::Paint::Solid(color) = paint {
        //     rgb::Rgba::from(color.to_vec4())
        // } else {
        //     panic!("Unsopported paint");
        // }
    }

    fn from_path(
        path: Path,
        fill_rule: Option<typst::visualize::FillRule>,
        fill: Option<&typst::visualize::Paint>,
        stroke: Option<&typst::visualize::FixedStroke>,
        span: typst::syntax::Span,
        transform: typst::layout::Transform,
        world: &World,
    ) -> TypstMobjectToken {
        let path = path.transform(glam::DAffine2::from_cols_array_2d(&[
            [transform.sx.get(), transform.ky.get()],
            [transform.kx.get(), transform.sy.get()],
            [transform.tx.to_pt(), transform.ty.to_pt()],
        ])); // TODO: pre transform or post transform
        let fill = fill.map(|fill| Fill {
            paint: Self::typst_paint_to_paint(fill, &path),
            options: match fill_rule.unwrap_or_default() {
                typst::visualize::FillRule::NonZero => lyon::tessellation::FillOptions::non_zero(),
                typst::visualize::FillRule::EvenOdd => lyon::tessellation::FillOptions::even_odd(),
            },
        });
        let stroke = stroke.map(|stroke| Stroke {
            dash_pattern: stroke.dash.as_ref().map(|dash_pattern| DashPattern {
                dashes: dash_pattern
                    .array
                    .iter()
                    .map(|length| length.to_pt())
                    .tuples()
                    .map(|(dash_length, space_length)| [dash_length, space_length])
                    .collect(),
                phase: dash_pattern.phase.to_pt(),
            }),
            // path: if let Some(dash_pattern) = &stroke.dash {
            //     path.dash(
            //         &dash_pattern
            //             .array
            //             .iter()
            //             .map(|length| length.to_pt())
            //             .collect_vec(),
            //         dash_pattern.phase.to_pt(),
            //     )
            // } else {
            //     path.clone()
            // },
            paint: Self::typst_paint_to_paint(&stroke.paint, &path),
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
        TypstMobjectToken {
            span: world.typst_world.range(span),
            mobject: ShapeMobject {
                transform: Transform::default(),
                path,
                fill,
                stroke,
            },
        }
    }

    fn from_typst_shape(
        shape: &typst::visualize::Shape,
        span: typst::syntax::Span,
        transform: typst::layout::Transform,
        world: &World,
    ) -> Vec<TypstMobjectToken> {
        let typst_path = match &shape.geometry {
            &typst::visualize::Geometry::Line(point) => {
                let mut path = typst::visualize::Path::new();
                path.line_to(point);
                Cow::Owned(path)
            }
            &typst::visualize::Geometry::Rect(size) => {
                Cow::Owned(typst::visualize::Path::rect(size))
            }
            &typst::visualize::Geometry::Path(ref path) => Cow::Borrowed(path),
        };
        vec![Self::from_path(
            Self::typst_path_to_path(&typst_path),
            Some(shape.fill_rule),
            shape.fill.as_ref(),
            shape.stroke.as_ref(),
            span,
            transform,
            world,
        )]
    }

    fn from_typst_text(
        text: &typst::text::TextItem,
        transform: typst::layout::Transform,
        world: &World,
    ) -> Vec<TypstMobjectToken> {
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
                Self::outline_glyph_to_path(&text.font, ttf_parser::GlyphId(glyph.id))
                    .into_iter()
                    .map(move |path| {
                        Self::from_path(
                            path,
                            None,
                            Some(&text.fill),
                            text.stroke.as_ref(),
                            glyph.span.0,
                            transform,
                            world,
                        )
                    })
            })
            .collect()
    }

    fn from_typst_frame(
        frame: &typst::layout::Frame,
        transform: typst::layout::Transform,
        world: &World,
    ) -> Vec<TypstMobjectToken> {
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
                let transform = transform
                    .pre_concat(typst::layout::Transform::translate(position.x, position.y));
                match item {
                    &typst::layout::FrameItem::Group(ref group) => {
                        if group.clip_path.is_some() {
                            panic!("Clip path not supported");
                        }
                        Self::from_typst_frame(
                            &group.frame,
                            transform.pre_concat(group.transform),
                            world,
                        )
                    }
                    &typst::layout::FrameItem::Text(ref text) => {
                        Self::from_typst_text(text, transform, world)
                    }
                    &typst::layout::FrameItem::Shape(ref shape, span) => {
                        Self::from_typst_shape(shape, span, transform, world)
                    }
                    &typst::layout::FrameItem::Image(_, _, _) => {
                        panic!("Unsopported item: image")
                    }
                    &typst::layout::FrameItem::Link(..) => Vec::new(),
                    &typst::layout::FrameItem::Tag(..) => Vec::new(),
                }
            })
            .collect()
    }

    fn from_typst_document(
        document: &typst::model::Document,
        world: &World,
    ) -> Vec<TypstMobjectToken> {
        document
            .pages
            .iter()
            .enumerate()
            .flat_map(|(i, page)| {
                Self::from_typst_frame(
                    &page.frame,
                    typst::layout::Transform::translate(
                        typst::layout::Abs::zero(),
                        i as f64 * page.frame.height(),
                    ),
                    world,
                )
            })
            .collect()
    }
}

impl Mobject for TypstMobject {
    fn render(&self, renderer: &Renderer) {
        println!("Rendered Typst!");
    }
}

#[cfg(test)]
mod typst_tests {
    use crate::toplevel::config::TypstConfig;

    use super::super::super::toplevel::config::StyleConfig;
    use super::super::super::toplevel::world::World;
    use super::TypstMobject;

    #[test]
    fn test_typst_mobject() -> () {
        let world = World::new(StyleConfig::default(), TypstConfig::default());
        let mob = TypstMobject::instantiate(
            "typst \\ text text #[text] text $ a b c - d^2 $".to_string(),
            &world,
        );
        // let mobs = typst_mobject("fish \\ #[f]ish");
        for token in mob.tokens {
            dbg!(token.span);
        }

        // frame.items().for_each(|a|);

        // println!("{content:?}");
        // println!("{document:?}");
        // let svg = typst_svg::svg_merged(&document, typst::layout::Abs::zero());
        // println!("{svg}");
    }
}

use core::range::Range;
use std::borrow::Cow;
use std::ops::Deref;
use std::path::PathBuf;

use comemo::Track;
use itertools::Itertools;
use morphing_core::config::Config;
use morphing_core::config::ConfigField;
use morphing_core::traits::Mobject;
use morphing_core::traits::MobjectBuilder;
use ttf_parser::OutlineBuilder;

use super::super::components::color::Color;
use super::super::components::fill::Fill;
use super::super::components::paint::Gradient;
use super::super::components::paint::Paint;
use super::super::components::path::Path;
use super::super::components::path::PathBuilder;
use super::super::components::stroke::DashPattern;
use super::super::components::stroke::Stroke;
use super::super::components::transform::Transform;
use super::shape::ShapeMobject;
use super::shape::VecPlanarTrianglesPresentation;

// Modified from typst/lib.rs, typst-cli/src/world.rs

#[derive(serde::Deserialize)]
struct TypstWorldInput {
    inputs: Vec<(String, String)>,
    include_system_fonts: bool,
    include_embedded_fonts: bool,
    font_paths: Vec<PathBuf>,
}

struct TypstWorld {
    library: typst::utils::LazyHash<typst::Library>,
    book: typst::utils::LazyHash<typst::text::FontBook>,
    fonts: Vec<typst_kit::fonts::FontSlot>,
    main_id: typst::syntax::FileId,
}

impl TypstWorld {
    fn new(typst_world_input: TypstWorldInput) -> Self {
        let inputs = typst_world_input
            .inputs
            .iter()
            .map(|(k, v)| {
                (
                    k.as_str().into(),
                    typst::foundations::IntoValue::into_value(v.as_str()),
                )
            })
            .collect();
        let fonts = typst_kit::fonts::FontSearcher::new()
            .include_system_fonts(typst_world_input.include_system_fonts)
            .include_embedded_fonts(typst_world_input.include_embedded_fonts)
            .search_with(typst_world_input.font_paths);
        Self {
            library: typst::utils::LazyHash::new(
                typst::Library::builder().with_inputs(inputs).build(),
            ),
            book: typst::utils::LazyHash::new(fonts.book),
            fonts: fonts.fonts,
            main_id: typst::syntax::FileId::new_fake(typst::syntax::VirtualPath::new("main.typ")),
        }
    }

    fn source(&self, text: String) -> typst::syntax::Source {
        typst::syntax::Source::new(self.main_id, text)
    }

    fn document(&self, source: &typst::syntax::Source) -> typst::model::Document {
        // Modified from reflexo-typst/src/error.rs
        fn eprint_diagnostics<I>(iter: I) -> !
        where
            I: IntoIterator<Item = typst::diag::SourceDiagnostic>,
        {
            for (index, diagnostic) in iter.into_iter().enumerate() {
                eprintln!("{index}. {diagnostic:?}");
                if !diagnostic.hints.is_empty() {
                    eprintln!("  - Hints: {}", diagnostic.hints.join(", "));
                }
            }
            panic!("Typst error. See diagnostics above.");
        }

        let styles = typst::foundations::StyleChain::new(&self.library().styles);
        let traced = typst::engine::Traced::default();
        let introspector = typst::introspection::Introspector::default();
        let world = self.track();
        let traced = traced.track();
        let introspector = introspector.track();

        let mut sink = typst::engine::Sink::new();
        let content = typst::eval::eval(
            world,
            traced,
            sink.track_mut(),
            typst::engine::Route::default().track(),
            source,
        )
        .unwrap_or_else(|errors| {
            sink.delay(errors);
            eprint_diagnostics(sink.delayed());
        })
        .content();

        let mut engine = typst::engine::Engine {
            world,
            introspector,
            traced,
            sink: sink.track_mut(),
            route: typst::engine::Route::default(),
        };
        let document = typst::layout::layout_document(&mut engine, &content, styles)
            .unwrap_or_else(|errors| {
                sink.delay(errors);
                eprint_diagnostics(sink.delayed());
            });

        let delayed = sink.delayed();
        if !delayed.is_empty() {
            eprint_diagnostics(delayed);
        }
        document
    }
}

impl Deref for TypstWorld {
    type Target = dyn typst::World;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl typst::World for TypstWorld {
    fn library(&self) -> &typst::utils::LazyHash<typst::Library> {
        &self.library
    }

    fn book(&self) -> &typst::utils::LazyHash<typst::text::FontBook> {
        &self.book
    }

    fn main(&self) -> typst::syntax::FileId {
        self.main_id
    }

    fn source(&self, _id: typst::syntax::FileId) -> typst::diag::FileResult<typst::syntax::Source> {
        Err(typst::diag::FileError::AccessDenied)
    }

    fn file(
        &self,
        _id: typst::syntax::FileId,
    ) -> typst::diag::FileResult<typst::foundations::Bytes> {
        Err(typst::diag::FileError::AccessDenied)
    }

    fn font(&self, index: usize) -> Option<typst::text::Font> {
        self.fonts[index].get()
    }

    fn today(&self, _: Option<i64>) -> Option<typst::foundations::Datetime> {
        None
    }
}

impl ConfigField for TypstWorld {
    const PATH: &'static str = "typst";

    fn parse(value: &toml::Value) -> Self {
        Self::new(value.clone().try_into().unwrap())
    }
}

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

    fn instantiate(self, config: &Config) -> Self::Instantiation {
        TypstMobject::instantiate(self.0, config)
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct TypstMobjectToken {
    span: Option<Range<usize>>,
    mobject: ShapeMobject,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct TypstMobject {
    text: String,
    tokens: Vec<TypstMobjectToken>,
}

impl TypstMobject {
    fn instantiate(text: String, config: &Config) -> Self {
        Self {
            text: text.clone(),
            tokens: config.operate(|typst_world: &TypstWorld| {
                let source = typst_world.source(text);
                let document = typst_world.document(&source);
                Self::from_typst_document(&document, &source)
            }),
        }
    }

    fn outline_glyph_to_path(font: &typst::text::Font, id: ttf_parser::GlyphId) -> Option<Path> {
        let mut builder = PathBuilder::new();
        font.ttf().outline_glyph(id, &mut builder)?;
        Some(builder.build())
    }

    fn typst_path_to_path(path: &typst::visualize::Path) -> Path {
        let mut builder = PathBuilder::new();
        for path_item in &path.0 {
            match path_item {
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
            }
        }
        builder.build()
    }

    fn typst_paint_to_paint(paint: &typst::visualize::Paint, path: &Path) -> Paint {
        match paint {
            typst::visualize::Paint::Solid(color) => Paint {
                color: color.to_rgb().to_vec4().into(),
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
                            from_position: nalgebra::Vector2::new(from.x as f32, from.y as f32),
                            to_position: nalgebra::Vector2::new(to.x as f32, to.y as f32),
                            radius_slope: 0.0,
                            radius_quotient: 1.0,
                            radial_stops: linear_gradient
                                .stops
                                .iter()
                                .map(|(color, ratio)| {
                                    (ratio.get() as f32, color.to_rgb().to_vec4().into())
                                })
                                .collect(),
                            angular_stops: Vec::new(),
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
                        let from = focal_center + focal_radius * direction;
                        let to = center + radius * direction;
                        Gradient {
                            from_position: nalgebra::Vector2::new(from.x as f32, from.y as f32),
                            to_position: nalgebra::Vector2::new(to.x as f32, to.y as f32),
                            radius_slope: ((to - from).length() / (radius - focal_radius)) as f32,
                            radius_quotient: (radius / focal_radius) as f32,
                            radial_stops: radial_gradient
                                .stops
                                .iter()
                                .map(|(color, ratio)| {
                                    (ratio.get() as f32, color.to_rgb().to_vec4().into())
                                })
                                .collect(),
                            angular_stops: Vec::new(),
                        }
                    }
                    typst::visualize::Gradient::Conic(conic_gradient) => {
                        let from = glam::DVec2 {
                            x: conic_gradient.center.x.get(),
                            y: conic_gradient.center.y.get(),
                        };
                        let to = from
                            + glam::DVec2 {
                                x: conic_gradient.angle.cos(),
                                y: conic_gradient.angle.sin(),
                            };
                        Gradient {
                            from_position: nalgebra::Vector2::new(from.x as f32, from.y as f32),
                            to_position: nalgebra::Vector2::new(to.x as f32, to.y as f32),
                            radius_slope: 0.0,
                            radius_quotient: 0.0,
                            radial_stops: Vec::new(),
                            angular_stops: conic_gradient
                                .stops
                                .iter()
                                .map(|(color, ratio)| {
                                    (ratio.get() as f32, color.to_rgb().to_vec4().into())
                                })
                                .collect(),
                        }
                    }
                };
                Paint {
                    color: Color::max(),
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
        source: &typst::syntax::Source,
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
            span: source.range(span),
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
        source: &typst::syntax::Source,
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
            source,
        )]
    }

    fn from_typst_text(
        text: &typst::text::TextItem,
        transform: typst::layout::Transform,
        source: &typst::syntax::Source,
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
                            source,
                        )
                    })
            })
            .collect()
    }

    fn from_typst_frame(
        frame: &typst::layout::Frame,
        transform: typst::layout::Transform,
        source: &typst::syntax::Source,
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
                            source,
                        )
                    }
                    &typst::layout::FrameItem::Text(ref text) => {
                        Self::from_typst_text(text, transform, source)
                    }
                    &typst::layout::FrameItem::Shape(ref shape, span) => {
                        Self::from_typst_shape(shape, span, transform, source)
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
        source: &typst::syntax::Source,
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
                    source,
                )
            })
            .collect()
    }
}

impl Mobject for TypstMobject {
    type MobjectPresentation = VecPlanarTrianglesPresentation;

    fn presentation(
        &self,
        device: &iced::widget::shader::wgpu::Device,
    ) -> Self::MobjectPresentation {
        self.tokens
            .iter()
            .flat_map(|TypstMobjectToken { mobject, .. }| mobject.presentation(device))
            .collect()
    }
}

#[cfg(test)]
mod typst_tests {
    use super::TypstMobject;
    use super::TypstWorld;
    use super::TypstWorldInput;

    #[test]
    fn test_typst_mobject() {
        let typst_world = TypstWorld::new(TypstWorldInput {
            inputs: Vec::new(),
            include_system_fonts: true,
            include_embedded_fonts: true,
            font_paths: Vec::new(),
        });
        let source =
            typst_world.source("typst \\ text text #[text] text $ a b c - d^2 $".to_string());
        let document = typst_world.document(&source);
        let tokens = TypstMobject::from_typst_document(&document, &source);
        // let mobs = typst_mobject("fish \\ #[f]ish");
        for token in tokens {
            dbg!(token.span);
        }

        // frame.items().for_each(|a|);

        // println!("{content:?}");
        // println!("{document:?}");
        // let svg = typst_svg::svg_merged(&document, typst::layout::Abs::zero());
        // println!("{svg}");
    }
}

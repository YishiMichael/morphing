use itertools::Itertools;
use std::borrow::Cow;
use std::ops::Range;
use ttf_parser::OutlineBuilder;

use super::fill::Fill;
use super::mobject::Mobject;
use super::path::Path;
use super::path::PathBuilder;
use super::stroke::Stroke;

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
    Box<dyn Mobject>,
    typst::layout::Transform,
    typst::syntax::Span,
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
    .map(|mobject| (mobject, transform, span))
    .collect()
}

fn typst_text_to_mobjects(
    text: &typst::text::TextItem,
    transform: typst::layout::Transform,
) -> Vec<(
    Box<dyn Mobject>,
    typst::layout::Transform,
    typst::syntax::Span,
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
                        .map(move |mobject| (mobject, transform, glyph.span.0))
                })
        })
        .collect()
}

fn typst_frame_to_mobjects(
    frame: &typst::layout::Frame,
    transform: typst::layout::Transform,
) -> Vec<(
    Box<dyn Mobject>,
    typst::layout::Transform,
    typst::syntax::Span,
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
                &typst::layout::FrameItem::Link(..) => Vec::new(),
                &typst::layout::FrameItem::Tag(..) => Vec::new(),
            }
        })
        .collect()
}

fn typst_document_to_mobjects(
    document: &typst::model::Document,
) -> Vec<(
    Box<dyn Mobject>,
    typst::layout::Transform,
    typst::syntax::Span,
)> {
    document
        .pages
        .iter()
        .enumerate()
        .flat_map(|(i, page)| {
            typst_frame_to_mobjects(
                &page.frame,
                typst::layout::Transform::translate(
                    typst::layout::Abs::zero(),
                    i as f64 * page.frame.height(),
                ),
            )
        })
        .collect()
}

pub fn typst_mobject(
    text: &str,
) -> Vec<(
    Box<dyn Mobject>,
    typst::layout::Transform,
    Option<Range<usize>>,
)> {
    let world = &world::WORLD;
    let source = typst::syntax::Source::new(world.main_id(), String::from(text));
    let document = world.document(&source);
    typst_document_to_mobjects(&document)
        .into_iter()
        .map(|(mobject, transform, span)| (mobject, transform, source.range(span)))
        .collect()
}

mod world {
    use comemo::Track;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::LazyLock;

    // Modified from typst/lib.rs, typst-cli/src/world.rs

    pub static WORLD: LazyLock<World> = LazyLock::new(|| World::default());

    pub struct World {
        root: PathBuf,
        library: typst::utils::LazyHash<typst::Library>,
        book: typst::utils::LazyHash<typst::text::FontBook>,
        fonts: Vec<typst_kit::fonts::FontSlot>,
        package_storage: typst_kit::package::PackageStorage,
        main_id: typst::syntax::FileId,
        source_slots:
            parking_lot::Mutex<HashMap<typst::syntax::FileId, SlotCell<typst::syntax::Source>>>,
        file_slots:
            parking_lot::Mutex<HashMap<typst::syntax::FileId, SlotCell<typst::foundations::Bytes>>>,
    }

    impl World {
        fn new(
            root: PathBuf,
            inputs: Vec<(String, String)>,
            // font
            font_paths: Vec<PathBuf>,
            include_system_fonts: bool,
            include_embedded_fonts: bool,
            // package
            package_path: Option<PathBuf>,
            package_cache_path: Option<PathBuf>,
        ) -> typst::diag::FileResult<Self> {
            let inputs = inputs
                .iter()
                .map(|(k, v)| {
                    (
                        k.as_str().into(),
                        typst::foundations::IntoValue::into_value(v.as_str()),
                    )
                })
                .collect();
            let fonts = typst_kit::fonts::FontSearcher::new()
                .include_system_fonts(include_system_fonts)
                .include_embedded_fonts(include_embedded_fonts)
                .search_with(font_paths);
            let user_agent = concat!("typst/", env!("CARGO_PKG_VERSION"));
            let downloader = match std::env::var("TYPST_CERT") {
                Ok(cert) => typst_kit::download::Downloader::with_path(user_agent, cert.into()),
                Err(_) => typst_kit::download::Downloader::new(user_agent),
            };
            let package_storage = typst_kit::package::PackageStorage::new(
                package_cache_path,
                package_path,
                downloader,
            );
            Ok(Self {
                root: root
                    .canonicalize()
                    .map_err(|_| typst::diag::FileError::NotFound(root))?,
                library: typst::utils::LazyHash::new(
                    typst::Library::builder().with_inputs(inputs).build(),
                ),
                book: typst::utils::LazyHash::new(fonts.book),
                fonts: fonts.fonts,
                package_storage,
                main_id: typst::syntax::FileId::new_fake(typst::syntax::VirtualPath::new(
                    "main.typ",
                )),
                source_slots: parking_lot::Mutex::new(HashMap::new()),
                file_slots: parking_lot::Mutex::new(HashMap::new()),
            })
        }

        pub fn main_id(&self) -> typst::syntax::FileId {
            self.main_id
        }

        pub fn document(&self, source: &typst::syntax::Source) -> typst::model::Document {
            self.source_slots
                .lock()
                .values_mut()
                .for_each(SlotCell::reset);
            self.file_slots
                .lock()
                .values_mut()
                .for_each(SlotCell::reset);

            let styles = typst::foundations::StyleChain::new(&self.library().styles);
            let traced = typst::engine::Traced::default();
            let introspector = typst::introspection::Introspector::default();
            let world = self.track();
            let traced = traced.track();
            let introspector = introspector.track();

            // TODO: handle unwrap
            let mut sink = typst::engine::Sink::new();
            let content = typst::eval::eval(
                world,
                traced,
                sink.track_mut(),
                typst::engine::Route::default().track(),
                &source,
            )
            .unwrap()
            .content();

            let mut engine = typst::engine::Engine {
                world,
                introspector,
                traced,
                sink: sink.track_mut(),
                route: typst::engine::Route::default(),
            };
            typst::layout::layout_document(&mut engine, &content, styles).unwrap()
        }

        fn read(&self, id: typst::syntax::FileId) -> typst::diag::FileResult<Vec<u8>> {
            let root = match id.package() {
                Some(spec) => &self
                    .package_storage
                    .prepare_package(spec, &mut typst_kit::download::ProgressSink)?,
                None => &self.root,
            };
            let path = id
                .vpath()
                .resolve(root)
                .ok_or(typst::diag::FileError::AccessDenied)?;
            std::fs::metadata(path.clone())
                .map_err(|_| typst::diag::FileError::AccessDenied)?
                .is_file()
                .then(|| std::fs::read(path).map_err(|_| typst::diag::FileError::AccessDenied))
                .unwrap_or(Err(typst::diag::FileError::IsDirectory))
        }
    }

    impl Default for World {
        fn default() -> Self {
            Self::new(
                PathBuf::from("."),
                Vec::new(),
                Vec::new(),
                true,
                true,
                None,
                None,
            )
            .unwrap()
        }
    }

    impl typst::World for World {
        fn library(&self) -> &typst::utils::LazyHash<typst::Library> {
            &self.library
        }

        fn book(&self) -> &typst::utils::LazyHash<typst::text::FontBook> {
            &self.book
        }

        fn main(&self) -> typst::syntax::FileId {
            self.main_id
        }

        fn source(
            &self,
            id: typst::syntax::FileId,
        ) -> typst::diag::FileResult<typst::syntax::Source> {
            fn decode_utf8(buf: &[u8]) -> typst::diag::FileResult<&str> {
                // Remove UTF-8 BOM.
                Ok(std::str::from_utf8(
                    buf.strip_prefix(b"\xef\xbb\xbf").unwrap_or(buf),
                )?)
            }

            let mut map = self.source_slots.lock();
            map.entry(id).or_insert_with(SlotCell::new).get_or_init(
                || self.read(id),
                |data| decode_utf8(&data).map(|text| typst::syntax::Source::new(id, text.into())),
            )
        }

        fn file(
            &self,
            id: typst::syntax::FileId,
        ) -> typst::diag::FileResult<typst::foundations::Bytes> {
            let mut map = self.file_slots.lock();
            map.entry(id)
                .or_insert_with(SlotCell::new)
                .get_or_init(|| self.read(id), |data| Ok(data.into()))
        }

        fn font(&self, index: usize) -> Option<typst::text::Font> {
            self.fonts[index].get()
        }

        fn today(&self, _: Option<i64>) -> Option<typst::foundations::Datetime> {
            None
        }
    }

    impl std::ops::Deref for World {
        type Target = dyn typst::World;

        fn deref(&self) -> &Self::Target {
            self
        }
    }

    /// Lazily processes data for a file.
    struct SlotCell<T> {
        /// The processed data.
        data: typst::diag::FileResult<T>,
        /// A hash of the raw file contents / access error.
        fingerprint: u128,
        /// Whether the slot has been accessed in the current compilation.
        accessed: bool,
    }

    impl<T: Clone> SlotCell<T> {
        /// Creates a new, empty cell.
        fn new() -> Self {
            Self {
                data: Err(typst::diag::FileError::Other(None)),
                fingerprint: 0,
                accessed: false,
            }
        }

        /// Marks the cell as not yet accessed in preparation of the next
        /// compilation.
        fn reset(&mut self) {
            self.accessed = false;
        }

        /// Gets the contents of the cell or initialize them.
        fn get_or_init(
            &mut self,
            load: impl FnOnce() -> typst::diag::FileResult<Vec<u8>>,
            process: impl FnOnce(Vec<u8>) -> typst::diag::FileResult<T>,
        ) -> typst::diag::FileResult<T> {
            if !std::mem::replace(&mut self.accessed, true) {
                // Read and hash the file.
                let result = load();
                let fingerprint = typst::utils::hash128(&result);
                if std::mem::replace(&mut self.fingerprint, fingerprint) != fingerprint {
                    // If the file contents changed, process data and cache.
                    self.data = result.and_then(process);
                }
            }
            self.data.clone()
        }
    }
}

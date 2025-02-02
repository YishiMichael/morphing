use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;

use comemo::Track;

use super::settings::StyleSettings;
use super::settings::TypstSettings;

pub struct World {
    pub style_settings: StyleSettings,
    pub typst_world: TypstWorld,
}

impl World {
    pub(crate) fn new(style_settings: StyleSettings, typst_settings: TypstSettings) -> Self {
        Self {
            style_settings,
            typst_world: TypstWorld::new(typst_settings),
        }
    }
}

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

// Modified from typst/lib.rs, typst-cli/src/world.rs

pub struct TypstWorld {
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

impl TypstWorld {
    fn new(settings: TypstSettings) -> Self {
        let inputs = settings
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
            .include_system_fonts(settings.include_system_fonts)
            .include_embedded_fonts(settings.include_embedded_fonts)
            .search_with(settings.font_paths);
        let user_agent = concat!("typst/", env!("CARGO_PKG_VERSION"));
        let downloader = match std::env::var("TYPST_CERT") {
            Ok(cert) => typst_kit::download::Downloader::with_path(user_agent, cert.into()),
            Err(_) => typst_kit::download::Downloader::new(user_agent),
        };
        let package_storage = typst_kit::package::PackageStorage::new(
            settings.package_cache_path,
            settings.package_path,
            downloader,
        );
        Self {
            root: settings.root,
            library: typst::utils::LazyHash::new(
                typst::Library::builder().with_inputs(inputs).build(),
            ),
            book: typst::utils::LazyHash::new(fonts.book),
            fonts: fonts.fonts,
            package_storage,
            main_id: typst::syntax::FileId::new_fake(typst::syntax::VirtualPath::new("main.typ")),
            source_slots: parking_lot::Mutex::new(HashMap::new()),
            file_slots: parking_lot::Mutex::new(HashMap::new()),
        }
    }

    pub fn source(&self, text: String) -> typst::syntax::Source {
        typst::syntax::Source::new(self.main_id, text)
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

    fn source(&self, id: typst::syntax::FileId) -> typst::diag::FileResult<typst::syntax::Source> {
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

impl Deref for TypstWorld {
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

impl<T> SlotCell<T>
where
    T: Clone,
{
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

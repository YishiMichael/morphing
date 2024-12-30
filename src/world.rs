use comemo::Track;
use std::collections::HashMap;
use std::path::PathBuf;

// Modified from typst-cli/src/world.rs

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
    //text: String,
}

impl TypstWorld {
    pub fn new(
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
        let package_storage =
            typst_kit::package::PackageStorage::new(package_cache_path, package_path, downloader);
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
            main_id: typst::syntax::FileId::new_fake(typst::syntax::VirtualPath::new("<main>")),
            source_slots: parking_lot::Mutex::new(HashMap::new()),
            file_slots: parking_lot::Mutex::new(HashMap::new()),
            // text: String::new(),
        })
    }

    pub fn document(&self, text: String) -> typst::model::Document {
        self.source_slots
            .lock()
            .values_mut()
            .for_each(SlotCell::reset);
        self.file_slots
            .lock()
            .values_mut()
            .for_each(SlotCell::reset);

        let source = typst::syntax::Source::new(self.main_id, text);
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
        let document = typst::layout::layout_document(&mut engine, &content, styles).unwrap();
        println!("{document:?}");
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

impl Default for TypstWorld {
    fn default() -> Self {
        Self::new(
            PathBuf::from("."),
            Vec::new(),
            Vec::new(),
            true,
            false,
            None,
            None,
        )
        .unwrap()
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

impl std::ops::Deref for TypstWorld {
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

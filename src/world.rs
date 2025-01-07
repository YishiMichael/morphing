use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;

use comemo::Track;

use super::scene::BakedWorldline;
use super::scene::Worldline;

pub(crate) static WORLD: LazyLock<World> = LazyLock::new(|| World::new());

pub(crate) struct World {
    cache_path: LazyLock<PathBuf>,
    typst_world: LazyLock<TypstWorld>,
}

impl World {
    fn new() -> Self {
        Self {
            cache_path: LazyLock::new(|| Self::init_cache_path().unwrap()),
            typst_world: LazyLock::new(|| TypstWorld::default()),
        }
    }

    fn init_cache_path() -> std::io::Result<PathBuf> {
        let temp_dir_path = std::env::temp_dir().join(format!(
            "{}-{}-CACHE",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ));
        let cache_path = temp_dir_path.join("cache.ron");
        if !std::fs::exists(&cache_path)? {
            if !std::fs::exists(&temp_dir_path)? {
                std::fs::create_dir(&temp_dir_path)?;
            }
        };
        Ok(cache_path)
    }

    pub(crate) fn read_cache(&self) -> HashMap<Worldline, BakedWorldline> {
        std::fs::read(&*self.cache_path)
            .map(|buf| ron::de::from_reader(&*buf).unwrap_or_default())
            .unwrap_or_default()
    }

    pub(crate) fn write_cache(&self, cache: HashMap<Worldline, BakedWorldline>) {
        let mut buf = Vec::new();
        ron::ser::to_writer(&mut buf, &cache).unwrap();
        std::fs::write(&*self.cache_path, buf).unwrap();
    }

    pub(crate) fn typst_world(&self) -> &TypstWorld {
        &self.typst_world
    }
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
            main_id: typst::syntax::FileId::new_fake(typst::syntax::VirtualPath::new("main.typ")),
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

impl Default for TypstWorld {
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

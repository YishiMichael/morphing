pub use config;
pub use inventory;

use super::root;
use convert_case::Casing;
use darling::FromMeta;

pub struct Symbol<T> {
    pub(crate) name: String,
    pub(crate) config: Vec<config::File<config::FileSourceString, config::FileFormat>>,
    pub(crate) content: T,
}

inventory::collect!(SceneSymbol);

fn f(i: u32) -> () {
    i
}

pub type SceneSymbol = Symbol<
    Box<
        dyn Fn(config::ConfigBuilder<config::builder::DefaultState>) -> Vec<Box<dyn Lifecycle>>
            + Sync,
    >,
>;

pub fn scene_symbol<C: 'static + serde::de::DeserializeOwned, const N: usize>(
    name: &str,
    config: [config::File<config::FileSourceString, config::FileFormat>; N],
    scene: fn(&mut Supervisor<C>),
) -> SceneSymbol {
    Symbol {
        name: name.into(),
        config: config.into(),
        content: Box::new(move |config_builder| {
            let configuration = config_builder.build().unwrap().try_deserialize().unwrap();
            let mut supervisor = Supervisor {
                time: 0.0,
                lifecycles: Vec::new(),
                config: configuration,
            };
            scene(&mut supervisor);
            supervisor.lifecycles
        }),
    }
}

pub type ChapterSymbol = Symbol<std::collections::HashMap<String, &'static SceneSymbol>>;

pub fn chapter_symbol<const N: usize>(
    name: &str,
    config: [config::File<config::FileSourceString, config::FileFormat>; N],
    scenes: inventory::iter<SceneSymbol>,
) -> ChapterSymbol {
    Symbol {
        name: name.into(),
        config: config.into(),
        content: scenes
            .into_iter()
            .map(|symbol| (symbol.name.clone(), symbol))
            .collect(),
    }
}

pub(crate) fn call_entrypoint(chapter_path: &str) -> ChapterSymbol {
    let func: libloading::Symbol<extern "Rust" fn() -> ChapterSymbol> = unsafe {
        let lib = libloading::Library::new(chapter_path).unwrap();
        lib.get(b"__morphing_entrypoint__\0").unwrap(); // expecting #[chapter] invocation
    };
    func()
}

// pub mod config_formats {
//     macro_rules! config_format {
//         ($name:ident = $format:expr) => {
//             pub fn $name(s: &str) -> config::File<config::FileSourceString, config::FileFormat> {
//                 config::File::from_str(s, $format)
//             }
//         };
//     }

//     config_format!(toml = config::FileFormat::Toml);
//     config_format!(json = config::FileFormat::Json);
//     config_format!(yaml = config::FileFormat::Yaml);
//     config_format!(ini = config::FileFormat::Ini);
//     config_format!(ron = config::FileFormat::Ron);
//     config_format!(json5 = config::FileFormat::Json5);
// }

#[derive(Default)]
struct NameValueList(Vec<syn::MetaNameValue>);

impl std::ops::Deref for NameValueList {
    type Target = Vec<syn::MetaNameValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<syn::MetaNameValue>> for NameValueList {
    fn from(v: Vec<syn::MetaNameValue>) -> Self {
        NameValueList(v)
    }
}

impl FromMeta for NameValueList {
    fn from_list(v: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        v.into_iter()
            .map(|nm| {
                if let darling::ast::NestedMeta::Meta(syn::Meta::NameValue(ref name_value)) = *nm {
                    Ok(name_value.clone())
                } else {
                    Err(darling::Error::unexpected_type("non-name-value").with_span(nm))
                }
            })
            .collect::<darling::Result<_>>()
            .map(NameValueList)
    }
}

#[derive(FromMeta)]
pub(crate) struct ConfigArgs {
    #[darling(default)]
    config: NameValueList,
}

fn expand_configs(config: NameValueList) -> proc_macro2::TokenStream {
    let items: Vec::<_> = config
        .iter()
        .map(|name_value| {
            let format = name_value
                .path
                .require_ident()
                .expect("Format specifier must be an identifier");
            let format = syn::Ident::new(
                &format.to_string().to_case(convert_case::Case::Pascal),
                format.span(),
            );
            let literal = &name_value.value;

            quote::quote! {
            	#root::__macros::link::config::File::from_str(#literal, #root::__macros::link::config::FileFormat::#format)
            }
        })
        .collect();

    quote::quote! {
        [#(#items),*]
    }
}

pub(crate) fn scene(args: ConfigArgs, item_fn: syn::ItemFn) -> proc_macro2::TokenStream {
    let ident = &item_fn.sig.ident;
    let name = ident.to_string();
    let config_expanded = expand_configs(args.config);

    quote::quote! {
        #item_fn

        #root::__macros::link::inventory::submit! {
            #root::__macros::link::scene_symbol(
                concat!(module_path!(), "::", #name),
                #config_expanded,
                #ident,
            )
        }
    }
}

pub(crate) fn chapter(
    args: ConfigArgs,
    item_extern_crate: syn::ItemExternCrate,
) -> proc_macro2::TokenStream {
    assert_eq!(item_extern_crate.ident.to_string(), "self");
    let root = root;
    let name = item_extern_crate
        .rename
        .as_ref()
        .map(|(_, rename)| rename.to_string())
        .unwrap_or_else(|| std::env::var("CARGO_PKG_NAME").unwrap());
    let config_expanded = expand_configs(args.config);

    quote::quote! {
        #[no_mangle]
        pub extern "Rust" fn __morphing_entrypoint__() -> #root::__macros::link::ChapterSymbol {
            #root::__macros::link::chapter_symbol(
                #name,
                #config_expanded,
                #root::__macros::link::inventory::iter,
            )
        }
    }
}

use convert_case::Casing;
use darling::FromMeta;

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
    let (formats, literals): (Vec<_>, Vec<_>) = config
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
            (format, literal)
        })
        .unzip();

    quote::quote! {
        [#(::morphing_core::link::config::File::from_str(#literals, ::morphing_core::link::config::FileFormat::#formats)),*]
    }
}

pub(crate) fn scene(args: ConfigArgs, item_fn: syn::ItemFn) -> proc_macro2::TokenStream {
    let ident = &item_fn.sig.ident;
    let name = ident.to_string();
    let config_expanded = expand_configs(args.config);

    quote::quote! {
        #item_fn

        ::morphing_core::link::inventory::submit! {
            ::morphing_core::link::scene_symbol(
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
    let name = item_extern_crate
        .rename
        .as_ref()
        .map(|(_, rename)| rename.to_string())
        .unwrap_or_else(|| std::env::var("CARGO_PKG_NAME").unwrap());
    let config_expanded = expand_configs(args.config);

    quote::quote! {
        #[no_mangle]
        pub extern "Rust" fn __morphing_entrypoint__() -> ::morphing_core::link::ChapterSymbol {
            ::morphing_core::link::chapter_symbol(
                #name,
                #config_expanded,
                ::morphing_core::link::inventory::iter,
            )
        }
    }
}

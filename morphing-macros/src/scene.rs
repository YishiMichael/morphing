use super::SceneArgs;

pub(crate) fn scene(args: SceneArgs, item_fn: syn::ItemFn) -> proc_macro2::TokenStream {
    let scene_name = item_fn.sig.ident.clone();
    let config_content = if let Some(config) = args.config {
        quote::quote! { Some(::std::include_str!(#config)) }
    } else {
        quote::quote! { None }
    };
    quote::quote! {
        #item_fn

        ::morphing_core::scene::inventory::submit! {
            ::morphing_core::scene::SceneModule {
                name: concat!(module_path!(), "::", stringify!(#scene_name)),
                config_content: #config_content,
                item_fn: #scene_name,
            }
        }
    }
}

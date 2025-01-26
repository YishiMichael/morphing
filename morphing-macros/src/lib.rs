use darling::FromMeta;
use proc_macro::TokenStream;

#[derive(FromMeta)]
struct SceneArgs {
    override_settings: Option<syn::Path>,
}

#[proc_macro_attribute]
pub fn scene(input: TokenStream, tokens: TokenStream) -> TokenStream {
    let args = match darling::ast::NestedMeta::parse_meta_list(input.into()) {
        Ok(args) => match SceneArgs::from_list(&args) {
            Ok(args) => args,
            Err(error) => return TokenStream::from(error.write_errors()),
        },
        Err(error) => return TokenStream::from(darling::Error::from(error).write_errors()),
    };
    let mut scene_fn = syn::parse_macro_input!(tokens as syn::ItemFn);
    let vis = std::mem::replace(&mut scene_fn.vis, syn::Visibility::Inherited);

    let scene_name = scene_fn.sig.ident.clone();
    let var_scene_settings = quote::format_ident!("__scene_settings");

    let override_settings_stmt = if let Some(override_settings_path) = args.override_settings {
        quote::quote! {
            let #var_scene_settings = #override_settings_path(#var_scene_settings);
        }
    } else {
        quote::quote! {}
    };
    let block = quote::quote! {
        #vis fn #scene_name(
            #var_scene_settings: ::morphing::toplevel::settings::SceneSettings,
        ) -> ::morphing::toplevel::scene::SceneTimelineCollection {
            #scene_fn
            #override_settings_stmt
            ::morphing::toplevel::scene::SceneTimelineCollection::new(stringify!(#scene_name), #var_scene_settings, #scene_name)
        }
    };
    block.into()
}

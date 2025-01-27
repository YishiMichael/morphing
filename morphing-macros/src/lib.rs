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
    let scene_fn = syn::parse_macro_input!(tokens as syn::ItemFn);

    let scene_name = scene_fn.sig.ident.clone();
    let override_settings = if let Some(override_settings) = args.override_settings {
        quote::quote! { Some(#override_settings) }
    } else {
        quote::quote! { None }
    };
    // let var_scene_settings = quote::format_ident!("scene_settings");

    // let override_settings_stmt = if let Some(override_settings_path) = args.override_settings {
    //     quote::quote! {
    //         let #var_scene_settings = #override_settings_path(#var_scene_settings);
    //     }
    // } else {
    //     quote::quote! {}
    // };
    quote::quote! {
        #scene_fn

        ::morphing::toplevel::scene::inventory::submit! {
            ::morphing::toplevel::scene::SceneModule {
                name: stringify!(#scene_name),
                override_settings: #override_settings,
                scene_fn: #scene_name,
            }
        }
    }
    .into()
}

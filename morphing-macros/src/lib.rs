// TODO: rate, world, layer, scene, wgpu_struct, wgpu_shader_types

mod rate;
mod scene;
mod structure;
mod wgpu;

use proc_macro::TokenStream;

use darling::FromMeta;

fn forward_macro_impl<I, T>(
    f: fn(I, T) -> proc_macro2::TokenStream,
    input: TokenStream,
    tokens: TokenStream,
) -> TokenStream
where
    I: darling::FromMeta,
    T: syn::parse::Parse,
{
    darling::ast::NestedMeta::parse_meta_list(input.into())
        .map_err(darling::Error::from)
        .and_then(|args| FromMeta::from_list(&args))
        .map_err(darling::Error::write_errors)
        .map(|input| {
            syn::parse(tokens)
                .map_or_else(syn::Error::into_compile_error, |tokens| f(input, tokens))
        })
        .unwrap_or_else(|token_stream| token_stream)
        .into()
}

macro_rules! forward {
    ($name:ident => $path:path) => {
        #[proc_macro_attribute]
        pub fn $name(input: TokenStream, tokens: TokenStream) -> TokenStream {
            forward_macro_impl($path, input, tokens)
        }
    };
}

#[derive(FromMeta)]
pub(crate) struct SceneArgs {
    config: Option<syn::LitStr>,
}

forward!(scene => scene::scene);

#[derive(FromMeta)]
pub(crate) struct RateArgs {
    normalized: darling::util::Flag,
    denormalized: darling::util::Flag,
    increasing: darling::util::Flag,
    assert: Option<syn::LitStr>,
}

forward!(rate => rate::rate);

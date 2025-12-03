// TODO: rate, world, layer, scene, wgpu_struct, wgpu_shader_types, pipeline?,

mod get_field;
mod link;
mod rate;
mod structure;
mod wgpu;

use proc_macro::TokenStream;

fn delegate_macro<T>(f: fn(T) -> proc_macro2::TokenStream, tokens: TokenStream) -> TokenStream
where
    T: syn::parse::Parse,
{
    syn::parse::Parse::parse(tokens)
        .map(f)
        .unwrap_or_else(darling::Error::write_errors)
        .into()
}

fn delegate_macro_attribute<I, T>(
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
        .and_then(|items| darling::FromMeta::from_list(&items))
        .map_err(darling::Error::write_errors)
        .map(|args| {
            syn::parse(tokens).map_or_else(syn::Error::into_compile_error, |tokens| f(args, tokens))
        })
        .unwrap_or_else(|token_stream| token_stream)
        .into()
}

fn delegate_macro_derive<T>(
    f: fn(T) -> proc_macro2::TokenStream,
    tokens: TokenStream,
) -> TokenStream
where
    T: darling::FromDeriveInput,
{
    T::from_derive_input(&syn::parse_macro_input!(tokens as T))
        .map(f)
        .unwrap_or_else(darling::Error::write_errors)
        .into()
}

#[proc_macro_attribute]
pub fn scene(input: TokenStream, tokens: TokenStream) -> TokenStream {
    delegate_macro_attribute(link::scene, input, tokens)
}

#[proc_macro_attribute]
pub fn chapter(input: TokenStream, tokens: TokenStream) -> TokenStream {
    delegate_macro_attribute(link::chapter, input, tokens)
}

#[proc_macro_derive(GetField)]
pub fn get_field_derive(tokens: TokenStream) -> TokenStream {
    delegate_macro_derive(get_field::get_field, tokens)
}

// #[proc_macro]

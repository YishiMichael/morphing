mod convert;
mod key;
mod parse;

fn delegate_macro<T>(
    f: fn(T) -> proc_macro2::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream
where
    T: syn::parse::Parse,
{
    match syn::parse(input) {
        Ok(input) => f(input).into(),
        Err(e) => proc_macro_error::abort!(e.span(), e),
    }
}

#[proc_macro_error::proc_macro_error]
#[proc_macro]
pub fn config(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    delegate_macro(parse::config_items_ts, input)
}

#[proc_macro_error::proc_macro_error]
#[proc_macro]
pub fn key(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    delegate_macro(key::Key::value_ts, input)
}

#[proc_macro_error::proc_macro_error]
#[proc_macro]
#[allow(non_snake_case)]
pub fn Key(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    delegate_macro(key::Key::type_ts, input)
}

#[proc_macro_derive(ConfigData)]
pub fn config_data(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    delegate_macro(convert::config_data, input)
}

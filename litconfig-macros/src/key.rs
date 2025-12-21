#[derive(Clone)]
pub(crate) struct Key {
    root: syn::LitStr,
    postfix: Vec<KeySegment>,
}

#[derive(Clone)]
enum KeySegment {
    Name(syn::LitStr),
    Index(syn::LitInt),
}

impl syn::parse::Parse for Key {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let root = input.parse()?;
        let mut postfix = vec![];
        while !input.is_empty() {
            postfix.push(input.parse()?);
        }
        Ok(Self { root, postfix })
    }
}

impl syn::parse::Parse for KeySegment {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.parse::<syn::Token![.]>().is_ok() {
            let name = input.parse()?;
            return Ok(KeySegment::Name(name));
        }
        if input.peek(syn::token::Bracket) {
            let index;
            syn::bracketed!(index in input);
            let index = input.parse()?;
            return Ok(KeySegment::Index(index));
        }
        Err(input.error("expected .<LitStr> | [<LitInt>]"))
    }
}

impl Key {
    pub(crate) fn type_ts(self) -> proc_macro2::TokenStream {
        fn recurse(root: &KeySegment, postfix: &[KeySegment]) -> proc_macro2::TokenStream {
            let root_type_ts = match root {
                KeySegment::Name(name) => {
                    let hash = const_fnv1a_hash::fnv1a_hash_str_64(name.value().as_str());
                    quote::quote! {
                        ::litconfig::__private::key_types::KeySegmentName<#hash>
                    }
                }
                KeySegment::Index(index) => {
                    quote::quote! {
                        ::litconfig::__private::key_types::KeySegmentIndex<#index>
                    }
                }
            };
            if let Some((root, postfix)) = postfix.split_first() {
                let postfix_type_ts = recurse(root, postfix);
                quote::quote! {
                    ::litconfig::__private::key_types::Cons<#root_type_ts, #postfix_type_ts>
                }
            } else {
                root_type_ts
            }
        }
        recurse(&KeySegment::Name(self.root), self.postfix.as_slice())
    }

    pub(crate) fn value_ts(self) -> proc_macro2::TokenStream {
        let type_ts = self.type_ts();
        quote::quote! {
            <#type_ts>::default()
        }
    }
}

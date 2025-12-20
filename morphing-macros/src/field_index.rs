use super::root;
use darling::{FromDeriveInput, FromField};

pub struct Key<const KEY: u64>;

#[derive(FromDeriveInput)]
#[darling(supports(struct_named))]
pub(crate) struct StructInfo {
    // vis: syn::Visibility,
    ident: syn::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<(), FieldInfo>, // TODO: ! type
}

#[derive(Debug, FromField)]
struct FieldInfo {
    // vis: syn::Visibility,
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

pub(crate) fn field_index_derive(struct_info: StructInfo) -> proc_macro2::TokenStream {
    let name = struct_info.ident;
    let generics_params = struct_info.generics.params;
    let generics_where_clause = struct_info.generics.where_clause;
    let fields_impl = struct_info
        .data
        .take_struct()
        .unwrap()
        .fields
        .iter()
        .map(|field_info| {
            let field_ident = field_info.ident.as_ref().unwrap();
            let field_ty = &field_info.ty;
            let key = const_fnv1a_hash::fnv1a_hash_str_64(&field_ident.to_string());

            quote::quote! {
                impl<#generics_params> ::std::ops::Index<#root::__macros::field_index::Key<#key>> for #name #generics_where_clause {
                    type Output = #field_ty;

                    fn index(&self) -> &Self::Output {
                        &self.#field_ident
                    }
                }

                impl<#generics_params> ::std::ops::IndexMut<#root::__macros::field_index::Key<#key>> for #name #generics_where_clause {
                    type Output = #field_ty;

                    fn index_mut(&mut self) -> &mut Self::Output {
                        &mut self.#field_ident
                    }
                }
            }
        })
        .collect();

    quote::quote! {
        impl<const __KEY: u64, __FP, #generics_params> Get<(#root::__macros::field_index::Key<__KEY>, __FP)> for #name
        where
            #name: ::std::ops::Index<#root::__macros::field_index::Key<KEY>>,
            T::Output: ::std::ops::Index<__FP>,
        {
            type Output = <T::Output as Get<K>>::Output;

            fn get(&self, key: (__Key<KEY>, K)) -> &Self::Output {
                self.get(key.0).get(key.1)
            }
        }
    }
}

pub(crate) fn field_path(
    punctuated: syn::punctuated::Punctuated<syn::Ident, syn::Token![.]>,
) -> proc_macro2::TokenStream {
    let mut keys: Vec<_> = punctuated
        .iter()
        .map(|ident| {
            let key = const_fnv1a_hash::fnv1a_hash_str_64(&ident.to_string());
            quote::quote! {
                #root::__macros::field_index::Key<#key>
            }
        })
        .collect();
    if keys.is_empty() {
        proc_macro2::TokenStream::new()
    } else {
        while keys.len() > 1 {
            let k = keys.pop().unwrap();
            let key = keys.pop().unwrap();
            keys.push(quote::quote! {
                (#key, #k)
            });
        }
        keys.pop().unwrap()
    }
}

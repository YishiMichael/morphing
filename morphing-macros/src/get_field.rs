use darling::{FromDeriveInput, FromField};

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

// #[doc(hidden)]
// pub use const_fnv1a_hash::fnv1a_hash_str_64;

// macro_rules! key_hash {
//     () => {};
// }

pub(crate) fn get_field(struct_info: StructInfo) -> proc_macro2::TokenStream {
    let name = struct_info.ident;
    let generics = struct_info.generics;
    struct_info
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
                impl #generics ::morphing_core::get::Get<::morphing_core::get::Key<#key>> for #name {
                    type Output = #field_ty;

                    fn get(&self) -> Self::Output {
                        &self.#field_ident
                    }
                }
            }
        })
        .collect()
}

use darling::FromMeta;

#[derive(FromMeta)]
pub(crate) struct RateArgs {
    normalized: darling::util::Flag,
    denormalized: darling::util::Flag,
    increasing: darling::util::Flag,
    assert: Option<syn::LitStr>,
}

// External crate dependencies:
// - serde
// - morphing_core
pub(crate) fn rate(args: RateArgs, item_fn: syn::ItemFn) -> proc_macro2::TokenStream {
    let fn_name = &item_fn.sig.ident;
    let vis = &item_fn.vis;
    let struct_name = quote::format_ident!("__{}_Rate", fn_name);
    let trait_name = quote::format_ident!("__{}_Trait", fn_name);

    let (struct_field_names, struct_field_types): (Vec<_>, Vec<_>) = item_fn
        .sig
        .inputs
        .iter()
        .skip(1)
        .filter_map(|fn_arg| {
            if let syn::FnArg::Typed(syn::PatType { pat, ty, .. }) = fn_arg {
                if let syn::Pat::Ident(ident) = &**pat {
                    Some((&ident.ident, &**ty))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unzip();

    let struct_definition = quote::quote! {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Debug, ::serde::Deserialize, ::serde::Serialize)]
        struct #struct_name {
            #(#struct_field_names: #struct_field_types,)*
        }
    };

    let impl_normalized_rate = args.normalized.is_present().then(|| quote::quote! {
        impl ::morphing_core::traits::Rate<::morphing_core::timer::NormalizedTimeMetric> for #struct_name {
            type OutputTimeMetric = ::morphing_core::timer::NormalizedTimeMetric;

            fn eval(&self, time_metric: ::morphing_core::timer::NormalizedTimeMetric) -> Self::OutputTimeMetric {
                #fn_name(*time_metric, #(self.#struct_field_names.clone()),*)
            }
        }
    }).unwrap_or_default();
    let impl_normalized_increasing_rate = (args.normalized.is_present() && args.increasing.is_present()).then(|| quote::quote! {
        impl ::morphing_core::traits::IncreasingRate<::morphing_core::timer::NormalizedTimeMetric> for #struct_name {}
    }).unwrap_or_default();
    let impl_denormalized_rate = args.denormalized.is_present().then(|| quote::quote! {
        impl ::morphing_core::traits::Rate<::morphing_core::timer::DenormalizedTimeMetric> for #struct_name {
            type OutputTimeMetric = ::morphing_core::timer::DenormalizedTimeMetric;

            fn eval(&self, time_metric: ::morphing_core::timer::DenormalizedTimeMetric) -> Self::OutputTimeMetric {
                #fn_name(*time_metric, #(self.#struct_field_names.clone()),*)
            }
        }
    }).unwrap_or_default();
    let impl_denormalized_increasing_rate = (args.denormalized.is_present() && args.increasing.is_present()).then(|| quote::quote! {
        impl ::morphing_core::traits::IncreasingRate<::morphing_core::timer::DenormalizedTimeMetric> for #struct_name {}
    }).unwrap_or_default();

    let assert_statement = args
        .assert
        .map(|assert_expression_str| {
            let expr = assert_expression_str.parse::<syn::Expr>().unwrap();
            quote::quote! {
                assert!(#expr);
            }
        })
        .unwrap_or_default();
    let trait_definition = quote::quote! {
        #[allow(non_camel_case_types)]
        #vis trait #trait_name<TM>: ::morphing_core::timeline::ApplyRate<TM>
        where
            TM: ::morphing_core::timer::TimeMetric,
            #struct_name: ::morphing_core::traits::Rate<TM>,
        {
            fn #fn_name(self, #(#struct_field_names: #struct_field_types),*) -> Self::Output<#struct_name> {
                #assert_statement
                self.apply_rate(#struct_name { #(#struct_field_names),* })
            }
        }
    };
    let blanket_impl = quote::quote! {
        impl<A, TM> #trait_name<TM> for A
        where
            A: ::morphing_core::timeline::ApplyRate<TM>,
            TM: ::morphing_core::timer::TimeMetric,
            #struct_name: ::morphing_core::traits::Rate<TM>,
        {}
    };

    quote::quote! {
        #item_fn
        #struct_definition
        #impl_normalized_rate
        #impl_normalized_increasing_rate
        #impl_denormalized_rate
        #impl_denormalized_increasing_rate
        #trait_definition
        #blanket_impl
    }
}

use super::key::KeySegment;
use config::{File, FileFormat, Map, Source, Value, ValueKind};

// enum VariantRepresentation {
//     Nil,
//     Bool(bool),
//     String(String),
//     Integer(i64),
//     Float(f64),
//     NamedTree(Vec<(syn::LitStr, Self)>),
//     UnnamedTree(Vec<(syn::LitInt, Self)>),
// }

// impl VariantRepresentation {
//     fn represent_variant(value: Value) -> Self {
//         match value.kind {
//             ValueKind::Nil => Self::Nil,
//             ValueKind::Boolean(value) => Self::Bool(value),
//             ValueKind::I64(value) => Self::Integer(value),
//             ValueKind::I128(value) => Self::Integer(value as i64),
//             ValueKind::U64(value) => Self::Integer(value as i64),
//             ValueKind::U128(value) => Self::Integer(value as i64),
//             ValueKind::Float(value) => Self::Float(value),
//             ValueKind::String(value) => Self::String(value),
//             ValueKind::Table(value) => Self::NamedTree(Self::represent_table(value)),
//             ValueKind::Array(value) => Self::UnnamedTree(Self::represent_array(value)),
//         }
//     }

//     fn represent_table(value: Map<String, Value>) -> Vec<(syn::LitStr, Self)> {
//         value
//             .into_iter()
//             .map(|(name, value)| {
//                 (
//                     syn::LitStr::new(name.as_str(), proc_macro2::Span::call_site()),
//                     Self::represent_variant(value),
//                 )
//             })
//             .collect()
//     }

//     fn represent_array(value: Vec<Value>) -> Vec<(syn::LitInt, Self)> {
//         value
//             .into_iter()
//             .enumerate()
//             .map(|(index, value)| {
//                 (
//                     syn::LitInt::new(format!("{index}").as_str(), proc_macro2::Span::call_site()),
//                     Self::represent_variant(value),
//                 )
//             })
//             .collect()
//     }

//     fn type_ts_variant(&self) -> proc_macro2::TokenStream {
//         match self {
//             Self::Nil => {
//                 quote::quote! {
//                     ::litconfig::__private::representation_types::Nil
//                 }
//             }
//             Self::Bool(_) => {
//                 quote::quote! {
//                     ::litconfig::__private::representation_types::Bool
//                 }
//             }
//             Self::String(_) => {
//                 quote::quote! {
//                     ::litconfig::__private::representation_types::StaticStr
//                 }
//             }
//             Self::Integer(_) => {
//                 quote::quote! {
//                     ::litconfig::__private::representation_types::Number
//                 }
//             }
//             Self::Float(_) => {
//                 quote::quote! {
//                     ::litconfig::__private::representation_types::Number
//                 }
//             }
//             Self::NamedTree(table) => Self::type_ts_container(table),
//             Self::UnnamedTree(array) => Self::type_ts_container(array),
//         }
//     }

//     fn type_ts_container<T>(container: &[(T, Self)]) -> proc_macro2::TokenStream {
//         let fields_type_ts = container.iter().map(|(_, field)| field.type_ts_variant());
//         quote::quote! {
//             (#(#fields_type_ts,)*)
//         }
//     }

//     fn value_ts_variant(&self) -> proc_macro2::TokenStream {
//         match self {
//             Self::Nil => {
//                 quote::quote! {
//                     ::litconfig::__private::representation_types::Nil
//                 }
//             }
//             Self::Bool(value) => {
//                 quote::quote! {
//                     ::litconfig::__private::representation_types::Bool(#value)
//                 }
//             }
//             Self::String(value) => {
//                 quote::quote! {
//                     ::litconfig::__private::representation_types::StaticStr(#value)
//                 }
//             }
//             Self::Integer(value) => {
//                 quote::quote! {
//                     ::litconfig::__private::representation_types::Number::Integer(#value)
//                 }
//             }
//             Self::Float(value) => {
//                 quote::quote! {
//                     ::litconfig::__private::representation_types::Number::Float(#value)
//                 }
//             }
//             Self::NamedTree(table) => Self::value_ts_container(table),
//             Self::UnnamedTree(array) => Self::value_ts_container(array),
//         }
//     }

//     fn value_ts_container<T>(container: &[(T, Self)]) -> proc_macro2::TokenStream {
//         let fields_value_ts = container.iter().map(|(_, field)| field.value_ts_variant());
//         quote::quote! {
//             (#(#fields_value_ts,)*)
//         }
//     }
// }

pub(crate) struct ConfigItems {
    items: Vec<ConfigItem>,
}

struct ConfigItem {
    attrs: Vec<syn::Attribute>,
    vis: syn::Visibility,
    static_token: syn::Token![static],
    name: syn::Ident,
    eq_token: syn::Token![=],
    sources: syn::punctuated::Punctuated<ConfigSource, syn::Token![+]>,
    semi_token: syn::Token![;],
}

enum ConfigSource {
    Include {
        file_name: syn::LitStr,
    },
    Literal {
        format: syn::Ident,
        content: syn::LitStr,
    },
}

impl syn::parse::Parse for ConfigItems {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut items = vec![];
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(Self { items })
    }
}

impl syn::parse::Parse for ConfigItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            attrs: input.call(syn::Attribute::parse_outer)?,
            vis: input.parse::<syn::Visibility>()?,
            static_token: input.parse::<syn::Token![static]>()?,
            name: input.parse::<syn::Ident>()?,
            eq_token: input.parse::<syn::Token![=]>()?,
            sources: syn::punctuated::Punctuated::parse_separated_nonempty(input)?,
            semi_token: input.parse::<syn::Token![;]>()?,
        })
    }
}

impl syn::parse::Parse for ConfigSource {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Ok(file_name) = input.parse() {
            return Ok(Self::Include { file_name });
        }
        if let Ok(syn::Macro {
            path: format,
            tokens: content_tokens,
            ..
        }) = input.parse()
        {
            let format = format.require_ident()?.clone();
            let content = syn::parse2(content_tokens)?;
            return Ok(Self::Literal { format, content });
        }
        Err(input.error("expected a string literal or a macro invocation"))
    }
}

fn parse_value(sources: impl IntoIterator<Item = ConfigSource>) -> Value {
    let mut value: Value = Map::<String, Value>::new().into();
    for source in sources {
        match source {
            ConfigSource::Include { file_name } => {
                if let Err(e) = File::with_name(file_name.value().as_str()).collect_to(&mut value) {
                    proc_macro_error::emit_error!(file_name, e);
                }
            }
            ConfigSource::Literal { format, content } => {
                let format = match format.to_string().as_str() {
                    "toml" => FileFormat::Toml,
                    "json" => FileFormat::Json,
                    "yaml" => FileFormat::Yaml,
                    "ini" => FileFormat::Ini,
                    "ron" => FileFormat::Ron,
                    "json5" => FileFormat::Json5,
                    _ => {
                        proc_macro_error::emit_error!(
                            format,
                            "supported config formats: toml, json, yaml, ini, ron, json5",
                        );
                        continue;
                    }
                };
                if let Err(e) =
                    File::from_str(content.value().as_str(), format).collect_to(&mut value)
                {
                    proc_macro_error::emit_error!(content, e);
                }
            }
        }
    }
    value
}

struct ConfigItemCollector {
    type_ts: proc_macro2::TokenStream,
    value_ts: proc_macro2::TokenStream,
    struct_items: Vec<proc_macro2::TokenStream>,
    select_impl_items: Vec<proc_macro2::TokenStream>,
    convert_impl_items: Vec<proc_macro2::TokenStream>,
}

impl ConfigItemCollector {
    fn from_value(value: Value, type_name: &str) -> Self {
        match value.kind {
            ValueKind::Nil => Self::from_nil(),
            ValueKind::Boolean(value) => Self::from_bool(value),
            ValueKind::I64(value) => Self::from_integer(value),
            ValueKind::I128(value) => Self::from_integer(value as i64),
            ValueKind::U64(value) => Self::from_integer(value as i64),
            ValueKind::U128(value) => Self::from_integer(value as i64),
            ValueKind::Float(value) => Self::from_float(value),
            ValueKind::String(value) => Self::from_string(value),
            ValueKind::Table(value) => Self::from_table(value, type_name),
            ValueKind::Array(value) => Self::from_array(value, type_name),
        }
    }

    fn from_nil() -> Self {
        Self::from_primitive(
            quote::quote! {
                ()
            },
            quote::quote! {
                ()
            },
        )
    }

    fn from_bool(value: bool) -> Self {
        Self::from_primitive(
            quote::quote! {
                bool
            },
            quote::quote! {
                #value
            },
        )
    }

    fn from_integer(value: i64) -> Self {
        Self::from_primitive(
            quote::quote! {
                i64
            },
            quote::quote! {
                #value
            },
        )
    }

    fn from_float(value: f64) -> Self {
        Self::from_primitive(
            quote::quote! {
                f64
            },
            quote::quote! {
                #value
            },
        )
    }

    fn from_string(value: String) -> Self {
        Self::from_primitive(
            quote::quote! {
                &'static str
            },
            quote::quote! {
                #value
            },
        )
    }

    fn from_table(value: Map<String, Value>, type_name: &str) -> Self {
        Self::from_container(
            value
                .into_iter()
                .map(|(name, value)| (KeySegment::name_type_ts(name.as_str()), name, value)),
            type_name,
        )
    }

    fn from_array(value: Vec<Value>, type_name: &str) -> Self {
        Self::from_container(
            value.into_iter().enumerate().map(|(index, value)| {
                (
                    KeySegment::index_type_ts(index),
                    format!("_{index}_"),
                    value,
                )
            }),
            type_name,
        )
    }

    fn from_primitive(
        type_ts: proc_macro2::TokenStream,
        value_ts: proc_macro2::TokenStream,
    ) -> Self {
        Self {
            type_ts,
            value_ts,
            struct_items: Vec::new(),
            select_impl_items: Vec::new(),
            convert_impl_items: Vec::new(),
        }
    }

    fn from_container(
        fields: impl Iterator<Item = (proc_macro2::TokenStream, String, Value)>,
        type_name: &str,
    ) -> Self {
        let type_ident = syn::Ident::new(type_name, proc_macro2::Span::call_site());
        let type_ts = quote::quote! { #type_ident };
        let mut names = Vec::new();
        let mut fields_type_ts = Vec::new();
        let mut fields_value_ts = Vec::new();
        let mut struct_items = Vec::new();
        let mut select_impl_items = Vec::new();
        let mut convert_impl_items = Vec::new();
        for (key_segment_type_ts, name, value) in fields {
            let ConfigItemCollector {
                type_ts: field_type_ts,
                value_ts: field_value_ts,
                struct_items: field_struct_items,
                select_impl_items: field_select_impl_items,
                convert_impl_items: field_convert_impl_items,
            } = Self::from_value(value, format!("{type_name}__{name}").as_str());
            let name = syn::Ident::new(name.as_str(), proc_macro2::Span::call_site());
            select_impl_items.push(quote::quote! {
                impl<'c> ::litconfig::__private::Select<'c, #key_segment_type_ts> for #type_ts {
                    type Representation = #field_type_ts;

                    fn select(&'c self, _key: #key_segment_type_ts) -> &'c Self::Representation {
                        &self.#name
                    }
                }
            });
            names.push(name);
            fields_type_ts.push(field_type_ts);
            fields_value_ts.push(field_value_ts);
            struct_items.extend(field_struct_items);
            select_impl_items.extend(field_select_impl_items);
            convert_impl_items.extend(field_convert_impl_items);
        }
        // convert_impl_items.push(quote::quote! {});
        struct_items.push(quote::quote! {
            struct #type_ts {
                #(#names: #fields_type_ts,)*
            }
        });
        let value_ts = quote::quote! {
            #type_ts {
                #(#names: #fields_value_ts,)*
            }
        };
        Self {
            type_ts,
            value_ts,
            struct_items,
            select_impl_items,
            convert_impl_items,
        }
    }
}

// fn iter_select_impls<'t>(
//     type_name: &'t syn::Ident,
//     table: &'t [(syn::LitStr, VariantRepresentation)],
// ) -> impl Iterator<Item = proc_macro2::TokenStream> + 't {
//     fn recurse<'v>(
//         type_name: &'v syn::Ident,
//         variant: &'v VariantRepresentation,
//         key: Key,
//         locs: Vec<usize>,
//     ) -> impl Iterator<Item = proc_macro2::TokenStream> + 'v {
//         let key_type_ts = key.clone().type_ts();
//         let type_ts = variant.type_ts_variant();
//         std::iter::once(quote::quote! {
//             impl<'c> ::litconfig::__private::Select<'c, #key_type_ts> for #type_name {
//                 type Representation = #type_ts;

//                 fn select(&'c self, _key: #key_type_ts) -> &'c Self::Representation {
//                     &self #(.#locs)*
//                 }
//             }
//         })
//         .chain::<Box<dyn Iterator<Item = _>>>(match variant {
//             VariantRepresentation::NamedTree(table) => Box::new(table.iter().enumerate().flat_map(
//                 move |(loc, (name, field))| {
//                     let mut key = key.clone();
//                     key.postfix.push(KeySegment::Name(name.clone()));
//                     let mut locs = locs.clone();
//                     locs.push(loc);
//                     recurse(type_name, field, key, locs)
//                 },
//             )),
//             VariantRepresentation::UnnamedTree(array) => Box::new(
//                 array
//                     .iter()
//                     .enumerate()
//                     .flat_map(move |(loc, (index, field))| {
//                         let mut key = key.clone();
//                         key.postfix.push(KeySegment::Index(index.clone()));
//                         let mut locs = locs.clone();
//                         locs.push(loc);
//                         recurse(type_name, field, key, locs)
//                     }),
//             ),
//             _ => Box::new(std::iter::empty()),
//         })
//     }
//     table.iter().enumerate().flat_map(|(loc, (name, field))| {
//         recurse(
//             type_name,
//             field,
//             Key {
//                 root: name.clone(),
//                 postfix: Vec::new(),
//             },
//             Vec::from([loc]),
//         )
//     })
// }

fn config_item_ts(config_item: ConfigItem) -> proc_macro2::TokenStream {
    let ConfigItem {
        attrs,
        vis,
        static_token,
        name,
        eq_token,
        sources,
        semi_token,
    } = config_item;
    let ConfigItemCollector {
        type_ts,
        value_ts,
        struct_items,
        select_impl_items,
        convert_impl_items,
    } = ConfigItemCollector::from_value(parse_value(sources), format!("__{name}").as_str());
    quote::quote! {
        #(#attrs)*
        #vis #static_token #name: #type_ts #eq_token #value_ts #semi_token

        #(#struct_items)*
        #(#select_impl_items)*
        #(#convert_impl_items)*
    }
}

pub(crate) fn config_items_ts(config_items: ConfigItems) -> proc_macro2::TokenStream {
    proc_macro2::TokenStream::from_iter(config_items.items.into_iter().map(config_item_ts))
}

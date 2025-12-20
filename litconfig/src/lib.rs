#[proc_macro_error::proc_macro_error]
#[proc_macro]
pub fn config(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match config2(input.into()) {
        Ok(ts) => ts.into(),
        Err(Error::Syn(e)) => proc_macro_error::abort!(e.span(), e.to_string()),
    }
}

fn config2(input: proc_macro2::TokenStream) -> Result<proc_macro2::TokenStream, Error> {
    let items: ConfigItems = syn::parse2(input).map_err(Error::Syn)?;
    let mut tokens = Vec::new();
    for item in items.items {
        let mut sources = Vec::new();
        for source in item.sources {
            let (source_str, format) = match source {
                ConfigSource::Include { path } => {
                    let cargo_manifest_dir =
                        std::env::var("CARGO_MANIFEST_DIR").map_err(Error::Env)?;
                    let path_buf = std::path::PathBuf::from(cargo_manifest_dir).join(path.value());
                    let source_str = std::fs::read_to_string(path_buf)
                        .map_err(|e| Error::FileReadError(path, e))?;
                    let format = path_buf
                        .extension()
                        .and_then(std::ffi::OsStr::to_str)
                        .and_then(select_format)
                        .ok_or(Error::FileExtensionUnsupported(path))?;
                    (source_str, format)
                }
                ConfigSource::Literal {
                    format, content, ..
                } => {
                    let format = select_format(format.to_string().as_str())
                        .ok_or(Error::FormatUnsupported(format))?;
                    (content.value(), format)
                }
            };
            // let file = config::FileSource
        }
    }
    Ok(proc_macro2::TokenStream::from_iter(tokens))
}

enum Error {
    Syn(syn::Error),
    Env(std::env::VarError),
    FileReadError(syn::LitStr, std::io::Error),
    FileExtensionUnsupported(syn::LitStr),
    FormatUnsupported(syn::Ident),
    ParseError(syn::LitStr, config::ConfigError),
}

// pub struct StaticToml(pub Vec<StaticTomlItem>);

struct ConfigItems {
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
        path: syn::LitStr,
    },
    Literal {
        format: syn::Ident,
        paren: syn::token::Paren,
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
        if let Ok(path) = input.parse() {
            Ok(ConfigSource::Include { path })
        } else {
            let inner;
            Ok(ConfigSource::Literal {
                format: input.parse()?,
                paren: syn::parenthesized!(inner in input),
                content: inner.parse()?,
            })
        }
        // let format: syn::Ident = input.parse()?;
        // input.parse::<syn::Token![!]>()?;
        // let format = config::FileFormat match format.to_string().as_str() {
        //     "toml" => config::FileFormat::Toml,
        //     "json" => config::FileFormat::Json,
        //     "yaml" => config::FileFormat::Yaml,
        //     "ini" => config::FileFormat::Ini,
        //     "ron" => config::FileFormat::Ron,
        //     "json5" => config::FileFormat::Json5,
        //     _ => {
        //         return Err(syn::Error::new_spanned(
        //             format,
        //             "supported config formats: toml, json, yaml, ini, ron, json5",
        //         ))
        //     }
        // };
        // let inner;
        // syn::parenthesized!(inner in input);
        // let content = input.parse::<ConfigSourceContent>()?;
        // Ok(Self { format, content })
    }
}

fn select_format(s: &str) -> Option<config::FileFormat> {
    Some(match s {
        "toml" => config::FileFormat::Toml,
        "json" => config::FileFormat::Json,
        "yaml" => config::FileFormat::Yaml,
        "yml" => config::FileFormat::Yaml,
        "ini" => config::FileFormat::Ini,
        "ron" => config::FileFormat::Ron,
        "json5" => config::FileFormat::Json5,
        _ => return None,
    })
}

use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

pub struct ConfigFallbackContent(pub &'static str);

impl Deref for ConfigFallbackContent {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

inventory::collect!(ConfigFallbackContent);

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct ConfigParsedContents(Vec<Arc<toml::Value>>);

impl ConfigParsedContents {
    pub fn parse_and_push(&mut self, content: &'static str) {
        self.0.push(Arc::new(toml::from_str(content).unwrap()));
    }

    fn read_value<CF>(&self) -> CF
    where
        CF: ConfigField,
    {
        CF::parse(
            self.0
                .iter()
                .rev()
                .filter_map(|toml_config| {
                    let mut option_value = Some(toml_config.as_ref());
                    for key in CF::PATH.split('.') {
                        option_value = option_value
                            .and_then(|value| value.as_table().and_then(|table| table.get(key)));
                    }
                    option_value
                })
                .next()
                .unwrap(),
        )
    }
}

#[derive(Debug, Default)]
pub struct Config {
    parsed_contents: ConfigParsedContents,
    storage: RefCell<typemap_rev::TypeMap<dyn typemap_rev::DebuggableStorage>>,
}

impl Config {
    pub fn new(parsed_contents: ConfigParsedContents) -> Self {
        Self {
            parsed_contents,
            storage: RefCell::new(typemap_rev::TypeMap::custom()),
        }
    }

    pub fn operate<CF, F, FO>(&self, f: F) -> FO
    where
        CF: ConfigField,
        F: FnOnce(&CF) -> FO,
    {
        if let Some(value) = self.storage.borrow().get::<ConfigFieldWrapper<CF>>() {
            f(value)
        } else {
            f(self
                .storage
                .borrow_mut()
                .entry::<ConfigFieldWrapper<CF>>()
                .or_insert(self.parsed_contents.read_value()))
        }
    }

    pub fn get_cloned<CF>(&self) -> CF
    where
        CF: Clone + ConfigField,
    {
        self.operate::<CF, _, _>(CF::clone)
    }
}

pub trait ConfigField: 'static + Debug + Send + Sized + Sync {
    const PATH: &'static str;

    fn parse(value: &toml::Value) -> Self;
}

struct ConfigFieldWrapper<CF>(CF);

impl<CF> typemap_rev::TypeMapKey for ConfigFieldWrapper<CF>
where
    CF: ConfigField,
{
    type Value = CF;
}

// impl ConfigField for i64 {
//     fn parse(value: &toml::Value) -> Self {
//         value.as_integer().unwrap()
//     }
// }

// impl ConfigField for f64 {
//     fn parse(value: &toml::Value) -> Self {
//         value.as_float().unwrap()
//     }
// }

// impl ConfigField for bool {
//     fn parse(value: &toml::Value) -> Self {
//         value.as_bool().unwrap()
//     }
// }

// impl ConfigField for String {
//     fn parse(value: &toml::Value) -> Self {
//         value.as_str().unwrap().into()
//     }
// }

// impl<T> ConfigField for Vec<T>
// where
//     T: ConfigField,
// {
//     fn parse(value: &toml::Value) -> Self {
//         value
//             .as_array()
//             .unwrap()
//             .into_iter()
//             .map(|element| T::parse(element))
//             .collect()
//     }
// // }

// impl ConfigField for PathBuf {
//     fn parse(value: &toml::Value) -> Self {
//         value.as_str().unwrap().into()
//     }
// }

// impl ConfigField for palette::Srgba {
//     fn parse(value: &toml::Value) -> Self {
//         palette::Srgba::from_str(value.as_str().unwrap())
//             .unwrap()
//             .into()
//     }
// }

// impl ConfigField for palette::Srgba<f64> {
//     fn parse(value: &toml::Value) -> Self {
//         palette::Srgba::from_str(value.as_str().unwrap())
//             .unwrap()
//             .into()
//     }
// }

// impl ConfigField for iced::widget::shader::wgpu::Color {
//     fn parse(value: &toml::Value) -> Self {
//         let color = palette::Srgba::<f64>::parse(value);
//         iced::widget::shader::wgpu::Color {
//             r: color.red,
//             g: color.green,
//             b: color.blue,
//             a: color.alpha,
//         }
//     }
// }

// impl ConfigField for iced::Size<u32> {
//     fn parse(value: &toml::Value) -> Self {
//         let (width, height) = value.as_str().unwrap().split_once('x').unwrap();
//         iced::Size::new(width.parse().unwrap(), height.parse().unwrap())
//     }
// }

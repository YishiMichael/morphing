use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct ConfigValues(Vec<Arc<toml::Value>>);

impl ConfigValues {
    pub(crate) fn overwrite(&mut self, content: &str) {
        self.0.push(Arc::new(toml::from_str(content).unwrap()));
    }

    fn read_value(&self, path: &'static str) -> &toml::Value {
        self.0
            .iter()
            .rev()
            .filter_map(|toml_config| {
                let mut option_value = Some(toml_config.as_ref());
                for key in path.split('.') {
                    option_value = option_value
                        .and_then(|value| value.as_table().and_then(|table| table.get(key)));
                }
                option_value
            })
            .next()
            .unwrap()
    }
}

#[derive(Debug, Default)]
pub struct Config {
    values: ConfigValues,
    storage: RefCell<type_map::TypeMap>,
}

impl Config {
    pub fn new(values: ConfigValues) -> Self {
        Self {
            values,
            storage: RefCell::new(type_map::TypeMap::new()),
        }
    }

    pub fn operate<F, T, OT>(&self, path: &'static str, f: F) -> OT
    where
        T: ConfigField,
        F: FnOnce(&T) -> OT,
    {
        f(self
            .storage
            .borrow_mut()
            .entry()
            .or_insert_with(HashMap::new)
            .entry(path)
            .or_insert_with(|| T::parse(self.values.read_value(path))))
    }
}

pub trait ConfigField: 'static + Sized {
    fn parse(value: &toml::Value) -> Self;
}

impl ConfigField for i64 {
    fn parse(value: &toml::Value) -> Self {
        value.as_integer().unwrap()
    }
}

impl ConfigField for f64 {
    fn parse(value: &toml::Value) -> Self {
        value.as_float().unwrap()
    }
}

impl ConfigField for bool {
    fn parse(value: &toml::Value) -> Self {
        value.as_bool().unwrap()
    }
}

impl<T> ConfigField for Vec<T>
where
    T: ConfigField,
{
    fn parse(value: &toml::Value) -> Self {
        value
            .as_array()
            .unwrap()
            .into_iter()
            .map(|element| T::parse(element))
            .collect()
    }
}

impl ConfigField for PathBuf {
    fn parse(value: &toml::Value) -> Self {
        PathBuf::from_str(value.as_str().unwrap()).unwrap().into()
    }
}

impl ConfigField for palette::Srgba {
    fn parse(value: &toml::Value) -> Self {
        palette::Srgba::from_str(value.as_str().unwrap())
            .unwrap()
            .into()
    }
}

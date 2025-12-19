use std::collections::HashMap;
use std::ops::RangeInclusive;
use std::sync::{OnceLock, RwLock};

pub fn default_options_list() -> &'static Vec<EngineOption<'static>> {
    static DEFAULTS: OnceLock<Vec<EngineOption>> = OnceLock::new();

    DEFAULTS.get_or_init(|| {
        vec![
            // Hash table size in megabytes
            EngineOption::new("Hash", OptionType::Spin(1..=2048, 128)),
            // The default depth the engine will search to
            EngineOption::new("Default Depth", OptionType::Spin(1..=64, 7)),
        ]
    })
}

fn default_options_map() -> &'static HashMap<String, OptionType> {
    static HASHMAP: OnceLock<HashMap<String, OptionType>> = OnceLock::new();

    HASHMAP.get_or_init(|| {
        let mut hashmap = HashMap::new();
        for option in default_options_list() {
            hashmap.insert(option.name.into(), option.data.clone());
        }
        hashmap
    })
}

#[derive(Debug)]
pub struct EngineOptions {
    hashmap: RwLock<HashMap<String, OptionType>>,
}

impl EngineOptions {
    pub fn print_defaults() {
        for option in default_options_list() {
            let name = option.name;
            let data = &option.data;
            println!(
                "option name {name} type {data} {}",
                match data {
                    OptionType::Button => "".into(),
                    OptionType::Check(check) => format!("default {check}",),
                    OptionType::String(string) => format!("default {string}",),
                    OptionType::Combo(items, val) => {
                        format!(
                            "default {val} {}",
                            items
                                .iter()
                                .map(|item| format!("var {item}"))
                                .collect::<Vec<String>>()
                                .join(" ")
                        )
                    }
                    OptionType::Spin(range, val) => {
                        format!("default {val} min {} max {}", range.start(), range.end())
                    }
                }
            );
        }
    }

    pub fn set<S: Into<String>>(&self, key: S, val: S) -> Result<(), anyhow::Error> {
        let key: String = key.into();
        let val: String = val.into();

        let mut options = self.hashmap.write().unwrap();
        if !options.contains_key(&key) {
            anyhow::bail!("unknown option: '{key}'");
        }

        let default_value = default_options_map().get(&key).unwrap();

        options.insert(
            key,
            match default_value {
                OptionType::Check(_) => OptionType::Check(val.parse()?),
                OptionType::String(_) => OptionType::String(val),
                OptionType::Combo(list, _) => {
                    if list.contains(&val) {
                        OptionType::Combo(list.to_vec(), val)
                    } else {
                        anyhow::bail!("not a valid list item");
                    }
                }
                OptionType::Spin(range, _) => {
                    let val: usize = val.trim().parse()?;
                    if val < *range.start() || val > *range.end() {
                        anyhow::bail!("value not in range");
                    } else {
                        OptionType::Spin(range.clone(), val)
                    }
                }
                OptionType::Button => OptionType::Button,
            },
        );

        Ok(())
    }

    pub fn get_number<S: Into<String>>(&self, key: S) -> usize {
        let key: String = key.into();
        let options = self.hashmap.read().unwrap();
        assert!(options.contains_key(&key));

        match options.get(&key).unwrap() {
            OptionType::Spin(_range, val) => *val,
            _ => panic!("Not a number value!"),
        }
    }

    pub fn get_string<S: Into<String>>(&self, key: S) -> String {
        let key: String = key.into();
        let options = self.hashmap.read().unwrap();
        assert!(options.contains_key(&key));

        match options.get(&key).unwrap() {
            OptionType::String(val) => val.clone(),
            OptionType::Combo(_list, val) => val.clone(),
            _ => panic!("Not a string value!"),
        }
    }

    pub fn get_bool<S: Into<String>>(&self, key: S) -> bool {
        let key: String = key.into();
        let options = self.hashmap.read().unwrap();
        assert!(options.contains_key(&key));

        match options.get(&key).unwrap() {
            OptionType::Check(val) => *val,
            _ => panic!("Not a boolean value!"),
        }
    }
}

impl Default for EngineOptions {
    fn default() -> Self {
        let mut hashmap = HashMap::new();

        for option in default_options_list() {
            hashmap.insert(option.name.into(), option.data.clone());
        }

        Self {
            hashmap: RwLock::new(hashmap),
        }
    }
}

pub struct EngineOption<'a> {
    name: &'a str,
    data: OptionType,
}

impl<'a> EngineOption<'a> {
    const fn new(name: &'a str, data: OptionType) -> EngineOption<'a> {
        Self { data, name }
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum OptionType {
    Button,
    Check(bool),
    String(String),
    Combo(Vec<String>, String),
    Spin(RangeInclusive<usize>, usize),
}

impl std::fmt::Display for OptionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OptionType::Button => "button",
                OptionType::Check(_) => "check",
                OptionType::String(_) => "string",
                OptionType::Combo(_, _) => "combo",
                OptionType::Spin(_, _) => "spin",
            }
        )
    }
}

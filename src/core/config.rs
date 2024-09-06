use super::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Struct holding the Config (the description of a command ands its subcommands/flags/args) with
/// some utility functions on top of it.
/// TODO: distinction between code in Machine, this file, and TokenMatching is rather arbritrary,
/// some very similar things (sub and token flattening) are done different ways.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabryConf {
    pub cmd: Option<String>,
    pub main: TabryConcreteSub,
    #[serde(default)]
    pub arg_includes: HashMap<String, TabryArgInclude>,
    #[serde(default)]
    pub option_includes: HashMap<String, Vec<TabryOpt>>,
}

#[derive(Error, Debug)]
pub enum TabryConfError {
    #[error("error reading config file")]
    IOError(#[from] std::io::Error),
    #[error("error parsing config file")]
    JsonError(#[from] serde_json::Error),
    #[error("internal error: {0}")]
    InternalError(String),
    #[error("internal error: {0}")]
    InvalidConfig(String),
    #[error("Missing include: {0}")]
    MissingInclude(String),
}

impl TabryConf {
    // TODO use thiserror instead of anyhow here.
    pub fn from_file(filename: &str) -> Result<Self, TabryConfError> {
        let conf_str = std::fs::read_to_string(filename)?;
        let conf: Self = serde_json::from_str(&conf_str)?;
        Ok(conf)
    }

    /// Get a TabryConcreteSub from the config, given it's "path" in the subcommand tree.
    /// This also resolves includes.
    /// e.g. given "sub foo { sub bar { .. } }", get the sub with dig_sub(["foo", "bar"])
    pub fn dig_sub(
        &self,
        sub_names_vec: &Vec<String>,
    ) -> Result<&TabryConcreteSub, TabryConfError> {
        let subs = self.dig_subs(sub_names_vec)?;
        return Ok(subs.last().unwrap());
    }

    // TODO switch to iterator without intermediate Vec
    /// Get all `TabryConcreteSub`s given a path in the subcommand tree, i.e., find the sub found
    /// by dig_sub along with all its ancestor subs.
    pub fn dig_subs(
        &self,
        sub_names_vec: &Vec<String>,
    ) -> Result<Vec<&TabryConcreteSub>, TabryConfError> {
        let mut result = vec![&self.main];

        for name in sub_names_vec {
            let sub = result.last().unwrap();
            let next =
                self.find_in_subs(&sub.subs, &sub.includes, name, false)?
                    .ok_or(TabryConfError::InternalError(
                        "sub not found in dig sub".to_owned(),
                    ))?;
            result.push(next);
        }

        Ok(result)
    }

    pub fn find_in_subs<'a>(
        &'a self,
        subs: &'a [TabrySub],
        includes: &'a Vec<String>,
        name: &String,
        check_aliases: bool,
    ) -> Result<Option<&TabryConcreteSub>, TabryConfError> {
        let concrete_subs: Vec<&TabryConcreteSub> = self.flatten_subs(subs, includes)?;

        for sub in concrete_subs {
            let sub_name = Self::unwrap_sub_name(sub)?;
            if name == sub_name || (check_aliases && sub.aliases.contains(name)) {
                return Ok(Some(sub));
            }
        }
        Ok(None)
    }

    pub fn unwrap_sub_name(sub: &TabryConcreteSub) -> Result<&str, TabryConfError> {
        match &sub.name {
            Some(s) => Ok(s.as_ref()),
            None => Err(TabryConfError::InvalidConfig(
                "sub without name not valid except as main sub".to_owned(),
            )),
        }
    }

    pub fn get_arg_include(&self, name: &str) -> Result<&TabryArgInclude, TabryConfError> {
        match self.arg_includes.get(name) {
            Some(inc) => Ok(inc),
            None => Err(TabryConfError::MissingInclude(name.to_string())),
        }
    }

    pub fn get_option_include(&self, name: &str) -> Result<&Vec<TabryOpt>, TabryConfError> {
        match self.option_includes.get(name) {
            Some(inc) => Ok(inc),
            None => Err(TabryConfError::MissingInclude(name.to_string())),
        }
    }

    pub fn flatten_subs<'a>(
        &'a self,
        subs: &'a [TabrySub],
        includes: &'a Vec<String>,
    ) -> Result<Vec<&TabryConcreteSub>, TabryConfError> {
        let mut vecofvecs = subs
            .iter()
            .map(|sub| match sub {
                TabrySub::TabryIncludeSub { include } => {
                    // Lookup include, which may return an error
                    let inc = self.get_arg_include(include)?;
                    // Flatten the include's subs recursively (which may return an error)
                    self.flatten_subs(&inc.subs, &inc.includes)
                }
                TabrySub::TabryConcreteSub(s) =>
                // This is a concrete sub, add it
                {
                    Ok(vec![s])
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        vecofvecs.extend(
            includes.iter().map(|include| {
                // Lookup include, which may return an error
                let inc = self.get_arg_include(include)?;
                // Flatten the include's subs recursively (which may return an error)
                self.flatten_subs(&inc.subs, &inc.includes)
            }).collect::<Result<Vec<_>, _>>()?,
        );

        // collect() will return an error if there were one, so now we just have flatten the
        // vectors
        Ok(vecofvecs.into_iter().flatten().collect::<Vec<_>>())
    }

    // TODO: Ugh, this is complicated with the Box and dyn. not sure of a better way. Seems
    // one flat_map call can return different types of iterators or something.
    pub fn expand_flags<'a>(
        &'a self,
        flags: &'a [TabryFlag],
        includes: &'a [String],
    ) -> Box<dyn Iterator<Item = &TabryConcreteFlag> + 'a> {
        let iter = flags.iter().flat_map(|flag| match flag {
            TabryFlag::TabryIncludeFlag { include } => {
                // TODO: bubble up error instead of unwrap (use get_arg_include)
                let include = self.arg_includes.get(include).unwrap();
                self.expand_flags(&include.flags, &include.includes)
            }
            TabryFlag::TabryConcreteFlag(concrete_flag) => Box::new(std::iter::once(concrete_flag)),
        });
        let iter = iter.chain(
            includes.iter().flat_map(move |include| {
                // TODO: bubble up error instead of unwrap
                let inc = self.arg_includes.get(include).unwrap();
                self.expand_flags(&inc.flags, &inc.includes)
            }),
        );

        Box::new(iter)
    }

    // TODO: this is an exact copy of the the above expand_flags()
    pub fn expand_args<'a>(
        &'a self,
        args: &'a [TabryArg],
    ) -> Box<dyn Iterator<Item = &TabryConcreteArg> + 'a> {
        let iter = args.iter().flat_map(|arg| match arg {
            TabryArg::TabryIncludeArg { include } => {
                // TODO: bubble up error instead of unwrap (use get_arg_include)
                let include = self.arg_includes.get(include).unwrap();
                self.expand_args(&include.args)
            }
            TabryArg::TabryConcreteArg(concrete_arg) => Box::new(std::iter::once(concrete_arg)),
        });
        Box::new(iter)
    }
}

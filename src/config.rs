use super::types::*;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use anyhow::Context; // TODO replace usage with thiserror

/// TODO: distinction between code in Machine, this file, and TokenMatching is rather arbritrary,
/// some very similar things (sub and token flattening) are done different ways.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabryConf {
    pub cmd: String,
    pub main: TabryConcreteSub,
    #[serde(default)]
    pub arg_includes: HashMap<String, TabryArgInclude>,
    #[serde(default)]
    pub option_includes: HashMap<String, Vec<TabryOpt>>,
}

impl TabryConf {
    // TODO use thiserror instead of anyhow here.
    pub fn from_file(filename: &str) -> anyhow::Result<Self> {
        let conf_str = std::fs::read_to_string(filename).
            with_context(|| "reading file failed")?;
        let conf: Self = serde_json::from_str(&conf_str).
            with_context(|| "parsing file failed")?;
        Ok(conf)
    }

    /// Get a TabryConcreteSub from the config, given it's "path" in the subcommand tree.
    /// This also resolves includes.
    /// e.g. given "sub foo { sub bar { .. } }", get the sub with dig_sub(["foo", "bar"])
    pub fn dig_sub(&self, sub_names_vec: &Vec<String>) -> Result<&TabryConcreteSub, &'static str> {
        let subs = self.dig_subs(sub_names_vec)?;
        return Ok(subs.last().unwrap());
    }

    // TODO switch to iterator without intermediate Vec
    /// Get all `TabryConcreteSub`s given a path in the subcommand tree.
    pub fn dig_subs(&self, sub_names_vec: &Vec<String>) -> Result<Vec<&TabryConcreteSub>, &'static str> {
        let mut result = vec![&self.main];

        for name in sub_names_vec {
            let subs_here = &result.last().unwrap().subs;
            let next = self.find_in_subs(subs_here, name, false)
                ?.ok_or("internal error: sub not found in dig sub")?;
            result.push(next);
        }

        Ok(result)
    }

    pub fn find_in_subs<'a>(&'a self, subs: &'a Vec<TabrySub>, name: &String, check_aliases: bool)
        -> Result<Option<&TabryConcreteSub>, &'static str> {
        let concrete_subs : Vec<&TabryConcreteSub> = self.flatten_subs(subs)?;

        for sub in concrete_subs {
            let sub_name = sub.name.as_ref().ok_or("sub without name not valid except as main sub")?;
            if name == sub_name || (check_aliases && sub.aliases.contains(name)) {
                return Ok(Some(sub));
            }
        }
        Ok(None)
    }

    pub fn flatten_subs<'a>(&'a self, subs: &'a Vec<TabrySub>) ->
        Result<Vec<&TabryConcreteSub>, &'static str> {

        let vecofvecs = subs.iter().map(|sub|
            match sub {
                TabrySub::TabryIncludeArg { include } => {
                    // Lookup include, which may return an error
                    let inc = self.arg_includes.get(include).ok_or("Error")?;
                    // Flatten the include's subs recursively (which may return an error)
                    self.flatten_subs(&inc.subs)
                },
                TabrySub::TabryConcreteSub(s) =>
                    // This is a concrete sub, add it
                    Ok(vec![s])
            }
        ).collect::<Result<Vec<_>,_>>()?;

        // collect() will return an error if there were one, so now we just have flatten the
        // vectors
        Ok(vecofvecs.into_iter().flatten().collect::<Vec<_>>())
    }


    // TODO: Ugh, this is complicated with the Box and dyn. not sure of a better way. Seems
    // one flat_map call can return different types of iterators or something.
    pub fn expand_flags<'a>(&'a self, flags: &'a Vec<TabryFlag>) -> Box<dyn Iterator<Item = &TabryConcreteFlag> + 'a> {
        let iter = flags.iter().flat_map(|flag|
            match flag {
                TabryFlag::TabryIncludeFlag { include } => {
                    // TODO: bubble up error instead of unwrap
                    let include = self.arg_includes.get(include).unwrap();
                    self.expand_flags(&include.flags).into_iter()
                }
                TabryFlag::TabryConcreteFlag(concrete_flag) =>
                    Box::new(std::iter::once(concrete_flag))
            }
        );
        Box::new(iter)
    }

}


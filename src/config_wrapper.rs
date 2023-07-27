use super::types;

/// Wrapper around Config struct that provides some methods for accessing it. In the future it may
/// have a cache.
pub struct ConfigWrapper {
    // TODO: this isn't modified. investigate "named lifetime parameters?" that would make sure we
    // couldn't modify it in here too which would be nice
    conf: types::TabryConf
}

impl ConfigWrapper {
    // In the future config wrapper will have a cache

    pub fn new(conf: types::TabryConf) -> ConfigWrapper {
        ConfigWrapper { conf }
    }

    // given "sub foo { sub bar { .. } }", get the sub with dig_sub(["foo", "bar"])
    pub fn dig_sub(&self, sub_names_vec: &Vec<String>) -> Result<&types::TabryConcreteSub, &'static str> {
        let subs = self.dig_subs(sub_names_vec)?;
        return Ok(subs.last().unwrap());
    }

    pub fn dig_subs(&self, sub_names_vec: &Vec<String>) -> Result<Vec<&types::TabryConcreteSub>, &'static str> {
        let mut result = vec![&self.conf.main];

        for name in sub_names_vec {
            let subs_here = &result.last().unwrap().subs;
            let next = self.find_in_subs(subs_here, name, false)
                ?.ok_or("internal error: sub not found in dig sub")?;
            result.push(next);
        }

        Ok(result)
    }

    pub fn find_in_subs<'a>(&'a self, subs: &'a Vec<types::TabrySub>, name: &String, check_aliases: bool)
        -> Result<Option<&types::TabryConcreteSub>, &'static str> {
        let concrete_subs : Vec<&types::TabryConcreteSub> = self.flatten_subs(subs)?;

        for sub in concrete_subs {
            let sub_name = sub.name.as_ref().ok_or("sub without name not valid except as main sub")?;
            if name == sub_name || (check_aliases && sub.aliases.contains(name)) {
                return Ok(Some(sub));
            }
        }
        Ok(None)
    }

    pub fn flatten_subs<'a>(&'a self, subs: &'a Vec<types::TabrySub>) ->
        Result<Vec<&types::TabryConcreteSub>, &'static str> {

        let vecofvecs = subs.iter().map(|sub|
            match sub {
                types::TabrySub::TabryIncludeArg { include } => {
                    // Lookup include, which may return an error
                    let inc = self.conf.arg_includes.get(include).ok_or("Error")?;
                    // Flatten the include's subs recursively (which may return an error)
                    self.flatten_subs(&inc.subs)
                },
                types::TabrySub::TabryConcreteSub(s) =>
                    // This is a concrete sub, add it
                    Ok(vec![s])
            }
        ).collect::<Result<Vec<_>,_>>()?;

        // collect() will return an error if there were one, so now we just have flatten the
        // vectors
        Ok(vecofvecs.into_iter().flatten().collect::<Vec<_>>())
    }
}


use std::fmt;
use std::collections::HashMap;
use std::mem::swap;

mod example_config_json;
mod types;

#[derive(Debug, Default, PartialEq)]
enum MachineStateMode {
    #[default]
    Subcommand,
    Flagarg {
        current_flag: String,
    },
}

#[derive(Default)]
struct MachineState {
    mode: MachineStateMode,
    subcommand_stack: Vec<String>,
    flags: HashMap<String, bool>,
    flag_args: HashMap<String, String>,
    args: Vec<String>,
    help: bool,
    dashdash: bool,
}

impl fmt::Debug for MachineState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = & mut f.debug_struct("MachineState");
        // TODO: macro to add to all if not default
        // except it doesn't work for vec... or bool, if serde_json is used...
        // there might even be another way to just not show defaults
        if !(self.mode == Default::default()) {
            res = res.field("mode", &self.mode);
        }
        if !self.subcommand_stack.is_empty() {
            res = res.field("subcommand_stack", &self.subcommand_stack);
        }
        if !(self.flags == Default::default()) {
            res = res.field("flags", &self.flags);
        }
        if !(self.flag_args == Default::default()) {
            res = res.field("flag_args", &self.flag_args);
        }
        if !self.args.is_empty() {
            res = res.field("args", &self.args);
        }
        if self.help {
            res = res.field("help", &self.help);
        }
        if self.dashdash {
            res = res.field("dashdash", &self.dashdash);
        }
        res.finish()
    }
}

struct Machine {
    config: ConfigWrapper,
    state: MachineState,
}

struct ConfigWrapper {
    // TODO: this isn't modified. investigate "named lifetime parameters?" that would make sure we
    // couldn't modify it in here too which would be nice
    conf: types::TabryConf
}

impl ConfigWrapper {
    // In the future config wrapper will have a cache

    fn dig_sub(&self, sub_names_vec: &Vec<String>) -> Result<&types::TabryConcreteSub, &'static str> {
        let mut current = &self.conf.main;

        for name in sub_names_vec {
            current = self.find_in_subs(&current.subs, name, false)?.ok_or("internal error: sub not found in dig sub")?;
        }

        Ok(current)
    }

    fn find_in_subs<'a>(&'a self, subs: &'a Vec<types::TabrySub>, name: &String, check_aliases: bool)
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

    fn flatten_subs<'a>(&'a self, subs: &'a Vec<types::TabrySub>) ->
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

impl Machine {
    // TODO: want to be able to pass a reference in here. need named lifetime. or can clone it...
    fn new(conf: types::TabryConf) -> Machine {
        Machine {
            config: ConfigWrapper { conf },
            state: MachineState::default()
        }
    }

    fn next(&mut self, token: &String) -> Result<(), &'static str> {
        match self.state.mode {
            MachineStateMode::Subcommand => self.match_mode_subcommand(token),
            MachineStateMode::Flagarg { .. } => Ok(self.match_mode_flagarg(token)),
        }
    }

    // TODO: error should be some class probably instead of a string
    fn match_mode_subcommand(&mut self, token: &String) -> Result<(), &'static str> {
        if self.match_subcommand(token)?
            || self.match_dashdash(token)
            || self.match_flag(token)?
            || self.match_help(token)
        {
            Ok(())
        } else {
            self.match_arg(token)
        }
    }


    /*
     * TODO using this doesn't work below
    fn current_sub(&mut self) -> Result<&types::TabryConcreteSub, &'static str> {
        self.config.dig_sub(&self.state.subcommand_stack)
    }
    */

    fn match_subcommand(&mut self, token: &String) -> Result<bool, &'static str> {
        if !self.state.args.is_empty() {
            return Ok(false);
        }

        // TODO using self.current_sub() causes weird borrow problem. But also want t
        // make self.find_in_subs etc. be able to mutate self which it can't right now
        // due to weird lifetime problem.
        let sub_here = self.config.dig_sub(&self.state.subcommand_stack)?;
        let subs_here = self.config.flatten_subs(&sub_here.subs);

        if let Some(sub) = self.config.find_in_subs(&sub_here.subs, token, true)? {
            let name = sub.name.as_ref().ok_or("sub must have name here")?;
            self.state.subcommand_stack.push(name.clone());
            self.log(format!("STEP subcommand, add {}", name));
            // TODO log
            Ok(true)
        } else {
            Ok(false)
        }

    }

    fn match_dashdash(&mut self, token: &String) -> bool {
        if !self.state.dashdash && token == "--" {
            self.state.dashdash = true;
            true
        } else {
            false
        }
    }

    fn match_flag(&mut self, token: &String) -> Result<bool, &'static str> {
        return Ok(false);
    }

    fn match_help(&mut self, token: &String) -> bool {
        if token == "help" {
            self.state.help = true;
            true
        } else {
            false
        }
    }

    fn match_arg(&mut self, token: &String) -> Result<(), &'static str> {
        self.log(format!("STEP fell back to argument {:?}", token));
        self.state.args.push(token.clone());
        return Ok(());
    }

    fn match_mode_flagarg(&mut self, token: &String) {
        // Set mode to subcommand and put string in flag_args
        let mut mode = MachineStateMode::Subcommand;
        swap(&mut mode, &mut self.state.mode);
        let MachineStateMode::Flagarg { current_flag } = mode else { unreachable!() };
        self.state.flag_args.insert(current_flag, token.clone());
    }

    fn log(&self, msg: String) {
        println!("{}; current state: {:?}", msg, self.state);
    }
}

fn main() {
    let mut machine = Machine::new(
        serde_json::from_str(example_config_json::STR).unwrap()
    );

    let tokens = ["move", "go", "vehicle1"];
    for token in tokens {
        machine.next(&String::from(token)).unwrap();
    }

    println!("{:?}", machine.state);
}

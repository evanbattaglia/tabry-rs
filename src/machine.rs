use crate::config_wrapper::ConfigWrapper;
use crate::types;
use std::collections::HashMap;
use std::mem::swap;
use std::fmt;

#[derive(Debug, Default, PartialEq)]
enum MachineStateMode {
    #[default]
    Subcommand,
    Flagarg {
        current_flag: String,
    },
}

#[derive(Default)]
pub struct MachineState {
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

pub struct Machine {
    config: ConfigWrapper,
    pub state: MachineState,
}

impl Machine {
    // TODO: want to be able to pass a reference in here. need named lifetime. or can clone it...
    pub fn new(conf: types::TabryConf) -> Machine {
        Machine {
            config: ConfigWrapper::new(conf),
            state: MachineState::default()
        }
    }

    pub fn next(&mut self, token: &String) -> Result<(), &'static str> {
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
        if self.state.dashdash {
            return Ok(false)
        }

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
        if let MachineStateMode::Flagarg { current_flag } = mode {
            self.state.flag_args.insert(current_flag, token.clone());
        } else {
            unreachable!();
        }
    }

    fn log(&self, msg: String) {
        println!("{}; current state: {:?}", msg, self.state);
    }
}


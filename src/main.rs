use std::collections::HashMap;
use std::mem::swap;

mod example_config_json;
mod types;

#[derive(Debug, Default)]
enum MachineStateMode {
    #[default]
    Subcommand,
    Flagarg {
        current_flag: String,
    },
}

#[derive(Debug, Default)]
struct MachineState {
    mode: MachineStateMode,
    subcommand_stack: Vec<String>,
    flags: HashMap<String, bool>,
    flag_args: HashMap<String, String>,
    args: Vec<String>,
    help: bool,
    dashdash: bool,
}

struct Machine {
    // TODO: this isn't modified. investigate "named lifetime parameters?"
    conf: types::TabryConf,
    state: MachineState,
}

impl Machine {
    fn new(conf: types::TabryConf) -> Machine {
        Machine { conf: conf, state: MachineState::default() }
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

    fn current_sub(&self) {
    }

    fn match_subcommand(&mut self, token: &String) -> Result<bool, &'static str> {
        if self.state.args.is_empty() {
            return Ok(false);
        }

        Ok(false)

        // let found = flatten_subs(self.current_sub()?.subs).find(token)
        // found.map( ... ).getOrElse(false)
        // state.subcommand_stack << sub.name
        // Tabry::Util.debug "MATCHED sub #{sub.name} ON token #{token.inspect}"
        // true
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
        return Ok(());
    }

    fn match_mode_flagarg(&mut self, token: &String) {
        // Set mode to subcommand and put string in flag_args
        let mut mode = MachineStateMode::Subcommand;
        swap(&mut mode, &mut self.state.mode);
        let MachineStateMode::Flagarg { current_flag } = mode else { unreachable!() };
        self.state.flag_args.insert(current_flag, token.clone());
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

    println!("{:?}", machine.state.args);
}

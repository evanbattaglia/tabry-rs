use super::config::TabryConf;
use super::machine_state::MachineState;

use super::types::TabryConcreteSub;

/// Encapsulates a TabryConfig and a TabryMachineState state, and provides
/// functionality relating to this state.
pub struct TabryResult {
    pub config: TabryConf,
    pub state: MachineState,

    // TODO: tried to make it a reference but got into a named lifetime mess... not sure why
    // when it only references things in TabryConf
    // maybe caching is not worth it if it takes a copy???
    pub current_sub: TabryConcreteSub,
    //pub sub_stack: Vector<TabryConcreteSub>,
}

impl TabryResult {
    pub fn new(config: TabryConf, state: MachineState) -> Self {
        let current_sub = config.dig_sub(&state.subcommand_stack).unwrap().clone();
        TabryResult { config, state, current_sub }
    }
}

/*
#[cfg(test)]
module tests {
    #[test]
    fn test_current_sub {
        let tabry_res = TabryResult {
        }
    }
}
*/

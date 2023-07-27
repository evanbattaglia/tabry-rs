use super::config_wrapper::ConfigWrapper;
use super::machine_state::MachineState;

use super::types;

/// Encapsulates a TabryConfig and a TabryMachineState state, and provides
/// functionality relating to this state.
pub struct TabryResult {
    pub config: ConfigWrapper,
    pub state: MachineState,

    // TODO: tried to make it a reference but got into a named lifetime mess... not sure why
    // when it only references things in ConfigWrapper.
    // maybe caching is not worth it if it takes a copy???
    pub current_sub: types::TabryConcreteSub,
}

impl TabryResult {
    pub fn new(config: ConfigWrapper, state: MachineState) -> Self {
        let current_sub = config.dig_sub(&state.subcommand_stack).unwrap().clone();
        TabryResult { config, state, current_sub }
    }
}

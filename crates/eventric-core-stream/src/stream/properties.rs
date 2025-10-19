use std::error::Error;

use eventric_core_state::{
    Keyspaces,
    Read,
};

// =================================================================================================
// Properties
// =================================================================================================

pub fn is_empty(keyspaces: &Keyspaces) -> Result<bool, Box<dyn Error>> {
    eventric_core_data::is_empty(&Read::new(keyspaces))
}

pub fn len(keyspaces: &Keyspaces) -> Result<u64, Box<dyn Error>> {
    eventric_core_data::len(&Read::new(keyspaces))
}

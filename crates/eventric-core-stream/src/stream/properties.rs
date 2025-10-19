use std::error::Error;

use eventric_core_state::Read;

// =================================================================================================
// Properties
// =================================================================================================

pub fn is_empty(read: &Read<'_>) -> Result<bool, Box<dyn Error>> {
    eventric_core_data::is_empty(read)
}

pub fn len(read: &Read<'_>) -> Result<u64, Box<dyn Error>> {
    eventric_core_data::len(read)
}

use derive_more::Debug;
use fancy_constructor::new;
use fjall::WriteBatch;

use crate::context::Keyspaces;

// =================================================================================================
// IO
// =================================================================================================

// Read/Write

#[derive(new, Debug)]
#[new(const_fn)]
pub struct Read<'a> {
    pub keyspaces: &'a Keyspaces,
}

#[derive(new, Debug)]
#[new(const_fn)]
pub struct Write<'a> {
    #[debug("Batch")]
    pub batch: &'a mut WriteBatch,
    pub keyspaces: &'a Keyspaces,
}

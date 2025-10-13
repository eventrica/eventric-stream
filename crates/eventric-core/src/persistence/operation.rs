use derive_more::Debug;
use fancy_constructor::new;
use fjall::WriteBatch;

use crate::persistence::Keyspaces;

// =================================================================================================
// Operation
// =================================================================================================

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct Read<'a> {
    pub keyspaces: &'a Keyspaces,
}

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct Write<'a> {
    #[debug("Batch")]
    pub batch: &'a mut WriteBatch,
    pub keyspaces: &'a Keyspaces,
}

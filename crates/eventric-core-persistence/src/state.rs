use derive_more::Debug;
use fancy_constructor::new;
use fjall::{
    Keyspace,
    WriteBatch,
};

// =================================================================================================
// State
// =================================================================================================

// Read/Write

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

// -------------------------------------------------------------------------------------------------

// Keyspaces

#[derive(new, Clone, Debug)]
#[new(vis(pub))]
pub struct Keyspaces {
    #[debug("Keyspace(\"{}\")", data.name)]
    pub data: Keyspace,
    #[debug("Keyspace(\"{}\")", index.name)]
    pub index: Keyspace,
    #[debug("Keyspace(\"{}\")", reference.name)]
    pub reference: Keyspace,
}

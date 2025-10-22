pub mod events;
pub mod indices;
pub mod references;

// =================================================================================================
// Data
// =================================================================================================

// Configuration

static HASH_LEN: usize = size_of::<u64>();
static ID_LEN: usize = size_of::<u8>();
static POSITION_LEN: usize = size_of::<u64>();

pub mod event;
pub mod query;
pub mod stream;

use rapidhash::v3::RapidSecrets;

// =================================================================================================
// Model
// =================================================================================================

// Configuration

static SEED: RapidSecrets = RapidSecrets::seed(0x2811_2017);

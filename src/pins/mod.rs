#[cfg(any(feature = "seed", feature = "seed_1_1", feature = "seed_1_2"))]
mod pins_seed;

#[cfg(any(feature = "seed", feature = "seed_1_1", feature = "seed_1_2"))]
pub use pins_seed::*;


#[cfg(feature = "patch_sm")]
mod pins_patch_sm;
#[cfg(feature = "patch_sm")]
pub use pins_patch_sm::*;

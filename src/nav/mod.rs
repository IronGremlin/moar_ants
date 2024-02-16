pub mod scentmap;
pub mod spatial_helper;

pub use spatial_helper::DistanceAwareQuery;

pub use scentmap::ScentMapPlugin;

pub mod scent {
    pub use crate::nav::scentmap::{ScentMap, ScentSettings, ScentType, WeightType};
}

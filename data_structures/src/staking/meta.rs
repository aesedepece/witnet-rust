use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

/// Versioned container for stake metadata.
///
/// Enables extensibility of stake entries with easy migrations that operate at a data structure
/// level instead of relying on mutating bytes on the fly when deserializing the whole `ChainState`.
#[derive(Copy, Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum StakeMeta<Epoch> {
    /// V2_0 used to carry no stake metadata.
    #[default]
    V2_0,
    /// V2_1 introduces stake metadata for the first time.
    V2_1(StakeMetaV2_1<Epoch>),
}

impl<Epoch> StakeMeta<Epoch>
where
    Epoch: Copy + Debug + Default + Display,
{
    /// Extract the value of the V2_1 "latest_active" piece of metadata.
    ///
    /// Gives `0` for older, non-migrated instances of `StakeMeta`.
    pub fn get_latest_active(&self) -> Epoch {
        match self {
            Self::V2_0 => Epoch::default(),
            Self::V2_1(StakeMetaV2_1 { latest_active }) => *latest_active,
        }
    }

    /// Update the value of the V2_1 "latest_active" piece of metadata.
    ///
    /// Does nothing for older, non-migrated instances of `StakeMeta`.
    pub fn set_latest_active(&mut self, epoch: Epoch) {
        if let Self::V2_1(meta) = self {
            meta.latest_active = epoch
        }
    }

    /// Make sure that stake metadata is upgraded to its latest version.
    pub fn migrate(&self) -> Self {
        match self {
            StakeMeta::V2_0 => {
                log::info!("Migrating stake entry from V2_0 to V2_1");

                StakeMeta::V2_1(StakeMetaV2_1::<Epoch> {
                    latest_active: Epoch::default(),
                })
            }
            StakeMeta::V2_1(_) => *self,
        }
    }
}

/// First version of stake metadata, as introduced by protocol version `V2_1`.
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct StakeMetaV2_1<Epoch> {
    /// The most recent epoch in which this entry has gotten a block accepted.
    ///
    /// Used for curating secure superblock voting committees.
    pub latest_active: Epoch,
}

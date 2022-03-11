//! Common requirements for crate consumers

pub use crate::{Error as Sx128xError, Sx128x, Sx128xSpi};

pub use crate::device::{Channel, Config, Modem, PacketInfo, RegulatorMode, State};

pub use crate::device::flrc::{FlrcChannel, FlrcConfig};
pub use crate::device::gfsk::{GfskChannel, GfskConfig};
pub use crate::device::lora::{LoRaChannel, LoRaConfig};


use super::common::*;

#[derive(Clone, PartialEq, Debug)]
pub struct GfskConfig {
    pub bitrate_bandwidth: GfskBleBitrate,
    pub modulation_index: GfskBleModIndex,
    pub modulation_shaping: ModShaping,
}

impl Default for GfskConfig {
    fn default() -> Self {
        Self {
            bitrate_bandwidth: GfskBleBitrate::GFSK_BLE_BR_0_250_BW_0_3,
            modulation_index: GfskBleModIndex::GFSK_BLE_MOD_IND_0_35,
            modulation_shaping: ModShaping::BtOFF,
        }   
    }
}

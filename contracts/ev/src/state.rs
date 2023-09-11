use crate::msg::{EnergyTransfer, EnergyTransferOffer};
use cw_storage_plus::{Map, Item};

pub const ENERGY_TRANSFER_OFFERS: Map<u64, EnergyTransferOffer> = Map::new("energy_transfer_offers");
pub const ENERGY_TRANSFER_OFFER_COUNT: Item<u64> = Item::new("energy_transfer_offer_count");
pub const ENERGY_TRANSFERS: Map<u64, EnergyTransfer> = Map::new("energy_transfers");
pub const ENERGY_TRANSFER_COUNT: Item<u64> = Item::new("energy_transfer_count");
pub const DENOM: Item<String> = Item::new("denom");

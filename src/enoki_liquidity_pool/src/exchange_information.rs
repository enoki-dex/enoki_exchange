use enoki_exchange_shared::is_managed::ManagementData;
use enoki_exchange_shared::{is_managed, types::*};

use crate::Principal;

pub fn assert_is_exchange() -> Result<()> {
    is_managed::assert_is_manager()
}

pub fn init_exchange_information(exchange: Principal) {
    is_managed::init_manager(ManagementData { manager: exchange })
}

pub fn export_stable_storage() -> (ManagementData,) {
    is_managed::export_stable_storage()
}

pub fn import_stable_storage(data: ManagementData) {
    is_managed::import_stable_storage(data);
}

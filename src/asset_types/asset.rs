use bus::BusReader;

#[derive(Debug, Clone, Copy)]
pub enum AssetChangeType {
    Modified,
}

pub trait Asset {
    fn get_change_receiver(&mut self) -> BusReader<AssetChangeType>;
    fn send_changes(&mut self, change_type: AssetChangeType);
    fn get_id(&self) -> u64;
}

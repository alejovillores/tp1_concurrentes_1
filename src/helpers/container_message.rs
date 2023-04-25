
pub trait ContainerMessage {
    fn is_not_ready(&self) -> bool;
    fn read(&mut self);
    fn ready_to_read(&mut self);
}
use super::resourse::Resourse;
use std::sync::{Arc, Condvar, Mutex};

pub trait Container {
    fn start(&mut self, monitor: Arc<(Mutex<Resourse>, Condvar)>);
}

use std_semaphore::Semaphore;

use super::resourse::Resourse;
use std::sync::{Arc, Condvar, Mutex};

pub trait Container {
    fn start(
        &mut self,
        request_monitor: Arc<(Mutex<Resourse>, Condvar)>,
        response_monitor: Arc<(Mutex<Resourse>, Condvar)>,
        bussy_sem: Arc<Semaphore>,
    );
}

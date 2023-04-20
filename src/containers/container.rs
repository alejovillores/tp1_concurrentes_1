use std_semaphore::Semaphore;

use std::sync::{Arc, Condvar, Mutex};

use crate::helpers::resourse::Resourse;

pub trait Container {
    fn start(
        &mut self,
        request_monitor: Arc<(Mutex<Resourse>, Condvar)>,
        response_monitor: Arc<(Mutex<Resourse>, Condvar)>,
        bussy_sem: Arc<Semaphore>,
    );
}

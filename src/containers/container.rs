use std_semaphore::Semaphore;

use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex},
};

use crate::helpers::{container_message::ContainerMessage, ingredients::Ingredients};

pub trait Container {
    fn start(
        &mut self,
        request_monitor: Arc<(Mutex<ContainerMessage>, Condvar)>,
        response_monitor: Arc<(Mutex<ContainerMessage>, Condvar)>,
        bussy_sem: Arc<Semaphore>,
        d_mute: Arc<Mutex<HashMap<Ingredients, i32>>>,
    );
}

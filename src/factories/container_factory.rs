use std::sync::Arc;

use std_semaphore::Semaphore;

use crate::containers::{coffe_container::CoffeContainer, container::Container};

pub struct ContainerFactory {}

pub enum Containers {
    Coffe,
    CoffeGrain,
}

impl ContainerFactory {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create(container_type: Containers, sem: Arc<Semaphore>) {
        match container_type {
            Containers::Coffe => todo!(),
            Containers::CoffeGrain => todo!(),
        }
    }
}

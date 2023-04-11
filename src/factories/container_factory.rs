use std::sync::Arc;

use std_semaphore::Semaphore;

pub struct ContainerFactory {}

pub enum Containers {
    Coffe,
    CoffeGrain,
}

impl ContainerFactory {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create(container_type: Containers, _sem: Arc<Semaphore>) {
        match container_type {
            Containers::Coffe => todo!(),
            Containers::CoffeGrain => todo!(),
        }
    }
}

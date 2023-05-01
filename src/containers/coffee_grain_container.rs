use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex},
};

use std_semaphore::Semaphore;

use crate::helpers::{
    container_message::{ContainerMessage, ContainerMessageType},
    ingredients::Ingredients,
};

use super::container::Container;

const FINISH_FLAG: i32 = -1;
const NO_MORE: i32 = 0;
const CAPACITY: i32 = 2500;

pub struct CoffeeGrainContainer {
    capacity: i32,
}

#[allow(clippy::new_without_default)]
impl CoffeeGrainContainer {
    pub fn new() -> Self {
        let capacity = CAPACITY;
        Self { capacity }
    }

    #[allow(dead_code)]
    fn refill(&mut self, amount: i32) -> i32 {
        if self.capacity >= amount && amount.is_positive() {
            self.capacity -= amount;
            return amount;
        };

        if self.capacity == NO_MORE {
            println!("[coffee grain container] - no more coffee grain sending FINISHING FLAG");
            return FINISH_FLAG;
        };

        if self.capacity <= amount {
            let result = self.capacity;
            self.capacity = 0;
            return result;
        };

        FINISH_FLAG
    }

    fn wait_refill(
        &mut self,
        lock: &Mutex<ContainerMessage>,
        cvar: &Condvar,
    ) -> Result<ContainerMessage, String> {
        if let Ok(guard) = lock.lock() {
            if let Ok(mut message) = cvar.wait_while(guard, |status| status.is_not_ready()) {
                message.read();
                let result = ContainerMessage::new(message.get_amount(), message.get_type());
                return Ok(result);
            }
        };
        Err("[error] - coffee grain container  monitor failed".to_string())
    }

    fn signal_refill(
        &self,
        lock: &Mutex<ContainerMessage>,
        cvar: &Condvar,
        resourse: ContainerMessage,
    ) -> Result<(), String> {
        if let Ok(mut old_resourse) = lock.lock() {
            *old_resourse = resourse;
            println!(
                "[coffee grain container] - send new coffee grain amount: {} response",
                old_resourse.get_amount()
            );
            old_resourse.ready_to_read();
            cvar.notify_all();
            return Ok(());
        };
        Err("[error] - coffee amount monitor failed in coffee dispenser".to_string())
    }

    fn check_capacity(&self) -> bool {
        let min_capacity = (CAPACITY as f32) * (0.2_f32);
        self.capacity as f32 <= min_capacity
    }

    fn save_status(&self, d_mutex: Arc<Mutex<HashMap<Ingredients, i32>>>) {
        if let Ok(mut guard) = d_mutex.lock() {
            guard.insert(Ingredients::CoffeGrain, self.capacity);
        }
    }
}

impl Container for CoffeeGrainContainer {
    fn start(
        &mut self,
        request_monitor: Arc<(Mutex<ContainerMessage>, Condvar)>,
        response_monitor: Arc<(Mutex<ContainerMessage>, Condvar)>,
        bussy_sem: Arc<Semaphore>,
        d_mutex: Arc<Mutex<HashMap<Ingredients, i32>>>,
    ) {
        loop {
            let (lock, cvar) = &*request_monitor;
            if let Ok(res) = self.wait_refill(lock, cvar) {
                let container_message_response: ContainerMessage = match res.get_type() {
                    ContainerMessageType::ResourseRequest => {
                        println!(
                            "[coffee grain container] - receving refill request {}",
                            res.get_amount()
                        );
                        let amounte_consumed = self.refill(res.get_amount());
                        ContainerMessage::new(
                            amounte_consumed,
                            ContainerMessageType::ResourseRequest,
                        )
                    }
                    ContainerMessageType::KillRequest => {
                        println!("[coffee grain container] - receiving FINISHING FLAG",);
                        ContainerMessage::new(FINISH_FLAG, ContainerMessageType::KillRequest)
                    }
                };
                if self.check_capacity() {
                    println!("[coffee grain container] - CAPACITY LOWER THAN 20% ")
                }

                let (res_lock, res_cvar) = &*response_monitor;
                if self
                    .signal_refill(res_lock, res_cvar, container_message_response)
                    .is_err()
                {
                    println!("[error] - error in coffee grain container")
                }

                if matches!(res.get_type(), ContainerMessageType::KillRequest) {
                    println!("[coffee grain container] - Kill Request - Killing thread ");
                    break;
                }
                self.save_status(d_mutex.clone());
                bussy_sem.release();
            }
        }
    }
}

#[cfg(test)]
mod coffee_grain_container_test {
    use super::CoffeeGrainContainer;

    #[test]
    fn it_should_return_100_when_amount_is_smaller_than_2500() {
        let mut coffee_grain_container = CoffeeGrainContainer::new();
        let amount = 100;
        let result = coffee_grain_container.refill(amount);

        assert_eq!(result, amount);
    }

    #[test]
    fn it_should_have_2400_when_amount_is_100() {
        let mut coffee_grain_container = CoffeeGrainContainer::new();
        let amount = 100;
        let _result = coffee_grain_container.refill(amount);
        let expected = 2400;

        assert_eq!(coffee_grain_container.capacity, expected);
    }

    #[test]
    fn it_should_return_40_when_capacity_is_40_and_amount_100() {
        let mut coffee_grain_container = CoffeeGrainContainer::new();
        coffee_grain_container.capacity = 40;

        let amount = 100;
        let result = coffee_grain_container.refill(amount);
        let expected = 40;

        assert_eq!(result, expected);
    }

    #[test]
    fn it_should_return_true_when_capacity_is_lower_than_20_percent() {
        let mut coffee_grain_container = CoffeeGrainContainer::new();
        /* 2500 is max capacity, 500 is 20% */
        coffee_grain_container.capacity = 500;
        assert!(coffee_grain_container.check_capacity())
    }
}

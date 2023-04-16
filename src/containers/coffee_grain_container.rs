use std::sync::{Arc, Condvar, Mutex};

use std_semaphore::Semaphore;

use super::{container::Container, resourse::Resourse};

const EMPTY: i32 = -1;
const CAPACITY: i32 = 2500;

pub struct CoffeeGrainContainer {
    capacity: i32,
    last_id_read: i32,
}

impl CoffeeGrainContainer {
    pub fn new() -> Self {
        let capacity = CAPACITY;
        let last_id_read = 0;
        Self {
            capacity,
            last_id_read,
        }
    }

    #[allow(dead_code)]
    fn refill(&mut self, amount: i32) -> i32 {
        if self.capacity >= amount && amount.is_positive() {
            println!("[coffee grain container] - amount valid");
            self.capacity -= amount;
            return amount;
        };

        if self.capacity == 0 {
            println!("[coffee grain container] - no more coffee grain");
            return EMPTY;
        };

        if amount.is_negative() {
            return EMPTY;
        }

        if self.capacity <= amount {
            let result = self.capacity;
            self.capacity = 0;
            return result;
        };

        EMPTY
    }

    fn wait_refill(&mut self, lock: &Mutex<Resourse>, cvar: &Condvar) -> Result<i32, String> {
        if let Ok(guard) = lock.lock() {
            if let Ok(mut resourse) = cvar.wait_while(guard, |status| status.is_not_ready()) {
                let coffee_amount = resourse.get_amount();
                println!(
                    "[coffee grain container] - coffe container asking for amount {}",
                    coffee_amount
                );
                resourse.read();
                return Ok(coffee_amount);
            };
        };

        Err("[error] - coffee container  monitor failed".to_string())
    }

    fn signal_refill(
        &self,
        lock: &Mutex<Resourse>,
        cvar: &Condvar,
        resourse: Resourse,
    ) -> Result<(), String> {
        if let Ok(mut old_resourse) = lock.lock() {
            *old_resourse = resourse;
            println!(
                "[coffee grain container] - send new coffee grain amount: {} request",
                old_resourse.get_amount()
            );
            old_resourse.ready_to_read();
            cvar.notify_all();
            return Ok(());
        };
        Err("[error] - coffee amount monitor failed in coffee dispenser".to_string())
    }
}

impl Container for CoffeeGrainContainer {
    fn start(
        &mut self,
        request_monitor: Arc<(Mutex<Resourse>, Condvar)>,
        response_monitor: Arc<(Mutex<Resourse>, Condvar)>,
        _bussy_sem: Arc<Semaphore>,
    ) {
        loop {
            let (lock, cvar) = &*request_monitor;
            if let Ok(amount) = self.wait_refill(lock, cvar) {
                let refill_amount = self.refill(amount);
                let resourse = Resourse::new(refill_amount);

                let (res_lock, res_cvar) = &*response_monitor;
                if self.signal_refill(res_lock, res_cvar, resourse).is_err() {
                    println!("[error] - error in coffee grain container")
                }

                if refill_amount == EMPTY {
                    break;
                };
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
}

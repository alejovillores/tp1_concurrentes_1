use std::sync::{Arc, Condvar, Mutex};

use super::{container::Container, resourse::Resourse};

const EMPTY: i32 = -1;
const CAPACITY: i32 = 2500;

pub struct CoffeeGrainContainer {
    capacity: i32,
}

impl CoffeeGrainContainer {
    pub fn new() -> Self {
        let capacity = CAPACITY;
        Self { capacity }
    }

    #[allow(dead_code)]
    fn refill(&mut self, amount: i32) -> i32 {
        if self.capacity >= amount {
            self.capacity -= amount;
            return amount;
        };

        if self.capacity == 0 {
            return EMPTY;
        };

        if self.capacity <= amount {
            let result = self.capacity;
            self.capacity = 0;
            return result;
        };

        EMPTY
    }

    #[allow(dead_code)]
    fn wait_refill(&mut self, lock: &Mutex<Resourse>, cvar: &Condvar) -> Result<i32, String> {
        if let Ok(guard) = lock.lock() {
            if let Ok(mut resourse) = cvar.wait_while(guard, |status| status.is_ready()) {
                let coffee_amount = resourse.get_amount();
                let result = self.refill(coffee_amount);
                *resourse = Resourse::new(result);
                resourse.ready();
                return Ok(result);
            };
        };

        Err("[error] - coffee container  monitor failed".to_string())
    }
}

impl Container for CoffeeGrainContainer {
    fn start(&mut self, _monitor: Arc<(Mutex<Resourse>, Condvar)>) {
        todo!()
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
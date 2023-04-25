use std::{
    sync::{Condvar, Mutex},
    thread,
    time::Duration,
};

use crate::helpers::{resourse::Resourse, container_message::ContainerMessage};

use super::container::Container;

const CAPACITY: i32 = 100;
const FINISH_FLAG: i32 = -1;

pub struct WaterContainer {
    capacity: i32,
}

#[allow(clippy::new_without_default)]
impl WaterContainer {
    pub fn new() -> Self {
        let capacity = 0;
        Self { capacity }
    }

    fn refill(&mut self, amount: i32) {
        println!("[water container] - waiting for more hot water");
        thread::sleep(Duration::from_secs(amount as u64));
        self.capacity = CAPACITY;
    }

    fn consume(&mut self, amount: i32) -> Result<i32, String> {
        if (self.capacity <= amount) && (amount <= CAPACITY) {
            let refill_amount = amount - self.capacity;
            println!(
                "[water container] - refilling {} units of water",
                refill_amount
            );
            self.refill(refill_amount);
            self.consume(amount)
        } else if (self.capacity >= amount) && (amount.is_positive()) {
            println!("[water container] - consuming {} units of water", amount);
            self.capacity -= amount;
            return Ok(amount);
        } else {
            return Ok(FINISH_FLAG);
        }
    }

    // Waits for dispenser to send new water request
    fn wait_dispenser(&mut self, lock: &Mutex<Resourse>, cvar: &Condvar) -> Result<i32, String> {
        if let Ok(guard) = lock.lock() {
            if let Ok(mut resourse) = cvar.wait_while(guard, |status| status.is_not_ready()) {
                let water_amount = resourse.get_amount();

                if water_amount == FINISH_FLAG {
                    println!("[water container] - dispenser sending FINISHING FLAG",);
                } else {
                    println!(
                        "[water container] - dispenser asking for {} units of water",
                        water_amount
                    );
                }
                resourse.read();
                return Ok(water_amount);
            }
        };
        Err("[error] - water container  monitor failed".to_string())
    }

    // Notify dispenser about new resourse avaliable
    fn notify_dispenser(&mut self, lock: &Mutex<Resourse>, cvar: &Condvar, res: Resourse) {
        if let Ok(mut resourse) = lock.lock() {
            *resourse = res;
            resourse.ready_to_read();
            if resourse.get_amount() == FINISH_FLAG {
                println!("[water container] - notifying dispenser FINISHING FLAG ",);
            } else {
                println!(
                    "[water container] - sending {} units to dispenser",
                    resourse.get_amount(),
                );
            }
            cvar.notify_all();
        }
    }
}

impl Container for WaterContainer {
    fn start(
        &mut self,
        request_monitor: std::sync::Arc<(Mutex<Resourse>, Condvar)>,
        response_monitor: std::sync::Arc<(Mutex<Resourse>, Condvar)>,
        bussy_sem: std::sync::Arc<std_semaphore::Semaphore>,
    ) {
        loop {
            let (lock, cvar) = &*request_monitor;
            println!("[water container] - waiting for request");
            if let Ok(res) = self.wait_dispenser(lock, cvar) {
                println!("[water container] - attempting to consume amount {}", res);

                if let Ok(amounte_consumed) = self.consume(res) {
                    let (res_lock, res_cvar) = &*response_monitor;
                    self.notify_dispenser(res_lock, res_cvar, Resourse::new(amounte_consumed));
                    if res == FINISH_FLAG {
                        println!("[water container] - finishing ");
                        break;
                    }
                    bussy_sem.release();
                    println!("[water container] - released sem")
                }
            }
        }
    }
}

#[cfg(test)]
mod water_container_test {
    use crate::containers::water_container::WaterContainer;

    #[test]
    fn it_should_init_with_0() {
        let water_container = WaterContainer::new();
        assert_eq!(water_container.capacity, 0)
    }

    #[test]
    fn it_should_refill_when_amount_is_bigger_than_capacity() {
        let mut water_container = WaterContainer::new();
        let amount = 10;
        water_container.consume(amount).unwrap();
        assert_eq!(water_container.capacity, 90)
    }

    #[test]
    fn it_should_consume_when_amount_is_smaller_than_capacity() {
        let mut water_container = WaterContainer::new();
        water_container.capacity = 20;
        let amount = 10;
        water_container.consume(amount).unwrap();
        assert_eq!(water_container.capacity, 10)
    }

    #[test]
    fn it_should_return_finish_flag_when_amount_negative() {
        let mut water_container = WaterContainer::new();
        let finish_flag = -1;
        let amount = -1;
        let res = water_container.consume(amount).unwrap();
        assert_eq!(res, finish_flag)
    }
}

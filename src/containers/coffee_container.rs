use std_semaphore::Semaphore;

use super::container::Container;
use crate::helpers::{
    container_message::{ContainerMessage, ContainerMessageType},
    ingredients::Ingredients,
};
use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex},
};

const CAPACITY: i32 = 100;
const FINISH_FLAG: i32 = -1;
const NO_MORE: i32 = 0;

pub struct CoffeContainer {
    capacity: i32,
    refill_req_monitor: Arc<(Mutex<ContainerMessage>, Condvar)>,
    refill_res_monitor: Arc<(Mutex<ContainerMessage>, Condvar)>,
    sem: Arc<Semaphore>,
}

#[allow(clippy::new_without_default)]
impl CoffeContainer {
    pub fn new(
        refill_req_monitor: Arc<(Mutex<ContainerMessage>, Condvar)>,
        refill_res_monitor: Arc<(Mutex<ContainerMessage>, Condvar)>,
        sem: Arc<Semaphore>,
    ) -> Self {
        let capacity = 0;
        Self {
            capacity,
            refill_req_monitor,
            refill_res_monitor,
            sem,
        }
    }

    // Attempst to refill container
    fn refill(
        &mut self,
        refill_req_monitor: Arc<(Mutex<ContainerMessage>, Condvar)>,
        refill_res_monitor: Arc<(Mutex<ContainerMessage>, Condvar)>,
    ) {
        // ask
        let req_resourse = ContainerMessage::new(CAPACITY, ContainerMessageType::ResourseRequest);
        let (req_lock, req_cvar) = &*refill_req_monitor;
        self.notify(req_lock, req_cvar, req_resourse);

        // wait
        let (res_lock, res_cvar) = &*refill_res_monitor;
        if let Ok(message) = self.wait(res_lock, res_cvar) {
            if message.get_amount() == FINISH_FLAG {
                println!("[coffee container] - out of coffe");
                self.capacity = FINISH_FLAG
            } else {
                println!("[coffee container] - refilling ");
                self.capacity += message.get_amount();
                println!("[coffee container] - refill complete");
            }
        }
    }

    fn consume(
        &mut self,
        refill_req_monitor: Arc<(Mutex<ContainerMessage>, Condvar)>,
        refill_res_monitor: Arc<(Mutex<ContainerMessage>, Condvar)>,
        amount: i32,
        sem: Arc<Semaphore>,
    ) -> Result<i32, String> {
        if self.capacity == NO_MORE {
            sem.acquire();
            self.refill(refill_req_monitor.clone(), refill_res_monitor.clone());
            return self.consume(refill_req_monitor, refill_res_monitor, amount, sem);
        }

        if self.capacity >= amount && amount.is_positive() {
            self.capacity -= amount;
            return Ok(amount);
        }

        if self.capacity == FINISH_FLAG {
            return Ok(NO_MORE);
        }

        Err("[error] - could not consume".to_string())
    }

    // Waits for dispenser to send new coffee request
    fn wait(
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
        Err("[error] - milk container  monitor failed".to_string())
    }

    // Notify container o dispenser about new resourse avaliable
    fn notify(&mut self, lock: &Mutex<ContainerMessage>, cvar: &Condvar, res: ContainerMessage) {
        if let Ok(mut resourse) = lock.lock() {
            *resourse = res;
            resourse.ready_to_read();
            cvar.notify_all();
        }
    }

    fn notify_end_message(&mut self, refill_req_monitor: Arc<(Mutex<ContainerMessage>, Condvar)>) {
        // send
        let req_resourse = ContainerMessage::new(FINISH_FLAG, ContainerMessageType::KillRequest);
        let (req_lock, req_cvar) = &*refill_req_monitor;
        self.notify(req_lock, req_cvar, req_resourse);
    }

    fn save_status(&self, d_mutex: Arc<Mutex<HashMap<Ingredients, i32>>>) {
        if let Ok(mut guard) = d_mutex.lock() {
            guard.insert(Ingredients::Coffee, self.capacity);
        }
    }
}

impl Container for CoffeContainer {
    fn start(
        &mut self,
        dispenser_req_monitor: Arc<(Mutex<ContainerMessage>, Condvar)>,
        dispenser_res_monitor: Arc<(Mutex<ContainerMessage>, Condvar)>,
        bussy_sem: Arc<Semaphore>,
        d_mutex: Arc<Mutex<HashMap<Ingredients, i32>>>,
    ) {
        loop {
            let (lock, cvar) = &*dispenser_req_monitor;
            println!("[coffee container] - waiting for request");
            if let Ok(res) = self.wait(lock, cvar) {
                let container_message_response: ContainerMessage;
                match res.get_type() {
                    ContainerMessageType::ResourseRequest => {
                        println!(
                            "[coffee container] - attempting to consume amount {}",
                            res.get_amount()
                        );
                        if let Ok(amounte_consumed) = self.consume(
                            self.refill_req_monitor.clone(),
                            self.refill_res_monitor.clone(),
                            res.get_amount(),
                            self.sem.clone(),
                        ) {
                            container_message_response = ContainerMessage::new(
                                amounte_consumed,
                                ContainerMessageType::ResourseRequest,
                            )
                        } else {
                            // consume fails --> kill the thread
                            container_message_response = ContainerMessage::new(
                                FINISH_FLAG,
                                ContainerMessageType::KillRequest,
                            )
                        }
                    }
                    ContainerMessageType::KillRequest => {
                        println!("[coffee container] - dispenser sending FINISHING FLAG",);
                        self.notify_end_message(self.refill_req_monitor.clone());
                        container_message_response =
                            ContainerMessage::new(FINISH_FLAG, ContainerMessageType::KillRequest)
                    }
                }
                let (res_lock, res_cvar) = &*dispenser_res_monitor;
                self.notify(res_lock, res_cvar, container_message_response);

                if matches!(res.get_type(), ContainerMessageType::KillRequest) {
                    println!("[milk container] - finishing ");
                    break;
                }
                self.save_status(d_mutex.clone());
                bussy_sem.release();
                println!("[milk container] - released sem");
            }
        }
    }
}

#[cfg(test)]
mod coffecontainer_test {
    use std::sync::{Arc, Condvar, Mutex};

    use std_semaphore::Semaphore;

    use crate::containers::coffee_container::{CoffeContainer, CAPACITY};
    use crate::helpers::container_message::{ContainerMessage, ContainerMessageType};

    #[test]
    fn it_should_init_with_0() {
        let refill_req_monitor = Arc::new((
            Mutex::new(ContainerMessage::new(
                100,
                ContainerMessageType::ResourseRequest,
            )),
            Condvar::new(),
        ));
        let refill_res_monitor = Arc::new((
            Mutex::new(ContainerMessage::new(
                100,
                ContainerMessageType::ResourseRequest,
            )),
            Condvar::new(),
        ));
        let sem = Arc::new(Semaphore::new(1));
        let coffee_container = CoffeContainer::new(refill_req_monitor, refill_res_monitor, sem);
        assert_eq!(coffee_container.capacity, 0)
    }

    #[test]
    fn it_should_refill_with_0_capacity() {
        let refill_req_monitor = Arc::new((
            Mutex::new(ContainerMessage::new(
                100,
                ContainerMessageType::ResourseRequest,
            )),
            Condvar::new(),
        ));
        let refill_res_monitor = Arc::new((
            Mutex::new(ContainerMessage::new(
                100,
                ContainerMessageType::ResourseRequest,
            )),
            Condvar::new(),
        ));
        let sem = Arc::new(Semaphore::new(1));
        let mut coffee_container =
            CoffeContainer::new(refill_req_monitor.clone(), refill_res_monitor.clone(), sem);
        coffee_container.capacity = 0;

        let (res_lock, res_cvar) = &*refill_res_monitor;
        if let Ok(mut resourse) = res_lock.lock() {
            resourse.ready_to_read();
            res_cvar.notify_all();
        }
        coffee_container.refill(refill_req_monitor, refill_res_monitor.clone());

        assert_eq!(coffee_container.capacity, CAPACITY)
    }

    #[test]
    fn it_should_has_value_with_valid_amount() {
        let refill_req_monitor = Arc::new((
            Mutex::new(ContainerMessage::new(
                100,
                ContainerMessageType::ResourseRequest,
            )),
            Condvar::new(),
        ));
        let refill_res_monitor = Arc::new((
            Mutex::new(ContainerMessage::new(
                100,
                ContainerMessageType::ResourseRequest,
            )),
            Condvar::new(),
        ));
        let sem = Arc::new(Semaphore::new(1));
        let mut coffee_container = CoffeContainer::new(refill_req_monitor, refill_res_monitor, sem);
        let mut res = ContainerMessage::new(10, ContainerMessageType::ResourseRequest);
        res.ready_to_read();
        let monitor = Arc::new((Mutex::new(res), Condvar::new()));
        let (lock, cvar) = &*monitor;

        match coffee_container.wait(lock, cvar) {
            Ok(r) => assert_eq!(r.get_amount(), 10),
            Err(_) => assert!(false),
        }
    }
}

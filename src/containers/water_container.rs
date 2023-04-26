use std::{
    sync::{Condvar, Mutex},
    thread,
    time::Duration,
};

use crate::helpers::container_message::{ContainerMessage, ContainerMessageType};

use super::container::Container;

const CAPACITY: i32 = 100;
const FINISH_FLAG: i32 = -1;
const NO_MORE: i32 = 0;

pub struct WaterContainer {
    capacity: i32,
}

#[allow(clippy::new_without_default)]
impl WaterContainer {
    pub fn new() -> Self {
        let capacity = 0;
        Self { capacity }
    }

    fn refill(&mut self) {
        println!("[water container] - waiting for more hot water");
        thread::sleep(Duration::from_millis(CAPACITY as u64));
        self.capacity = CAPACITY;
    }

    fn consume(&mut self, amount: i32) -> Result<i32, String> {
        if (self.capacity < amount) && (self.capacity > NO_MORE) {
            Ok(NO_MORE)
        } else if self.capacity == NO_MORE {
            println!("[water container] - refilling water ");
            self.refill();
            return self.consume(amount);
        } else {
            self.capacity -= amount;
            Ok(amount)
        }
    }

    // Waits for dispenser to send new water request
    fn wait_dispenser(
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
        Err("[error] - water container  monitor failed".to_string())
    }

    // Notify dispenser about new resourse avaliable
    fn notify_dispenser(
        &mut self,
        lock: &Mutex<ContainerMessage>,
        cvar: &Condvar,
        res: ContainerMessage,
    ) {
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
        request_monitor: std::sync::Arc<(Mutex<ContainerMessage>, Condvar)>,
        response_monitor: std::sync::Arc<(Mutex<ContainerMessage>, Condvar)>,
        bussy_sem: std::sync::Arc<std_semaphore::Semaphore>,
    ) {
        loop {
            let (lock, cvar) = &*request_monitor;
            println!("[water container] - waiting for request");
            if let Ok(res) = self.wait_dispenser(lock, cvar) {
                let container_message_response: ContainerMessage;
                match res.get_type() {
                    ContainerMessageType::ResourseRequest => {
                        println!(
                            "[water container] - attempting to consume amount {}",
                            res.get_amount()
                        );
                        if let Ok(amounte_consumed) = self.consume(res.get_amount()) {
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
                    ContainerMessageType::DataRequest => {
                        container_message_response =
                            ContainerMessage::new(self.capacity, ContainerMessageType::DataRequest)
                    }
                    ContainerMessageType::KillRequest => {
                        println!("[water container] - dispenser sending FINISHING FLAG",);
                        container_message_response =
                            ContainerMessage::new(FINISH_FLAG, ContainerMessageType::KillRequest)
                    }
                }

                let (res_lock, res_cvar) = &*response_monitor;
                self.notify_dispenser(res_lock, res_cvar, container_message_response);
                if matches!(res.get_type(), ContainerMessageType::KillRequest) {
                    println!("[milk container] - finishing ");
                    break;
                }
                bussy_sem.release();
                println!("[water container] - released sem")
            }
        }
    }
}

#[cfg(test)]
mod water_container_test {
    use std::sync::{Arc, Condvar, Mutex};

    use crate::{
        containers::water_container::{WaterContainer, CAPACITY, FINISH_FLAG},
        helpers::container_message::{ContainerMessage, ContainerMessageType},
    };

    #[test]
    fn it_should_init_with_0() {
        let water_container = WaterContainer::new();
        assert_eq!(water_container.capacity, 0)
    }

    #[test]
    fn it_should_send_0_when_amount_is_bigger_than_capacity() {
        let mut water_container = WaterContainer::new();
        water_container.capacity = 9;
        let amount = 10;
        let res = water_container.consume(amount).unwrap();
        assert_eq!(res, 0)
    }

    #[test]
    fn it_should_refill_when_capacity_is_0() {
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

    #[test]
    fn it_should_wait_for_resourse_is_ready_and_return_message() {
        let mut milk_container: WaterContainer = WaterContainer::new();
        let mut resourse = ContainerMessage::new(10, ContainerMessageType::ResourseRequest);
        resourse.ready_to_read();

        let monitor = Arc::new((Mutex::new(resourse), Condvar::new()));
        let (lock, cvar) = &*monitor;

        let result: ContainerMessage = milk_container.wait_dispenser(lock, cvar).unwrap();

        assert_eq!(result.get_amount(), 10);
    }
    #[test]
    fn it_should_wait_for_data_request_is_ready_and_return_resourse() {
        let mut cacao_container = WaterContainer::new();
        let mut resourse = ContainerMessage::new(0, ContainerMessageType::DataRequest);
        resourse.ready_to_read();

        let monitor: Arc<(Mutex<ContainerMessage>, Condvar)> =
            Arc::new((Mutex::new(resourse), Condvar::new()));
        let (lock, cvar) = &*monitor;

        let result = cacao_container.wait_dispenser(lock, cvar).unwrap();

        assert_eq!(result.get_amount(), 0);
    }

    #[test]
    fn it_should_wait_for_kill_request_is_ready_and_return_resourse() {
        let mut cacao_container = WaterContainer::new();
        let mut resourse = ContainerMessage::new(FINISH_FLAG, ContainerMessageType::KillRequest);
        resourse.ready_to_read();

        let monitor: Arc<(Mutex<ContainerMessage>, Condvar)> =
            Arc::new((Mutex::new(resourse), Condvar::new()));
        let (lock, cvar) = &*monitor;

        let result = cacao_container.wait_dispenser(lock, cvar).unwrap();

        assert_eq!(result.get_amount(), FINISH_FLAG);
    }
}

use std::sync::{Condvar, Mutex};

use crate::helpers::container_message::{ContainerMessage, ContainerMessageType};

use super::container::Container;

const CAPACITY: i32 = 1500;
const FINISH_FLAG: i32 = -1;
const NO_MORE: i32 = 0;

pub struct MilkContainer {
    capacity: i32,
}

#[allow(clippy::new_without_default)]
impl MilkContainer {
    pub fn new() -> Self {
        let capacity = CAPACITY;
        Self { capacity }
    }

    fn consume(&mut self, amount: i32) -> Result<i32, String> {
        println!("[milk container] - attempting to consume amount {}", amount);
        if amount <= self.capacity {
            self.capacity -= amount;
            Ok(amount)
        } else {
            Ok(NO_MORE)
        }
    }

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
        Err("[error] - milk container  monitor failed".to_string())
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
                println!("[milk container] - notifying FINISHING FLAG ",);
            } else {
                println!("[milk container] - sending {} units", resourse.get_amount(),);
            }
            cvar.notify_all();
        }
    }

    fn check_capacity(&self) -> bool {
        let min_capacity = (CAPACITY as f32) * (0.2_f32);
        self.capacity as f32 <= min_capacity
    }
}

impl Container for MilkContainer {
    fn start(
        &mut self,
        request_monitor: std::sync::Arc<(
            std::sync::Mutex<crate::helpers::container_message::ContainerMessage>,
            std::sync::Condvar,
        )>,
        response_monitor: std::sync::Arc<(
            std::sync::Mutex<crate::helpers::container_message::ContainerMessage>,
            std::sync::Condvar,
        )>,
        bussy_sem: std::sync::Arc<std_semaphore::Semaphore>,
    ) {
        loop {
            let (lock, cvar) = &*request_monitor;
            println!("[milk container] - waiting for request");
            if let Ok(res) = self.wait_dispenser(lock, cvar) {
                let container_message_response: ContainerMessage;
                match res.get_type() {
                    ContainerMessageType::ResourseRequest => {
                        println!(
                            "[milk container] - attempting to consume amount {}",
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
                        println!("[milk container] - dispenser sending FINISHING FLAG",);
                        container_message_response =
                            ContainerMessage::new(FINISH_FLAG, ContainerMessageType::KillRequest)
                    }
                }
                let (res_lock, res_cvar) = &*response_monitor;
                self.notify_dispenser(res_lock, res_cvar, container_message_response);
                if self.check_capacity() {
                    println!("[milk container] - CAPACITY LOWER THAN 20% ")
                }
                if matches!(res.get_type(), ContainerMessageType::KillRequest) {
                    println!("[milk container] - finishing ");
                    break;
                }
                bussy_sem.release();
                println!("[milk container] - released sem");
            }
        }
    }
}

#[cfg(test)]
mod milk_container_test {
    use std::sync::{Arc, Condvar, Mutex};

    use crate::{
        containers::milk_container::{MilkContainer, CAPACITY, FINISH_FLAG, NO_MORE},
        helpers::container_message::{ContainerMessage, ContainerMessageType},
    };

    #[test]
    fn it_should_init_with_n() {
        let n: i32 = 1500;
        let cacao_container = MilkContainer::new();
        assert_eq!(cacao_container.capacity, n)
    }

    #[test]
    fn it_should_consume_when_amount_is_smaller_than_capacity() {
        let mut cacao_container = MilkContainer::new();
        let amount = 10;
        cacao_container.consume(amount).unwrap();
        assert_eq!(cacao_container.capacity, (CAPACITY - amount))
    }

    #[test]
    fn it_should_send_finish_flag_when_finish_flag() {
        let mut milk_container = MilkContainer::new();
        let amount = -1;
        let res = milk_container.consume(amount).unwrap();
        assert_eq!(res, FINISH_FLAG)
    }

    #[test]
    fn it_should_send_no_more_flag_when_no_capacity() {
        let mut milk_container = MilkContainer::new();
        let amount = 2200;
        let res = milk_container.consume(amount).unwrap();
        assert_eq!(res, NO_MORE)
    }

    #[test]
    fn it_should_wait_for_resourse_is_ready_and_return_message() {
        let mut milk_container: MilkContainer = MilkContainer::new();
        let mut resourse = ContainerMessage::new(10, ContainerMessageType::ResourseRequest);
        resourse.ready_to_read();

        let monitor = Arc::new((Mutex::new(resourse), Condvar::new()));
        let (lock, cvar) = &*monitor;

        let result: ContainerMessage = milk_container.wait_dispenser(lock, cvar).unwrap();

        assert_eq!(result.get_amount(), 10);
    }
    #[test]
    fn it_should_wait_for_data_request_is_ready_and_return_resourse() {
        let mut cacao_container = MilkContainer::new();
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
        let mut cacao_container = MilkContainer::new();
        let mut resourse = ContainerMessage::new(FINISH_FLAG, ContainerMessageType::KillRequest);
        resourse.ready_to_read();

        let monitor: Arc<(Mutex<ContainerMessage>, Condvar)> =
            Arc::new((Mutex::new(resourse), Condvar::new()));
        let (lock, cvar) = &*monitor;

        let result = cacao_container.wait_dispenser(lock, cvar).unwrap();

        assert_eq!(result.get_amount(), FINISH_FLAG);
    }

    #[test]
    fn it_should_notify_for_resourse_is_ready() {
        let mut milk_container = MilkContainer::new();
        let resourse_req = ContainerMessage::new(0, ContainerMessageType::ResourseRequest);
        let resourse_res = ContainerMessage::new(10, ContainerMessageType::ResourseRequest);
        let monitor = Arc::new((Mutex::new(resourse_req), Condvar::new()));
        let (lock, cvar) = &*monitor;

        milk_container.notify_dispenser(lock, cvar, resourse_res);

        if let Ok(g) = cvar.wait_while(lock.lock().unwrap(), |s| s.is_not_ready()) {
            assert_eq!(g.get_amount(), 10);
        };
    }

    #[test]
    fn it_should_return_true_when_capacity_is_lower_than_20_percent() {
        let mut coffee_grain_container = MilkContainer::new();
        /* 1500 is max capacity, 300 is 20% */
        coffee_grain_container.capacity = 300;
        assert!(coffee_grain_container.check_capacity())
    }
}

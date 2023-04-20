use std::sync::{Mutex, Condvar};

use super::{container::Container, resourse::Resourse};

const N: i32 = 1000;
const FINISH_FLAG: i32 = -1;

pub struct CacaoContainer {
    capacity: i32,
}

impl CacaoContainer {
    pub fn new() -> Self {
        let capacity = N;
        Self { capacity }
    }

    fn consume(&mut self, amount: i32) -> Result<i32, String> {
        if (amount.is_positive()) && (amount <= self.capacity) {
            self.capacity -= amount;
            return Ok(amount);
        } else {
            return Ok(FINISH_FLAG);
        }
    }

    fn wait_dispenser(&mut self, lock: &Mutex<Resourse>, cvar: &Condvar) -> Result<i32, String> {
        if let Ok(guard) = lock.lock() {
            if let Ok(mut resourse) = cvar.wait_while(guard, |status| status.is_not_ready()) {
                let cacao_consumed = resourse.get_amount();

                if cacao_consumed == FINISH_FLAG {
                    println!("[cacao container] - dispenser sending FINISHING FLAG",);
                } else {
                    println!(
                        "[cacao container] - dispenser asking for {} units of cacao",
                        cacao_consumed
                    );
                }
                resourse.read();
                return Ok(cacao_consumed);
            }
        };
        Err("[error] - cacao container  monitor failed".to_string())
    }

    // Notify dispenser about new resourse avaliable
    fn notify_dispenser(&mut self, lock: &Mutex<Resourse>, cvar: &Condvar, res: Resourse) {
        if let Ok(mut resourse) = lock.lock() {
            *resourse = res;
            resourse.ready_to_read();
            if resourse.get_amount() == FINISH_FLAG {
                println!("[cacao container] - notifying dispenser FINISHING FLAG ",);
            } else {
                println!(
                    "[cacao container] - sending {} units to dispenser",
                    resourse.get_amount(),
                );
            }
            cvar.notify_all();
        }
    }

}

impl Container for CacaoContainer {
    fn start(
        &mut self,
        request_monitor: std::sync::Arc<(std::sync::Mutex<super::resourse::Resourse>, std::sync::Condvar)>,
        response_monitor: std::sync::Arc<(std::sync::Mutex<super::resourse::Resourse>, std::sync::Condvar)>,
        bussy_sem: std::sync::Arc<std_semaphore::Semaphore>,
    )
    {
        loop {
        let (lock, cvar) = &*request_monitor;
        println!("[cacao container] - waiting for request");
        if let Ok(res) = self.wait_dispenser(lock, cvar) {
            println!("[cacao container] - attempting to consume amount {}", res);

            if let Ok(amounte_consumed) = self.consume(res) {
                let (res_lock, res_cvar) = &*response_monitor;
                self.notify_dispenser(res_lock, res_cvar, Resourse::new(amounte_consumed));
                if res == FINISH_FLAG {
                    println!("[cacao container] - finishing ");
                    break;
                }
                bussy_sem.release();
                println!("[cacao container] - released sem")
            }
        }
    }
}}


#[cfg(test)]
mod cacao_container_test {
    use std::sync::{Mutex, Arc, Condvar};

    use crate::containers::{cacao_container::CacaoContainer, resourse::Resourse};

    #[test]
    fn it_should_init_with_n() {
        let n: i32 = 1000;
        let cacao_container = CacaoContainer::new();
        assert_eq!(cacao_container.capacity, n)
    }

    #[test]
    fn it_should_consume_when_amount_is_smaller_than_capacity() {
        let mut cacao_container = CacaoContainer::new();
        let amount = 10;
        cacao_container.consume(amount).unwrap();
        assert_eq!(cacao_container.capacity, 990)
    }

    #[test]
    fn it_should_return_finish_flag_when_amount_negative() {
        let mut cacao_container = CacaoContainer::new();
        let finish_flag = -1;
        let amount = -1;
        let res = cacao_container.consume(amount).unwrap();
        assert_eq!(res, finish_flag)
    }

    #[test]
    fn it_should_wait_for_resourse_is_ready() {
        let mut cacao_container = CacaoContainer::new();
        let mut resourse = Resourse::new(10);
        resourse.ready_to_read();

        let monitor = Arc::new((Mutex::new(resourse), Condvar::new()));
        let (lock, cvar) = &*monitor;
        
        let result =  cacao_container.wait_dispenser(lock, cvar).unwrap();

        assert_eq!(result,10);
    }

    #[test]
    fn it_should_notify_for_resourse_is_ready() {
        let mut cacao_container = CacaoContainer::new();
        let resourse_req = Resourse::new(0);
        let resourse_res = Resourse::new(10);
        let monitor = Arc::new((Mutex::new(resourse_req), Condvar::new()));
        let (lock, cvar) = &*monitor;
        
        cacao_container.notify_dispenser(lock, cvar,resourse_res);

        if let Ok(g) = cvar.wait_while(lock.lock().unwrap(), |s| s.is_not_ready()){
            assert_eq!(g.get_amount(),10);
        };
    }


}

use std::{sync::{Arc, Mutex, Condvar}, time::Duration, thread};

use super::{order_manager::OrderManager, order};



pub struct StatsPresenter {
    time: u64
}

impl StatsPresenter {

    pub fn new(time: u64) -> Self {
        Self {
            time
        }
    }

    pub fn start(&self,order_monitor: Arc<(Mutex<OrderManager>, Condvar)>){
        loop {
            thread::sleep(Duration::from_secs(self.time));
            let (order_lock, cvar) = &*order_monitor;
            if let Ok(order_manager) = order_lock.lock(){
                let orders_made = order_manager.orders_made();
                let orders_unmade = order_manager.orders_in_qeue();
                
                println!("\n \t---------------- Stats -------------");
                println!("\tCOFFE ORDERS MADE:    {}", orders_made);
                println!("\tCOFFE ORDERS IN QEUE: {}", orders_unmade);
                println!("\t------------------------------------\n");
                

                
                if order_manager.no_more_orders() {
                    break;
                }
            }
        }
        
    }
}
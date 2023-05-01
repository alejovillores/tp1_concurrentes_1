use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
};

use super::{ingredients::Ingredients, order_manager::OrderManager};

const INGREDIENTS: [Ingredients; 6] = [
    Ingredients::CoffeGrain,
    Ingredients::Coffee,
    Ingredients::Milk,
    Ingredients::Water,
    Ingredients::Foam,
    Ingredients::Cacao,
];

pub struct StatsPresenter {
    time: u64,
}

impl StatsPresenter {
    pub fn new(time: u64) -> Self {
        Self { time }
    }

    fn present_machine_stats(&self, orders_made: i32, orders_unmade: usize) {
        println!("\n \t---------------- Machine Stats -------------");
        println!("\tCOFFE ORDERS MADE:    {}", orders_made);
        println!("\tCOFFE ORDERS IN QEUE: {}", orders_unmade);
        println!("\n \t---------------- Containers Stats -------------");
    }

    fn present_ingredient(&self, ingredient: Ingredients, amount: &i32) {
        println!("\t {:?} container units: {}", ingredient, amount);
    }

    pub fn start(
        &self,
        order_monitor: Arc<(Mutex<OrderManager>, Condvar)>,
        container_data: Arc<Mutex<HashMap<Ingredients, i32>>>,
    ) {
        loop {
            thread::sleep(Duration::from_secs(self.time));
            let (order_lock, _cvar) = &*order_monitor;
            if let Ok(order_manager) = order_lock.lock() {
                let orders_made = order_manager.orders_made();
                let orders_unmade = order_manager.orders_in_qeue();

                if let Ok(container_data) = container_data.lock() {
                    self.present_machine_stats(orders_made, orders_unmade);

                    for i in INGREDIENTS.iter().copied() {
                        let amount = container_data.get(&i).expect("NO COFFE DATA");
                        self.present_ingredient(i, amount)
                    }
                    println!("\t------------------------------------\n");
                }

                if order_manager.no_more_orders() {
                    break;
                }
            }
        }
    }
}

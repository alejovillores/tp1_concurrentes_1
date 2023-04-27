use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
};

use super::{ingredients::Ingredients, order_manager::OrderManager};

pub struct StatsPresenter {
    time: u64,
}

impl StatsPresenter {
    pub fn new(time: u64) -> Self {
        Self { time }
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
                    println!("\n \t---------------- Machine Stats -------------");
                    println!("\tCOFFE ORDERS MADE:    {}", orders_made);
                    println!("\tCOFFE ORDERS IN QEUE: {}", orders_unmade);
                    println!("\n \t---------------- Containers Stats -------------");
                    println!(
                        "\tCOFFE CONTAINER UNITS: {}",
                        container_data
                            .get(&Ingredients::Coffee)
                            .expect("NO COFFE DATA")
                    );
                    println!(
                        "\tCOFFE CONTAINER GRAIN UNITS: {}",
                        container_data
                            .get(&Ingredients::CoffeGrain)
                            .expect("NO COFFE GRAIN DATA")
                    );
                    println!(
                        "\tMILK CONTAINER UNITS: {}",
                        container_data
                            .get(&Ingredients::Milk)
                            .expect("NO MILK DATA")
                    );
                    println!(
                        "\tCACAO CONTAINER UNITS: {}",
                        container_data
                            .get(&Ingredients::Cacao)
                            .expect("NO CACAO DATA")
                    );
                    println!(
                        "\tFOAM CONTAINER UNITS: {}",
                        container_data
                            .get(&Ingredients::Foam)
                            .expect("NO FOAM DATA")
                    );
                    println!(
                        "\tWATER CONTAINER UNITS: {}",
                        container_data
                            .get(&Ingredients::Water)
                            .expect("NO WATER DATA")
                    );
                    println!("\t------------------------------------\n");
                }

                if order_manager.no_more_orders() {
                    break;
                }
            }
        }
    }
}

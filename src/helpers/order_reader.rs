use std::collections::VecDeque;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use super::order::Order;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
struct OrderJSON {
    coffee_amount: i32,
    water_amount: i32,
    cacao_amount: i32,
    milk_amount: i32,
    foam_amount: i32,
}

impl OrderJSON {
    pub fn to_order(&self) -> Order {
        Order::new(
            self.coffee_amount,
            self.water_amount,
            self.cacao_amount,
            self.milk_amount,
            self.foam_amount,
        )
    }
}

pub struct OrderReader {
    path: String,
    orders: VecDeque<OrderJSON>,
}

impl OrderReader {
    pub fn new(path: String) -> Self {
        let orders: VecDeque<OrderJSON> = VecDeque::new();
        Self { path, orders }
    }

    #[allow(unused_must_use)]
    fn read_file(&self) -> Result<String, String> {
        match File::open(self.path.clone()) {
            Ok(f) => {
                let mut buf_reader = BufReader::new(f);
                let mut contents = String::new();

                match buf_reader.read_to_string(&mut contents) {
                    Ok(_u) => Ok(contents),
                    Err(_) => Err("could not read file".to_string()),
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    fn make_orders(&self, orders: String) -> Result<VecDeque<OrderJSON>, String> {
        match serde_json::from_str::<VecDeque<OrderJSON>>(&orders) {
            Ok(r) => Ok(r),
            Err(_) => Err("could not convert json to object".to_string()),
        }
    }

    #[allow(unused_must_use)]
    pub fn read_json(&mut self) -> Result<(), String> {
        match self.read_file() {
            Ok(s) => {
                if let Ok(o) = self.make_orders(s) {
                    self.orders = o;
                    Ok(())
                } else {
                    Err("could not make orders".to_string())
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn get_order(&mut self) -> Option<Order> {
        match self.orders.pop_front() {
            Some(o) => {
                let mut order = o.to_order();
                if self.orders.is_empty() {
                    order.last();
                }
                Some(order)
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod order_reader_test {

    use super::OrderReader;

    #[test]
    fn it_should_read_order_with_2_cacao_amount() {
        let o_reader = OrderReader::new("res/orders.test1.json".to_owned());

        let res = o_reader.read_file().unwrap();
        let result = o_reader.make_orders(res).unwrap();

        assert_eq!(result.get(0).unwrap().cacao_amount, 2)
    }

    #[test]
    fn it_should_read_order_with_5_coffee_amount() {
        let o_reader = OrderReader::new("res/orders.test1.json".to_owned());

        let res = o_reader.read_file().unwrap();
        let result = o_reader.make_orders(res).unwrap();

        assert_eq!(result.get(0).unwrap().coffee_amount, 5)
    }

    #[test]
    fn it_should_read_order_with_8_water_amount() {
        let o_reader = OrderReader::new("res/orders.test1.json".to_owned());

        let res = o_reader.read_file().unwrap();
        let result = o_reader.make_orders(res).unwrap();

        assert_eq!(result.get(0).unwrap().water_amount, 8)
    }

    #[test]
    fn it_should_read_order_with_3_milk_amount() {
        let o_reader = OrderReader::new("res/orders.test1.json".to_owned());

        let res = o_reader.read_file().unwrap();
        let result = o_reader.make_orders(res).unwrap();

        assert_eq!(result.get(0).unwrap().milk_amount, 3)
    }

    #[test]
    fn it_should_read_order_with_1_foam_amount() {
        let o_reader = OrderReader::new("res/orders.test1.json".to_owned());

        let res = o_reader.read_file().unwrap();
        let result = o_reader.make_orders(res).unwrap();

        assert_eq!(result.get(0).unwrap().foam_amount, 1)
    }

    #[test]
    fn it_should_read_4_orders() {
        let o_reader = OrderReader::new("res/orders.test2.json".to_owned());

        let res = o_reader.read_file().unwrap();
        let result = o_reader.make_orders(res).unwrap();

        assert_eq!(result.len(), 4)
    }

    #[test]
    fn it_should_init_orders() {
        let mut o_reader = OrderReader::new("res/orders.test2.json".to_owned());
        o_reader.read_json().unwrap();

        assert_eq!(o_reader.orders.len(), 4)
    }

    #[test]
    fn it_should_map_order_json_to_order() {
        let mut o_reader = OrderReader::new("res/orders.test2.json".to_owned());
        o_reader.read_json().unwrap();
        let order = o_reader.get_order().unwrap();

        assert!(order.is_not_ready())
    }

    #[test]
    fn it_should_not_have_last_order_first() {
        let mut o_reader = OrderReader::new("res/orders.test2.json".to_owned());
        o_reader.read_json().unwrap();
        let order = o_reader.get_order().unwrap();

        assert!(!order.is_last())
    }

    #[test]
    fn it_should_have_last_order() {
        let mut o_reader = OrderReader::new("res/orders.test1.json".to_owned());
        o_reader.read_json().unwrap();
        let order = o_reader.get_order().unwrap();
        assert!(order.is_last())
    }
}

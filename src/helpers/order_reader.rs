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
}

impl OrderJSON {
    pub fn new(coffee_amount: i32, water_amount: i32, cacao_amount: i32) -> Self {
        Self {
            coffee_amount,
            water_amount,
            cacao_amount,
        }
    }
}

pub struct OrderReader {
    path: String,
}

impl OrderReader {
    pub fn new(path: String) -> Self {
        Self { path }
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

    fn make_orders(&self, orders: String) -> Result<Vec<OrderJSON>, String> {
        match serde_json::from_str::<Vec<OrderJSON>>(&orders) {
            Ok(r) => Ok(r),
            Err(_) => Err("could not convert json to object".to_string()),
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
    fn it_should_read_order_with_10_coffee_amount() {
        let o_reader = OrderReader::new("res/orders.test1.json".to_owned());

        let res = o_reader.read_file().unwrap();
        let result = o_reader.make_orders(res).unwrap();

        assert_eq!(result.get(0).unwrap().coffee_amount, 10)
    }

    #[test]
    fn it_should_read_order_with_20_water_amount() {
        let o_reader = OrderReader::new("res/orders.test1.json".to_owned());

        let res = o_reader.read_file().unwrap();
        let result = o_reader.make_orders(res).unwrap();

        assert_eq!(result.get(0).unwrap().water_amount, 20)
    }
}

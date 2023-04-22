use super::order::Order;
use std::collections::VecDeque;

#[derive(Debug)]
enum StatusFlag {
    LastOrder,
    NoMoreOrders,
    NotEmpty,
    Empty,
}

pub struct OrderManager {
    status: StatusFlag,
    orders: VecDeque<Order>,
    orders_extracted: i32,
}

impl OrderManager {
    pub fn new() -> Self {
        let status = StatusFlag::Empty;
        let orders: VecDeque<Order> = VecDeque::new();
        let orders_extracted = 0;

        Self {
            status,
            orders,
            orders_extracted,
        }
    }

    pub fn add(&mut self, ticket: Order) {
        if ticket.is_last() {
            println!("[order manager] - last packet in qeue");
            self.status = StatusFlag::LastOrder;
            println!("[order manager] - status: {:?}", self.status);
        } else {
            println!("[order manager] - new packet in qeue");
            self.status = StatusFlag::NotEmpty;
        }
        self.orders.push_back(ticket)
    }

    pub fn extract(&mut self) -> Option<Order> {
        match self.orders.pop_front() {
            Some(t) => {
                match self.status {
                    StatusFlag::LastOrder => {
                        self.status = StatusFlag::NoMoreOrders;
                    }
                    _ => {
                        if self.orders.is_empty() {
                            self.status = StatusFlag::Empty;
                        } else {
                            self.status = StatusFlag::NotEmpty;
                        }
                    }
                }

                self.orders_extracted += 1;
                Some(t)
            }
            None => {
                match self.status {
                    StatusFlag::NoMoreOrders => {}
                    _ => self.status = StatusFlag::Empty,
                }
                None
            }
        }
    }

    pub fn empty(&self) -> bool {
        matches!(self.status, StatusFlag::Empty)
    }

    pub fn no_more_orders(&self) -> bool {
        println!("[order manager] - status: {:?}", self.status);
        matches!(self.status, StatusFlag::NoMoreOrders)
    }

    pub fn orders_in_qeue(&self) -> usize {
        self.orders.len()
    }

    pub fn orders_made(&self) -> i32 {
        self.orders_extracted
    }
}

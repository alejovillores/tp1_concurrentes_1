use std::collections::VecDeque;
use super::order::Order;

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
        if ticket.last() {
            self.status = StatusFlag::LastOrder;
        } else {
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
                        if self.orders.is_empty(){
                            self.status = StatusFlag::Empty;
                        }
                        else {
                            self.status = StatusFlag::NotEmpty;
                            
                        }
                    }
                }

                self.orders_extracted += 1;
                return Some(t);
            }
            None => {
                self.status = StatusFlag::Empty;
                return None;
            }
        }
    }

    pub fn empty(&self) -> bool {
        match self.status {
            StatusFlag::Empty => return true,
            _ => return false,
        }
    }

    pub fn no_more_orders(&self) ->bool {
        match self.status {
            StatusFlag::NoMoreOrders => return true,
            _ => return false,
        }
    }

    pub fn orders_in_qeue(&self) -> usize{
        self.orders.len()
    }

    pub fn orders_made(&self) -> i32{
        self.orders_extracted
    }

}

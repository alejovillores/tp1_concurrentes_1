use std::collections::VecDeque;

use super::ticket::Ticket;

enum StatusFlag {
    LastOrder,
    NoMoreOrders,
    NotEmpty,
    Empty,
}

pub struct OrderManager {
    status: StatusFlag,
    orders: VecDeque<Ticket>,
}

impl OrderManager {
    pub fn new() -> Self {
        let status = StatusFlag::Empty;
        let orders: VecDeque<Ticket> = VecDeque::new();
        Self { status, orders }
    }

    pub fn add(&mut self, ticket: Ticket) {
        if ticket.last() {
            self.status = StatusFlag::LastOrder;
        } else {
            self.status = StatusFlag::NotEmpty;
        }
        self.orders.push_back(ticket)
    }

    pub fn extract(&mut self) -> Option<Ticket> {
        match self.orders.pop_front() {
            Some(t) => {
                match self.status {
                    StatusFlag::LastOrder => {
                        self.status = StatusFlag::NoMoreOrders;
                    }
                    _ => {}
                }
                return Some(t);
            }
            None => {
                self.status = StatusFlag::NoMoreOrders;
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
}

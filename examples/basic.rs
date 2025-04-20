#![no_std]
#![no_main]

extern crate alloc;

use alloc::sync::Arc;
use core::fmt::Display;

use autons::prelude::*;
use autons_controller::prelude::*;
use vexide::{prelude::*, sync::Mutex};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Category {
    Category1,
    Category2,
    Category3,
}
impl Display for Category {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Category::Category1 => write!(f, "Category 1"),
            Category::Category2 => write!(f, "Category 2"),
            Category::Category3 => write!(f, "Category 3"),
        }
    }
}

struct Robot {}

impl Robot {
    async fn route_1(&mut self) {
        println!("Route 1 selected");
    }
    async fn route_2(&mut self) {
        println!("Route 2 selected");
    }
    async fn route_3(&mut self) {
        println!("Route 3 selected");
    }
    async fn route_4(&mut self) {
        println!("Route 4 selected");
    }
    async fn route_5(&mut self) {
        println!("Route 5 selected");
    }
    async fn route_6(&mut self) {
        println!("Route 6 selected");
    }
}

impl SelectCompete for Robot {}

#[vexide::main]
async fn main(peripherals: Peripherals) {
    let robot = Robot {};

    robot
        .compete(ControllerSelect::new(
            Arc::new(Mutex::new(peripherals.primary_controller)),
            [
                route!(Category::Category1, "Route 1", Robot::route_1),
                route!(Category::Category2, "Route 2", Robot::route_2),
                route!(Category::Category3, "Route 3", Robot::route_3),
                route!(Category::Category1, "Route 4", Robot::route_4),
                route!(Category::Category1, "Route 5", Robot::route_5),
                route!(Category::Category1, "Route 6", Robot::route_6),
            ],
        ))
        .await;
}

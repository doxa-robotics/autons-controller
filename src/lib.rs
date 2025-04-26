//! Simple controller-based autonomous route selector.

#![no_std]
#![feature(never_type)]

extern crate alloc;

use alloc::{boxed::Box, format, rc::Rc, vec, vec::Vec};
use core::{cell::RefCell, fmt::Display, time::Duration};

use autons::Selector;
use display::{horizontal_picker, simple_dialog};
use vexide::{
    devices::controller::*,
    task::{self, Task},
    time::sleep,
};

use crate::alloc::string::ToString;

mod display;
mod route;

pub use route::*;

struct SelectorState<R: 'static, C: Clone + Display + PartialEq, const N: usize> {
    routes: [Route<R, C>; N],
    selection: Option<usize>,
}

enum SelectorStateType<R: 'static, C: Clone + Display + PartialEq> {
    Category,
    Route(C),
    Confirm(Route<R, C>),
    Done(Route<R, C>),
}

pub struct ControllerSelect<
    R: 'static,
    C: Clone + Display + PartialEq + PartialOrd + 'static,
    const N: usize,
> {
    state: Rc<RefCell<SelectorState<R, C, N>>>,
    _task: Task<()>,
}

impl<R, C: Clone + Display + PartialEq + Ord + 'static, const N: usize> ControllerSelect<R, C, N> {
    /// Creates a new selector from a [`Display`] peripheral and array of routes.
    pub fn new(
        controller: Rc<RefCell<Controller>>,
        is_selecting: Rc<RefCell<bool>>,
        routes: [Route<R, C>; N],
    ) -> Self {
        const {
            assert!(N > 0, "ControllerSelect requires at least one route.");
        }

        let mut categories = routes
            .iter()
            .map(|route| route.category.clone())
            .collect::<Vec<_>>();
        categories.sort();
        categories.dedup();

        let state = Rc::new(RefCell::new(SelectorState {
            routes,
            selection: None,
        }));

        Self {
            state: state.clone(),
            _task: task::spawn(async move {
                let mut state_type = SelectorStateType::Category;
                loop {
                    // If we're connected to a comp control system, we should
                    // exit since picker should happen before we plug in
                    if vexide::competition::is_connected() || !*is_selecting.borrow() {
                        is_selecting.replace(false);
                        sleep(Duration::from_millis(400)).await;
                        _ = controller.borrow_mut().screen.try_clear_screen();
                        break;
                    }
                    match state_type {
                        SelectorStateType::Category => {
                            match horizontal_picker(
                                controller.clone(),
                                is_selecting.clone(),
                                "- Pick category -",
                                categories
                                    .clone()
                                    .into_iter()
                                    .map(|c| c.to_string())
                                    .collect::<Vec<_>>(),
                            )
                            .await
                            {
                                Some(selected) => {
                                    state_type =
                                        SelectorStateType::Route(categories[selected].clone());
                                }
                                None => state_type = SelectorStateType::Category,
                            }
                        }
                        SelectorStateType::Route(category) => {
                            let route_picker_header = format!("- {} -", category);
                            let routes = state.borrow().routes.clone();
                            let filtered_routes = routes
                                .clone()
                                .into_iter()
                                .filter(|route| route.category == category)
                                .collect::<Vec<_>>();
                            match horizontal_picker(
                                controller.clone(),
                                is_selecting.clone(),
                                &route_picker_header,
                                filtered_routes
                                    .iter()
                                    .map(|route| route.name.to_string())
                                    .collect::<Vec<_>>(),
                            )
                            .await
                            {
                                Some(selected) => {
                                    let selected = &filtered_routes[selected];
                                    state_type = SelectorStateType::Confirm(selected.clone());
                                }
                                None => state_type = SelectorStateType::Category,
                            }
                        }
                        SelectorStateType::Confirm(route) => {
                            let header = format!("Pick in {}:", route.category);
                            match horizontal_picker(
                                controller.clone(),
                                is_selecting.clone(),
                                &header,
                                vec!["Cancel".to_string(), format!("Confirm {}", route.name)],
                            )
                            .await
                            {
                                Some(selected) => {
                                    if selected == 0 {
                                        state_type = SelectorStateType::Route(route.category);
                                    } else {
                                        let mut state = state.borrow_mut();
                                        state.selection = Some(
                                            state
                                                .routes
                                                .iter()
                                                .position(|r| {
                                                    r.name == route.name
                                                        && r.category == route.category
                                                })
                                                .unwrap(),
                                        );
                                        state_type = SelectorStateType::Done(route.clone());
                                    }
                                }
                                None => state_type = SelectorStateType::Route(route.category),
                            }
                        }
                        SelectorStateType::Done(ref route) => {
                            let description = format!("{} / {}", route.category, route.name);
                            is_selecting.replace(true);
                            simple_dialog(
                                controller.clone(),
                                is_selecting.clone(),
                                "- Route selected -",
                                &description,
                                Some("Good luck!"),
                            )
                            .await;
                        }
                    }
                }
            }),
        }
    }

    /// Programmatically selects an autonomous route by index.
    pub fn select(&mut self, index: usize) {
        assert!(index < N, "Invalid route selection index.");
        let mut state = self.state.borrow_mut();
        state.selection = Some(index);
    }
}

impl<R, C: Clone + Display + PartialEq + PartialOrd + 'static, const N: usize> Selector<R>
    for ControllerSelect<R, C, N>
{
    async fn run(&self, robot: &mut R) {
        {
            let state = self.state.borrow();
            if let Some(selection) = state.selection {
                (state.routes[selection].callback)(robot)
            } else {
                Box::pin(async {})
            }
        }
        .await;
    }
}

/// This module is intended to be glob imported.
pub mod prelude {
    pub use super::{
        route::{route, Route},
        ControllerSelect,
    };
}

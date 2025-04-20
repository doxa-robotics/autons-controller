use alloc::boxed::Box;
use core::{fmt::Display, future::Future, pin::Pin};

type RouteFn<Shared> = for<'s> fn(&'s mut Shared) -> Pin<Box<dyn Future<Output = ()> + 's>>;

/// Route entry for [`SimpleSelect`].
///
/// These are provided to [`SimpleSelect`] in the form of an array passed to [`SimpleSelect`].
/// Route entries contain a function pointer to the provided route function, as well as a human-readable
/// name for the route that is displayed in the selector's UI.
///
/// It's recommended to use the [`route!()`] macro to aid in creating instances of this struct.
///
/// [`SimpleSelect`]: crate::simple::SimpleSelect
#[derive(Debug, Eq, PartialEq)]
pub struct Route<R, C: Display + Clone + PartialEq> {
    pub category: C,
    pub name: &'static str,
    pub callback: RouteFn<R>,
}

impl<R, C: Display + Clone + PartialEq> Clone for Route<R, C> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            category: self.category.clone(),
            callback: self.callback,
        }
    }
}

impl<R, C: Display + Clone + PartialEq> Route<R, C> {
    pub fn new(category: C, name: &'static str, callback: RouteFn<R>) -> Self {
        Self {
            category,
            name,
            callback,
        }
    }
}

/// Concisely creates an instance of a [`SimpleSelectRoute`].
///
/// # Example
///
/// ```
/// #[derive(Clone, Copy)]
/// enum Category {
///     Category1,
///     Category2,
/// }
///
/// impl Display for Category {
///     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
///         match self {
///             Category::Category1 => write!(f, "Category 1"),
///             Category::Category2 => write!(f, "Category 2"),
///         }
///     }
/// }
///
/// let routes = [
///     route!(Category::Category1, "Route 1", Robot::route_1),
///     route!(Category::Category2, "Route 2", Robot::route_2),
/// ];
/// ```
#[macro_export]
macro_rules! route {
    ($category:expr, $name:expr, $func:path) => {{
        ::autons_controller::Route::new($category, $name, |robot| {
            ::alloc::boxed::Box::pin($func(robot))
        })
    }};
}
pub use route;

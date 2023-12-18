/// A `Subscriber` can subscribe to events of type `E` that are published.
/// 
/// Subscribers must implement the `receive()` method, which is called with a borrowed reference to
/// the `E` object that was published. They must also implement `as_any()`, but this crate provides
/// the `as_any!()` macro to cut down boilerplate. 
/// 
/// # Examples
/// 
/// This example shows a `Dog` object, which subscribes to any published `Food` events and
/// accumulates the amount of food eaten. Note that `impl` uses the `as_any!()` macro to stand in
/// for the requirement to implement the `as_any` function.
/// ```
/// # use pubsubhub_macros::as_any;
/// # use pubsubhub::Subscriber;
/// 
/// struct Food { amount: i32 }
/// 
/// struct Dog { total_eaten: i32 }
/// impl Subscriber<Food> for Dog {
///     fn receive(&mut self, event: &Food) {
///         self.total_eaten += event.amount;
///     }
///     as_any!();
/// }
/// ```
/// 
/// If you wanted to subscribe to multiple events, simply provide an `impl` for each disambiguated
/// `Subscriber`. Here's a `Dog` that listens for `Food` and `Sleep` events:
/// ```
/// # use pubsubhub_macros::as_any;
/// # use pubsubhub::Subscriber;
/// 
/// struct Food { amount: i32 }
/// struct Sleep { }
/// 
/// struct Dog { 
///     total_eaten: i32,
///     times_slept: u32,
/// }
/// impl Subscriber<Food> for Dog {
///     fn receive(&mut self, event: &Food) {
///         self.total_eaten += event.amount;
///     }
///     as_any!();
/// }
/// impl Subscriber<Sleep> for Dog {
///     fn receive(&mut self, event: &Sleep) {
///         self.times_slept += 1;
///     }
///     as_any!();
/// }
pub trait Subscriber<E> {
    /// This method is called for each subscriber in a `PubSubHub` when an event of type `E` is
    /// published.
    fn receive(&mut self, event: &E);

    /// `as_any` provides a cast from this type to `std::any::Any`. This isn't directly used by
    /// a PubSub system but may be useful if you want to recover a reference to the object after
    /// subscribing it to a PubSub.
    /// 
    /// # Example
    /// Here we hold on to the `Arc` returned by the PubSub's subscription function and use the
    /// `as_any` function to access it outside of publishing events:
    /// ```
    /// # #[macro_use] extern crate pubsubhub;
    /// # use pubsubhub_macros::as_any;
    /// # use pubsubhub_macros::publishes;
    /// # use pubsubhub::Subscriber;
    /// # use std::any::Any;
    /// struct Food { amount: i32 }
    /// 
    /// struct Dog { total_eaten: i32 }
    /// impl Subscriber<Food> for Dog { 
    ///     fn receive(&mut self, event: &Food) {
    ///         self.total_eaten += event.amount;
    ///     }
    ///     fn as_any(&self) -> &dyn Any { self }
    /// }
    /// 
    /// #[publishes(Food)]
    /// struct PubSub { }
    /// 
    /// fn main() {
    ///     let d = Dog { total_eaten: 0 };
    ///     let mut p = PubSub::new();
    ///     let arc_to_d = p.subscribe_Food(Box::new(d));
    ///     let boxed = arc_to_d.lock().unwrap();
    ///     let d = boxed.as_any().downcast_ref::<Dog>().unwrap();
    ///     assert_eq!(d.total_eaten, 0);
    /// }
    /// ```
    fn as_any(&self) -> &dyn std::any::Any;
}
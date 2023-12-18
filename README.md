# PubSubHub-rs

A library that implements a publish/subscribe system in rust, as a learning exercise.

# Subscribers
This library provides a trait called `Subscriber` with a type parameter `A`. A type that implements
`Subscriber<A>` provides a method called `receive` which is called by a pubsub system and passed
a reference to an instance of `A`.

Here's an example subscriber implementation:

```rust
struct Food {
    amount: i32,
}

struct Dog {
    total_eaten: i32,
}
impl Subscriber<Food> for Dog {
    pub fn receive(&mut self, event: &Food) {
        self.total_eaten += amount;
    }
    as_any!()
}
```

When a `Dog` is subscribed to a pubsub, and a `Food` event is published, the `receive` method will be called.

# PubSub
The library doesn't actually define a particular PubSub struct, but allows you to turn any struct
into a pubsub system with a macro listing the types it should publish.

```rust
struct Food { amount: i32 }
struct Sleep { time: i32 }

#[publishes(Food, Sleep)]
struct PubSubHub {
    /* Any other struct fields you want in your pubsub system */
}
```

The macro will fill in new methods that can be used to publish and subscribe to the events supported
by that struct. So, for example, you could use the `PubSubHub` to `publish_Food` or `publish_Sleep`:

```rust
let mut p = PubSubHub::new();
let d = Dog { } // Assume dog implements Subscriber<Food> and Subscriber<Sleep>

p.subscribe_Food(Box::new(d));
p.subscribe_Sleep(Box::new(d));

let f = Food { amount: 80 }
p.publish_Food(&f) // Will call the receive method for the Subscriber<Food> implementation on Dog
```
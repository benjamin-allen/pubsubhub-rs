#[cfg(test)]
mod tests {
    use pubsubhub::*;
    use pubsubhub_macros::*;

    struct A { a: i32 }
    
    struct Listener { }
    impl Subscriber<A> for Listener {
        fn receive(&mut self, event: &A) {
            assert_ne!(event.a, 123);
        }
        as_any!();
    }

    #[publishes(A)]
    struct PubSub { }

    #[test]
    pub fn test_can_subscribe() {
        let mut pubsub = PubSub::new();
        let l = Listener { };
        pubsub.subscribe_A(Box::new(l));

        assert_eq!(pubsub.__subscriptions_A.iter().count(), 1);

        pubsub.subscribe_A(Box::new(Listener { }));
        assert_eq!(pubsub.__subscriptions_A.iter().count(), 2);
    }

    #[test]
    pub fn test_can_publish() {
        let mut pubsub = PubSub::new();
        let l = Listener { };
        pubsub.subscribe_A(Box::new(l));

        for i in 0..122 {
            let event = A { a: i };
            pubsub.publish_A(&event);
        }

        let pubsub_immut = &pubsub;
        let result = std::panic::catch_unwind(|| pubsub_immut.publish_A(&A { a: 123 }));
        assert_eq!(result.is_err(), true);
    }

    #[test]
    pub fn test_can_unsubscribe() {
        let mut pubsub = PubSub::new();
        let l1 = Listener { };
        let l2 = Listener { };

        let l1_arc = pubsub.subscribe_A(Box::new(l1));
        let l2_arc = pubsub.subscribe_A(Box::new(l2));

        assert_eq!(pubsub.__subscriptions_A.iter().count(), 2);

        pubsub.unsubscribe_A(&l1_arc);

        assert_eq!(pubsub.__subscriptions_A.iter().count(), 1);

        pubsub.unsubscribe_A(&l2_arc);

        assert_eq!(pubsub.__subscriptions_A.iter().count(), 0);

        pubsub.publish_A(&A { a: 123 });
    }
}
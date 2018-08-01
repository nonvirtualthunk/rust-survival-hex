use anymap::AnyMap;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::iter::Iterator;
use itertools::Itertools;

#[allow(dead_code)]
struct ListenerContainer<T, EventType> {
    callbacks : Vec<fn(&mut T, &EventType)>,
    offset : usize
}


struct EventHolder<EventType>(EventType, usize);

#[derive(PartialEq,Eq,Hash,Clone,Copy)]
pub struct ConsumerHandle(usize, usize);

#[allow(dead_code)]
pub struct EventBus<EventType> {
    event_count: usize,
    events : VecDeque<EventHolder<EventType>>,
    callbacks : AnyMap
}

impl <EventType> Default for EventBus<EventType> {
    fn default() -> Self {
        EventBus::new()
    }
}

impl <EventType> EventBus<EventType> {
    pub fn new() -> EventBus<EventType> {
        EventBus {
            event_count: 0,
            events : VecDeque::new(),
            callbacks : AnyMap::new()
        }
    }


    pub fn push_event(&mut self, evt : EventType) {
        self.event_count += 1;
        self.events.push_front(EventHolder(evt, self.event_count));
    }

    pub fn register_consumer(&self, start_at_beginning : bool) -> ConsumerHandle {
        let offset = if start_at_beginning { 0 } else { self.event_count };
        ConsumerHandle(0, offset)
    }

    pub fn events_for(&self, consumer : &mut ConsumerHandle) -> impl Iterator<Item=&EventType> {
        let stop_point = consumer.1;
        consumer.1 = self.event_count;

        self.events.iter().take_while(move |e| e.1 > stop_point).map(|e| &e.0).collect_vec().into_iter().rev()
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use spectral::prelude::*;

    #[derive(PartialEq,Eq,Clone,Debug)]
    pub enum TestEvents {
        Foo,
        Bar(i32)
    }

    #[test]
    pub fn test_event_bus() {
        let mut bus = EventBus::new();

        bus.push_event(TestEvents::Foo);
        bus.push_event(TestEvents::Bar(1));

        let mut start_at_end_consumer = bus.register_consumer(false);
        let mut start_at_beginning_consumer = bus.register_consumer(true);

        {
            assert_that(&bus.events_for(&mut start_at_end_consumer).collect_vec().is_empty()).is_equal_to(&true);
            let from_beginning_events = bus.events_for(&mut start_at_beginning_consumer).collect_vec();
            assert_that(&from_beginning_events).has_length(2);
            assert_that(&from_beginning_events[0]).is_equal_to(&TestEvents::Foo);
            assert_that(&from_beginning_events[1]).is_equal_to(&TestEvents::Bar(1));

            assert_that(&bus.events_for(&mut start_at_beginning_consumer).collect_vec()).has_length(0);
        }

        bus.push_event(TestEvents::Bar(3));

        let from_beginning_events = bus.events_for(&mut start_at_beginning_consumer).collect_vec();
        assert_that(&from_beginning_events).has_length(1);
        assert_that(&from_beginning_events[0]).is_equal_to(&TestEvents::Bar(3));

        let from_end_events = bus.events_for(&mut start_at_end_consumer).collect_vec();
        assert_that(&from_end_events).has_length(1);
        assert_that(&from_end_events[0]).is_equal_to(&TestEvents::Bar(3));
    }


}
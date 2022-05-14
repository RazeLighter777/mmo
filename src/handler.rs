use crate::game_event;
use crate::event_collector;
pub trait HandlerInterface {
    fn handle(&self, handler : &event_collector::EventCollector);
}
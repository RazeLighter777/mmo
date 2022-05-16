use crate::event_collector;
use crate::game_event;
pub trait HandlerInterface: Send {
    fn handle(&self, handler: &event_collector::EventCollector);
}

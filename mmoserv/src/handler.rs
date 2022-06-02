use crate::event_collector;
pub trait HandlerInterface: Send + Sync {
    fn handle(&self, handler: &event_collector::EventCollector);
}

use std::collections::HashMap;

use serde_json::map::{OccupiedEntry, VacantEntry};
use std::collections::hash_map::Entry::{Occupied, Vacant};

use crate::game::Game;
use mmolib;
pub struct EventCollector {
    events_by_type: HashMap<
        mmolib::game_event::EventTypeId,
        Vec<Box<dyn mmolib::game_event::GameEventInterface>>,
    >,
}
impl EventCollector {
    pub fn new() -> Self {
        Self {
            events_by_type: HashMap::new(),
        }
    }
    pub fn add_events(&mut self, evs: Vec<Box<dyn mmolib::game_event::GameEventInterface>>) {
        for ev in evs {
            match self.events_by_type.entry(ev.get_type_id()) {
                Vacant(mut ent) => {
                    ent.insert(vec![ev]);
                }
                Occupied(mut ent) => {
                    ent.get_mut().push(ev);
                }
            }
        }
    }
    pub fn clear(&mut self) {
        self.events_by_type.clear();
    }
    pub fn get_events_of_type<T: mmolib::game_event::GameEventType + 'static>(
        &self,
    ) -> Vec<&mmolib::game_event::GameEvent<T>> {
        let mut res = Vec::new();
        match self
            .events_by_type
            .get(&mmolib::game_event::get_type_id::<T>())
        {
            Some(ent) => {
                for i in ent {
                    match i
                        .as_any()
                        .downcast_ref::<mmolib::game_event::GameEvent<T>>()
                    {
                        Some(cast) => {
                            res.push(cast);
                        }
                        None => {}
                    }
                }
            }
            None => {}
        }
        res
    }
}

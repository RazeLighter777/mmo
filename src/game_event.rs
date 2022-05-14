use crate::entity;
use crate::hashing;
pub type EventTypeId = u64;
pub trait GameEventType {}

pub trait GameEventInterface : Send + Sync {
    fn get_type_id(&self) -> EventTypeId;
    fn get_targets(&self) -> &Vec<entity::EntityId>;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_mutable(&mut self) -> &mut dyn std::any::Any;
}
pub const fn get_type_id<DataType: 'static + GameEventType>() -> u64 {
    hashing::string_hash(std::any::type_name::<DataType>())
}
pub struct GameEvent<T : GameEventType> {
    data : T,
    tid : EventTypeId,
    targets : Vec<entity::EntityId>
}
impl<DataType: 'static + GameEventType + Send + Sync> GameEventInterface for GameEvent<DataType> {
    fn get_type_id(&self) -> EventTypeId {
        self.tid
    }
    fn get_targets(&self) -> &Vec<entity::EntityId> {
        &self.targets
    }
     /// Returns an any trait reference
     fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }
    /// Returns an any mutable trait reference
    fn as_mutable(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
}
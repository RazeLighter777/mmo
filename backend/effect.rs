use serde::Deserialize;

#[derive(Deserialize, Clone,Debug)]
pub enum Effect {
    Poison = 0,
    Fire = 1,
    Haste = 2,
    Stink = 3,
    Strength = 4,
}
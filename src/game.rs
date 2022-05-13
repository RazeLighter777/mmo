use crate::world;
use crate::generator;

pub struct Game {
    world : world::World,
    generators : Vec<Box<dyn generator::Generator>>
}

impl Game {
    pub fn new() -> Self {
        Game { world: world::World::new(), generators: Vec::new() }
    }
    pub fn add_generator(&mut self, generator : Box<dyn generator::Generator>) {
        self.generators.push(generator);
    }
    pub fn tick(&mut self) {
        self.world.process(&self.generators);
    }
    pub fn get_world(&mut self) -> &mut world::World { 
        &mut self.world
    }
}
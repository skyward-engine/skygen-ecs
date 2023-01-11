use crate::{entity::EntityContainer, SkywardError};
use std::cell::RefCell;

#[derive(Debug, PartialEq, Hash, Eq)]
pub enum SystemType {
    Init,
    Loop,
}

pub trait System {
    fn update(&self, entity_container: &EntityContainer) -> Result<(), SkywardError>;
}

pub struct World {
    pub container: EntityContainer,
    init_systems: Vec<RefCell<Box<dyn System>>>,
    loop_systems: Vec<RefCell<Box<dyn System>>>,
}

impl World {
    pub fn new(counts: [usize; 2]) -> Self {
        Self {
            container: EntityContainer::new(Some(counts[0]), Some(counts[1])),
            init_systems: vec![],
            loop_systems: vec![],
        }
    }

    pub fn system<T>(&mut self, system_type: SystemType, system: T) -> &mut Self
    where
        T: System + 'static,
    {
        let systems = match system_type {
            SystemType::Init => &mut self.init_systems,
            SystemType::Loop => &mut self.loop_systems,
        };

        systems.push(RefCell::new(Box::new(system)));
        self
    }

    pub fn update(&self, system_type: SystemType) -> Result<(), SkywardError> {
        let systems = match system_type {
            SystemType::Init => &self.init_systems,
            SystemType::Loop => &self.loop_systems,
        };

        for system in systems {
            system.borrow().update(&self.container)?;
        }

        Ok(())
    }
}

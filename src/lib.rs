pub mod borrow;
pub mod entity;
pub mod world;

#[derive(Debug)]
pub enum SkywardError {
    BitMaskExhausted,
    DowncastError,
    BorrowError,
    NoComponent,
}

#[cfg(test)]
pub mod test {
    use crate::{
        entity::EntityContainer,
        world::{System, SystemType, World},
        SkywardError,
    };

    struct Named(&'static str);
    struct Position {
        x: f32,
        y: f32,
    }

    #[test]
    pub fn ecs_test() -> Result<(), SkywardError> {
        const TEST_ENTITY: usize = 0;

        struct TestEntitySystem;
        impl System for TestEntitySystem {
            fn update(&self, container: &EntityContainer) -> Result<(), SkywardError> {
                let mut positions = container.borrow_mut::<Position>()?;
                let named = container.borrow::<Named>()?;

                let named = named.get(TEST_ENTITY).ok_or(SkywardError::NoComponent)?;
                let position = positions
                    .get_mut(TEST_ENTITY)
                    .ok_or(SkywardError::NoComponent)?;

                position.x += 0.3;
                position.y += 0.3;

                println!("{} is at x:{}, y:{}", named.0, position.x, position.y);

                Ok(())
            }
        }

    let mut world = World::new([1, 63]);
        world.system(SystemType::Loop, TestEntitySystem);

        let container = &mut world.container;
        let entity = container.entity();

        container
            .with::<Named>(entity, Named("NV6"))?
            .with::<Position>(entity, Position { x: 39.4, y: 21.3 })?;

        for _ in 0..5 {
            world.update(SystemType::Loop)?;
        }

        Ok(())
    }
}

use std::{
    any::{Any, TypeId},
    cell::RefCell,
};

use hashbrown::HashMap;
use persist_o_vec::Persist;

use crate::{
    borrow::{MapMut, MapRef},
    SkywardError,
};

type EntityMaskType = u64;
const ENTITY_MASK_MAX: EntityMaskType = 63;

#[derive(Clone, Copy, Hash, Eq, Ord, PartialEq, PartialOrd)]
struct Entity {
    pub(crate) entity_id: EntityMaskType,
}
pub struct EntityContainer {
    active_entities: Vec<usize>,
    entity_masks: Vec<EntityMaskType>,
    components: Vec<RefCell<Box<dyn Any>>>,
    next_free_entity: usize,
    vacated_slots: Vec<usize>,
    type_masks: HashMap<TypeId, (EntityMaskType, EntityMaskType)>,
    next_type_mask: EntityMaskType,
}

impl Default for EntityContainer {
    fn default() -> Self {
        EntityContainer::new(None, None)
    }
}

impl EntityContainer {
    pub fn new(entity_count: Option<usize>, component_count: Option<usize>) -> Self {
        if let Some(count) = component_count {
            if count as EntityMaskType > ENTITY_MASK_MAX {
                panic!(
                    "Initial part count too large. Maximum of {} allowed",
                    ENTITY_MASK_MAX
                );
            }
        }
        Self {
            entity_masks: vec![0; entity_count.unwrap_or(0)],
            active_entities: vec![],
            components: Vec::with_capacity(component_count.unwrap_or(0)),
            next_free_entity: 0,
            vacated_slots: Vec::new(),
            type_masks: HashMap::with_capacity(component_count.unwrap_or(0)),
            next_type_mask: 1,
        }
    }

    #[inline]
    pub fn entity(&mut self) -> usize {
        let entity: usize;

        match self.vacated_slots.pop() {
            Some(slot) => entity = slot,
            None => {
                entity = self.next_free_entity;
                self.next_free_entity += 1;
            }
        }
        self.active_entities.push(entity);
        entity
    }

    pub fn with<T>(&mut self, entity: usize, component: T) -> Result<&mut Self, SkywardError>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        let entry = {
            if !self.type_masks.contains_key(&type_id) {
                if (self.type_masks.len() as EntityMaskType) >= ENTITY_MASK_MAX {
                    return Err(SkywardError::BitMaskExhausted);
                }

                let persistant =
                    Persist::<T>::with_capacity(self.entity_masks.len() + self.vacated_slots.len());
                let boxed = RefCell::new(Box::new(persistant));

                // todo: add check if component is a duplicate!
                self.components.push(boxed);
                self.type_masks.insert(
                    type_id,
                    (
                        self.next_type_mask,
                        (self.components.len() - 1).try_into().unwrap(),
                    ),
                );

                self.next_type_mask <<= 1;
            }

            self.type_masks[&type_id]
        };

        let (type_mask, type_index) = entry;

        // we checked whether the current_entity contains a value at the beginning of this function.
        let comp = self.components.get_mut(type_index as usize);

        if let Some(comp) = comp {
            comp.borrow_mut()
                .downcast_mut::<Persist<T>>()
                .ok_or(SkywardError::DowncastError)?
                .insert(entity, component);

            let old = self.entity_masks[entity];
            self.entity_masks[entity] = old | type_mask;
        }

        Ok(self)
    }

    #[inline]
    pub fn has(&self, entity: usize) -> bool {
        match self.entity_masks.get(entity) {
            Some(entity) => entity > &0,
            None => false,
        }
    }

    #[inline]
    pub fn entity_has<T>(&self, entity: usize) -> bool
    where
        T: 'static,
    {
        self.entity_has_option::<T>(entity).is_some()
    }

    #[inline]
    pub fn entity_has_option<T>(&self, entity: usize) -> Option<()>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();

        let entity_mask = self.entity_masks.get(entity)?;
        let (type_mask, _) = self.type_masks.get(&type_id)?;

        return if entity_mask & type_mask == *type_mask {
            Some(())
        } else {
            None
        };
    }

    #[inline]
    pub fn remove_component<T>(&mut self, entity: usize) -> Result<(), SkywardError>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        let (type_mask, type_index) = self
            .type_masks
            .get(&type_id)
            .ok_or(SkywardError::NoComponent)?;
        let map = self
            .components
            .get_mut(*type_index as usize)
            .ok_or(SkywardError::NoComponent)?;

        map.borrow_mut()
            .downcast_mut::<Persist<T>>()
            .ok_or(SkywardError::DowncastError)?
            .remove(entity);

        self.entity_masks[entity] ^= *type_mask;

        if self.entity_masks[entity] == 0 {
            self.vacated_slots.push(entity);
        }

        Ok(())
    }

    #[inline]
    pub fn borrow<T>(&self) -> Result<MapRef<T>, SkywardError>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        let type_index = self
            .type_masks
            .get(&type_id)
            .ok_or(SkywardError::NoComponent)?
            .1;

        let components = self
            .components
            .get(type_index as usize)
            .ok_or(SkywardError::NoComponent)?;

        Ok(MapRef::new(components)?)
    }

    #[inline]
    pub fn borrow_mut<T>(&self) -> Result<MapMut<T>, SkywardError>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        let type_index = self
            .type_masks
            .get(&type_id)
            .ok_or(SkywardError::NoComponent)?
            .1;

        let components = self
            .components
            .get(type_index as usize)
            .ok_or(SkywardError::NoComponent)?;

        Ok(MapMut::new(components)?)
    }

    pub fn entities(&self) -> &Vec<usize> {
        &self.active_entities
    }
}

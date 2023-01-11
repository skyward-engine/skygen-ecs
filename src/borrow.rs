use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    ops::{Deref, DerefMut},
};

use persist_o_vec::Persist;

use crate::SkywardError;

pub struct MapRef<'a, T> {
    _borrow: Ref<'a, dyn Any>,
    value: &'a Persist<T>,
}

impl<'a, T> MapRef<'a, T>
where
    T: 'static,
{
    pub fn new(value: &'a RefCell<Box<dyn Any>>) -> Result<MapRef<'a, T>, SkywardError> {
        let borrow = value.try_borrow().or(Err(SkywardError::BorrowError))?;
        let value = unsafe { value.as_ptr().as_ref() }
            .ok_or(SkywardError::DowncastError)?
            .downcast_ref::<Persist<T>>()
            .ok_or(SkywardError::DowncastError)?;

        let map_ref = Self {
            value,
            _borrow: borrow,
        };

        Ok(map_ref)
    }
}

impl<'a, T> Deref for MapRef<'a, T> {
    type Target = Persist<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

pub struct MapMut<'a, T> {
    _borrow: RefMut<'a, dyn Any>,
    value: &'a mut Persist<T>,
}

impl<'a, T> MapMut<'a, T>
where
    T: 'static,
{
    pub fn new(value: &'a RefCell<Box<dyn Any>>) -> Result<MapMut<'a, T>, SkywardError> {
        let borrow = value.try_borrow_mut().or(Err(SkywardError::BorrowError))?;
        let value = unsafe { value.as_ptr().as_mut() }
            .ok_or(SkywardError::DowncastError)?
            .downcast_mut::<Persist<T>>()
            .ok_or(SkywardError::DowncastError)?;

        let map_ref = Self {
            value,
            _borrow: borrow,
        };

        Ok(map_ref)
    }
}

impl<'a, T> Deref for MapMut<'a, T> {
    type Target = Persist<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T> DerefMut for MapMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

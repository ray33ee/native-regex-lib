use std::iter::Enumerate;

//Half vector and half hash map, this absolute abomination allows super fast insertions, removal and clearing with absolutely no collision detection!
#[derive(Debug, Clone)]
pub struct VectorMap<T> {
    _map: Vec<Option<T>>,
    _state: u64,
}

impl<T> VectorMap<T> {

    #[inline(always)]
    pub fn new(n: usize) -> Self {
        assert!(n < 64);

        //Is there a better way to fill a vector with `n` default values?
        let mut _map = (0..n).map(|_| None).collect::<Vec<_>>();

        let _state = 0;

        VectorMap {
            _map,
            _state,
        }
    }

    #[inline(always)]
    pub fn get(& self, index: usize) -> Option<& T> {
        if (self._state & (1 << index)) != 0  {
            self._map.get(index).unwrap().as_ref()
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn insert(& mut self, index: usize, item: T) {
        self._map[index] = Some(item);
        self._state = self._state | (1 << index); //Set the nth bit
    }

    #[inline(always)]
    pub fn remove(& mut self, index: usize) {
        self._state = self._state & !(1 << index); //Clear the nth bit
    }

    #[inline(always)]
    pub fn clear(& mut self) {
        self._state = 0;
    }

    #[inline(always)]
    pub fn iter(&self) -> Iter<T> {
        Iter {
            _iterator: self._map.iter().enumerate(),
            _state: self._state,
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self._map.len()
    }

}

pub struct Iter<'a, T> {
    _iterator: Enumerate<std::slice::Iter<'a, Option<T>>>,
    _state: u64,
}

impl<'a, T> Iterator for Iter<'a, T>
{
    type Item = Option<& 'a T>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let (index, next_item) = self._iterator.next()?;

        Some(if (self._state & (1 << index)) != 0  {
            next_item.as_ref()
        } else {
            None
        })
    }
}

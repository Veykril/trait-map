#[doc(hidden)]
pub mod __private {
    pub use core::any::TypeId;
    pub use core::default::Default;
    pub use std::boxed::Box;
    pub use std::collections::HashMap;
}

#[macro_export]
macro_rules! trait_map {
    ( $(#[$meta:meta])* $vis:vis struct $ident:ident<dyn $trait:ident $(+ $bound:ident)*> ) => {
        $(#[$meta])*
        $vis struct $ident {
            map: $crate::__private::HashMap<$crate::__private::TypeId, $crate::__private::Box<dyn $trait $(+ $bound)* + 'static>>,
        }

        #[allow(dead_code)]
        impl $ident {
            $vis fn new() -> Self {
                $ident {
                    map: $crate::__private::HashMap::new()
                }
            }

            $vis fn with_capacity(cap: usize) -> Self {
                $ident {
                    map: $crate::__private::HashMap::with_capacity(cap)
                }
            }

            $vis fn reserve(&mut self, additional: usize) {
                self.map.reserve(additional);
            }

            $vis fn capacity(&self) -> usize {
                self.map.capacity()
            }

            $vis fn len(&self) -> usize {
                self.map.len()
            }

            $vis fn is_empty(&self) -> bool {
                self.map.is_empty()
            }

            $vis fn clear(&mut self) {
                self.map.clear();
            }

            $vis fn shrink_to_fit(&mut self) {
                self.map.shrink_to_fit();
            }
        }

        #[allow(dead_code)]
        impl $ident {
            $vis fn contains<T: $trait $(+ $bound)* + 'static>(&self) -> bool {
                self.map.contains_key(&$crate::__private::TypeId::of::<T>())
            }

            $vis fn get<T: $trait $(+ $bound)* + 'static>(&self) -> Option<&T> {
                self.map
                    .get(&$crate::__private::TypeId::of::<T>())
                    .map(|it| unsafe { Self::downcast_ref_unchecked(&**it) })
            }

            $vis fn get_mut<T: $trait $(+ $bound)* + 'static>(&mut self) -> Option<&mut T> {
                self.map
                    .get_mut(&$crate::__private::TypeId::of::<T>())
                    .map(|it| unsafe { Self::downcast_mut_unchecked(&mut **it) })
            }

            $vis fn insert<T: $trait $(+ $bound)* + 'static>(&mut self, t: T) -> Option<T> {
                self.map
                    .insert($crate::__private::TypeId::of::<T>(), Box::new(t))
                    .map(|it| unsafe { *Self::downcast_unchecked(it) })
            }

            $vis fn remove<T: $trait $(+ $bound)* + 'static>(&mut self) -> Option<T> {
                self.map
                    .remove(&$crate::__private::TypeId::of::<T>())
                    .map(|it| unsafe { *Self::downcast_unchecked(it) })
            }

            $vis fn iter(&self) -> impl Iterator<Item = ($crate::__private::TypeId, &(dyn $trait $(+ $bound)*))> + '_ {
                self.map.iter().map(|(&k, v)| (k, &**v))
            }

            $vis fn iter_mut(&mut self) -> impl Iterator<Item = ($crate::__private::TypeId, &mut (dyn $trait $(+ $bound)* + 'static))> + '_ {
                self.map.iter_mut().map(|(&k, v)| (k, &mut **v))
            }
        }

        impl $crate::__private::Default for $ident {
            fn default() -> Self {
                Self::new()
            }
        }

        #[allow(dead_code)]
        impl $ident {
            #[inline]
            unsafe fn downcast_ref_unchecked<T: $trait $(+ $bound)*>(it: &dyn $trait) -> &T {
                &*(it as *const dyn $trait as *const T)
            }
            #[inline]
            unsafe fn downcast_mut_unchecked<T: $trait $(+ $bound)*>(it: &mut dyn $trait) -> &mut T {
                &mut *(it as *mut dyn $trait as *mut T)
            }
            #[inline]
            unsafe fn downcast_unchecked<T: $trait $(+ $bound)*>(it: Box<dyn $trait>) -> Box<T> {
                $crate::__private::Box::from_raw($crate::__private::Box::into_raw(it) as *mut T)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    pub trait Trait {}

    impl Trait for u8 {}
    impl Trait for u16 {}
    impl Trait for u32 {}
    impl Trait for u64 {}
    impl Trait for u128 {}

    trait_map!(struct TraitMap<dyn Trait>);
    trait_map!(struct SendMap<dyn Trait + Send>);
    trait_map!(struct SyncMap<dyn Trait + Sync>);
    trait_map!(struct SendSyncMap<dyn Trait + Send + Sync>);

    pub trait CloneTrait: dyn_clone::DynClone {}
    dyn_clone::clone_trait_object!(CloneTrait);
    trait_map!(#[derive(Clone)] struct CloneTraitMap<dyn CloneTrait>);

    static_assertions::assert_not_impl_any!(TraitMap: Clone, Send, Sync);
    static_assertions::assert_impl_all!(SendMap: Send);
    static_assertions::assert_impl_all!(SyncMap: Sync);
    static_assertions::assert_impl_all!(SendSyncMap: Send, Sync);
    static_assertions::assert_impl_all!(CloneTraitMap: Clone);

    #[test]
    fn insert_different() {
        let mut map = TraitMap::new();
        assert_eq!(None, map.insert(8u8));
        assert_eq!(None, map.insert(16u16));
        assert_eq!(None, map.insert(32u32));
        assert_eq!(None, map.insert(64u64));
        assert_eq!(None, map.insert(128u128));
        assert_eq!(Some(8), map.remove::<u8>());
        assert_eq!(Some(16), map.remove::<u16>());
        assert_eq!(Some(32), map.remove::<u32>());
        assert_eq!(Some(64), map.remove::<u64>());
        assert_eq!(Some(128), map.remove::<u128>());
    }

    #[test]
    fn overwrite() {
        let mut map = TraitMap::new();
        let source = 1..128u32;
        let mut expected = 0..128u32;
        assert_eq!(None, map.insert(0u32));
        for n in source {
            assert_eq!(map.insert(n), expected.next());
        }
        assert_eq!(map.remove::<u32>(), expected.next());
    }
}

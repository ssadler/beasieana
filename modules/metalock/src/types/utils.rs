
#[macro_export]
macro_rules! byte_ref {
    ($val:expr, $size:expr) => {
        unsafe { &*(std::ptr::addr_of!($val) as *const [u8; $size]) }
    };
}

#[macro_export]
macro_rules! impl_deref {
    ( [$($impl_generics:tt)*], $type:ty, $target:ty, $field:tt) => {
        $crate::impl_deref_const!([$($impl_generics)*], $type, $target, $field);

        impl<$($impl_generics)*> std::ops::DerefMut for $type {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.$field
            }
        }
    };
}

#[macro_export]
macro_rules! impl_deref_const {
    ( [$($impl_generics:tt)*], $type:ty, $target:ty, $field:tt) => {
        impl<$($impl_generics)*> std::ops::Deref for $type {
            type Target = $target;
            fn deref(&self) -> &Self::Target {
                &self.$field
            }
        }
    }
}

macro_rules! field_ptr {
    ($ptr:expr, $type:ty, $field:ident) => {{
        use std::ptr;

        #[allow(unused_unsafe)]
        unsafe {
            if true {
                $ptr.cast::<u8>()
                    .add(offset_of!($type, $field))
                    .cast()
            } else {
                #[allow(deref_nullptr)]
                {
                    ptr::addr_of!((*ptr::null::<$type>()).$field)
                }
            }
        }
    }};
}

pub(super) use field_ptr;
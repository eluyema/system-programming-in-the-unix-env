#[macro_export]
macro_rules! offset_of {
    ($type:ty, $field:ident) => {{
        use std::mem::MaybeUninit;
        use std::ptr;
        let data = MaybeUninit::<$type>::uninit();
        #[allow(unused_unsafe)]
        unsafe {
            ptr::addr_of!((*data.as_ptr()).$field)
                .cast::<u8>()
                .offset_from(data.as_ptr().cast::<u8>()) as usize
        }
    }};
}
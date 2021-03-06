use crate::exception::Exception;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use core::fmt::Display;
use core::ptr::{self, NonNull};
use core::result::Result as StdResult;
use core::slice;
use core::str;

#[repr(C)]
#[derive(Copy, Clone)]
struct PtrLen {
    ptr: NonNull<u8>,
    len: usize,
}

#[repr(C)]
pub union Result {
    err: PtrLen,
    ok: *const u8, // null
}

pub unsafe fn r#try<T, E>(ret: *mut T, result: StdResult<T, E>) -> Result
where
    E: Display,
{
    match result {
        Ok(ok) => {
            ptr::write(ret, ok);
            Result { ok: ptr::null() }
        }
        Err(err) => to_c_error(err.to_string()),
    }
}

unsafe fn to_c_error(msg: String) -> Result {
    let mut msg = msg;
    msg.as_mut_vec().push(b'\0');
    let ptr = msg.as_ptr();
    let len = msg.len();

    extern "C" {
        #[link_name = "cxxbridge1$error"]
        fn error(ptr: *const u8, len: usize) -> *const u8;
    }

    let copy = error(ptr, len);
    let slice = slice::from_raw_parts(copy, len);
    let string = str::from_utf8_unchecked(slice);
    let err = PtrLen {
        ptr: NonNull::from(string).cast::<u8>(),
        len: string.len(),
    };
    Result { err }
}

impl Result {
    pub unsafe fn exception(self) -> StdResult<(), Exception> {
        if self.ok.is_null() {
            Ok(())
        } else {
            let err = self.err;
            let slice = slice::from_raw_parts_mut(err.ptr.as_ptr(), err.len);
            let s = str::from_utf8_unchecked_mut(slice);
            Err(Exception {
                what: Box::from_raw(s),
            })
        }
    }
}

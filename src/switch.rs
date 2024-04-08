//! Switch C type definitions


pub const INT_MIN: i32 = -2147483648;
pub const INT_MAX: i32 = 2147483647;

#[repr(C)]
struct Reent {
    errno: i32,
}

extern "C" {
    fn __getreent() -> *mut Reent;
}

// Ported from https://github.com/devkitPro/newlib/pull/23
unsafe fn resolve_path(
    reent: &mut Reent,
    path: *mut ::c_char,
    mut extra: *const ::c_char,
    max_length: i32,
) -> i32 {
    let mut path_length = ::strnlen(path, max_length as _);

    if path_length >= max_length as usize {
        reent.errno = ::ENAMETOOLONG;
        return -1;
    }

    let mut path_end = path.add(path_length as usize);
    if *path_end.sub(1) != '/' as _ {
        *path_end = '/' as _;
        path_end = path_end.add(1)
    }

    let mut extra_end = extra;

    if *extra == '/' as _ {
        path_end = ::strchr(path, '/' as _).add(1);
        *path_end = 0;
    }

    let mut extra_size = 0;

    loop {
        let directory_this = ".\x00".as_ptr();
        let directory_parent = "..\x00".as_ptr();
        let directory_this_len = 1;
        let directory_parent_len = 2;

        while *extra == '/' as _ {
            extra = extra.add(1);
        }

        extra_end = ::strchr(extra, '/' as _);
        if extra_end.is_null() {
            extra_end = ::strrchr(extra, 0);
        } else {
            extra_end = extra_end.add(1);
        }

        extra_size = extra_end.offset_from(extra) as usize;
        if extra_size == 0 {
            break;
        }

        if (::strncmp(extra, directory_this, directory_this_len) == 0)
            && ((*extra.add(directory_this_len) == '/' as _)
                || (*extra.add(directory_this_len) == 0))
        {
        } else if (::strncmp(extra, directory_parent, directory_parent_len) == 0)
            && ((*extra.add(directory_parent_len) == '/' as _)
                || (*extra.add(directory_parent_len) == 0))
        {
            if *path_end.sub(1) == '/' as _ {
                path_end = path_end.sub(1);
            }
            path_end = ::strrchr(path, '/' as _).add(1);
            if path_end.is_null() {
                reent.errno = ::ENOENT;
                return -1;
            }

            path_length = path_end.offset_from(path) as usize;
            path_end = path_end.add(1);
        } else {
            path_length += extra_size;
            if path_length >= max_length as usize {
                reent.errno = ::ENAMETOOLONG;
                return -1;
            }
            ::strncpy(path_end, extra, extra_size);
            path_end = path_end.add(extra_size);
        }

        *path_end = 0;
        extra = extra.add(extra_size);

        if extra_size == 0 {
            break;
        }
    }

    0
}

pub fn realpath(mut pathname: *const ::c_char, resolved: *mut ::c_char) -> *mut ::c_char {
    use core::ptr::*;

    unsafe {
        let reent = &mut *__getreent();
        let mut stack = [0u8; 2048];
        let mut path_position: *const ::c_char = null();

        if pathname.is_null() {
            reent.errno = ::ENOENT;
            return null_mut();
        }

        let len = ::strnlen(pathname, 1024);
        if len == 0 {
            reent.errno = ::ENOENT;
            return null_mut();
        }
        if len >= 1024 {
            reent.errno = ::ENAMETOOLONG;
            return null_mut();
        }

        if !::strchr(pathname, ':' as _).is_null() {
            ::strncpy(stack.as_mut_ptr(), pathname, 1024 - 1);
            pathname = ::strchr(pathname, ':' as _).add(1);
        } else {
            ::getcwd(stack.as_mut_ptr(), 1024);
        }

        path_position = ::strchr(stack.as_ptr(), ':' as _);

        if path_position.is_null() {
            path_position = stack.as_ptr();
        } else {
            path_position = path_position.add(1);
        }

        if *path_position != '/' as _ {
            reent.errno = ::ENOENT;
            return null_mut();
        }

        if resolve_path(reent, stack.as_mut_ptr(), pathname, 1024) == -1 {
            return null_mut();
        }

        if !resolved.is_null() {
            ::strncpy(resolved, stack.as_ptr(), 1024);
            return resolved;
        }

        ::strndup(stack.as_ptr(), 1024)
    }
}

pub use ffi::c_void;

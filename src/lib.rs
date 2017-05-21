// Copyright 2017 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, you can
// obtain one at http://mozilla.org/MPL/2.0/.


extern crate libc;


use std::{fmt, mem};
use std::ops::{Drop, Range};


const CHUNK_SIZE: isize = 32;


/// Dynamic array that allows efficient insertion and removal operations
/// that are near the same location. Ideal for text editors.
pub struct GapBuffer {
    buf_start: *mut u8,
    gap_start: *mut u8,
    gap_end: *mut u8,
    buf_end: *mut u8
}

impl GapBuffer {
    /// Inserts `s` into the buffer at `offset`.
    pub fn insert_str(&mut self, offset: usize, s: &str) {
        let s_len = s.len() as isize;
        if s_len > self.gap_len() {
            self.grow_gap(s_len);
        }

        self.move_gap_to(offset as isize);

        let src_ptr = s.as_bytes().as_ptr();
        unsafe {
            libc::memcpy(self.gap_start as *mut libc::c_void,
                         src_ptr as *const libc::c_void,
                         s_len as usize);
            self.gap_start = self.gap_start.offset(s_len);
        }
    }

    /// Removes `range` from the buffer.
    pub fn remove(&mut self, range: Range<usize>) {
        let buf_len = self.buf_len() as usize;
        assert!(range.start < range.end, "Invalid range: {:?}", range);
        assert!(range.start < buf_len);
        assert!(range.end <= buf_len);

        let s = self.to_string();
        let head = &s[0..range.start];
        let tail = &s[range.end..];

        self.clear();
        self.insert_str(0, head);
        self.insert_str(head.len(), tail);
    }

    /// Creates a new buffer with a `capacity` sized allocation.
    ///
    /// # Panics
    ///
    /// * If `malloc` returns `NULL`.
    pub fn with_capacity(capacity: usize) -> GapBuffer {
        let buffer = unsafe {
            let size = mem::size_of::<u8>() * capacity;
            libc::malloc(size) as *mut u8
        };

        // malloc will return NULL if called with zero.
        if buffer.is_null() && capacity != 0 {
            panic!("Unable to allocate requested capacity");
        }

        GapBuffer {
            buf_start: buffer,
            gap_start: buffer,
            gap_end: unsafe { buffer.offset(capacity as isize) },
            buf_end: unsafe { buffer.offset(capacity as isize) }
        }
    }

    fn allocate_extra(&mut self, extra: isize) {
        let current_size = ptr_diff(self.buf_end, self.buf_start);
        let new_size = mem::size_of::<u8>()
            * extra as usize
            + current_size as usize;

        let new_buf = unsafe {
            libc::realloc(self.buf_start as *mut libc::c_void,
                          new_size) as *mut u8
        };

        assert!(!new_buf.is_null(), "Out of memory");

        self.buf_start = new_buf;
    }

    fn buf_len(&self) -> isize {
        let head_len = ptr_diff(self.gap_start, self.buf_start);
        let tail_len = ptr_diff(self.buf_end, self.gap_end);
        head_len + tail_len
    }

    fn clear(&mut self) {
        self.gap_start = self.buf_start;
        self.gap_end = self.buf_end;
    }

    fn gap_len(&self) -> isize {
        ptr_diff(self.gap_end, self.gap_start)
    }

    fn grow_gap(&mut self, size: isize) {
        let available = self.gap_len();
        let needed = size - available;

        let mut chunk = (needed as f32 / CHUNK_SIZE as f32).ceil() as isize;
        chunk *= CHUNK_SIZE;

        let head_len = ptr_diff(self.gap_start, self.buf_start);
        let tail_len = ptr_diff(self.buf_end, self.gap_end);
        let new_gap_size = self.gap_len() + chunk;
        let buf_len = head_len + tail_len;

        self.allocate_extra(chunk);
        unsafe {
            libc::memmove(self.gap_start as *mut libc::c_void,
                          self.gap_end as *const libc::c_void,
                          tail_len as usize);
            self.gap_start = self.buf_start.offset(buf_len);
            self.gap_end = self.gap_start.offset(new_gap_size);
            self.buf_end = self.gap_end;
        }
    }

    fn head(&self) -> String {
        let head_len = ptr_diff(self.gap_start, self.buf_start) as usize;
        string_from_segment(self.buf_start, head_len)
    }

    fn move_gap_to(&mut self, offset: isize) {
        let gap_len = self.gap_len();
        let new_pos = unsafe { self.buf_start.offset(offset) };

        let diff = ptr_diff(new_pos, self.gap_start);

        if diff == 0 { return; }

        if diff < 0 {
            unsafe {
                self.gap_start = new_pos;
                self.gap_end = self.gap_start.offset(gap_len);
                libc::memmove(self.gap_end as *mut libc::c_void,
                              self.gap_start as *mut libc::c_void,
                              diff.abs() as usize);
            }
        } else {
            unsafe {
                self.gap_end = self.gap_end.offset(diff);
                self.gap_start = self.gap_start.offset(diff);
                libc::memmove(new_pos as *mut libc::c_void,
                              self.gap_start as *mut libc::c_void,
                              diff as usize);
            }
        }
    }

    fn tail(&self) -> String {
        let tail_len = ptr_diff(self.buf_end, self.gap_end) as usize;
        string_from_segment(self.gap_end, tail_len)
    }
}

impl fmt::Display for GapBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.head(), self.tail())
    }
}

impl Drop for GapBuffer {
    fn drop(&mut self) {
        unsafe { libc::free(self.buf_start as *mut libc::c_void); }
    }
}

fn ptr_to_isize(p: *const u8) -> isize {
    unsafe { mem::transmute::<*const u8, isize>(p) }
}

fn ptr_diff(p: *const u8, q: *const u8) -> isize {
    ptr_to_isize(p) - ptr_to_isize(q)
}

fn string_from_segment(start: *mut u8, len: usize) -> String {
    let mut s = String::with_capacity(len);
    let tmp = unsafe { String::from_raw_parts(start, len, len) };
    s.push_str(&tmp);
    mem::forget(tmp);
    s
}


#[cfg(test)]
mod tests {
    use super::GapBuffer;


    #[test]
    fn insert_str_1() {
        let gap_buf = buf_from_str("12345678");
        let text = gap_buf.to_string();
        assert!(text == "12345678");
    }

    #[test]
    fn insert_str_2() {
        let mut gap_buf = buf_from_str("12345678");
        gap_buf.insert_str(7, "9");

        let text = gap_buf.to_string();
        assert!(text == "123456798");
    }

    #[test]
    fn insert_str_3() {
        let mut gap_buf = buf_from_str("12345678");
        gap_buf.insert_str(0, "0");

        let text = gap_buf.to_string();
        assert!(text == "012345678");
    }

    #[test]
    fn insert_str_4() {
        let mut gap_buf = buf_from_str("0123456789.0123456789.0123456789");
        gap_buf.insert_str(0, "9876543210.");

        let text = gap_buf.to_string();
        assert!(text == "9876543210.0123456789.0123456789.0123456789");
    }

    #[test]
    fn insert_str_5() {
        let mut gap_buf = buf_from_str("0123456789.0123456789.0123456789");
        gap_buf.insert_str(11, "9876543210.");

        let text = gap_buf.to_string();
        assert!(text == "0123456789.9876543210.0123456789.0123456789");
    }

    #[test]
    fn remove_1() {
        let mut gap_buf = buf_from_str("12345678");
        gap_buf.remove(0..8);

        let text = gap_buf.to_string();
        assert!(text == "");
    }

    #[test]
    fn remove_2() {
        let mut gap_buf = buf_from_str("12345678");
        gap_buf.remove(0..1);

        let text = gap_buf.to_string();
        assert!(text == "2345678");
    }

    #[test]
    fn remove_3() {
        let mut gap_buf = buf_from_str("12345678");
        gap_buf.remove(7..8);

        let text = gap_buf.to_string();
        assert!(text == "1234567");
    }

    #[test]
    fn remove_4() {
        let mut gap_buf = buf_from_str("12345678");
        gap_buf.remove(3..6);

        let text = gap_buf.to_string();
        assert!(text == "12378");
    }

    #[test]
    #[should_panic]
    fn remove_5() {
        let mut gap_buf = buf_from_str("12345678");
        gap_buf.remove(0..9);
    }

    fn buf_from_str(s: &str) -> GapBuffer {
        let mut buf = GapBuffer::with_capacity(s.len());
        buf.insert_str(0, s);
        buf
    }
}

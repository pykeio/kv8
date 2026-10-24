// Copyright 2019-2021 the Deno authors. All rights reserved. MIT license.
use crate::ArrayBuffer;
use crate::Local;
use crate::PinScope;
use crate::TypedArray;
use crate::binding::v8__TypedArray__kMaxByteLength;
use crate::support::size_t;

unsafe extern "C" {
  fn v8__TypedArray__Length(this: *const TypedArray) -> size_t;
}

impl TypedArray {
  /// The largest supported typed array byte size. Each subclass defines a
  /// type-specific max_length for the maximum length that can be passed to new.
  pub const MAX_BYTE_LENGTH: usize = v8__TypedArray__kMaxByteLength;

  /// Number of elements in this typed array
  /// (e.g. for Int16Array, |ByteLength|/2).
  #[inline(always)]
  pub fn length(&self) -> usize {
    unsafe { v8__TypedArray__Length(self) }
  }
}

macro_rules! typed_array {
  ($name:ident; new = $new:ident, max_length = $max_length:ident) => {
    use crate::$name;

    unsafe extern "C" {
      fn $new(
        buf_ptr: *const ArrayBuffer,
        byte_offset: usize,
        length: usize,
      ) -> *const $name;
    }

    impl $name {
      #[inline(always)]
      pub fn new<'s>(
        scope: &PinScope<'s, '_>,
        buf: Local<ArrayBuffer>,
        byte_offset: usize,
        length: usize,
      ) -> Option<Local<'s, $name>> {
        unsafe { scope.cast_local(|_| $new(&*buf, byte_offset, length)) }
      }

      #[doc = concat!("The largest ", stringify!($name), " size that can be constructed using `new`.")]
      pub const MAX_LENGTH: usize = crate::binding::$max_length;
    }
  };
}

typed_array!(Uint8Array; new = v8__Uint8Array__New, max_length = v8__Uint8Array__kMaxLength);
typed_array!(Uint8ClampedArray; new = v8__Uint8ClampedArray__New, max_length = v8__Uint8ClampedArray__kMaxLength);
typed_array!(Int8Array; new = v8__Int8Array__New, max_length = v8__Int8Array__kMaxLength);
typed_array!(Uint16Array; new = v8__Uint16Array__New, max_length = v8__Uint16Array__kMaxLength);
typed_array!(Int16Array; new = v8__Int16Array__New, max_length = v8__Int16Array__kMaxLength);
typed_array!(Uint32Array; new = v8__Uint32Array__New, max_length = v8__Uint32Array__kMaxLength);
typed_array!(Int32Array; new = v8__Int32Array__New, max_length = v8__Int32Array__kMaxLength);
typed_array!(Float16Array; new = v8__Float16Array__New, max_length = v8__Float16Array__kMaxLength);
typed_array!(Float32Array; new = v8__Float32Array__New, max_length = v8__Float32Array__kMaxLength);
typed_array!(Float64Array; new = v8__Float64Array__New, max_length = v8__Float64Array__kMaxLength);
typed_array!(BigUint64Array; new = v8__BigUint64Array__New, max_length = v8__BigUint64Array__kMaxLength);
typed_array!(BigInt64Array; new = v8__BigInt64Array__New, max_length = v8__BigInt64Array__kMaxLength);

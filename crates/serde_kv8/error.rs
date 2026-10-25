// Copyright 2018-2026 the Deno authors. MIT license.

use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
  Message(String),
  ExpectedBoolean(&'static str),
  ExpectedInteger(&'static str),
  ExpectedNumber(&'static str),
  ExpectedString(&'static str),
  ExpectedArray(&'static str),
  ExpectedMap(&'static str),
  ExpectedEnum(&'static str),
  ExpectedObject(&'static str),
  ExpectedBuffer(&'static str),
  ExpectedDetachable(&'static str),
  ExpectedExternal(&'static str),
  #[cfg(feature = "bigint")]
  ExpectedBigInt(&'static str),
  ExpectedUtf8,
  ExpectedLatin1,
  UnsupportedType,
  LengthMismatch(usize, usize),
  RecursionLimitExceeded,
  V8Exception,
  ResizableBackingStoreNotSupported,
  Custom(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Message(s) => f.write_str(s),
      Self::ExpectedBoolean(t) => {
        f.write_fmt(format_args!("invalid type; expected: boolean, got: {t}"))
      }
      Self::ExpectedInteger(t) => {
        f.write_fmt(format_args!("invalid type; expected: integer, got: {t}"))
      }
      Self::ExpectedNumber(t) => {
        f.write_fmt(format_args!("invalid type; expected: number, got: {t}"))
      }
      Self::ExpectedString(t) => {
        f.write_fmt(format_args!("invalid type; expected: string, got: {t}"))
      }
      Self::ExpectedArray(t) => {
        f.write_fmt(format_args!("invalid type; expected: array, got: {t}"))
      }
      Self::ExpectedMap(t) => {
        f.write_fmt(format_args!("invalid type; expected: map, got: {t}"))
      }
      Self::ExpectedEnum(t) => {
        f.write_fmt(format_args!("invalid type; expected: enum, got: {t}"))
      }
      Self::ExpectedObject(t) => {
        f.write_fmt(format_args!("invalid type; expected: object, got: {t}"))
      }
      Self::ExpectedBuffer(t) => {
        f.write_fmt(format_args!("invalid type; expected: buffer, got: {t}"))
      }
      Self::ExpectedDetachable(t) => f.write_fmt(format_args!(
        "invalid type; expected: detachable, got: {t}"
      )),
      Self::ExpectedExternal(t) => {
        f.write_fmt(format_args!("invalid type; expected: external, got: {t}"))
      }
      #[cfg(feature = "bigint")]
      Self::ExpectedBigInt(t) => {
        f.write_fmt(format_args!("invalid type; expected: bigint, got: {t}"))
      }
      Self::ExpectedUtf8 => f.write_str("invalid type; expected: utf8"),
      Self::ExpectedLatin1 => f.write_str("invalid type; expected: latin1"),
      Self::UnsupportedType => f.write_str("unsupported type"),
      Self::LengthMismatch(got, expected) => f.write_fmt(format_args!(
        "length mismatch, got: {got}, expected: {expected}"
      )),
      Self::RecursionLimitExceeded => f.write_str("recursion limit exceeded"),
      Self::V8Exception => f.write_str("exception during value conversion"),
      Self::ResizableBackingStoreNotSupported => {
        f.write_str("can't create slice from resizable ArrayBuffer")
      }
      Self::Custom(e) => e.fmt(f),
    }
  }
}

impl serde_core::ser::Error for Error {
  fn custom<T: Display>(msg: T) -> Self {
    Error::Message(msg.to_string())
  }
}

impl serde_core::de::Error for Error {
  fn custom<T: Display>(msg: T) -> Self {
    Error::Message(msg.to_string())
  }
}

impl std::error::Error for Error {}

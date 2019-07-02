//! dbase is rust library meant to read and write
//!
//! # Reading
//!
//! To Read the whole file at once you should use the [read](fn.read.html) function.
//!
//! Once you have access to the records, you will have to `match` against the real
//! [FieldValue](enum.FieldValue.html)
//!
//! # Examples
//!
//! ```
//! use dbase::FieldValue;
//! let records = dbase::read("tests/data/line.dbf").unwrap();
//! for record in records {
//!     for (name, value) in record {
//!         println!("{} -> {:?}", name, value);
//!         match value {
//!             FieldValue::Character(string) => println!("Got string: {}", string),
//!             FieldValue::Numeric(value) => println!("Got numeric value of  {}", value),
//!             _ => {}
//!         }
//!     }
//!}
//! ```
//!
//! You can also create a [Reader](reading/Reader.struct.html) and iterate over the records.
//!
//! ```
//! let reader = dbase::Reader::from_path("tests/data/line.dbf").unwrap();
//! for record_result in reader {
//!     let record = record_result.unwrap();
//!     for (name, value) in record {
//!         println!("name: {}, value: {:?}", name, value);
//!     }
//! }
//!
//! ```
//!

//https://dbfviewer.com/dbf-file-structure/

extern crate byteorder;

pub use reading::{read, FieldValueReader, Reader, Record};
pub use record::field::{Date, FieldType, FieldValue, SizeableField};
pub use writing::{write_to, write_to_path, Writer};

mod header;
mod reading;
mod record;
mod writing;

/// Errors that may happen when reading a .dbf
#[derive(Debug)]
pub enum Error {
    /// Wrapper of `std::io::Error` to forward any reading/writing error
    IoError(std::io::Error),
    /// Wrapper to forward errors whe trying to parse a float from the file
    ParseFloatError(std::num::ParseFloatError),
    /// Wrapper to forward errors whe trying to parse an integer value from the file
    ParseIntError(std::num::ParseIntError),
    /// The Field as an invalid FieldType
    InvalidFieldType(char),
    InvalidDate,
    FieldLengthTooLong,
    FieldNameTooLong,
    FieldTypeNotAsExpected(FieldType)
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(p: std::num::ParseFloatError) -> Self {
        Error::ParseFloatError(p)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(p: std::num::ParseIntError) -> Self {
        Error::ParseIntError(p)
    }
}

pub trait DBaseRecord {
    fn from_field_reader<T>(r: &mut T) -> Result<Self, Error>
    where
        Self: Sized,
        T: FieldValueReader;

    fn fields_info() -> Vec<(String, FieldType)>;

    fn fields_length(&self, fields_length: &mut [u8]);

    fn fields_values(self, fields_value: &mut [FieldValue]);
}

#[macro_export]
macro_rules! extract_field_value {
    ($field_value_option:expr, FieldValue::$expected_variant:ident) => {
        match $field_value_option {
            Some(Ok(FieldValue::$expected_variant(value))) => value,
            Some(Ok(v)) => return Err(Error::FieldTypeNotAsExpected(v.field_type())),
            Some(Err(e)) => return Err(e),
            None => panic!("not enough members"),
        }
    };
}

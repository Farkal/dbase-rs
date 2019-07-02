//! Module with all structs & functions charged of writing .dbf file content
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use byteorder::WriteBytesExt;

use header::Header;
use reading::TERMINATOR_VALUE;
use record::RecordFieldInfo;
use {DBaseRecord, FieldValue};
use {Error, Record};

/// A dbase file ends with this byte
const FILE_TERMINATOR: u8 = 0x1A;

/// Struct that handles the writing of records to any destination
/// that supports the `Write` trait
pub struct Writer<T: Write> {
    dest: T,
}

#[allow(dead_code)]
// Just here for documentation purposes, we only write not_deleted flag
const DELETION_FLAG_DELETED: u8 = 0x2a;
const DELETION_FLAG_NOT_DELETED: u8 = 0x20;

impl<T: Write> Writer<T> {
    /// Creates a new Writer
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// let writer = dbase::Writer::new(Cursor::new(Vec::<u8>::new()));
    /// ```
    pub fn new(dest: T) -> Self {
        Self { dest }
    }

    /// Writes the collection of records
    ///
    /// # Returns
    /// Returns the `dest` provided when constructing the writer, in case you need it.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let mut fst = dbase::Record::new();
    /// fst.insert("Name".to_string(), dbase::FieldValue::Character("Fallujah".to_string()));
    /// let records = vec![fst];
    ///
    /// let writer = dbase::Writer::new(Cursor::new(Vec::<u8>::new()));
    /// let cursor = writer.write(&records).unwrap();
    /// ```
    pub fn write(mut self, records: &Vec<Record>) -> Result<(T), Error> {
        if records.is_empty() {
            return Ok(self.dest);
        }
        let fields_name: Vec<&String> = records[0].keys().collect();

        let mut fields_info = Vec::<RecordFieldInfo>::with_capacity(fields_name.len());
        for (field_name, field_value) in &records[0] {
            let field_length = field_value.size_in_bytes();

            if field_length > std::u8::MAX as usize {
                return Err(Error::FieldLengthTooLong);
            }

            fields_info.push(RecordFieldInfo::with_length(
                field_name.to_owned(),
                field_value.field_type(),
                field_length as u8,
            ));
        }

        // TODO check that for the same field, the field type is the same
        for record in &records[1..records.len()] {
            for (field_name, record_info) in fields_name.iter().zip(&mut fields_info) {
                let field_value = record.get(*field_name).unwrap(); // TODO: Should return an Err()
                let field_length = field_value.size_in_bytes();
                if field_length > std::u8::MAX as usize {
                    return Err(Error::FieldLengthTooLong);
                }

                record_info.field_length = record_info.field_length.max(field_length as u8);
            }
        }

        self.write_header_and_fields_info(&fields_info, records.len())?;

        let mut fields_values = (0..fields_info.len())
            .map(|_i| FieldValue::Numeric(0.0))
            .collect::<Vec<FieldValue>>();

        for record in records {
            for (i, field_name) in fields_name.iter().enumerate() {
                fields_values[i] = record.get(*field_name).unwrap().clone();
            }
            self.write_field_values(&fields_info, &fields_values)?;
        }

        self.dest.write_u8(FILE_TERMINATOR)?;
        Ok(self.dest)
    }

    pub fn write_records<R: DBaseRecord>(mut self, records: Vec<R>) -> Result<T, Error> {
        if records.is_empty() {
            return Ok(self.dest);
        }

        let mut fields_infos = R::fields_info()
            .into_iter()
            .map(|(name, field_type)| RecordFieldInfo::new(name, field_type))
            .collect::<Vec<RecordFieldInfo>>();

        // Get the max field length for each records & fields
        let mut fields_sizes = vec![0u8; fields_infos.len()];
        for record in &records {
            record.fields_length(&mut fields_sizes);
            fields_infos
                .iter_mut()
                .zip(&fields_sizes)
                .for_each(|(info, size)| {
                    info.field_length = info.field_length.max(*size);
                });
        }

        self.write_header_and_fields_info(&fields_infos, records.len())?;

        let mut fields_values = (0..fields_infos.len())
            .map(|_i| FieldValue::Numeric(0.0))
            .collect::<Vec<FieldValue>>();

        for record in records {
            record.fields_values(&mut fields_values);
            self.write_field_values(&fields_infos, &fields_values)?;
        }
        self.dest.write_u8(FILE_TERMINATOR)?;
        Ok(self.dest)
    }

    fn write_header_and_fields_info(
        &mut self,
        fields_info: &Vec<RecordFieldInfo>,
        num_records: usize,
    ) -> Result<(), Error> {
        let offset_to_first_record =
            Header::SIZE + (fields_info.len() * RecordFieldInfo::SIZE) + std::mem::size_of::<u8>();
        let size_of_record = fields_info
            .iter()
            .fold(0u16, |s, ref info| s + info.field_length as u16);
        let mut header = Header::new(
            num_records as u32,
            offset_to_first_record as u16,
            size_of_record,
        );

        header.write_to(&mut self.dest)?;
        for record_info in fields_info {
            record_info.write_to(&mut self.dest)?;
        }
        self.dest.write_u8(TERMINATOR_VALUE)?;
        Ok(())
    }

    fn write_field_values(
        &mut self,
        fields_infos: &Vec<RecordFieldInfo>,
        fields_values: &[FieldValue],
    ) -> Result<(), Error> {
        self.dest.write_u8(DELETION_FLAG_NOT_DELETED)?;
        for (field_value, record_info) in fields_values.iter().zip(fields_infos.iter()) {
            if field_value.field_type() != record_info.field_type {
                panic!(
                    "Field Value type given '{:?}' does not match expected field type '{:?}'",
                    field_value.field_type(),
                    record_info.field_type
                );
            }

            let bytes_written = field_value.write_to(&mut self.dest)?;
            if bytes_written > std::u8::MAX as usize {
                panic!("FieldValue was too long");
            }

            if bytes_written > record_info.field_length as usize {
                panic!("record length was miscalculated");
            }

            let mut bytes_to_pad = record_info.field_length - bytes_written as u8;
            while bytes_to_pad > 0 {
                //FIXME I think the padded byte values changes depending on the FieldType
                self.dest.write_u8(0x20)?; // pad with space
                bytes_to_pad -= 1;
            }
        }
        Ok(())
    }
}

impl Writer<BufWriter<File>> {
    /// Creates a new writer that will write the to a new filed
    /// # Examples
    /// ```
    /// let writer = dbase::Writer::from_path("new_records.dbf").unwrap();
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        Ok(Writer::new(BufWriter::new(File::create(path)?)))
    }
}

/// Writes the records to the dest
///
/// # Examples
///
/// ```
/// use std::io::Cursor;
///
/// let mut fst = dbase::Record::new();
/// fst.insert("Name".to_string(), dbase::FieldValue::Character("The Flesh PrevailsFallujah".to_string()));
/// fst.insert("Price".to_string(), dbase::FieldValue::Numeric(9.99));
/// let records = vec![fst];
///
/// let cursor = Cursor::new(Vec::<u8>::new());
/// let cursor = dbase::write_to(&records, cursor).unwrap();
/// ```
pub fn write_to<T: Write>(records: &Vec<Record>, dest: T) -> Result<T, Error> {
    let writer = Writer::new(dest);
    writer.write(&records)
}

/// Writes all the records to the a new file at path
///
/// # Examples
///
/// ```
/// let mut fst = dbase::Record::new();
/// fst.insert("Name".to_string(), dbase::FieldValue::Character("The Flesh PrevailsFallujah".to_string()));
/// fst.insert("Price".to_string(), dbase::FieldValue::Numeric(9.99));
/// let records = vec![fst];
///
/// dbase::write_to_path(&records, "albums.dbf").unwrap();
/// ```
pub fn write_to_path<P: AsRef<Path>>(records: &Vec<Record>, path: P) -> Result<(), Error> {
    let writer = Writer::from_path(path)?;
    writer.write(&records)?;
    Ok(())
}

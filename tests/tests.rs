const LINE_DBF: &str = "./tests/data/line.dbf";

extern crate dbase;

use std::collections::HashMap;
use std::io::{Cursor, Seek, SeekFrom};
use dbase::{DBaseRecord, FieldValue, Error, FieldValueReader};


#[test]
fn test_simple_file() {
    let records = dbase::read(LINE_DBF).unwrap();
    assert_eq!(records.len(), 1);
    let mut expected_fields = HashMap::new();
    expected_fields.insert("name".to_owned(), dbase::FieldValue::Character("linestring1".to_owned()));

    assert_eq!(records[0], expected_fields);
}

#[test]
fn test_read_write_simple_file() {
    let mut expected_fields = HashMap::new();
    expected_fields.insert("name".to_owned(), dbase::FieldValue::Character("linestring1".to_owned()));

    use std::fs::File;
    let records = dbase::read(LINE_DBF).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0], expected_fields);

    let file = File::create("lol.dbf").unwrap();
    let writer = dbase::Writer::new(file);
    writer.write(&records).unwrap();

    let records = dbase::read("lol.dbf").unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0], expected_fields);
}


struct ArtistRecord {
    name: String,
}


impl DBaseRecord for ArtistRecord {
    fn from_field_reader<T: FieldValueReader>(r: &mut T) -> Result<Self, Error> {
        let name = match r.read_next_value() {
            Some(Ok(FieldValue::Character(value))) => value,
            Some(Ok(_)) => panic!("value mismatch"),
            Some(Err(e)) => return Err(e),
            None => panic!("not enough members")
        };
/*
        let age = match r.read_next_value() {
            Some(Ok(FieldValue::Numeric(value))) => value,
            Some(Ok(_)) => panic!("value mismatch"),
            Some(Err(e)) => return Err(e),
            None => panic!("not enough members")
        };*/


        Ok(Self {name})
    }
}


#[test]
fn from_scratch() {
    let mut fst = dbase::Record::new();
    fst.insert("Name".to_string(), dbase::FieldValue::Character("Fallujah".to_string()));

    let mut scnd = dbase::Record::new();
    scnd.insert("Name".to_string(), dbase::FieldValue::Character("Beyond Creation".to_string()));

    let records = vec![fst, scnd];

    let cursor = Cursor::new(Vec::<u8>::new());
    let writer = dbase::Writer::new(cursor);
    let mut cursor = writer.write(&records).unwrap();
    cursor.seek(SeekFrom::Start(0)).unwrap();

    let reader = dbase::Reader::new(cursor).unwrap();
    let read_records = reader.read_as::<ArtistRecord>().unwrap();

    assert_eq!(read_records.len(), 2);

    assert_eq!(read_records[0].name, "Fallujah");
    assert_eq!(read_records[1].name, "Beyond Creation");
}


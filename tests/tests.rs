const LINE_DBF: &str = "./tests/data/line.dbf";

extern crate dbase;

use std::collections::HashMap;
use std::io::{Cursor, Seek, SeekFrom};
use dbase::{DBaseRecord, FieldValue, FieldType, Error, FieldValueReader, SizeableField};


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


struct AlbumRecord {
    name: String,
    artist: String,
}


impl DBaseRecord for AlbumRecord {
    fn from_field_reader<T: FieldValueReader>(r: &mut T) -> Result<Self, Error> {
        let name = match r.read_next_value() {
            Some(Ok(FieldValue::Character(value))) => value,
            Some(Ok(_)) => panic!("value mismatch"),
            Some(Err(e)) => return Err(e),
            None => panic!("not enough members")
        };


        let artist = match r.read_next_value() {
            Some(Ok(FieldValue::Character(value))) => value,
            Some(Ok(_)) => panic!("value mismatch"),
            Some(Err(e)) => return Err(e),
            None => panic!("not enough members")
        };


        Ok(Self {name, artist})
    }

    fn fields_info() -> Vec<(String, FieldType)> {
        vec![
            ("name".to_owned(), FieldType::Character),
            ("artist".to_owned(), FieldType::Character),
        ]
    }

    fn fields_length(&self, fields_length: &mut [u8]) {
        fields_length[0] = self.name.dbase_size_of();
        fields_length[1] = self.artist.dbase_size_of();
    }

    fn fields_values(self, fields_value: &mut [FieldValue]) {
        fields_value[0] = FieldValue::Character(self.name);
        fields_value[1] = FieldValue::Character(self.artist);
    }
}


#[test]
fn from_scratch() {
    let mut fst = dbase::Record::new();
    fst.insert("Name".to_string(), dbase::FieldValue::Character("The Flesh Prevails".to_string()));
    fst.insert("Artist".to_string(), dbase::FieldValue::Character("Fallujah".to_string()));

    let mut scnd = dbase::Record::new();
    scnd.insert("Name".to_string(), dbase::FieldValue::Character("Earthborn Evolution".to_string()));
    scnd.insert("Artist".to_string(), dbase::FieldValue::Character("Beyond Creation".to_string()));

    let records = vec![fst, scnd];

    let cursor = Cursor::new(Vec::<u8>::new());
    let writer = dbase::Writer::new(cursor);
    let mut cursor = writer.write(&records).unwrap();
    cursor.seek(SeekFrom::Start(0)).unwrap();

    let reader = dbase::Reader::new(cursor).unwrap();
    let read_records = reader.read_as::<AlbumRecord>().unwrap();

    assert_eq!(read_records.len(), 2);

    assert_eq!(read_records[0].artist, "Fallujah");
    assert_eq!(read_records[1].artist, "Beyond Creation");

    assert_eq!(read_records[0].name, "The Flesh Prevails");
    assert_eq!(read_records[1].name, "Earthborn Evolution");
}


#[test]
fn from_scratch2() {

    let records = vec![
        AlbumRecord {name: "The Flesh Prevails".to_string(), artist: "Fallujah".to_string()},
        AlbumRecord {name: "Earthborn Evolution".to_string(), artist: "Beyond Creation".to_string()}
    ];

    let writer = dbase::Writer::new(Cursor::new(Vec::<u8>::new()));
    let mut cursor = writer.write_records(records).unwrap();
    cursor.seek(SeekFrom::Start(0)).unwrap();

    let reader = dbase::Reader::new(cursor).unwrap();
    let read_records = reader.read_as::<AlbumRecord>().unwrap();

    assert_eq!(read_records.len(), 2);

    assert_eq!(read_records[0].name, "The Flesh Prevails");
    assert_eq!(read_records[0].artist, "Fallujah");
    assert_eq!(read_records[1].name, "Earthborn Evolution");
    assert_eq!(read_records[1].artist, "Beyond Creation");
}




// https://achdevguide.nacha.org/ach-file-details

use crate::string_reader::StringReader;
use log::{error, info};
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

#[derive(Debug)]
pub struct AchError {}

pub trait AchRecord: std::fmt::Debug {}

#[derive(Debug)]
pub(crate) enum AchRecordType {
    Header,
    CompanyBatchHeader,
    EntryDetail,
    Addenda,
    CompanyBatchTrailer,
    Trailer,
    Unknown, // Only really used for defaults and the From<&str> impl
}

impl From<&str> for AchRecordType {
    fn from(s: &str) -> Self {
        match s {
            "Header" => AchRecordType::Header,
            "CompanyBatchHeader" => AchRecordType::CompanyBatchHeader,
            "EntryDetail" => AchRecordType::EntryDetail,
            "Addenda" => AchRecordType::Addenda,
            "CompanyBatchTrailer" => AchRecordType::CompanyBatchTrailer,
            "Trailer" => AchRecordType::Trailer,
            _ => AchRecordType::Unknown,
        }
    }
}

fn checked_read_line(file: &mut BufReader<File>) -> Result<String, AchError> {
    let mut line = "".to_string();
    match file.read_line(&mut line) {
        Ok(s) => {
            info!("Successfully read {} bytes to line", s)
        }
        Err(e) => {
            error!("Could not read line: {}", e);
            return Err(AchError {});
        }
    };

    Ok(line)
}

fn checked_read_type(file: &mut BufReader<File>) -> Result<char, AchError> {
    let mut record_type_code: [u8; 1] = [0];
    match file.read_exact(&mut record_type_code) {
        Ok(_) => {
            info!("Successfully read record type code from file")
        }
        Err(e) => {
            error!("Could not read record type code: {}", e);
            return Err(AchError {});
        }
    };

    Ok(record_type_code[0] as char)
}

impl Display for AchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            _ => write!(f, "ach error"),
        }
    }
}

impl std::error::Error for AchError {}

#[derive(Debug, Clone)]
pub struct AchFile {
    header: Header,
    records: Vec<CompanyBatch>,
    trailer: Trailer,
}

#[test]
fn test_achfile_clone() {
    let ach1 = AchFile {
        header: Header {
            record_type_code: Default::default(),
            priority_code: Default::default(),
            immediate_dest: Field::from("-test_case1-"),
            immediate_orig: Default::default(),
            file_creation_date: Default::default(),
            file_creation_time: Default::default(),
            file_id_modifier: Default::default(),
            record_size: Default::default(),
            blocking_factor: Default::default(),
            format_code: Default::default(),
            immediate_dest_name: Default::default(),
            immediate_orig_name: Default::default(),
            reference_code: Default::default(),
        },
        records: vec![CompanyBatch {
            batch_header: Default::default(),
            batch_records: vec![EntryDetail {
                record_type_code: Default::default(),
                transactions_code: Default::default(),
                receiving_dfi_id: Field::from("-test_case2-"),
                check_digit: Default::default(),
                dfi_account: Default::default(),
                amount: Default::default(),
                individual_id: Default::default(),
                individual_name: Default::default(),
                discretionary_data: Default::default(),
                addenda_indicator: Default::default(),
                trace: Default::default(),
                addenda: vec![Addenda {
                    record_type_code: Default::default(),
                    addenda_type: Default::default(),
                    payment_related_info: Field::from("-test_case3-"),
                    addenda_sequence: Default::default(),
                    batch: Default::default(),
                }],
            }],
            batch_trailer: Default::default(),
        }],
        trailer: Default::default(),
    };
    let ach2 = ach1.clone();

    assert_eq!(format!("{:?}", ach1), format!("{:?}", ach2))
}

impl AchFile {
    pub fn len(&self) -> usize {
        let mut len = 2; // Start with +1 for each, header and footer
        for record in &self.records {
            len += record.len();
        }
        len
    }

    pub fn split(&self, company_ids: Vec<String>) -> Result<(AchFile, AchFile), AchError> {
        let mut ach_files = (AchFile::default(), AchFile::default());
        ach_files.0.header = self.header.clone();
        ach_files.1.header = self.header.clone();

        for company_batch in &self.records {
            if company_ids.contains(&company_batch.batch_header.company_id.content) {
                println!("found company id {}", company_batch.batch_header.company_id)
            } else {
                //println!("company id {} does not match any of {:?}", company_batch.batch_header.company_id, company_ids)
            }
        }

        Ok(ach_files)
    }
}

impl Default for AchFile {
    fn default() -> Self {
        AchFile {
            header: Default::default(),
            records: vec![],
            trailer: Default::default(),
        }
    }
}

impl Display for AchFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.header)?;
        for record in &self.records {
            writeln!(f, "{}", record)?;
        }

        writeln!(f, "{}", self.trailer)?;

        for _ in 0..(10 - (self.len() % 10)) {
            writeln!(f, "{}", "9".repeat(94))?;
        }
        write!(f, "")
    }
}

impl TryFrom<&Path> for AchFile {
    type Error = AchError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        info!("Trying to create AchFile from file");
        let mut file = BufReader::new(match File::open(path) {
            Ok(file) => file,
            Err(e) => {
                error!("Could not open file: {}", e);
                return Err(Self::Error {});
            }
        });

        let mut header = Header {
            ..Default::default()
        };
        let mut records: Vec<CompanyBatch> = vec![];
        let trailer;

        loop {
            let record_type_code = checked_read_type(&mut file)?;

            match record_type_code {
                '1' => header = Header::try_from(&mut file)?,
                '5' => records.push(CompanyBatch::try_from(&mut file)?),
                '9' => {
                    trailer = Trailer::try_from(&mut file)?;
                    break; // Assume end of file, break
                }
                t => {
                    error!("Unrecognized record type code! found: {}", t);
                    return Err(AchError {});
                }
            }
        }

        Ok(AchFile {
            header,
            records,
            trailer,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Field {
    content: String,
    size: usize,
    left_justified: bool,
}

impl Field {
    fn left_just(mut self, justification: bool) -> Self {
        self.left_justified = justification;
        self
    }
}

impl PartialEq<&str> for Field {
    fn eq(&self, other: &&str) -> bool {
        self.content == *other
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        if self.left_justified {
            out.push_str(&*self.content);
            for _ in 0..(self.size - self.content.len()) {
                out.push(' ');
            }
        } else {
            for _ in 0..(self.size - self.content.len()) {
                out.push(' ');
            }
            out.push_str(&*self.content);
        }

        write!(f, "{}", out)
    }
}

#[test]
fn test_field_display() {
    let mut field = Field::from("lorem ipsum dolor sit amet");
    assert_eq!(format!("{}", field), "lorem ipsum dolor sit amet");

    field.size = 30;
    assert_eq!(format!("{}", field), "    lorem ipsum dolor sit amet");
    field.left_justified = true;
    assert_eq!(format!("{}", field), "lorem ipsum dolor sit amet    ")
}

impl Default for Field {
    fn default() -> Self {
        Self {
            content: "".to_string(),
            size: 0,
            left_justified: false,
        }
    }
}

impl From<&str> for Field {
    fn from(content: &str) -> Self {
        Field {
            content: content.to_string(),
            size: content.len(),
            left_justified: false,
        }
    }
}

impl From<u32> for Field {
    fn from(size: u32) -> Self {
        Field {
            content: String::new(),
            size: size as usize,
            left_justified: false,
        }
    }
}

impl From<String> for Field {
    fn from(string: String) -> Self {
        Self {
            size: string.len(),
            content: string,
            left_justified: false,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct Header {
    record_type_code: Field,    // content: "1", size: 1
    priority_code: Field,       // content:  "01", size: 2
    immediate_dest: Field,      // size: 10
    immediate_orig: Field,      // size: 10
    file_creation_date: Field,  // size: 6
    file_creation_time: Field,  // size: 4
    file_id_modifier: Field,    // size: 1
    record_size: Field,         // content: "094", size: 3
    blocking_factor: Field,     // content: "10", size: 2
    format_code: Field,         // content: "1", size: 1
    immediate_dest_name: Field, // size: 23
    immediate_orig_name: Field, // size: 23
    reference_code: Field,      // content: "", size: 8
}

impl AchRecord for Header {}

impl Display for Header {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.record_type_code)?;
        write!(f, "{}", self.priority_code)?;
        write!(f, "{}", self.immediate_dest)?;
        write!(f, "{}", self.immediate_orig)?;
        write!(f, "{}", self.file_creation_date)?;
        write!(f, "{}", self.file_creation_time)?;
        write!(f, "{}", self.file_id_modifier)?;
        write!(f, "{}", self.record_size)?;
        write!(f, "{}", self.blocking_factor)?;
        write!(f, "{}", self.format_code)?;
        write!(f, "{}", self.immediate_dest_name)?;
        write!(f, "{}", self.immediate_orig_name)?;
        write!(f, "{}", self.reference_code)
    }
}

#[test]
fn test_header_display() {
    let header = Header {
        record_type_code: Field::from("1"),
        priority_code: Field::from("01"),
        immediate_dest: Field::from("-imm_dest-"),
        immediate_orig: Field::from("-imm_orig-"),
        file_creation_date: Field::from("112233"),
        file_creation_time: Field::from("1122"),
        file_id_modifier: Field::from("1"),
        record_size: Field::from("094"),
        blocking_factor: Field::from("10"),
        format_code: Field::from("1"),
        immediate_dest_name: Field::from("-!immediate_dest_name!-"),
        immediate_orig_name: Field::from("-!immediate_orig_name!-"),
        reference_code: Field::from(8),
    };
    assert_eq!(format!("{}", header), "101-imm_dest--imm_orig-11223311221094101-!immediate_dest_name!--!immediate_orig_name!-        ")
}

impl From<StringReader> for Header {
    fn from(mut reader: StringReader) -> Self {
        Header {
            record_type_code: Field::from("1"),
            priority_code: Field::from(reader.read(2)),
            immediate_dest: Field::from(reader.read(10)),
            immediate_orig: Field::from(reader.read(10)),
            file_creation_date: Field::from(reader.read(6)),
            file_creation_time: Field::from(reader.read(4)),
            file_id_modifier: Field::from(reader.read(1)),
            record_size: Field::from(reader.read(3)),
            blocking_factor: Field::from(reader.read(2)),
            format_code: Field::from(reader.read(1)),
            immediate_dest_name: Field::from(reader.read(23)),
            immediate_orig_name: Field::from(reader.read(23)),
            reference_code: Field::from(8),
        }
    }
}

impl TryFrom<&mut BufReader<File>> for Header {
    type Error = AchError;

    fn try_from(file: &mut BufReader<File>) -> Result<Self, Self::Error> {
        info!("Trying to build Header from file");

        Ok(Header::from(StringReader::new(checked_read_line(
            &mut *file,
        )?)))
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct CompanyBatch {
    batch_header: CompanyBatchHeader,
    batch_records: Vec<EntryDetail>,
    batch_trailer: CompanyBatchTrailer,
}

impl CompanyBatch {
    fn len(&self) -> usize {
        let mut len = 2; // +1 for header and footer
        for record in &self.batch_records {
            len += record.len()
        }
        len
    }
}

impl Display for CompanyBatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.batch_header)?;
        for record in &self.batch_records {
            writeln!(f, "{}", record)?;
        }
        write!(f, "{}", self.batch_trailer)
    }
}

impl TryFrom<&mut BufReader<File>> for CompanyBatch {
    type Error = AchError;

    fn try_from(file: &mut BufReader<File>) -> Result<Self, Self::Error> {
        info!("Trying to build CompanyBatch from file");

        let line = checked_read_line(&mut *file)?;
        let batch_header = CompanyBatchHeader::from(StringReader::new(line));

        let mut batch_records: Vec<EntryDetail> = vec![];
        let mut batch_trailer = CompanyBatchTrailer {
            ..Default::default()
        };

        loop {
            let record_type_code = checked_read_type(&mut *file)?;

            match record_type_code {
                '6' => batch_records.push(EntryDetail::try_from(&mut *file)?),
                '8' => batch_trailer = CompanyBatchTrailer::try_from(&mut *file)?,
                _ => {
                    file.seek_relative(-1).unwrap();
                    break;
                }
            }
        }

        Ok(CompanyBatch {
            batch_header,
            batch_records,
            batch_trailer,
        })
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct CompanyBatchHeader {
    record_type_code: Field,           // content: "5", size: 1
    service_class_code: Field,         // size: 3
    company_name: Field,               // size: 16
    company_discretionary_data: Field, // size: 20
    company_id: Field,                 // size: 10
    sec: Field,                        // size: 3
    entry_desc: Field,                 // size: 10
    company_descriptive_date: Field,   // size: 6
    effective_entry_date: Field,       // size: 6
    settlement_date: Field,            // size: 3
    originator_status_code: Field,     // size: 1
    odfi_id: Field,                    // size: 8
    batch_number: Field,               // size: 7
}

impl AchRecord for CompanyBatchHeader {}

impl Display for CompanyBatchHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.record_type_code)?;
        write!(f, "{}", self.service_class_code)?;
        write!(f, "{}", self.company_name)?;
        write!(f, "{}", self.company_discretionary_data)?;
        write!(f, "{}", self.company_id)?;
        write!(f, "{}", self.sec)?;
        write!(f, "{}", self.entry_desc)?;
        write!(f, "{}", self.company_descriptive_date)?;
        write!(f, "{}", self.effective_entry_date)?;
        write!(f, "{}", self.settlement_date)?;
        write!(f, "{}", self.originator_status_code)?;
        write!(f, "{}", self.odfi_id)?;
        write!(f, "{}", self.batch_number)
    }
}

impl From<StringReader> for CompanyBatchHeader {
    fn from(mut reader: StringReader) -> CompanyBatchHeader {
        CompanyBatchHeader {
            record_type_code: Field::from("5"),
            service_class_code: Field::from(reader.read(3)),
            company_name: Field::from(reader.read(16)),
            company_discretionary_data: Field::from(reader.read(20)),
            company_id: Field::from(reader.read(10)),
            sec: Field::from(reader.read(3)),
            entry_desc: Field::from(reader.read(10)),
            company_descriptive_date: Field::from(reader.read(6)),
            effective_entry_date: Field::from(reader.read(6)),
            settlement_date: Field::from(reader.read(3)),
            originator_status_code: Field::from(reader.read(1)),
            odfi_id: Field::from(reader.read(8)),
            batch_number: Field::from(reader.read(7)),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct EntryDetail {
    record_type_code: Field,   // content: "6", size: 1
    transactions_code: Field,  // size: 2
    receiving_dfi_id: Field,   // size: 8
    check_digit: Field,        // size: 1
    dfi_account: Field,        // size: 17
    amount: Field,             // size: 10
    individual_id: Field,      // size: 15
    individual_name: Field,    // size: 22
    discretionary_data: Field, // size: 2
    addenda_indicator: Field,  // size: 1
    trace: Field,              // size: 15

    addenda: Vec<Addenda>,
}

impl AchRecord for EntryDetail {}

impl EntryDetail {
    fn len(&self) -> usize {
        let mut len = 1; // +1 for basic record
        for _ in &self.addenda {
            len += 1;
        }
        len
    }
}

impl Display for EntryDetail {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.record_type_code)?;
        write!(f, "{}", self.transactions_code)?;
        write!(f, "{}", self.receiving_dfi_id)?;
        write!(f, "{}", self.check_digit)?;
        write!(f, "{}", self.dfi_account)?;
        write!(f, "{}", self.amount)?;
        write!(f, "{}", self.individual_id)?;
        write!(f, "{}", self.individual_name)?;
        write!(f, "{}", self.discretionary_data)?;
        write!(f, "{}", self.addenda_indicator)?;
        write!(f, "{}", self.trace)?;

        if self.addenda_indicator == "1" {
            writeln!(f, "")?;
            let mut written = false;
            for a in &self.addenda {
                if written {
                    writeln!(f, "")?;
                } else {
                    written = true;
                }
                write!(f, "{}", a)?;
            }
        }

        write!(f, "")
    }
}

impl From<StringReader> for EntryDetail {
    fn from(mut reader: StringReader) -> EntryDetail {
        EntryDetail {
            record_type_code: Field::from("6"),
            transactions_code: Field::from(reader.read(2)),
            receiving_dfi_id: Field::from(reader.read(8)),
            check_digit: Field::from(reader.read(1)),
            dfi_account: Field::from(reader.read(17)),
            amount: Field::from(reader.read(10)),
            individual_id: Field::from(reader.read(15)),
            individual_name: Field::from(reader.read(22)),
            discretionary_data: Field::from(reader.read(2)),
            addenda_indicator: Field::from(reader.read(1)),
            trace: Field::from(reader.read(15)),

            addenda: vec![],
        }
    }
}

impl TryFrom<&mut BufReader<File>> for EntryDetail {
    type Error = AchError;

    fn try_from(file: &mut BufReader<File>) -> Result<Self, Self::Error> {
        info!("Trying to build EntryDetail from file");

        let line = checked_read_line(&mut *file)?;
        let mut entry_detail = EntryDetail::from(StringReader::new(line));

        let mut addenda_vec: Vec<Addenda> = vec![];
        loop {
            let record_type_code = checked_read_type(&mut *file)?;

            match record_type_code {
                '7' => addenda_vec.push(Addenda::try_from(&mut *file)?),
                _ => {
                    file.seek_relative(-1).unwrap();
                    break;
                }
            }
        }

        entry_detail.addenda.append(&mut addenda_vec);
        Ok(entry_detail)
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct Addenda {
    record_type_code: Field,     // content: "7", size: 1
    addenda_type: Field,         // size: 2
    payment_related_info: Field, // size: 80
    addenda_sequence: Field,     // size: 4
    batch: Field,                // size: 7
}

impl AchRecord for Addenda {}

impl Display for Addenda {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.record_type_code)?;
        write!(f, "{}", self.addenda_type)?;
        write!(f, "{}", self.payment_related_info)?;
        write!(f, "{}", self.addenda_sequence)?;
        write!(f, "{}", self.batch)
    }
}

impl From<StringReader> for Addenda {
    fn from(mut reader: StringReader) -> Addenda {
        Addenda {
            record_type_code: Field::from("7"),
            addenda_type: Field::from(reader.read(2)),
            payment_related_info: Field::from(reader.read(80)),
            addenda_sequence: Field::from(reader.read(4)),
            batch: Field::from(reader.read(7)),
        }
    }
}

impl TryFrom<&mut BufReader<File>> for Addenda {
    type Error = AchError;

    fn try_from(file: &mut BufReader<File>) -> Result<Self, Self::Error> {
        info!("Trying to build Addenda from file");

        Ok(Addenda::from(StringReader::new(checked_read_line(
            &mut *file,
        )?)))
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct CompanyBatchTrailer {
    record_type_code: Field,        // content: "8", size: 1
    service_class_code: Field,      // size: 3
    entry_and_addenda_count: Field, // size: 6  (sum of [EntryDetail] and [Addenda] since [CompanyBatchHeader])
    entry_hash: Field, // size: 10 (Sum of each [EntryDetail.receiving_dfi_id], left justify)
    total_debit_amount: Field, // size: 12 (Sum of [EntryDetail.amount]s for debits since [CompanyBatchHeader])
    total_credit_amount: Field, // size: 12 (Sum of [EntryDetail.amount]s for credits since [CompanyBatchHeader])
    company_id: Field,          // size: 10
    message_auth_code: Field,   // size: 19
    reserved: Field,            // size: 6
    originating_dfi_id_num: Field, // size: 8
    batch_num: Field,           // size: 7
}

impl AchRecord for CompanyBatchTrailer {}

impl Display for CompanyBatchTrailer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.record_type_code)?;
        write!(f, "{}", self.service_class_code)?;
        write!(f, "{}", self.entry_and_addenda_count)?;
        write!(f, "{}", self.entry_hash)?;
        write!(f, "{}", self.total_debit_amount)?;
        write!(f, "{}", self.total_credit_amount)?;
        write!(f, "{}", self.company_id)?;
        write!(f, "{}", self.message_auth_code)?;
        write!(f, "{}", self.reserved)?;
        write!(f, "{}", self.originating_dfi_id_num)?;
        write!(f, "{}", self.batch_num)
    }
}

impl From<StringReader> for CompanyBatchTrailer {
    fn from(mut reader: StringReader) -> CompanyBatchTrailer {
        CompanyBatchTrailer {
            record_type_code: Field::from("8"),
            service_class_code: Field::from(reader.read(3)),
            entry_and_addenda_count: Field::from(reader.read(6)),
            entry_hash: Field::from(reader.read(10)).left_just(true),
            total_debit_amount: Field::from(reader.read(12)),
            total_credit_amount: Field::from(reader.read(12)),
            company_id: Field::from(reader.read(10)),
            message_auth_code: Field::from(reader.read(19)),
            reserved: Field::from(reader.read(6)),
            originating_dfi_id_num: Field::from(reader.read(8)),
            batch_num: Field::from(reader.read(7)),
        }
    }
}

impl TryFrom<&mut BufReader<File>> for CompanyBatchTrailer {
    type Error = AchError;

    fn try_from(file: &mut BufReader<File>) -> Result<Self, Self::Error> {
        info!("Trying to build CompanyBatchTrailer from file");

        Ok(CompanyBatchTrailer::from(StringReader::new(
            checked_read_line(&mut *file)?,
        )))
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct Trailer {
    record_type_code: Field,        // content: "9", size: 1
    batch_count: Field,             // size: 6 (total count of [CompanyBatchHeader] records)
    block_count: Field,             // size: 6 (a block is defined as 10 records.)
    entry_and_addenda_count: Field, // size: 8 (sum of [EntryDetail] and [Addenda])
    entry_hash: Field,              // size: 10 (sum of [EntryDetail.receiving_dfi_id]s )
    total_debits: Field,            // size: 12 (sum of [EntryDetail.amount]s for debits)
    total_credits: Field,           // size: 12 (sum of [EntryDetail.amount]s for credits)
    reserved: Field,                // size: 39
}

impl AchRecord for Trailer {}

impl Display for Trailer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.record_type_code)?;
        write!(f, "{}", self.batch_count)?;
        write!(f, "{}", self.block_count)?;
        write!(f, "{}", self.entry_and_addenda_count)?;
        write!(f, "{}", self.entry_hash)?;
        write!(f, "{}", self.total_debits)?;
        write!(f, "{}", self.total_credits)?;
        write!(f, "{}", self.reserved)
    }
}

impl From<StringReader> for Trailer {
    fn from(mut reader: StringReader) -> Trailer {
        Trailer {
            record_type_code: Field::from("9"),
            batch_count: Field::from(reader.read(6)),
            block_count: Field::from(reader.read(6)),
            entry_and_addenda_count: Field::from(reader.read(8)),
            entry_hash: Field::from(reader.read(10)),
            total_debits: Field::from(reader.read(12)),
            total_credits: Field::from(reader.read(12)),
            reserved: Field::from(reader.read(39)),
        }
    }
}

impl TryFrom<&mut BufReader<File>> for Trailer {
    type Error = AchError;

    fn try_from(file: &mut BufReader<File>) -> Result<Self, Self::Error> {
        info!("Trying to build Trailer from file");

        Ok(Trailer::from(StringReader::new(checked_read_line(
            &mut *file,
        )?)))
    }
}

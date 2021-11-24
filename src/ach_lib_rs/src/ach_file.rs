// https://achdevguide.nacha.org/ach-file-details

use log::error;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
//use std::io::prelude::*;
use crate::string_reader::StringReader;
use std::io::{self, BufRead};
use std::path::Path;

trait Record<T> {
    fn new(mut reader: StringReader) -> T;
}

#[derive(Debug)]
struct AchError {}

impl Display for AchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            _ => write!(f, "ach error"),
        }
    }
}

impl std::error::Error for AchError {}

struct AchFile {
    header: Header,
    records: Vec<CompanyBatch>,
    trailer: Trailer,
}

impl AchFile {
    fn new(path: &Path) -> Result<Self, AchError> {
        let file = io::BufReader::new(match File::open(path) {
            Ok(file) => file,
            Err(e) => {
                error!("Could not open file: {}", e);
                return Err(AchError {});
            }
        })
        .lines();

        let mut ach = AchFile {
            header: Default::default(),
            records: vec![],
            trailer: Default::default(),
        };

        for line in file {
            let mut line = StringReader::new(match line {
                Ok(l) => l,
                Err(e) => {
                    error!("Error reading file: {}", e);
                    panic!("Error reading file: {}", e);
                }
            });

            match &*line.read(1) {
                "1" => {
                    Header::new(line);
                }
                _ => {}
            };
        }

        todo!("complete this function")
    }
}

struct Field {
    content: String,
    size: u32,
    left_justified: bool,
}

impl Field {
    fn left_just(mut self, justification: bool) -> self {
        self.left_justified = justification
    }
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
            size: content.len() as u32,
            left_justified: false,
        }
    }
}

impl From<u32> for Field {
    fn from(size: u32) -> Self {
        Field {
            content: String::new(),
            size,
            left_justified: false,
        }
    }
}

impl From<String> for Field {
    fn from(string: String) -> Self {
        Self {
            size: string.len() as u32,
            content: string,
            left_justified: false,
        }
    }
}

#[derive(Default)]
struct Header {
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

impl Record<Header> for Header {
    fn new(mut reader: StringReader) -> Header {
        Header {
            record_type_code: Field::from("1"),
            priority_code: Field::from(reader.read(2)),
            immediate_dest: Field::from(reader.read(10)),
            immediate_orig: Field::from(reader.read(10)),
            file_creation_date: Field::from(reader.read(6)),
            file_creation_time: Field::from(reader.read(4)),
            file_id_modifier: Field::from(reader.read(1)),
            record_size: Field::from("094"),
            blocking_factor: Field::from("10"),
            format_code: Field::from("10"),
            immediate_dest_name: Field::from(reader.read(23)),
            immediate_orig_name: Field::from(reader.read(23)),
            reference_code: Field::new_empty(8),
        }
    }
}

#[derive(Default)]
struct CompanyBatch {
    batch_header: CompanyBatchHeader,
    batch_records: Vec<EntryDetail>,
    batch_trailer: CompanyBatchTrailer,
}

#[derive(Default)]
struct CompanyBatchHeader {
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

impl Record<CompanyBatchHeader> for CompanyBatchHeader {
    fn new(mut reader: StringReader) -> CompanyBatchHeader {
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

#[derive(Default)]
struct EntryDetail {
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
}

impl Record<EntryDetail> for EntryDetail {
    fn new(mut reader: StringReader) -> EntryDetail {
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
        }
    }
}

#[derive(Default)]
struct Addenda {
    record_type_code: Field,     // content: "7", size: 1
    addenda_type: Field,         // size: 2
    payment_related_info: Field, // size: 80
    addenda_sequence: Field,     // size: 4
    batch: Field,                // size: 7
}

impl Record<Addenda> for Addenda {
    fn new(mut reader: StringReader) -> Addenda {
        Addenda {
            record_type_code: Field::from("7"),
            addenda_type: Field::from(reader.read(2)),
            payment_related_info: Field::from(reader.read(80)),
            addenda_sequence: Field::from(reader.read(4)),
            batch: Field::from(reader.read(7)),
        }
    }
}

#[derive(Default)]
struct CompanyBatchTrailer {
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

impl Record<CompanyBatchTrailer> for CompanyBatchTrailer {
    fn new(mut reader: StringReader) -> CompanyBatchTrailer {
        CompanyBatchTrailer {
            record_type_code: Field::from("8"),
            service_class_code: Field::from(reader.read(3)),
            entry_and_addenda_count: Field::from(reader.read(3)),
            entry_hash: Field::from(reader.read(10)).left_just(true),
            total_debit_amount: Field::from(reader.read(12)),
            total_credit_amount: Field::from(reader.read(12)),
            company_id: Field::from(reader.read(10)),
            message_auth_code: Field::from(reader.read(19)),
            reserved: Field::from(reader.read(6)),
            originating_dfi_id_num: Field::from(reader.read(8)),
            batch_num: Field::from(reader.read(7)),           // size: 7
        }
    }
}

#[derive(Default)]
struct Trailer {
    record_type_code: Field,        // content: "9", size: 1
    batch_count: Field,             // size: 6 (total count of [CompanyBatchHeader] records)
    block_count: Field,             // size: 6 (a block is defined as 10 records.)
    entry_and_addenda_count: Field, // size: 8 (sum of [EntryDetail] and [Addenda])
    entry_hash: Field,              // size: 10 (todo: learn how this is derived )
    total_debits: Field,            // size: 12 (sum of [EntryDetail.amount]s for debits)
    total_credits: Field,           // size: 12 (sum of [EntryDetail.amount]s for credits)
    reserved: Field,                // size: 39
}

impl Record<Trailer> for Trailer {
    fn new(mut reader: StringReader) -> Trailer {
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

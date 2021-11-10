// https://achdevguide.nacha.org/ach-file-details

trait Record {
	const REC_TYPE_CODE: Field;
}

struct File {
	header: Header,
	records: Vec<Box<dyn Record>>,
	
}

struct Field {
	content: String,
	size: u32,
}

impl Field {
	fn new(content: &str) -> Self{
		Field {
			content: content.to_string(),
			size: 1
		}
	}
}

struct Header {
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

impl Record for Header {
	const REC_TYPE_CODE: Field = Field::new("1");
}

struct CompanyBatchHeader {
	service_class_code: Field,          // size: 3
	company_name: Field,                // size: 16
	company_discretionary_data: Field,  // size: 20
	company_id: Field,                  // size: 10
	sec: Field,                         // size: 3
	entry_desc: Field,                  // size: 10
	company_descriptive_date: Field,    // size: 6
	effective_entry_date: Field,        // size: 6
	settlement_date: Field,             // size: 3
	originator_status_code: Field,      // size: 1
	odfi_id: Field,                     // size: 8
	batch_number: Field,                // size: 7
}

impl Record for CompanyBatchHeader {
	const REC_TYPE_CODE: Field = Field::new("5");
}

struct EntryDetail {
	transactions_code: Field,   // size: 2
	receiving_dfi_id: Field,    // size: 8
	check_digit: Field,         // size: 1
	dfi_account
}

impl Record for EntryDetail {
	const REC_TYPE_CODE: Field = Field::new("6");
}

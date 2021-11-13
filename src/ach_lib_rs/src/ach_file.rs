// https://achdevguide.nacha.org/ach-file-details

trait Record {
	const REC_TYPE_CODE: Field;
}

struct File {
	header: Header,
	records: Vec<Box<dyn Record>>,
}

impl From<std::Path> for File {
	fn from(_: std::Path) -> Self {
		todo!()
	}
}

struct Field {
	content: String,
	size: u32,
	left_justified: bool,
}

impl Field {
	fn new(content: &str) -> Self{
		Field {
			content: content.to_string(),
			size: content.len() as u32,
			..Default::default()
		}
	}
}

impl Default for Field {
	fn default() -> Self {
		Self {
			content: "".to_string(),
			size: 0,
			left_justified: false
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
	dfi_account: Field,         // size: 17
	amount: Field,              // size: 10
	individual_id: Field,       // size: 15
	individual_name: Field,     // size: 22
	discretionary_data: Field,  // size: 2
	addenda_indicator: Field,   // size: 1
	trace: Field,               // size: 15
}

impl Record for EntryDetail {
	const REC_TYPE_CODE: Field = Field::new("6");
}

struct Addenda {
	addenda_type: Field,            // size: 2
	payment_related_info: Field,    // size: 80
	addenda_sequence: Field,        // size: 4
	batch: Field,                   // size: 7
}

impl Record for Addenda {
	const REC_TYPE_CODE: Field = Field::new("7");
}

struct CompanyBatchTrailer {
	service_class_code: Field,      // size: 3
	entry_and_addenda_count: Field, // size: 6  (sum of [EntryDetail] and [Addenda] since [CompanyBatchHeader])
	entry_hash: Field,              // size: 10 (Sum of each [EntryDetail.receiving_dfi_id], left justify)
	total_debit_amount: Field,      // size: 12 (Sum of [EntryDetail.amount]s for debits since [CompanyBatchHeader])
	total_credit_amount: Field,     // size: 12 (Sum of [EntryDetail.amount]s for credits since [CompanyBatchHeader])
	company_id: Field,              // size: 10
	message_auth_code: Field,       // size: 19
	reserved: Field,                // size: 6
	originating_dfi_id_num: Field,  // size: 8
	batch_num: Field,               // size: 7
}

impl Record for CompanyBatchTrailer {
	const REC_TYPE_CODE: Field = Field::new("8");
}

struct FileTrailer {
	batch_count: Field,             // size: 6 (total count of [CompanyBatchHeader] records)
	block_count: Field,             // size: 6 (a block is defined as 10 records.)
	entry_and_addenda_count: Field, // size: 8 (sum of [EntryDetail] and [Addenda])
	entry_hash: Field,              // size: 10 (todo: learn how this is derived )
	total_debits: Field,            // size: 12 (sum of [EntryDetail.amount]s for debits)
	total_credits: Field,           // size: 12 (sum of [EntryDetail.amount]s for credits)
	reserved: Field,                // size: 39
}

impl Record for FileTrailer {
	const REC_TYPE_CODE: Field = Field::new("9");
}

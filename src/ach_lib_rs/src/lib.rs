// https://dev-ach-guide.pantheonsite.io/ach-file-details
// Nacha ACH file implemented in Rust

trait AchRecord{ }

struct AchFile {
    header: AchHeader,
    records: Vec<Box<dyn AchRecord>>,

}

struct AchField {
    field_num: u8,
    content: String,
    size: u32,
}

struct AchHeader {
    rec_type_code: AchField,        // content: "1", size: 1
    priority_code: AchField,        // content:  "01", size: 2
    immediate_dest: AchField,       // size: 10
    immediate_orig: AchField,       // size: 10
    file_creation_date: AchField,   // size: 6
    file_creation_time: AchField,   // size: 4
    file_id_modifier: AchField,     // size: 1
    record_size: AchField,          // content: "094", size: 3
    blocking_factor: AchField,      // content: "10", size: 2
    format_code: AchField,          // content: "1", size: 1
    immediate_dest_name: AchField,  // size: 23
    immediate_orig_name: AchField,  // size: 23
    reference_code: AchField,       // content: "", size: 8
}

impl AchHeader { }

impl AchRecord for AchHeader { }


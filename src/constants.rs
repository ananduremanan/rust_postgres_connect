use once_cell::sync::Lazy;
use std::collections::HashMap;

// Define a static HashMap to store function names
pub static FUNCTION_NAMES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("get_students", "get_student_names");
    m.insert("set_students", "set_student");
    m.insert("delete_student", "set_student");
    m.insert("update_student", "set_student");
    m.insert("mock_costly_operation", "get_student_names");
    m.insert("delete_by_id", "set_student");
    m
});
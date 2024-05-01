use std::collections::HashMap;
use std::io::Write;
use std::{
    collections::hash_map::Entry,
    ffi::{CStr, CString},
    fs::File,
    os::raw::c_void,
    ptr::null_mut,
};

use babeltrace_sys::*;

struct CSVDumper {
    file: File,
    header_dumped: bool,
}

impl CSVDumper {
    pub fn new(name: &str) -> CSVDumper {
        let csv_name = name.replace(':', "_");
        CSVDumper {
            file: File::create(format!("{csv_name}.csv")).unwrap(),
            header_dumped: false,
        }
    }
}

struct DbConverter {
    message_iterator: *mut bt_message_iterator,
    index: u64,
    files: HashMap<String, CSVDumper>,
}
/// # Panics
#[no_mangle]
pub extern "C" fn db_converter_consume(
    sink: *mut bt_self_component_sink,
) -> bt_component_class_sink_consume_method_status {
    let mut messages: bt_message_array_const = null_mut();
    let mut count: u64 = 0;

    let dbconvert = unsafe {
        &mut *(bt_self_component_get_data(sink.cast::<bt_self_component>()).cast::<DbConverter>())
    };

    match unsafe { bt_message_iterator_next(dbconvert.message_iterator, &mut messages, &mut count) }
    {
        bt_message_iterator_next_status::BT_MESSAGE_ITERATOR_NEXT_STATUS_AGAIN => {
            bt_component_class_sink_consume_method_status::BT_COMPONENT_CLASS_SINK_CONSUME_METHOD_STATUS_AGAIN
        }
        bt_message_iterator_next_status::BT_MESSAGE_ITERATOR_NEXT_STATUS_END => {
            bt_component_class_sink_consume_method_status::BT_COMPONENT_CLASS_SINK_CONSUME_METHOD_STATUS_END
        }
        bt_message_iterator_next_status::BT_MESSAGE_ITERATOR_NEXT_STATUS_MEMORY_ERROR => {
            bt_component_class_sink_consume_method_status::BT_COMPONENT_CLASS_SINK_CONSUME_METHOD_STATUS_MEMORY_ERROR
        }
        bt_message_iterator_next_status::BT_MESSAGE_ITERATOR_NEXT_STATUS_ERROR => {
            bt_component_class_sink_consume_method_status::BT_COMPONENT_CLASS_SINK_CONSUME_METHOD_STATUS_ERROR
        }
        bt_message_iterator_next_status::BT_MESSAGE_ITERATOR_NEXT_STATUS_OK => {
            let messages: &[*const bt_message] =
                unsafe { std::slice::from_raw_parts_mut(messages, usize::try_from(count).unwrap()) };

            for &message in messages {
                if (unsafe { bt_message_get_type(message) }
                    == bt_message_type::BT_MESSAGE_TYPE_EVENT)
                {
                    let event =
                        unsafe { &*bt_message_event_borrow_event_const(message) as &bt_event };
                    let event_class =
                        unsafe { &*bt_event_borrow_class_const(event) as &bt_event_class };
                    let payload_field =
                        unsafe { &*bt_event_borrow_payload_field_const(event) as &bt_field };

                    let class_name = unsafe { CStr::from_ptr(bt_event_class_get_name(event_class)) };
                    let class_name = class_name.to_str().unwrap();

                    let dumper = match dbconvert.files.entry(class_name.to_string()) {
                        Entry::Occupied(o) => o.into_mut(),
                        Entry::Vacant(v) => v.insert(CSVDumper::new(class_name)),
                    };

                    if unsafe { bt_field_get_class_type(payload_field) == bt_field_class_type::BT_FIELD_CLASS_TYPE_STRUCTURE } {
                            let struct_class: *const bt_field_class = unsafe {
                                bt_field_borrow_class_const(payload_field)
                            };
                            let member_count = unsafe { bt_field_class_structure_get_member_count(struct_class) };

                            if !dumper.header_dumped {
                                let mut header_line="timestamp,".to_owned();

                                for i in 0..member_count {
                                    let member = unsafe {
                                        CStr::from_ptr(bt_field_class_structure_member_get_name(bt_field_class_structure_borrow_member_by_index_const(struct_class, i)))
                                    };
                                    header_line.push_str(&format!(",{}", member.to_str().unwrap()));
                                }
                                header_line.push('\n');

                                dumper.file.write_all(header_line.as_bytes()).unwrap();
                                dumper.header_dumped = true;
                            }


                            let timestamp = unsafe { bt_message_event_borrow_default_clock_snapshot_const(message) };
                            let mut nanoseconds: i64 = 0;
                            unsafe { bt_clock_snapshot_get_ns_from_origin(timestamp, &mut nanoseconds); }

                            let mut csv_line: String = format!("{nanoseconds}");

                            for i in 0..member_count {
                                let member_field = unsafe { bt_field_structure_borrow_member_field_by_index_const(payload_field, i) };
                                match unsafe { bt_field_get_class_type(member_field) } {
                                    bt_field_class_type::BT_FIELD_CLASS_TYPE_UNSIGNED_INTEGER => {
                                        let member_value = unsafe { bt_field_integer_unsigned_get_value(member_field) };
                                        csv_line.push_str(format!(",{member_value}").as_str());
                                    },
                                    bt_field_class_type::BT_FIELD_CLASS_TYPE_SIGNED_INTEGER => {
                                        let member_value = unsafe { bt_field_integer_signed_get_value(member_field) };
                                        csv_line.push_str(format!(",{member_value}").as_str());
                                    },
                                    bt_field_class_type::BT_FIELD_CLASS_TYPE_SINGLE_PRECISION_REAL => {
                                        let member_value = unsafe { bt_field_real_single_precision_get_value(member_field) };
                                        csv_line.push_str(format!(",{member_value}").as_str());
                                    },
                                    bt_field_class_type::BT_FIELD_CLASS_TYPE_DOUBLE_PRECISION_REAL => {
                                        let member_value = unsafe { bt_field_real_double_precision_get_value(member_field) };
                                        csv_line.push_str(format!(",{member_value}").as_str());
                                    },
                                    bt_field_class_type::BT_FIELD_CLASS_TYPE_STRING => {
                                        let member_value = unsafe { CStr::from_ptr(bt_field_string_get_value(member_field)) };
                                        csv_line.push_str(member_value.to_str().unwrap());
                                    },
                                    _ => {}
                                }
                            }
                            csv_line.push('\n');
                            dumper.file.write_all(csv_line.as_bytes()).unwrap();
                    }

                    dbconvert.index += 1;
                }

                unsafe { babeltrace_sys::bt_message_put_ref(message.cast()) };
            }

            bt_component_class_sink_consume_method_status::BT_COMPONENT_CLASS_SINK_CONSUME_METHOD_STATUS_OK
        }
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn db_converter_graph_is_configured(
    sink: *mut bt_self_component_sink,
) -> bt_component_class_sink_graph_is_configured_method_status {
    let dbconvert = unsafe {
        &mut *bt_self_component_get_data(sink.cast::<bt_self_component>()).cast::<DbConverter>()
    };

    let port = unsafe { &mut *bt_self_component_sink_borrow_input_port_by_index(sink, 0) };

    unsafe {
        bt_message_iterator_create_from_sink_component(sink, port, &mut dbconvert.message_iterator);
    }

    bt_component_class_sink_graph_is_configured_method_status::BT_COMPONENT_CLASS_SINK_GRAPH_IS_CONFIGURED_METHOD_STATUS_OK
}

/// # Panics
#[no_mangle]
pub extern "C" fn db_converter_finalize(sink: *mut bt_self_component_sink) {
    let dbconvert = unsafe {
        &mut *bt_self_component_get_data(sink.cast::<bt_self_component>()).cast::<DbConverter>()
    };

    for dumper in dbconvert.files.values_mut() {
        dumper.file.flush().unwrap();
    }

    unsafe {
        drop(Box::from_raw(dbconvert));
    }
}

/// # Safety
/// # Panics
/// panic when fails to alloate memory for `CString`.
#[no_mangle]
pub unsafe extern "C" fn db_converter_initialize(
    sink: *mut bt_self_component_sink,
    _configuration: *mut bt_self_component_sink_configuration,
    _params: *const bt_value,
    _initialize_method_data: *mut c_void,
) -> bt_component_class_initialize_method_status {
    let dbconverter = Box::new(DbConverter {
        message_iterator: null_mut(),
        index: 0,
        files: HashMap::new(),
    });

    unsafe {
        bt_self_component_set_data(
            sink.cast::<bt_self_component>(),
            Box::into_raw(dbconverter).cast::<c_void>(),
        );
        let in_arg = CString::new("in").expect("CString::new failed");
        bt_self_component_sink_add_input_port(sink, in_arg.as_ptr(), null_mut(), null_mut());
    };

    bt_component_class_initialize_method_status::BT_COMPONENT_CLASS_INITIALIZE_METHOD_STATUS_OK
}

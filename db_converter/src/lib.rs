use std::{
    ffi::{CStr, CString},
    os::raw::c_void,
    ptr::null_mut,
};

use babeltrace_sys::{
    bt_component_class_sink_consume_method_status,
    bt_component_class_sink_graph_is_configured_method_status, bt_event,
    bt_event_borrow_class_const, bt_event_borrow_payload_field_const, bt_event_class,
    bt_event_class_get_name, bt_field, bt_field_borrow_class_const,
    bt_field_class_structure_get_member_count, bt_message, bt_message_array_const,
    bt_message_event_borrow_event_const, bt_message_iterator_create_from_sink_component,
    bt_message_iterator_next, bt_message_iterator_next_status, bt_message_type, bt_self_component,
    bt_self_component_set_data, bt_self_component_sink, bt_self_component_sink_add_input_port,
    bt_self_component_sink_borrow_input_port_by_index, bt_self_component_sink_configuration,
    bt_value,
};

struct DbConverter {
    message_iterator: *mut babeltrace_sys::bt_message_iterator,
    index: u64,
}

#[no_mangle]
pub extern "C" fn db_converter_consume(
    sink: *mut bt_self_component_sink,
) -> bt_component_class_sink_consume_method_status {
    let mut messages: bt_message_array_const = null_mut();
    let mut count: u64 = 0;

    let dbconvert = unsafe {
        &mut *(babeltrace_sys::bt_self_component_get_data(sink.cast::<bt_self_component>())
            .cast::<DbConverter>())
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
        _ => {
            let messages: &[*const bt_message] =
                unsafe { std::slice::from_raw_parts_mut(messages, usize::try_from(count).unwrap()) };

            for &message in messages {
                if (unsafe { babeltrace_sys::bt_message_get_type(message) }
                    == bt_message_type::BT_MESSAGE_TYPE_EVENT)
                {
                    let event =
                        unsafe { &*bt_message_event_borrow_event_const(message) as &bt_event };
                    let event_class =
                        unsafe { &*bt_event_borrow_class_const(event) as &bt_event_class };
                    let payload_field =
                        unsafe { &*bt_event_borrow_payload_field_const(event) as &bt_field };
                    let member_count = unsafe {
                        bt_field_class_structure_get_member_count(bt_field_borrow_class_const(
                            payload_field,
                        ))
                    };
                    let class_name = unsafe { CStr::from_ptr(bt_event_class_get_name(event_class)) };

                    println!("#{}: {} ({} payload member(s))", dbconvert.index, class_name.to_str().unwrap(), member_count);

                    dbconvert.index += 1;
                }
            }

            bt_component_class_sink_consume_method_status::BT_COMPONENT_CLASS_SINK_CONSUME_METHOD_STATUS_OK
        }
    }
}

#[no_mangle]
pub extern "C" fn db_converter_graph_is_configured(
    sink: *mut bt_self_component_sink,
) -> bt_component_class_sink_graph_is_configured_method_status {
    let dbconvert = unsafe {
        &mut *babeltrace_sys::bt_self_component_get_data(sink.cast::<bt_self_component>())
            .cast::<DbConverter>()
    };

    let port = unsafe { &mut *bt_self_component_sink_borrow_input_port_by_index(sink, 0) };

    unsafe {
        bt_message_iterator_create_from_sink_component(sink, port, &mut dbconvert.message_iterator);
    }

    bt_component_class_sink_graph_is_configured_method_status::BT_COMPONENT_CLASS_SINK_GRAPH_IS_CONFIGURED_METHOD_STATUS_OK
}

#[no_mangle]
pub extern "C" fn db_converter_finalize(sink: *mut bt_self_component_sink) {
    let dbconvert = unsafe {
        babeltrace_sys::bt_self_component_get_data(sink.cast::<bt_self_component>())
            .cast::<bt_self_component>()
    };

    unsafe {
        drop(Box::from_raw(dbconvert));
    }
    /* Free the allocated structure */
}

#[no_mangle]
pub extern "C" fn db_converter_initialize(
    sink: *mut bt_self_component_sink,
    _configuration: *mut bt_self_component_sink_configuration,
    _params: *const bt_value,
    _initialize_method_data: *mut c_void,
) {
    let dbconverter = Box::new(DbConverter {
        message_iterator: null_mut(),
        index: 0,
    });

    unsafe {
        bt_self_component_set_data(
            sink.cast::<bt_self_component>(),
            Box::into_raw(dbconverter).cast::<c_void>(),
        );
        let in_arg = CString::new("in").expect("CString::new failed");
        bt_self_component_sink_add_input_port(sink, in_arg.as_ptr(), null_mut(), null_mut());
    };
}

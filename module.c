#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>
#include <inttypes.h>
#include <string.h>
#include <babeltrace2/babeltrace.h>


extern bt_component_class_initialize_method_status db_converter_initialize(
        bt_self_component_sink *self_component_sink,
        bt_self_component_sink_configuration *configuration,
        const bt_value *params, void *initialize_method_data);
extern void db_converter_finalize(bt_self_component_sink *self_component_sink);
extern bt_component_class_sink_consume_method_status db_converter_consume(
        bt_self_component_sink *self_component_sink);
extern bt_component_class_sink_graph_is_configured_method_status
db_converter_graph_is_configured(bt_self_component_sink *self_component_sink);

/* Mandatory */
BT_PLUGIN_MODULE();
 
/* Define the `epitome` plugin */
BT_PLUGIN(db_converter);

BT_PLUGIN_DESCRIPTION("Convert CTF to csv");
BT_PLUGIN_AUTHOR("Minhu Wang <minhuw@acm.org>");
BT_PLUGIN_LICENSE("MIT");

/* Define the `output` sink component class */
BT_PLUGIN_SINK_COMPONENT_CLASS(output, db_converter_consume);

/* Set some of the `output` sink component class's optional methods */
BT_PLUGIN_SINK_COMPONENT_CLASS_INITIALIZE_METHOD(output,
    db_converter_initialize);
BT_PLUGIN_SINK_COMPONENT_CLASS_FINALIZE_METHOD(output, db_converter_finalize);
BT_PLUGIN_SINK_COMPONENT_CLASS_GRAPH_IS_CONFIGURED_METHOD(output,
    db_converter_graph_is_configured);
mod util;
pub mod node_int;
pub mod node_dynamic_array;
pub mod node_dynamic_bytes;
pub mod node_enum;
pub mod node_fixed_range;
pub mod node_fixed_bytes;
pub mod node_serial;
pub mod node_const;
pub mod node_rust;
pub mod node_zero;
pub mod node;
pub mod object;
pub mod schema;

pub fn new() -> schema::Schema {
    return schema::Schema::new();
}

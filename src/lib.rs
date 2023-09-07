mod util;
pub mod node;
pub mod scope;
pub mod schema;

pub fn new() -> schema::Schema {
    return schema::Schema::new();
}

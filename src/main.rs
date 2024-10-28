use std::collections::HashMap;

use vel::VelInstance;

fn main() {
    let mut instance = VelInstance::new(HashMap::from([("test", "value")]));
    dbg!(instance.render("test".to_owned(), HashMap::new()));
}

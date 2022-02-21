
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;

pub mod tests;
pub mod queries;

///// Expose DatabaseActor Actor /////
pub use gm::db::actor::{
    DatabaseActor,
    GetPool,
    GetPoolError,
};
pub use queries::*;

use gm::db;


#[test]
fn zip_hashmap_example() {

    use itertools::zip;
    use std::collections::HashMap;

    let v1 = vec!['a', 'b', 'c', 'd'];
    let v2 = vec![6, 7, 8, 9];

    let hmap = zip(v1.iter(), v2.iter()).collect::<HashMap<_,_>>();
    println!("hmap from zip: {:?}", hmap);

}
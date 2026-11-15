use super::*;
use crate::{
    columns::network_info::NetworkInfoColumn, stores::state::StateStore, tests::TempDBDir,
};

#[test_log::test]
fn put_all_refuses_a_column_family_that_is_not_writable() {
    let tmp = TempDBDir::new();
    let db = StateStore::init_db(&tmp.path, None).unwrap();
    let access = DbAccess::new(&db, &[]);

    assert!(matches!(
        access.put_all::<NetworkInfoColumn>([]),
        Err(Error::WritingReadOnlyCf(_))
    ));
}

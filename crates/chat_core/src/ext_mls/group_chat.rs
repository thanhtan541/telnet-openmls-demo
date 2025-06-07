use openmls::group::GroupId;

use crate::helpers::setup_group;

#[test]
fn welcome_based_flow() {
    let number_of_members = 3;
    let group_name = "alice_init";
    let members = setup_group(group_name, number_of_members);
    assert_eq!(members.len(), number_of_members);

    let group_id = GroupId::from_slice(group_name.as_bytes());

    for member in members.iter() {
        let (group, info) = member;

        assert_eq!(
            group.group_id(),
            &group_id,
            "Group ID is not matched between owner group and member group"
        );
        assert_eq!(
            group.group_id(),
            &info.group_id,
            "Group ID is not matched between group and member info"
        );
    }
}

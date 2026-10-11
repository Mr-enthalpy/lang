//! Visibility declared on actual structural members. This coordinate is
//! independent of member residency, registered roles and callability.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StructuralMemberVisibility {
    Default,
    Public,
    Private,
}

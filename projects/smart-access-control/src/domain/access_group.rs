//! Groups of users that share door permissions (e.g. "Staff", "Cleaners").
//!
//! Membership is a plain user-to-group relation, managed by the application
//! layer, so it is not modelled as an entity here.

use super::{AccessGroupId, text::bounded_text};

bounded_text!(
    /// An access group's name, e.g. "Night Security".
    GroupName,
    field = "name",
    max = 100
);

bounded_text!(
    /// Free-text notes about a group.
    Description,
    field = "description",
    max = 500
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessGroup {
    id: AccessGroupId,
    name: GroupName,
    description: Option<Description>,
}

impl AccessGroup {
    pub fn create(name: GroupName, description: Option<Description>) -> Self {
        Self {
            id: AccessGroupId::generate(),
            name,
            description,
        }
    }

    /// Rebuilds a group from stored values. For repositories only.
    pub fn restore(id: AccessGroupId, name: GroupName, description: Option<Description>) -> Self {
        Self {
            id,
            name,
            description,
        }
    }

    pub fn id(&self) -> AccessGroupId {
        self.id
    }

    pub fn name(&self) -> &GroupName {
        &self.name
    }

    /// `Option<&T>` rather than `&Option<T>`: callers get the common,
    /// more flexible shape, and the field's layout stays private.
    pub fn description(&self) -> Option<&Description> {
        self.description.as_ref()
    }

    pub fn rename(&mut self, name: GroupName) {
        self.name = name;
    }

    pub fn set_description(&mut self, description: Option<Description>) {
        self.description = description;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_edit() {
        let mut group = AccessGroup::create(GroupName::parse("Staff").unwrap(), None);
        assert_eq!(group.name().as_str(), "Staff");
        assert_eq!(group.description(), None);

        group.rename(GroupName::parse("All Staff").unwrap());
        group.set_description(Some(Description::parse("Everyone employed").unwrap()));
        assert_eq!(group.name().as_str(), "All Staff");
        assert_eq!(
            group.description().map(Description::as_str),
            Some("Everyone employed")
        );
    }
}

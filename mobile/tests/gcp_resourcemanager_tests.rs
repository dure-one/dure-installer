//! Tests for api/gcp/resourcemanager module

use dure::api::gcp::resourcemanager::{Project, ProjectList};

#[test]
fn test_project_structure() {
    let project = Project {
        name: Some("projects/my-project-123".to_string()),
        project_id: "my-project-123".to_string(),
        display_name: Some("My Test Project".to_string()),
        state: Some("ACTIVE".to_string()),
        labels: std::collections::HashMap::new(),
    };

    assert_eq!(project.project_id, "my-project-123");
    assert_eq!(project.id(), "my-project-123");
    assert_eq!(project.display_name(), "My Test Project");
    assert_eq!(project.state(), "ACTIVE");
    assert!(project.is_active());
}

#[test]
fn test_project_list_structure() {
    let list = ProjectList {
        projects: vec![],
        next_page_token: None,
    };

    assert!(list.projects.is_empty());
    assert!(list.next_page_token.is_none());
}

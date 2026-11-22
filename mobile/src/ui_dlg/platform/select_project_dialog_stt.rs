#[derive(Default)]
pub struct SelectProjectDialog {
    pub open: bool,
    pub platform_name: String,
    pub project_list: Vec<(String, String)>, // (project_id, project_name)
    pub selected: Option<usize>,
    pub loading: bool,
}

pub enum SelectProjectAction {
    Select {
        platform_name: String,
        project_id: String,
    },
    CreateNew {
        platform_name: String,
    },
    Cancel,
    None,
}

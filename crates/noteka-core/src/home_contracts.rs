use crate::contracts::{Classification, EntityKind, PersonFields, ProjectState};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HomeView {
    TasksToday,
    TasksInbox,
    TasksAll,
    TasksDone,
    TasksWeek,
    TasksMonth,
    TasksScheduled,
    TasksNoDue,
    TasksActiveProjects,
    TasksProjectsInProgress,
    NotesAll,
    NotesInbox,
    NotesLibrary,
    NotesFavorites,
    NotesClips,
    NotesVoice,
    NotesJournal,
    NotesMeetings,
    ProjectsActive,
    ProjectsPlanned,
    ProjectsBoard,
    ProjectsDeadlines,
    ProjectsCompleted,
    ProjectsOnHold,
    ProjectsByTag,
    ProjectsByPeople,
    TagsRecent,
    TagsFavorites,
    TagsAlphabetical,
    TagsTypes,
    GoalsActivity,
    GoalsDeadline,
    GoalsByTag,
    GoalsAchieved,
    MilestonesCompleted,
    MilestonesAll,
    PeopleContacts,
    PeopleNew,
    PeopleUpdated,
    PeopleCheckIns,
    PeopleBirthdays,
    PeoplePipeline,
    PeopleActiveProjects,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HomeQuery {
    pub view: HomeView,
    pub as_of: Option<String>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityRef {
    pub id: String,
    pub kind: EntityKind,
    pub title: String,
    pub revision: u32,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_color: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct HomeMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal: Option<crate::contracts::GoalFields>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub milestone: Option<crate::contracts::MilestoneFields>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_activity_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_end_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub person: Option<PersonFields>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birthday_occurrence: Option<String>,
    pub note_type: Option<String>,
    pub favorite: Option<bool>,
    pub note_library: Option<bool>,
    pub source_updated_at: Option<String>,
    pub source_updated_days_ago: Option<i64>,
    pub state: Option<String>,
    pub inbox: Option<bool>,
    pub priority: Option<String>,
    pub due_date: Option<String>,
    pub planned_date: Option<String>,
    pub classification: Option<String>,
    pub updated_days_ago: Option<i64>,
    pub issues: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HomeSummary {
    Goal {
        available: bool,
        total: Option<u32>,
        done: Option<u32>,
        progress_tenths: Option<u32>,
    },
    Project {
        available: bool,
        total: Option<u32>,
        pending: Option<u32>,
        done: Option<u32>,
        overdue: Option<u32>,
        progress_percent: Option<u32>,
        progress_tenths: Option<u32>,
    },
    Tag {
        available: bool,
        note_count: Option<u32>,
        last_note: Option<EntityRef>,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HomeItem {
    pub entity: EntityRef,
    pub metadata: HomeMetadata,
    pub context: Vec<EntityRef>,
    pub context_total: u32,
    pub summary: Option<HomeSummary>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HomeGroup {
    pub key: String,
    pub label: String,
    pub entity_ids: Vec<String>,
    pub project_id: Option<String>,
    pub project_archived: bool,
    pub project_state: Option<ProjectState>,
    pub classification: Option<Classification>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related_entity: Option<EntityRef>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HomePage {
    pub view: HomeView,
    pub date: String,
    pub timezone: String,
    pub items: Vec<HomeItem>,
    pub groups: Vec<HomeGroup>,
    pub matching_count: u32,
    pub unavailable_count: u32,
    pub offset: u32,
    pub next_cursor: Option<String>,
}

use crate::error::AppResult;
use crate::services::git_service::{
    GitBranch, GitCommit, GitDiffEntry, GitRemote, GitStashEntry, GitStatusEntry, GitTag,
};
use crate::{validate_project_path, DbState, GitState, ServicesState};

fn validate(state: &DbState, path: &str) -> AppResult<std::path::PathBuf> {
    validate_project_path(path, state)
}

#[tauri::command]
pub fn git_get_branches(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
) -> AppResult<Vec<GitBranch>> {
    services.audit_logger.timed("git:get_branches", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.get_branches(&project_path)
    })
}

#[tauri::command]
pub fn git_checkout_branch(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
    branch_name: String,
) -> AppResult<()> {
    services.audit_logger.timed("git:checkout_branch", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.checkout_branch(&project_path, &branch_name)
    })
}

#[tauri::command]
pub fn git_create_branch(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
    branch_name: String,
) -> AppResult<()> {
    services.audit_logger.timed("git:create_branch", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.create_branch(&project_path, &branch_name)
    })
}

#[tauri::command]
pub fn git_delete_branch(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
    branch_name: String,
) -> AppResult<()> {
    services.audit_logger.timed("git:delete_branch", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.delete_branch(&project_path, &branch_name)
    })
}

#[tauri::command]
pub fn git_get_status(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
) -> AppResult<Vec<GitStatusEntry>> {
    services.audit_logger.timed("git:get_status", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.get_status(&project_path)
    })
}

#[tauri::command]
pub fn git_stage_file(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
    file_path: String,
) -> AppResult<()> {
    services.audit_logger.timed("git:stage_file", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.stage_file(&project_path, &file_path)
    })
}

#[tauri::command]
pub fn git_unstage_file(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
    file_path: String,
) -> AppResult<()> {
    services.audit_logger.timed("git:unstage_file", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.unstage_file(&project_path, &file_path)
    })
}

#[tauri::command]
pub fn git_stage_all(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
) -> AppResult<()> {
    services.audit_logger.timed("git:stage_all", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.stage_all(&project_path)
    })
}

#[tauri::command]
pub fn git_commit(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
    message: String,
    author_name: String,
    author_email: String,
) -> AppResult<String> {
    services.audit_logger.timed("git:commit", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state
            .git_service
            .commit(&project_path, &message, &author_name, &author_email)
    })
}

#[tauri::command]
pub fn git_push(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
    remote_name: String,
) -> AppResult<()> {
    services.audit_logger.timed("git:push", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.push(&project_path, &remote_name)
    })
}

#[tauri::command]
pub fn git_pull(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
) -> AppResult<()> {
    services.audit_logger.timed("git:pull", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.pull(&project_path)
    })
}

#[tauri::command]
pub fn git_clone(
    db_state: tauri::State<DbState>,
    services: tauri::State<ServicesState>,
    url: String,
    dest_path: String,
) -> AppResult<()> {
    services.audit_logger.timed("git:clone", || {
        let dest = std::path::Path::new(&dest_path);
        let parent = dest.parent().unwrap_or(dest);
        let _resolved = validate(&db_state, &parent.to_string_lossy())?;
        crate::services::git_service::GitService::git_clone_remote(&url, &dest_path)
    })
}

#[tauri::command]
pub fn git_get_remotes(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
) -> AppResult<Vec<GitRemote>> {
    services.audit_logger.timed("git:get_remotes", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.get_remotes(&project_path)
    })
}

#[tauri::command]
pub fn git_add_remote(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
    name: String,
    url: String,
) -> AppResult<()> {
    services.audit_logger.timed("git:add_remote", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.add_remote(&project_path, &name, &url)
    })
}

#[tauri::command]
pub fn git_get_diff_summary(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
) -> AppResult<Vec<GitDiffEntry>> {
    services.audit_logger.timed("git:get_diff_summary", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.get_diff_summary(&project_path)
    })
}

#[tauri::command]
pub fn git_get_log(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
    max_count: usize,
) -> AppResult<Vec<GitCommit>> {
    services.audit_logger.timed("git:get_log", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.get_log(&project_path, max_count)
    })
}

#[tauri::command]
pub fn git_stash(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
    message: Option<String>,
) -> AppResult<()> {
    services.audit_logger.timed("git:stash", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.stash(&project_path, message.as_deref())
    })
}

#[tauri::command]
pub fn git_stash_list(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
) -> AppResult<Vec<GitStashEntry>> {
    services.audit_logger.timed("git:stash_list", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.stash_list(&project_path)
    })
}

#[tauri::command]
pub fn git_stash_pop(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
    stash_index: usize,
) -> AppResult<()> {
    services.audit_logger.timed("git:stash_pop", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.stash_pop(&project_path, stash_index)
    })
}

#[tauri::command]
pub fn git_stash_drop(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
    stash_index: usize,
) -> AppResult<()> {
    services.audit_logger.timed("git:stash_drop", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.stash_drop(&project_path, stash_index)
    })
}

#[tauri::command]
pub fn git_stash_apply(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
    stash_index: usize,
) -> AppResult<()> {
    services.audit_logger.timed("git:stash_apply", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.stash_apply(&project_path, stash_index)
    })
}

#[tauri::command]
pub fn git_get_tags(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
) -> AppResult<Vec<GitTag>> {
    services.audit_logger.timed("git:get_tags", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.get_tags(&project_path)
    })
}

#[tauri::command]
pub fn git_diff_file(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
    file_path: String,
) -> AppResult<String> {
    services.audit_logger.timed("git:diff_file", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.diff_file(&project_path, &file_path)
    })
}

#[tauri::command]
pub fn git_get_commit_diff(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
    commit_oid: String,
) -> AppResult<String> {
    services.audit_logger.timed("git:get_commit_diff", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state
            .git_service
            .get_commit_diff(&project_path, &commit_oid)
    })
}

#[tauri::command]
pub fn git_get_branch_ahead_behind(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
    branch_name: String,
) -> AppResult<(usize, usize)> {
    services.audit_logger.timed("git:get_branch_ahead_behind", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state
            .git_service
            .get_branch_ahead_behind(&project_path, &branch_name)
    })
}

#[tauri::command]
pub fn git_get_current_branch(
    db_state: tauri::State<DbState>,
    git_state: tauri::State<GitState>,
    services: tauri::State<ServicesState>,
    project_path: String,
) -> AppResult<String> {
    services.audit_logger.timed("git:get_current_branch", || {
        let _resolved = validate(&db_state, &project_path)?;
        git_state.git_service.get_current_branch(&project_path)
    })
}

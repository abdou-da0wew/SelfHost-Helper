use std::path::Path;

use git2::{
    BranchType, DiffOptions, IndexAddOption, Oid, Repository, Sort, Status,
    StatusOptions, StashFlags,
};

use crate::error::{AppError, AppResult};

pub struct GitService;

#[derive(Debug, Clone, serde::Serialize)]
pub struct GitBranch {
    pub name: String,
    pub is_head: bool,
    pub upstream: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GitCommit {
    pub oid: String,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GitDiffEntry {
    pub path: String,
    pub status: String,
    pub insertions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GitStatusEntry {
    pub path: String,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GitRemote {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GitStashEntry {
    pub index: usize,
    pub message: String,
    pub branch: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GitTag {
    pub name: String,
    pub target_oid: String,
    pub message: Option<String>,
}

impl GitService {
    pub fn new() -> Self {
        Self
    }

    fn get_repo(&self, project_path: &str) -> AppResult<Repository> {
        let path = std::path::Path::new(project_path);
        if !path.is_dir() {
            return Err(AppError::Validation("Invalid project path".into()));
        }
        Repository::open(project_path)
            .map_err(|e| AppError::Git(format!("Failed to open repository: {}", e)))
    }

    pub fn get_branches(&self, project_path: &str) -> AppResult<Vec<GitBranch>> {
        let repo = self.get_repo(project_path)?;
        let head_ref = repo.head().ok();
        let head_name = head_ref
            .as_ref()
            .and_then(|h| h.shorthand().map(|s| s.to_string()));
        let mut branches = Vec::new();
        for branch_result in repo.branches(Some(BranchType::Local))? {
            let (branch, _) = branch_result?;
            let name = branch.name()?.unwrap_or_default().to_string();
            let is_head = head_name.as_deref() == Some(name.as_str());
            let upstream = branch
                .upstream()
                .ok()
                .and_then(|u| u.name().ok().flatten().map(|s| s.to_string()));
            branches.push(GitBranch {
                name,
                is_head,
                upstream,
            });
        }
        Ok(branches)
    }

    pub fn checkout_branch(&self, project_path: &str, branch_name: &str) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        let head = repo.head()?;
        let current_branch = head.shorthand().unwrap_or_default();
        if current_branch == branch_name {
            return Ok(());
        }
        let branch_ref = repo
            .find_branch(branch_name, git2::BranchType::Local)?
            .into_reference();
        let commit = branch_ref.peel_to_commit()?;
        repo.checkout_tree(commit.as_object(), None)
            .map_err(|e| AppError::Git(format!("Failed to checkout tree: {}", e)))?;
        repo.set_head(branch_ref.name().ok_or_else(|| {
            AppError::Git("Invalid branch name".into())
        })?)?;
        Ok(())
    }

    pub fn create_branch(&self, project_path: &str, branch_name: &str) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;
        repo.branch(branch_name, &commit, false)?;
        Ok(())
    }

    pub fn delete_branch(&self, project_path: &str, branch_name: &str) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        let mut branch = repo.find_branch(branch_name, BranchType::Local)?;
        branch.delete()?;
        Ok(())
    }

    pub fn get_status(&self, project_path: &str) -> AppResult<Vec<GitStatusEntry>> {
        let repo = self.get_repo(project_path)?;
        let mut opts = StatusOptions::new();
        opts.include_untracked(true).recurse_untracked_dirs(true);
        let statuses = repo.statuses(Some(&mut opts))?;
        let entries = statuses
            .iter()
            .filter_map(|entry| {
                let path = entry.path()?.to_string();
                let status = format_status(entry.status());
                Some(GitStatusEntry { path, status })
            })
            .collect();
        Ok(entries)
    }

    pub fn stage_file(&self, project_path: &str, file_path: &str) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        let mut index = repo.index()?;
        index
            .add_path(Path::new(file_path))
            .map_err(|e| AppError::Git(format!("Failed to stage file: {}", e)))?;
        index.write()?;
        Ok(())
    }

    pub fn unstage_file(&self, project_path: &str, file_path: &str) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        let head = repo.head()?.peel_to_commit()?;
        repo.reset_default(Some(head.as_object()), &[Path::new(file_path)])
            .map_err(|e| AppError::Git(format!("Failed to unstage file: {}", e)))?;
        Ok(())
    }

    pub fn stage_all(&self, project_path: &str) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        let mut index = repo.index()?;
        index.add_all(["."], IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(())
    }

    pub fn commit(
        &self,
        project_path: &str,
        message: &str,
        author_name: &str,
        author_email: &str,
    ) -> AppResult<String> {
        let repo = self.get_repo(project_path)?;
        let mut index = repo.index()?;
        index.write()?;
        let tree_oid = index.write_tree()?;
        let tree = repo.find_tree(tree_oid)?;
        let sig = git2::Signature::now(author_name, author_email)?;
        let head = repo.head().ok();
        let parent = head.and_then(|h| h.peel_to_commit().ok());
        let parents: Vec<&git2::Commit> = parent.iter().collect();
        let commit_oid = repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)?;
        Ok(commit_oid.to_string())
    }

    pub fn push(&self, project_path: &str, remote_name: &str) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        let mut remote = repo.find_remote(remote_name).map_err(|e| {
            AppError::Git(format!("Remote '{}' not found: {}", remote_name, e))
        })?;
        let head = repo.head()?;
        let branch = head.shorthand().unwrap_or("main");
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch, branch);
        remote
            .push(&[refspec], None)
            .map_err(|e| AppError::Git(format!("Push failed: {}", e)))?;
        Ok(())
    }

    pub fn pull(&self, project_path: &str) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        let branch = repo.head()?.shorthand().unwrap_or("main").to_string();
        let mut remote = repo.find_remote("origin")?;
        remote.fetch(&[branch.as_str()], None, None)?;
        let fetch_commit = repo
            .find_reference("FETCH_HEAD")?
            .peel_to_commit()?;
        let head_commit = repo.head()?.peel_to_commit()?;

        if head_commit.id() == fetch_commit.id() {
            return Ok(());
        }

        let merge_base = repo.merge_base(head_commit.id(), fetch_commit.id())?;
        if merge_base == head_commit.id() {
            // Fast-forward
            repo.head()?
                .set_target(fetch_commit.id(), "fast-forward merge")?;
            repo.checkout_tree(fetch_commit.as_object(), None)?;
        } else {
            // 3-way merge
            let annotated = repo.find_annotated_commit(fetch_commit.id())?;
            let mut merge_opts = git2::MergeOptions::new();
            repo.merge(
                &[&annotated],
                Some(&mut merge_opts),
                None,
            )
            .map_err(|e| {
                if repo.index().is_ok_and(|i| i.has_conflicts()) {
                    AppError::Git("Merge conflicts detected — resolve manually".into())
                } else {
                    AppError::Git(format!("Merge failed: {}", e))
                }
            })?;
            if repo.index()?.has_conflicts() {
                return Err(AppError::Git(
                    "Merge conflicts detected — resolve manually".into(),
                ));
            }
            let tree_oid = repo.index()?.write_tree()?;
            let tree = repo.find_tree(tree_oid)?;
            let sig =
                git2::Signature::now("SelfHost Helper", "dev@selfhost")?;
            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                &format!("Merge remote-tracking branch 'origin/{}'", branch),
                &tree,
                &[&head_commit, &fetch_commit],
            )?;
        }
        Ok(())
    }

    pub fn git_clone_remote(url: &str, dest_path: &str) -> AppResult<()> {
        if dest_path.is_empty() {
            return Err(AppError::Validation(
                "Destination path cannot be empty".into(),
            ));
        }
        let dest = std::path::Path::new(dest_path);
        if let Some(parent) = dest.parent() {
            if !parent.is_dir() {
                return Err(AppError::Validation(
                    "Destination parent directory does not exist".into(),
                ));
            }
        }
        Repository::clone(url, dest_path)
            .map_err(|e| AppError::Git(format!("Clone failed: {}", e)))?;
        Ok(())
    }

    pub fn get_remotes(&self, project_path: &str) -> AppResult<Vec<GitRemote>> {
        let repo = self.get_repo(project_path)?;
        let mut remotes = Vec::new();
        for remote_result in repo.remotes()?.iter() {
            if let Some(name) = remote_result {
                if let Ok(remote) = repo.find_remote(name) {
                    remotes.push(GitRemote {
                        name: name.to_string(),
                        url: remote.url().unwrap_or_default().to_string(),
                    });
                }
            }
        }
        Ok(remotes)
    }

    pub fn add_remote(&self, project_path: &str, name: &str, url: &str) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        repo.remote(name, url)?;
        Ok(())
    }

    pub fn get_diff_summary(&self, project_path: &str) -> AppResult<Vec<GitDiffEntry>> {
        let repo = self.get_repo(project_path)?;
        let head = repo.head()?.peel_to_tree()?;
        let mut opts = DiffOptions::new();
        let diff = repo.diff_tree_to_workdir(Some(&head), Some(&mut opts))?;
        let mut entries = Vec::new();
        for delta_idx in 0..diff.deltas().len() {
            let delta = diff
                .deltas()
                .nth(delta_idx)
                .ok_or_else(|| AppError::Internal("bad delta index".into()))?;
            let path = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            let status = format_diff_status(delta.status());
            entries.push(GitDiffEntry {
                path,
                status,
                insertions: 0,
                deletions: 0,
            });
        }
        Ok(entries)
    }

    pub fn get_log(&self, project_path: &str, max_count: usize) -> AppResult<Vec<GitCommit>> {
        let repo = self.get_repo(project_path)?;
        let head = repo.head()?;
        let mut revwalk = repo.revwalk()?;
        revwalk.set_sorting(Sort::TIME)?;
        revwalk.push(
            head.target()
                .ok_or_else(|| AppError::Git("HEAD has no target".into()))?,
        )?;
        let mut commits = Vec::new();
        for (i, oid_result) in revwalk.enumerate() {
            if i >= max_count {
                break;
            }
            let oid = oid_result?;
            let commit = repo.find_commit(oid)?;
            let author = commit.author();
            commits.push(GitCommit {
                oid: oid.to_string(),
                message: commit.summary().unwrap_or("").to_string(),
                author_name: author.name().unwrap_or("").to_string(),
                author_email: author.email().unwrap_or("").to_string(),
                timestamp: author.when().seconds(),
            });
        }
        Ok(commits)
    }

    pub fn stash(&self, project_path: &str, message: Option<&str>) -> AppResult<()> {
        let mut repo = self.get_repo(project_path)?;
        let signature = repo.signature()?;
        let mut index = repo.index()?;
        index.write()?;
        let stash_msg = message.unwrap_or("WIP");
        repo.stash_save(&signature, stash_msg, Some(StashFlags::DEFAULT))?;
        Ok(())
    }

    pub fn stash_list(&self, project_path: &str) -> AppResult<Vec<GitStashEntry>> {
        let mut repo = self.get_repo(project_path)?;
        let mut entries = Vec::new();
        repo.stash_foreach(|index, name, _oid| {
            entries.push(GitStashEntry {
                index,
                message: name.to_string(),
                branch: String::new(),
                timestamp: 0,
            });
            true
        })?;
        Ok(entries)
    }

    pub fn stash_pop(&self, project_path: &str, stash_index: usize) -> AppResult<()> {
        let mut repo = self.get_repo(project_path)?;
        repo.stash_pop(stash_index, None)?;
        Ok(())
    }

    pub fn stash_drop(&self, project_path: &str, stash_index: usize) -> AppResult<()> {
        let mut repo = self.get_repo(project_path)?;
        repo.stash_drop(stash_index)?;
        Ok(())
    }

    pub fn stash_apply(&self, project_path: &str, stash_index: usize) -> AppResult<()> {
        let mut repo = self.get_repo(project_path)?;
        repo.stash_apply(stash_index, None)?;
        Ok(())
    }

    pub fn get_tags(&self, project_path: &str) -> AppResult<Vec<GitTag>> {
        let repo = self.get_repo(project_path)?;
        let mut tags = Vec::new();
        for tag_result in &repo.tag_names(None)? {
            if let Some(name) = tag_result {
                if let Ok(reference) = repo.find_reference(&format!("refs/tags/{}", name)) {
                    if let Ok(commit) = reference.peel_to_commit() {
                        tags.push(GitTag {
                            name: name.to_string(),
                            target_oid: commit.id().to_string(),
                            message: reference
                                .peel(git2::ObjectType::Any)
                                .ok()
                                .and_then(|obj| obj.peel_to_tag().ok())
                                .and_then(|tag| tag.message().map(|s| s.to_string())),
                        });
                    }
                }
            }
        }
        Ok(tags)
    }

    pub fn diff_file(&self, project_path: &str, file_path: &str) -> AppResult<String> {
        let repo = self.get_repo(project_path)?;
        let head = repo.head()?.peel_to_tree()?;
        let mut opts = DiffOptions::new();
        opts.pathspec(file_path);
        let diff = repo.diff_tree_to_workdir(Some(&head), Some(&mut opts))?;
        let mut diff_text = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            diff_text.push_str(&String::from_utf8_lossy(line.content()));
            true
        })?;
        Ok(diff_text)
    }

    pub fn get_commit_diff(
        &self,
        project_path: &str,
        commit_oid_str: &str,
    ) -> AppResult<String> {
        let repo = self.get_repo(project_path)?;
        let oid = Oid::from_str(commit_oid_str)?;
        let commit = repo.find_commit(oid)?;
        let tree = commit.tree()?;
        let parent_tree = commit.parent(0).ok().map(|p| p.tree()).transpose()?;
        let mut opts = DiffOptions::new();
        let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut opts))?;
        let mut diff_text = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            diff_text.push_str(&String::from_utf8_lossy(line.content()));
            true
        })?;
        Ok(diff_text)
    }

    pub fn get_branch_ahead_behind(
        &self,
        project_path: &str,
        branch_name: &str,
    ) -> AppResult<(usize, usize)> {
        let repo = self.get_repo(project_path)?;
        let local = repo.find_branch(branch_name, BranchType::Local)?;
        let upstream = local.upstream().ok();
        let (ahead, behind) = if let Some(upstream_ref) = upstream {
            let local_oid = local
                .get()
                .target()
                .ok_or_else(|| AppError::Git("no local oid".into()))?;
            let upstream_oid = upstream_ref
                .get()
                .target()
                .ok_or_else(|| AppError::Git("no upstream oid".into()))?;
            repo.graph_ahead_behind(local_oid, upstream_oid)?
        } else {
            (0, 0)
        };
        Ok((ahead, behind))
    }

    pub fn get_current_branch(&self, project_path: &str) -> AppResult<String> {
        let repo = self.get_repo(project_path)?;
        let head = repo.head()?;
        Ok(head.shorthand().unwrap_or("HEAD").to_string())
    }
}

fn format_status(status: Status) -> String {
    if status.contains(Status::INDEX_NEW) {
        "added".into()
    } else if status.contains(Status::INDEX_MODIFIED) {
        "modified".into()
    } else if status.contains(Status::INDEX_DELETED) {
        "deleted".into()
    } else if status.contains(Status::INDEX_RENAMED) {
        "renamed".into()
    } else if status.contains(Status::WT_NEW) {
        "untracked".into()
    } else if status.contains(Status::WT_MODIFIED) {
        "changed".into()
    } else if status.contains(Status::WT_DELETED) {
        "deleted".into()
    } else if status.contains(Status::CONFLICTED) {
        "conflict".into()
    } else {
        "unknown".into()
    }
}

fn format_diff_status(status: git2::Delta) -> String {
    match status {
        git2::Delta::Added => "added",
        git2::Delta::Modified => "modified",
        git2::Delta::Deleted => "deleted",
        git2::Delta::Renamed => "renamed",
        git2::Delta::Copied => "copied",
        git2::Delta::Typechange => "typechange",
        _ => "unknown",
    }
    .into()
}

use super::model::{CoordinatorResult, text};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureClass {
    Transient,
    Validation,
    Permission,
    Configuration,
    Conflict,
    UncertainEffect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskResult {
    Succeeded,
    Failed(FailureClass),
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionState {
    Pending,
    Running,
    Finished(TaskResult),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub(super) id: String,
    pub(super) dependencies: Vec<String>,
    pub(super) write_scopes: Vec<String>,
    state: ExecutionState,
    attempts: u8,
}

impl ScheduledTask {
    pub fn new(
        id: &str,
        dependencies: Vec<String>,
        write_scopes: Vec<String>,
    ) -> CoordinatorResult<Self> {
        text(id, 128)?;
        if dependencies.len() > 128 || write_scopes.len() > 64 {
            return Err("task exceeds bounds".into());
        }
        for scope in &write_scopes {
            text(scope, 512)?;
            if scope.starts_with('/')
                || scope.contains(['\\', ':'])
                || scope
                    .split('/')
                    .any(|s| s == ".." || (s == "." && scope != ".") || s.is_empty())
            {
                return Err("write scope must be a normalized project-relative path".into());
            }
        }
        Ok(Self {
            id: id.into(),
            dependencies,
            write_scopes,
            state: ExecutionState::Pending,
            attempts: 0,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn state(&self) -> ExecutionState {
        self.state
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    tasks: BTreeMap<String, ScheduledTask>,
}

impl Schedule {
    pub fn new(tasks: Vec<ScheduledTask>) -> CoordinatorResult<Self> {
        if tasks.is_empty() || tasks.len() > 256 {
            return Err("schedule requires 1..256 tasks".into());
        }
        let count = tasks.len();
        let tasks: BTreeMap<_, _> = tasks.into_iter().map(|t| (t.id.clone(), t)).collect();
        if tasks.len() != count {
            return Err("duplicate task id".into());
        }
        for task in tasks.values() {
            if task.dependencies.iter().any(|id| !tasks.contains_key(id)) {
                return Err("unknown dependency".into());
            }
        }
        let mut visited = BTreeSet::new();
        loop {
            let before = visited.len();
            for task in tasks.values() {
                if task.dependencies.iter().all(|id| visited.contains(id)) {
                    visited.insert(task.id.clone());
                }
            }
            if visited.len() == count {
                return Ok(Self { tasks });
            }
            if visited.len() == before {
                return Err("task dependency cycle".into());
            }
        }
    }

    pub fn tasks(&self) -> impl Iterator<Item = &ScheduledTask> {
        self.tasks.values()
    }

    pub fn claim_ready(&mut self, capacity: usize) -> CoordinatorResult<Vec<String>> {
        if capacity == 0 || capacity > 32 {
            return Err("capacity must be 1..32".into());
        }
        let mut occupied: Vec<_> = self
            .tasks
            .values()
            .filter(|t| t.state == ExecutionState::Running)
            .flat_map(|t| t.write_scopes.clone())
            .collect();
        let running = self
            .tasks
            .values()
            .filter(|t| t.state == ExecutionState::Running)
            .count();
        let mut selected = Vec::new();
        for task in self.tasks.values() {
            if selected.len() + running >= capacity {
                break;
            }
            if task.state != ExecutionState::Pending {
                continue;
            }
            if !task
                .dependencies
                .iter()
                .all(|id| self.tasks[id].state == ExecutionState::Finished(TaskResult::Succeeded))
            {
                continue;
            }
            if task
                .write_scopes
                .iter()
                .any(|a| occupied.iter().any(|b| overlaps(a, b)))
            {
                continue;
            }
            occupied.extend(task.write_scopes.clone());
            selected.push(task.id.clone());
        }
        for id in &selected {
            let task = self.tasks.get_mut(id).unwrap();
            task.state = ExecutionState::Running;
            task.attempts += 1;
        }
        Ok(selected)
    }

    pub fn finish(&mut self, id: &str, result: TaskResult) -> CoordinatorResult<()> {
        let task = self.tasks.get_mut(id).ok_or("unknown task")?;
        if task.state != ExecutionState::Running {
            return Err("task is not running".into());
        }
        task.state = ExecutionState::Finished(result);
        Ok(())
    }

    pub fn retry(&mut self, id: &str) -> CoordinatorResult<()> {
        let task = self.tasks.get_mut(id).ok_or("unknown task")?;
        if task.state != ExecutionState::Finished(TaskResult::Failed(FailureClass::Transient))
            || task.attempts >= 3
        {
            return Err("only known transient failures with attempts remaining can retry".into());
        }
        task.state = ExecutionState::Pending;
        Ok(())
    }

    pub fn interrupt(&mut self) {
        for task in self
            .tasks
            .values_mut()
            .filter(|t| t.state == ExecutionState::Running)
        {
            task.state =
                ExecutionState::Finished(TaskResult::Failed(FailureClass::UncertainEffect));
        }
    }

    pub fn succeeded(&self) -> bool {
        self.tasks
            .values()
            .all(|t| t.state == ExecutionState::Finished(TaskResult::Succeeded))
    }
}

fn overlaps(a: &str, b: &str) -> bool {
    a == "."
        || b == "."
        || a == b
        || a.starts_with(&format!("{b}/"))
        || b.starts_with(&format!("{a}/"))
}

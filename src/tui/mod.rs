use anyhow::{Context, Result};
use crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, widgets::ListState};
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashSet},
    env,
    fmt::Display,
    iter,
};

use crate::{
    branch::{prep_available_branches, prep_available_remote_branches, prep_available_worktrees},
    config::{Config, QuickSelectHint, RefSource},
    fuzzy_match::{MatchRecord, fuzzy_match},
    git::read_cached_remote_branches,
    model::Model,
    types::{Branch, Head, Worktree},
};

mod cursor_nav;
mod render;
pub mod terminal;
mod theme;

enum ModifierKey {
    Alt,
    Option,
}

impl Display for ModifierKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let modifier = match self {
            ModifierKey::Alt => "Alt",
            ModifierKey::Option => "⌥",
        };
        write!(f, "{modifier}")
    }
}

pub struct InteractiveApp {
    vim_mode: bool,
    quick_select_hint: QuickSelectHint,

    modifier: ModifierKey,

    character_index: usize,
    input_mode: InputMode,
    alert: String,

    rows: Vec<Row>,
    // view state (technically a copy of the source data)
    view: SearchView,
}

#[derive(Clone)]
pub enum SearchView {
    Idle,
    Filtered {
        search_string: String, // invariant: non-empty
        list: Vec<Row>,
    },
}

impl SearchView {
    fn search_string(&self) -> &str {
        match self {
            SearchView::Idle => "",
            SearchView::Filtered { search_string, .. } => search_string,
        }
    }
}

pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug, Clone)]
pub enum Row {
    /// the currently checked out branch
    Current(Head),
    /// branches you can jump to (does not include currently checked out branch)
    LocalBranch(Branch),
    /// remote branches you are not tracking locally
    RemoteBranch { remote: String, branch: Branch },
    /// branches checked out in linked worktrees
    Worktree(Worktree),
}

impl Row {
    fn label<'a>(&'a self) -> Cow<'a, str> {
        match self {
            Row::Current(head) => Cow::Borrowed(head.label()),
            Row::LocalBranch(branch) => Cow::Borrowed(&branch.name),
            Row::Worktree(worktree) => Cow::Borrowed(worktree.head.label()),
            Row::RemoteBranch { remote, branch } => Cow::Owned(format!("{remote}/{}", branch.name)),
        }
    }
}

/// invariant: `search_string` is not empty
pub fn generate_ranked_rows(rows: &[Row], search_string: &str) -> Vec<Row> {
    assert!(!search_string.is_empty());

    let scored_rows: Vec<_> = rows
        .iter()
        .map(|r| MatchRecord {
            match_score: fuzzy_match(&r.label(), search_string),
            item: r,
        })
        .filter(|r| r.match_score > 0)
        .collect();

    let (mut worktrees, mut rest): (Vec<_>, Vec<_>) = scored_rows
        .into_iter()
        .partition(|r| matches!(r.item, Row::Worktree(_)));

    let by_score = |a: &MatchRecord<&Row>, b: &MatchRecord<&Row>| {
        b.match_score
            .cmp(&a.match_score)
            .then_with(|| a.item.label().cmp(&b.item.label()))
    };

    worktrees.sort_by(by_score);
    rest.sort_by(by_score);

    rest.append(&mut worktrees);

    rest.into_iter().map(|r| r.item.to_owned()).collect()
}

#[derive(Debug)]
pub enum AppExitStatus {
    Cancelled,
    StayedOnDetached,
    SelectedLocal(Branch),
    SelectedRemote(Branch, String),
    LocatedAt(Worktree),
}

impl InteractiveApp {
    pub fn new(state: &Model, app_config: &Config, active_head: &Head) -> Result<Self> {
        let branches = prep_available_branches(&state.branches, &state.worktrees);

        let local_branches: HashSet<String> = branches.iter().map(|b| b.name.clone()).collect();

        let cached_remote_branches =
            read_cached_remote_branches().context("Could not read local remote cache")?;

        let remotes: Vec<String> = app_config
            .general
            .sources
            .iter()
            .filter_map(|s| match s {
                RefSource::Remote(r) => Some(r.clone()),
                _ => None,
            })
            .collect();

        let remote_branches = prep_available_remote_branches(
            &cached_remote_branches,
            &remotes,
            &local_branches,
            active_head,
        );

        let worktrees = prep_available_worktrees(&state.worktrees, &state.active_worktree);

        Ok(Self {
            quick_select_hint: app_config.appearance.quick_select_hint.to_owned(),
            modifier: if env::consts::OS == "macos" {
                ModifierKey::Option
            } else {
                ModifierKey::Alt
            },
            vim_mode: app_config.general.vim_mode,
            input_mode: if app_config.general.vim_mode {
                InputMode::Normal
            } else {
                InputMode::Editing
            },

            character_index: 0,
            alert: String::from(""),

            rows: Self::make_rows(active_head.to_owned(), branches, remote_branches, worktrees),

            view: SearchView::Idle,
        })
    }

    pub fn make_rows(
        head: Head,
        branches: Vec<Branch>,
        remote_branches: BTreeMap<String, Vec<Branch>>,
        worktrees: Vec<Worktree>,
    ) -> Vec<Row> {
        iter::once(Row::Current(head))
            .chain(branches.into_iter().map(Row::LocalBranch))
            .chain(remote_branches.into_iter().flat_map(|(r, bs)| {
                let mut rows: Vec<Row> = bs
                    .into_iter()
                    .map(move |b| Row::RemoteBranch {
                        remote: r.clone(),
                        branch: b,
                    })
                    .collect();

                rows.sort_by(|a, b| a.label().cmp(&b.label()));
                rows
            }))
            .chain(worktrees.into_iter().map(Row::Worktree))
            .collect()
    }

    fn move_cursor_left_n(&mut self, n: usize) {
        let cursor_moved_left = self.character_index.saturating_sub(n);
        self.character_index = self.clamp_cursor(self.view.search_string(), cursor_moved_left);
    }

    fn move_cursor_left(&mut self) {
        self.move_cursor_left_n(1);
    }

    fn move_cursor_right_n(&mut self, n: usize) {
        let cursor_moved_right = self.character_index.saturating_add(n);
        self.character_index = self.clamp_cursor(self.view.search_string(), cursor_moved_right);
    }

    fn move_cursor_right(&mut self) {
        self.move_cursor_right_n(1);
    }

    fn set_search(&mut self, new_search: String) -> bool {
        if self.view.search_string() == new_search.as_str() {
            return false;
        }

        self.view = if new_search.is_empty() {
            SearchView::Idle
        } else {
            SearchView::Filtered {
                list: generate_ranked_rows(&self.rows, &new_search),
                search_string: new_search,
            }
        };
        true
    }

    fn enter_char(&mut self, new_char: char) -> bool {
        let mut s = self.view.search_string().to_string();
        s.insert(self.byte_index(&s), new_char);
        let search_changed = self.set_search(s);
        self.move_cursor_right();
        search_changed
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self, s: &str) -> usize {
        s.char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(s.len())
    }

    fn delete_char(&mut self) -> bool {
        let s = self.view.search_string();
        if self.character_index == 0 || s.is_empty() {
            return false;
        }
        let before = s.chars().take(self.character_index - 1);
        let after = s.chars().skip(self.character_index);
        let search_changed = self.set_search(before.chain(after).collect());
        self.move_cursor_left();
        search_changed
    }

    fn clamp_cursor(&self, s: &str, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, s.chars().count())
    }

    const fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn indexed_rows(&self) -> Vec<(Row, Option<usize>)> {
        let rows = match &self.view {
            SearchView::Idle => self.rows.clone(),
            SearchView::Filtered { list, .. } => list.clone(),
        };

        let mut i: usize = 0;
        rows.into_iter()
            .map(|r| match r {
                Row::LocalBranch(_) | Row::RemoteBranch { .. } if i < 10 => {
                    let idx = i;
                    i += 1;
                    (r, Some(idx))
                }
                _ => (r, None),
            })
            .collect()
    }

    fn quick_select(&self, digit: usize) -> Option<AppExitStatus> {
        self.indexed_rows().into_iter().find_map(|x| match x {
            (Row::LocalBranch(branch), Some(i)) if i == digit => {
                Some(AppExitStatus::SelectedLocal(branch))
            }
            (Row::RemoteBranch { remote, branch }, Some(i)) if i == digit => {
                Some(AppExitStatus::SelectedRemote(branch, remote))
            }
            _ => None,
        })
    }

    fn make_selection(&mut self, list_state: &ListState) -> Result<AppExitStatus> {
        let idx = list_state.selected().unwrap_or(0);

        let (row, _) = self
            .indexed_rows()
            .into_iter()
            .nth(idx)
            .context("selected branch no longer available")?;

        Ok(match row {
            Row::Current(_) => AppExitStatus::StayedOnDetached,
            Row::LocalBranch(branch) => AppExitStatus::SelectedLocal(branch),
            Row::RemoteBranch { branch, remote } => AppExitStatus::SelectedRemote(branch, remote),
            Row::Worktree(worktree) => AppExitStatus::LocatedAt(worktree),
        })
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<AppExitStatus> {
        let mut list_state = ListState::default();
        list_state.select_first();

        loop {
            terminal.draw(|frame| self.render(frame, &mut list_state))?;
            if let Some(key) = event::read()?.as_key_press_event() {
                match self.input_mode {
                    InputMode::Normal => match (key.code, key.modifiers) {
                        (KeyCode::Char('q'), KeyModifiers::NONE)
                        | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                            return Ok(AppExitStatus::Cancelled);
                        }

                        (KeyCode::Char('i'), KeyModifiers::NONE) => {
                            self.input_mode = InputMode::Editing
                        }

                        (KeyCode::Char('j'), KeyModifiers::NONE)
                        | (KeyCode::Down, KeyModifiers::NONE) => list_state.select_next(),

                        (KeyCode::Char('k'), KeyModifiers::NONE)
                        | (KeyCode::Up, KeyModifiers::NONE) => list_state.select_previous(),

                        (KeyCode::Enter, KeyModifiers::NONE) => {
                            return self.make_selection(&list_state);
                        }

                        (KeyCode::Char(c @ '0'..='9'), KeyModifiers::ALT) => {
                            if let Some(status) = self.quick_select(c as usize - '0' as usize) {
                                return Ok(status);
                            }
                        }

                        _ => {}
                    },

                    InputMode::Editing if key.kind == KeyEventKind::Press => {
                        match (key.code, key.modifiers) {
                            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                                return Ok(AppExitStatus::Cancelled);
                            }

                            // --- cursor movement ---

                            // Home / beginning of line
                            (KeyCode::Home, KeyModifiers::NONE)
                            | (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                                self.reset_cursor();
                            }

                            // End / end of line
                            (KeyCode::End, KeyModifiers::NONE)
                            | (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                                self.character_index = self.view.search_string().chars().count();
                            }

                            // Word left
                            (KeyCode::Left, KeyModifiers::ALT)
                            | (KeyCode::Char('b'), KeyModifiers::ALT) => {
                                self.character_index = cursor_nav::prev_word_boundary(
                                    self.view.search_string(),
                                    self.character_index,
                                );
                            }

                            // Word right
                            (KeyCode::Right, KeyModifiers::ALT)
                            | (KeyCode::Char('f'), KeyModifiers::ALT) => {
                                self.character_index = cursor_nav::next_word_boundary(
                                    self.view.search_string(),
                                    self.character_index,
                                );
                            }

                            // Char left
                            (KeyCode::Left, KeyModifiers::NONE) => self.move_cursor_left(),

                            // Char right
                            (KeyCode::Right, KeyModifiers::NONE) => self.move_cursor_right(),

                            (KeyCode::Down, KeyModifiers::NONE) => list_state.select_next(),

                            (KeyCode::Up, KeyModifiers::NONE) => list_state.select_previous(),

                            // --- deletion ---

                            // Delete word backward
                            (KeyCode::Backspace, KeyModifiers::ALT) => {
                                let stop = cursor_nav::prev_word_boundary(
                                    self.view.search_string(),
                                    self.character_index,
                                );
                                let before = self.view.search_string().chars().take(stop);
                                let after =
                                    self.view.search_string().chars().skip(self.character_index);
                                if self.set_search(before.chain(after).collect()) {
                                    list_state.select_first();
                                }
                                self.character_index = stop;
                            }

                            // Delete entire line
                            (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
                                if self.set_search("".to_string()) {
                                    list_state.select_first();
                                }
                                self.reset_cursor();
                            }

                            // Delete from cursor to end of line
                            (KeyCode::Char('k'), KeyModifiers::CONTROL)
                                if self.set_search(
                                    self.view
                                        .search_string()
                                        .chars()
                                        .take(self.character_index)
                                        .collect(),
                                ) =>
                            {
                                list_state.select_first();
                            }

                            // Delete from cursor to beginning of line
                            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                                if self.set_search(
                                    self.view
                                        .search_string()
                                        .chars()
                                        .skip(self.character_index)
                                        .collect(),
                                ) {
                                    list_state.select_first();
                                }
                                self.reset_cursor();
                            }

                            // Forward delete
                            (KeyCode::Delete, KeyModifiers::NONE) => {
                                let count = self.view.search_string().chars().count();
                                if self.character_index < count {
                                    let before = self
                                        .view
                                        .search_string()
                                        .chars()
                                        .take(self.character_index);
                                    let after = self
                                        .view
                                        .search_string()
                                        .chars()
                                        .skip(self.character_index + 1);
                                    if self.set_search(before.chain(after).collect()) {
                                        list_state.select_first();
                                    }
                                }
                            }

                            // Backspace
                            (KeyCode::Backspace, KeyModifiers::NONE) if self.delete_char() => {
                                list_state.select_first();
                            }

                            // --- quick select ---
                            (KeyCode::Char(c @ '0'..='9'), KeyModifiers::ALT) => {
                                if let Some(status) = self.quick_select(c as usize - '0' as usize) {
                                    return Ok(status);
                                }
                            }

                            // --- text input ---
                            (KeyCode::Char(to_insert), KeyModifiers::NONE)
                            | (KeyCode::Char(to_insert), KeyModifiers::SHIFT)
                                if self.enter_char(to_insert) =>
                            {
                                list_state.select_first();
                            }

                            // --- mode / confirm ---
                            (KeyCode::Enter, KeyModifiers::NONE) => {
                                return self.make_selection(&list_state);
                            }

                            (KeyCode::Esc, KeyModifiers::NONE) if self.vim_mode => {
                                self.input_mode = InputMode::Normal;
                            }

                            _ => {}
                        }
                    }

                    InputMode::Editing => {}
                }
            }
        }
    }
}

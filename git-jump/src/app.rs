use std::{iter, usize};

use crate::{
    input::{next_word_boundary, prev_word_boundary},
    list::generate_ranked_list,
    types::{Branch, Head, RankedSearchList, Worktree},
    ui::BRANCH_INDEX_PADD,
};
use anyhow::{Context, Result};
use crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{List, ListState},
};

pub struct InteractiveApp {
    pub character_index: usize,
    pub input_mode: InputMode,
    pub alert: String,

    /// the currently checked out branch
    pub head: Head,
    /// branches you can jump to (does not include currently checked out branch)
    pub branches: Vec<Branch>,
    /// branches checked out in linked worktrees
    pub worktrees: Vec<Worktree>,

    // view state (technically a copy of the source data)
    pub view: SearchView,
}

pub enum SearchView {
    Idle,
    Filtered {
        search_string: String, // invariant: non-empty
        list: RankedSearchList,
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

#[derive(Clone)]
enum Row {
    Current(Head),
    Branch { branch: Branch, index: BranchIndex },
    Worktree(Worktree),
}

#[derive(Clone)]
enum BranchIndex {
    QuickSelect(usize),
    Head,
    Bare,
}

pub enum AppExitStatus {
    Cancelled,
    StayedOnDetached,
    Selected(Branch),
    LocatedAt(Worktree),
}

impl InteractiveApp {
    pub fn new(head: Head, branches: Vec<Branch>, worktrees: Vec<Worktree>) -> Self {
        Self {
            character_index: 0,
            input_mode: InputMode::Normal,
            alert: String::from("no alert"),
            head,
            branches,
            worktrees,
            view: SearchView::Idle,
        }
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

    fn set_search(&mut self, new_search: String) {
        self.view = if new_search.is_empty() {
            SearchView::Idle
        } else {
            SearchView::Filtered {
                list: generate_ranked_list(&self.branches, &self.worktrees, &new_search),
                search_string: new_search,
            }
        };
    }

    fn enter_char(&mut self, new_char: char) {
        let mut s = self.view.search_string().to_string();
        s.insert(self.byte_index(&s), new_char);
        self.set_search(s);
        self.move_cursor_right();
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

    fn delete_char(&mut self) {
        let s = self.view.search_string();
        if self.character_index == 0 || s.is_empty() {
            return;
        }
        let before = s.chars().take(self.character_index - 1);
        let after = s.chars().skip(self.character_index);
        self.set_search(before.chain(after).collect());
        self.move_cursor_left();
    }

    fn clamp_cursor(&self, s: &str, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, s.chars().count())
    }

    const fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn rows(&self) -> Vec<Row> {
        match &self.view {
            SearchView::Idle => iter::once(Row::Current(self.head.clone()))
                .chain(self.branches.iter().enumerate().map(|(i, b)| Row::Branch {
                    branch: b.clone(),
                    index: if i <= 9 {
                        BranchIndex::QuickSelect(i)
                    } else {
                        BranchIndex::Bare
                    },
                }))
                .chain(self.worktrees.iter().map(|w| Row::Worktree(w.clone())))
                .collect(),

            SearchView::Filtered { list, .. } => {
                let mut i = 0;

                list.available
                    .iter()
                    .map(|b| Row::Branch {
                        branch: b.clone(),
                        index: if b.is_head(&self.head) {
                            BranchIndex::Head
                        } else if i <= 9 {
                            let idx = i;
                            i += 1;
                            BranchIndex::QuickSelect(idx)
                        } else {
                            BranchIndex::Bare
                        },
                    })
                    .chain(list.worktrees.iter().map(|w| Row::Worktree(w.clone())))
                    .collect()
            }
        }
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
                            let idx = list_state.selected().unwrap_or(0);
                            let row = self
                                .rows()
                                .into_iter()
                                .nth(idx)
                                .context("selected branch no longer available")?;

                            return Ok(match row {
                                Row::Current(_) => AppExitStatus::StayedOnDetached,
                                Row::Branch { branch, .. } => AppExitStatus::Selected(branch),
                                Row::Worktree(worktree) => AppExitStatus::LocatedAt(worktree),
                            });
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
                                let curr = self.character_index;
                                let next = prev_word_boundary(
                                    self.view.search_string(),
                                    self.character_index,
                                );
                                self.character_index = next;
                                self.alert = format!("move from {} to {}", curr, next)
                            }

                            // Word right
                            (KeyCode::Right, KeyModifiers::ALT)
                            | (KeyCode::Char('f'), KeyModifiers::ALT) => {
                                let curr = self.character_index;
                                let next = next_word_boundary(
                                    self.view.search_string(),
                                    self.character_index,
                                );
                                self.character_index = next;
                                self.alert = format!("move from {} to {}", curr, next)
                            }

                            // Char left
                            (KeyCode::Left, KeyModifiers::NONE) => self.move_cursor_left(),

                            // Char right
                            (KeyCode::Right, KeyModifiers::NONE) => self.move_cursor_right(),

                            // --- deletion ---

                            // Delete word backward
                            (KeyCode::Backspace, KeyModifiers::ALT) => {
                                let stop = prev_word_boundary(
                                    self.view.search_string(),
                                    self.character_index,
                                );
                                let before = self.view.search_string().chars().take(stop);
                                let after =
                                    self.view.search_string().chars().skip(self.character_index);
                                self.set_search(before.chain(after).collect());
                                self.character_index = stop;
                            }

                            // Delete entire line
                            (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
                                self.set_search("".to_string());
                                self.reset_cursor();
                            }

                            // Delete from cursor to end of line
                            (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                                self.set_search(
                                    self.view
                                        .search_string()
                                        .chars()
                                        .take(self.character_index)
                                        .collect(),
                                );
                            }

                            // Delete from cursor to beginning of line
                            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                                self.set_search(
                                    self.view
                                        .search_string()
                                        .chars()
                                        .skip(self.character_index)
                                        .collect(),
                                );
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
                                    self.set_search(before.chain(after).collect());
                                }
                            }

                            // Backspace
                            (KeyCode::Backspace, KeyModifiers::NONE) => {
                                self.delete_char();
                            }

                            // --- text input ---
                            (KeyCode::Char(to_insert), KeyModifiers::NONE)
                            | (KeyCode::Char(to_insert), KeyModifiers::SHIFT) => {
                                self.enter_char(to_insert);
                            }

                            // --- mode / confirm ---
                            (KeyCode::Enter, KeyModifiers::NONE) => {
                                // TODO: confirm selection
                            }

                            (KeyCode::Esc, KeyModifiers::NONE) => {
                                self.input_mode = InputMode::Normal;
                            }

                            // --- debug catch-all ---
                            (code, mods) => {
                                self.alert = format!("code={:?} mods={:?}", code, mods);
                            }
                        }
                    }

                    InputMode::Editing => {}
                }
            }
        }
    }

    fn render(&self, frame: &mut Frame, list_state: &mut ListState) {
        let num_branches = self.branches.len();
        let num_worktrees = self.worktrees.len();
        let num_rows = (1 + num_branches + num_worktrees) as u16;

        let constraints: Vec<Constraint> = vec![
            Constraint::Length(1),
            Constraint::Length(num_rows),
            Constraint::Length(1),
        ];

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(constraints);

        let [search_area, list, status_area] = frame.area().layout(&layout);

        self.render_search_row(frame, search_area);

        self.render_scrollable_list(frame, list, list_state);

        self.render_status(frame, status_area);
    }

    fn render_search_row(&self, frame: &mut Frame, area: Rect) {
        let search_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(3),
                Constraint::Fill(1),
                Constraint::Length(20),
            ]);

        let [_, search_area, hint_area] = area.layout(&search_layout);

        let search_string = if self.view.search_string().is_empty() {
            "Search"
        } else {
            &self.view.search_string()
        };

        let cursor_style = match self.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => {
                frame.set_cursor_position(Position::new(
                    search_area.x + self.character_index as u16,
                    search_area.y,
                ));
                Style::default().add_modifier(Modifier::SLOW_BLINK)
            }
        };

        frame.render_widget(
            Text::from(search_string)
                .bg(Color::DarkGray)
                .patch_style(cursor_style),
            search_area,
        );

        frame.render_widget(
            Text::from(format!("⌥+0..{} quick select", self.branches.len())).bg(Color::DarkGray),
            hint_area,
        );
    }

    fn render_status(&self, frame: &mut Frame, area: Rect) {
        let status_span = match self.input_mode {
            InputMode::Normal => "   NORMAL",
            InputMode::Editing => "   INPUT",
        };

        let alert_span = format!("     {}", self.alert);

        frame.render_widget(
            Line::from(vec![Span::from(status_span), Span::from(alert_span)])
                .bg(Color::Rgb(15, 23, 43)),
            area,
        );
    }

    fn render_scrollable_list(&self, frame: &mut Frame, area: Rect, list_state: &mut ListState) {
        let longest_branch_name = self
            .branches
            .iter()
            .map(|x| x.name.len())
            .max()
            .unwrap_or(0);
        let longest_worktree_name = self
            .worktrees
            .iter()
            .map(|x| x.head.label().len())
            .max()
            .unwrap_or(0);
        let longest_entry = self
            .head
            .label()
            .len()
            .max(longest_branch_name)
            .max(longest_worktree_name);
        let longest_dir = self
            .worktrees
            .iter()
            .map(|x| x.dir.to_str().unwrap().len())
            .max()
            .unwrap_or(0);

        let rows = self.rows();

        let items: Vec<_> = rows
            .iter()
            .map(|r| match r {
                Row::Current(h) => render_head(h),
                Row::Branch { branch, index } => match index {
                    BranchIndex::QuickSelect(idx) => {
                        render_branch(branch, format!(" {idx} "), longest_entry)
                    }
                    BranchIndex::Bare => {
                        render_branch(branch, BRANCH_INDEX_PADD.to_string(), longest_entry)
                    }
                    BranchIndex::Head => render_head(&self.head),
                },

                Row::Worktree(w) => render_worktree(w, longest_entry, longest_dir),
            })
            .collect();

        let list = List::new(items).highlight_style(Style::new().bg(Color::LightGreen));

        frame.render_stateful_widget(list, area, list_state);
    }
}

fn render_head(h: &'_ Head) -> Line<'_> {
    Line::from(vec![
        Span::from(BRANCH_INDEX_PADD),
        Span::from(h.label()).bg(Color::Cyan),
    ])
}

fn render_branch(b: &'_ Branch, index_label: String, max_entry_len: usize) -> Line<'_> {
    Line::from(vec![
        Span::from(index_label).bg(Color::DarkGray),
        Span::from(format!("{:width$}", b.name, width = max_entry_len)).bg(Color::White),
    ])
}

fn render_worktree(w: &'_ Worktree, max_entry_len: usize, max_dir_len: usize) -> Line<'_> {
    Line::from(vec![
        "   ".into(),
        Span::from(format!("{:width$}", w.head.label(), width = max_entry_len)).bg(Color::Blue),
        Span::from(format!(
            "  {:width$}",
            w.dir.to_string_lossy(),
            width = max_dir_len
        ))
        .bg(Color::DarkGray),
    ])
}

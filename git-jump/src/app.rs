use std::iter;

use crate::{
    input::{next_word_boundary, prev_word_boundary},
    list::{SearchList, generate_list},
    types::{Branch, Head, Worktree},
};
use anyhow::Result;
use crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{List, ListState},
};

enum SearchListState {
    Idle {
        /// the currently checked out branch
        head: Head,
        /// branches you can jump to (does not include currently checked out branch)
        available_branches: Vec<Branch>,
        /// branches checked out in linked worktrees
        in_worktrees: Vec<Worktree>,
    },
    InSearch {
        /// non-empty search string
        search_string: String,
        /// all branches you can jump to that match the search input
        available: Vec<Head>,
        /// all worktrees that match the search input
        worktrees: Vec<Worktree>,
    },
}

pub struct InteractiveApp {
    pub input: String,
    /// book keeping
    pub character_index: usize,
    pub input_mode: InputMode,
    /// actual data
    pub list: SearchList,
    /// testing new shape
    // pub data: SearchListState,
    /// debug
    pub alert: String,
}

pub enum InputMode {
    Normal,
    Editing,
}

impl InteractiveApp {
    fn new(
        current_head: Head,
        branches: Vec<Branch>,
        worktrees: Vec<Worktree>,
        search_string: &str,
    ) -> Self {
        Self {
            input: search_string.to_string(),
            input_mode: InputMode::Normal,
            alert: String::from("no alert"),
            character_index: 0,
            list: generate_list(current_head, branches, worktrees, search_string),
        }
    }

    fn move_cursor_left_n(&mut self, n: usize) {
        let cursor_moved_left = self.character_index.saturating_sub(n);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_left(&mut self) {
        self.move_cursor_left_n(1);
    }

    fn move_cursor_right_n(&mut self, n: usize) {
        let cursor_moved_right = self.character_index.saturating_add(n);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn move_cursor_right(&mut self) {
        self.move_cursor_right_n(1);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        if self.character_index != 0 {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    const fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let mut list_state = ListState::default();
        list_state.select_first();

        loop {
            terminal.draw(|frame| self.render(frame, &mut list_state))?;
            if let Some(key) = event::read()?.as_key_press_event() {
                match self.input_mode {
                    InputMode::Normal => match (key.code, key.modifiers) {
                        (KeyCode::Char('q'), KeyModifiers::NONE)
                        | (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(()),

                        (KeyCode::Char('i'), KeyModifiers::NONE) => {
                            self.input_mode = InputMode::Editing
                        }

                        (KeyCode::Char('j'), KeyModifiers::NONE)
                        | (KeyCode::Down, KeyModifiers::NONE) => list_state.select_next(),

                        (KeyCode::Char('k'), KeyModifiers::NONE)
                        | (KeyCode::Up, KeyModifiers::NONE) => list_state.select_previous(),

                        _ => {}
                    },

                    InputMode::Editing if key.kind == KeyEventKind::Press => {
                        match (key.code, key.modifiers) {
                            (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(()),

                            // --- cursor movement ---

                            // Home / beginning of line
                            (KeyCode::Home, KeyModifiers::NONE)
                            | (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                                self.reset_cursor();
                            }

                            // End / end of line
                            (KeyCode::End, KeyModifiers::NONE)
                            | (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                                self.character_index = self.input.chars().count();
                            }

                            // Word left
                            (KeyCode::Left, KeyModifiers::ALT)
                            | (KeyCode::Char('b'), KeyModifiers::ALT) => {
                                let curr = self.character_index;
                                let next = prev_word_boundary(&self.input, self.character_index);
                                self.character_index = next;
                                self.alert = format!("move from {} to {}", curr, next)
                            }

                            // Word right
                            (KeyCode::Right, KeyModifiers::ALT)
                            | (KeyCode::Char('f'), KeyModifiers::ALT) => {
                                let curr = self.character_index;
                                let next = next_word_boundary(&self.input, self.character_index);
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
                                let stop = prev_word_boundary(&self.input, self.character_index);
                                let before = self.input.chars().take(stop);
                                let after = self.input.chars().skip(self.character_index);
                                self.input = before.chain(after).collect();
                                self.character_index = stop;
                            }

                            // Delete entire line
                            (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
                                self.input.clear();
                                self.reset_cursor();
                            }

                            // Delete from cursor to end of line
                            (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                                self.input =
                                    self.input.chars().take(self.character_index).collect();
                            }

                            // Delete from cursor to beginning of line
                            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                                self.input =
                                    self.input.chars().skip(self.character_index).collect();
                                self.reset_cursor();
                            }

                            // Forward delete
                            (KeyCode::Delete, KeyModifiers::NONE) => {
                                let count = self.input.chars().count();
                                if self.character_index < count {
                                    let before = self.input.chars().take(self.character_index);
                                    let after = self.input.chars().skip(self.character_index + 1);
                                    self.input = before.chain(after).collect();
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

        render_scrollable_list(
            frame,
            list,
            list_state,
            self.current_head.clone(),
            self.branches.clone(),
            self.worktrees.clone(),
        );

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

        let search_string = if self.input.is_empty() {
            "Search"
        } else {
            &self.input
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
}

fn render_scrollable_list(
    frame: &mut Frame,
    area: Rect,
    list_state: &mut ListState,
    current_head: String,
    branches: Vec<String>,
    worktrees: Vec<Worktree>,
) {
    let longest_branch_len = branches.iter().map(|x| x.len()).max().unwrap_or(0);
    let longest_worktree_name_len = worktrees
        .iter()
        .map(|x| x.head.into_label().len())
        .max()
        .unwrap_or(0);
    let longest_entry_len = current_head
        .len()
        .max(longest_branch_len)
        .max(longest_worktree_name_len);

    let longest_worktree_dir_len = worktrees
        .iter()
        .map(|x| x.dir.to_str().unwrap().len())
        .max()
        .unwrap_or(0);

    let cur_span = Span::from(current_head).bg(Color::Cyan);
    let current_head_line = Line::from(vec!["   ".into(), cur_span]);

    let branch_lines: Vec<Line> = branches
        .iter()
        .enumerate()
        .map(|(i, branch)| {
            Line::from(vec![
                Span::from(format!(" {i} ")).bg(Color::DarkGray),
                Span::from(format!("{branch:width$}", width = longest_entry_len)).bg(Color::White),
            ])
        })
        .collect();

    let worktree_lines: Vec<Line> = worktrees
        .iter()
        .map(|worktree| {
            Line::from(vec![
                "   ".into(),
                Span::from(format!(
                    "{:width$}",
                    worktree.head.into_label(),
                    width = longest_entry_len
                ))
                .bg(Color::Blue),
                Span::from(format!(
                    "  {:width$}",
                    worktree.dir.to_str().unwrap(),
                    width = longest_worktree_dir_len
                ))
                .bg(Color::DarkGray),
            ])
        })
        .collect();

    let items: Vec<_> = iter::once(current_head_line)
        .chain(branch_lines)
        .chain(worktree_lines)
        .collect();

    let list = List::new(items).highlight_style(Style::new().bg(Color::LightGreen));

    frame.render_stateful_widget(list, area, list_state);
}

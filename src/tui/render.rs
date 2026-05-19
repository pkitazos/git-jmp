use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{List, ListState},
};

use crate::{
    config::QuickSelectHint,
    print::BRANCH_INDEX_PADD,
    tui::{BranchIndex, InputMode, InteractiveApp, Row},
    types::{Branch, Head, Worktree},
};

impl InteractiveApp {
    pub fn render(&self, frame: &mut Frame, list_state: &mut ListState) {
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
            .constraints(constraints);

        let [search_area, list, status_area] = frame.area().layout(&layout);

        self.render_search_row(frame, search_area);

        self.render_scrollable_list(frame, list, list_state);

        if self.vim_mode {
            self.render_status(frame, status_area);
        }
    }

    fn render_search_row(&self, frame: &mut Frame, area: Rect) {
        let quick_select_hint = match self.quick_select_hint {
            QuickSelectHint::Full => {
                format!("{}+0..{} quick select", self.modifier, self.branches.len())
            }
            QuickSelectHint::Compact => format!("{}+0..{}", self.modifier, self.branches.len()),
            QuickSelectHint::Hidden => "".to_string(),
        };

        let search_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(3),
                Constraint::Fill(1),
                Constraint::Length(quick_select_hint.len() as u16),
            ]);

        let [_, search_area, hint_area] = area.layout(&search_layout);

        let search_string = if self.view.search_string().is_empty() {
            "Search"
        } else {
            self.view.search_string()
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

        frame.render_widget(Text::from(quick_select_hint).bg(Color::DarkGray), hint_area);
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

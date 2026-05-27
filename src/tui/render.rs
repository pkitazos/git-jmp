use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{List, ListState},
};

use super::theme;
use crate::{
    config::QuickSelectHint,
    print::INDEX_PADD,
    tui::{InputMode, InteractiveApp, Row, SearchView},
    types::{Branch, Head, Worktree},
};

impl InteractiveApp {
    pub fn render(&self, frame: &mut Frame, list_state: &mut ListState) {
        let rows = self.indexed_rows();

        let constraints: Vec<Constraint> = vec![
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ];

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints);

        let [search_area, list, status_area] = frame.area().layout(&layout);

        self.render_search_row(&rows, frame, search_area);

        self.render_scrollable_list(&rows, frame, list, list_state);

        if self.vim_mode {
            self.render_status(frame, status_area);
        }
    }

    fn render_search_row(&self, rows: &[(Row, Option<usize>)], frame: &mut Frame, area: Rect) {
        let selectable_count = rows
            .iter()
            .filter(|(r, _)| matches!(r, Row::LocalBranch(_) | Row::RemoteBranch { .. }))
            .count();

        let max_quick_select = selectable_count.min(10).saturating_sub(1);

        let quick_select_hint = if selectable_count == 0 {
            "".to_string()
        } else {
            let range = if selectable_count == 1 {
                format!("{}+0", self.modifier)
            } else {
                format!("{}+0..{max_quick_select}", self.modifier)
            };

            match self.quick_select_hint {
                QuickSelectHint::Full => format!("{range} quick select"),
                QuickSelectHint::Compact => range,
                QuickSelectHint::Hidden => "".to_string(),
            }
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
                Style::default()
            }
        };

        frame.render_widget(
            Text::from(search_string)
                .fg(theme::MUTED)
                .patch_style(cursor_style),
            search_area,
        );

        frame.render_widget(Text::from(quick_select_hint).fg(theme::MUTED), hint_area);
    }

    fn render_status(&self, frame: &mut Frame, area: Rect) {
        let mode = match self.input_mode {
            InputMode::Normal => "[NORMAL]",
            InputMode::Editing => "[INPUT]",
        };

        let total = self.rows.len();
        let count = match &self.view {
            SearchView::Filtered { list, .. } => format!("{}/{total}", list.len()),
            SearchView::Idle => total.to_string(),
        };

        let status_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(count.len() as u16 + 1),
            ]);

        let [left, right] = area.layout(&status_layout);

        frame.render_widget(
            Line::from(vec![
                Span::from(format!("{INDEX_PADD}{mode}")),
                Span::from(self.alert.to_string()),
            ])
            .fg(theme::STATUS_BAR),
            left,
        );

        frame.render_widget(Text::from(format!("{count} ")).fg(theme::STATUS_BAR), right);
    }

    fn render_scrollable_list(
        &self,
        rows: &[(Row, Option<usize>)],
        frame: &mut Frame,
        area: Rect,
        list_state: &mut ListState,
    ) {
        let SpanWidths {
            head,
            local_branch,
            remote_branch,
            worktree_branch,
            worktree_dir: longest_dir,
        } = span_widths(rows);

        let longest_entry = head
            .max(local_branch)
            .max(remote_branch)
            .max(worktree_branch);

        let items: Vec<_> = rows
            .iter()
            .map(|x| match x {
                (Row::Current(h), _) => render_head(h),

                (Row::LocalBranch(branch), Some(i)) => {
                    render_branch(branch, format!(" {i} "), longest_entry)
                }

                (Row::LocalBranch(branch), None) => {
                    render_branch(branch, INDEX_PADD.to_string(), longest_entry)
                }

                (Row::RemoteBranch { branch, remote }, Some(i)) => render_remote_branch(
                    branch,
                    format!(" {i} "),
                    remote.to_string(),
                    longest_entry,
                ),
                (Row::RemoteBranch { branch, remote }, None) => render_remote_branch(
                    branch,
                    INDEX_PADD.to_string(),
                    remote.to_string(),
                    longest_entry,
                ),

                (Row::Worktree(w), _) => render_worktree(w, longest_entry, longest_dir),
            })
            .collect();

        let list = List::new(items).highlight_style(Style::new().fg(theme::HIGHLIGHT));

        frame.render_stateful_widget(list, area, list_state);
    }
}

fn render_head(h: &'_ Head) -> Line<'_> {
    Line::from(vec![
        Span::from(INDEX_PADD),
        Span::from(h.label()).fg(theme::HEAD),
    ])
}

fn render_branch(b: &'_ Branch, index_label: String, max_entry_len: usize) -> Line<'_> {
    Line::from(vec![
        Span::from(index_label).fg(theme::BRANCH_INDEX),
        Span::from(format!("{:width$}", b.name, width = max_entry_len)).fg(theme::BRANCH),
    ])
}

fn render_remote_branch(
    b: &'_ Branch,
    index_label: String,
    remote: String,
    max_entry_len: usize,
) -> Line<'_> {
    let branch_name_len = max_entry_len.saturating_sub(remote.len() + 1);

    Line::from(vec![
        Span::from(index_label).fg(theme::BRANCH_INDEX),
        Span::from(format!("{remote}/")).fg(theme::REMOTE),
        Span::from(format!("{:width$}", b.name, width = branch_name_len)).fg(theme::BRANCH),
    ])
}

fn render_worktree(w: &'_ Worktree, max_entry_len: usize, max_dir_len: usize) -> Line<'_> {
    Line::from(vec![
        Span::from(INDEX_PADD),
        Span::from(format!("{:width$}", w.head.label(), width = max_entry_len)).fg(theme::WORKTREE),
        Span::from(format!(
            "  {:width$}",
            w.dir.to_string_lossy(),
            width = max_dir_len
        ))
        .fg(theme::WORKTREE_DIR),
    ])
}

struct SpanWidths {
    head: usize,
    local_branch: usize,
    remote_branch: usize,
    worktree_branch: usize,
    worktree_dir: usize,
}

fn span_widths(rows: &[(Row, Option<usize>)]) -> SpanWidths {
    let mut head: usize = 0;
    let mut local_branch: usize = 0;
    let mut remote_branch: usize = 0;
    let mut worktree_branch: usize = 0;
    let mut worktree_dir: usize = 0;

    for (r, _) in rows {
        match &r {
            Row::Current(_) => head = head.max(r.label().len()),
            Row::LocalBranch(_) => local_branch = local_branch.max(r.label().len()),
            Row::RemoteBranch { .. } => remote_branch = remote_branch.max(r.label().len()),
            Row::Worktree(w) => {
                worktree_branch = worktree_branch.max(r.label().len());
                worktree_dir = worktree_dir.max(w.dir.to_string_lossy().len())
            }
        }
    }

    SpanWidths {
        head,
        local_branch,
        remote_branch,
        worktree_branch,
        worktree_dir,
    }
}

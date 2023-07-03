use std::fmt::{Display};

use crossterm::event::{KeyCode, KeyEvent};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use tui::widgets::ListState;

use crate::app::Focusable;

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
    pub items_filtered: Vec<T>,
    pub filter: String
}

impl<T> StatefulList<T> where T: Copy + Display + PartialEq<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        let copy_of_items = items.clone();

        let mut state = ListState::default();
        state.select(Some(0));

        StatefulList {
            state,
            items,
            items_filtered: copy_of_items,
            filter: String::new(),
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i + 1 >= self.items.len() {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn get_selected(&self) -> Option<&T> {
        self.state.selected().and_then(|index| { self.items.get(index)})
    }

    fn filter(&mut self) {
        let matcher = SkimMatcherV2::default();
        let selection = self.state.selected()
            .and_then(|x| self.items_filtered.get(x)).copied();
        self.items_filtered = self.items.iter()
            .map(|x| (x, matcher.fuzzy_match(format!("{x}").as_str(), self.filter.as_str())))
            .filter(|(_, fuzz)| fuzz.is_some())
            .map(|(x, _)| *x)
            .collect();

        self.state = ListState::default();
        if let Some(sel) = selection {
            let new_selection = self.items_filtered.iter().position(|&x| x == sel);
            self.state.select(new_selection);
        }
    }
}

impl<T> Focusable for StatefulList<T> where T: Copy + Display + PartialEq<T> {
    fn on_input(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char(char) => {
                self.filter += &char.to_string();
                self.filter();
            }
            KeyCode::Backspace => {
                self.filter.pop();
                self.filter();
            }
            _ => {}
        }
    }


}

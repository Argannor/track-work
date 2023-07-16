use std::fmt::{Display};

use crossterm::event::{KeyCode, KeyEvent};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use tui::widgets::ListState;

use crate::app::Focusable;

#[derive(Debug, Clone)]
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
    pub items_filtered: Vec<T>,
    pub filter: String
}

impl<T> StatefulList<T> where T: Copy + Display + PartialEq<T> {
    pub fn with_items<I: IntoIterator<Item = T>>(items: I) -> StatefulList<T> {
        let items: Vec<T> = items.into_iter().collect();
        let copy_of_items: Vec<T> = items.clone();

        let mut state = ListState::default();

        if !items.is_empty() {
            state.select(Some(0));
        }

        StatefulList {
            state,
            items,
            items_filtered: copy_of_items,
            filter: String::new(),
        }
    }

    pub fn next(&mut self) {
        let item_count = self.items.len();
        if item_count == 0 {
            self.state.select(None);
        }

        let i = match self.state.selected() {
            Some(i) if i + 1 >= item_count => 0,
            Some(i) => i + 1,
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let item_count = self.items.len();
        if item_count == 0 {
            self.state.select(None);
        }

        let i = match self.state.selected() {
            Some(i) if i == 0 => item_count - 1,
            Some(i) => i - 1,
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
            .and_then(|x| self.items_filtered.get(x))
            .cloned();
        let filter = self.filter.as_str();
        self.items_filtered = self.items.iter()
            .map(|x| (x, matcher.fuzzy_match(format!("{x}").as_str(), filter)))
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
    fn on_input(&mut self, event: &KeyEvent) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_previous_next() {
        let mut list = StatefulList::with_items(vec!["a", "b", "c"]);

        assert_eq!(list.get_selected(), Some(&"a"));
        list.next();
        assert_eq!(list.get_selected(), Some(&"b"));
        list.next();
        assert_eq!(list.get_selected(), Some(&"c"));
        list.next();
        assert_eq!(list.get_selected(), Some(&"a"));
        list.previous();
        assert_eq!(list.get_selected(), Some(&"c"));
        list.previous();
        assert_eq!(list.get_selected(), Some(&"b"));
        list.previous();
        assert_eq!(list.get_selected(), Some(&"a"));
    }

    #[test]
    fn test_empty_list() {
        let mut list: StatefulList<&'static str> = StatefulList::with_items(vec![]);

        assert_eq!(list.get_selected(), None);
        list.next();
        assert_eq!(list.get_selected(), None);
        list.previous();
        assert_eq!(list.get_selected(), None);
    }
}
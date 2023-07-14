
use chrono::{Datelike, Days, IsoWeek, NaiveDate, Utc, Weekday};
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::widgets::{Paragraph, StatefulWidget, Widget};

pub struct WeekPickerState {
    pub date: NaiveDate,
}

impl WeekPickerState {
    pub fn increment(&mut self) {
        self.date = self.date + Days::new(7);
    }
    pub fn decrement(&mut self) {
        self.date = self.date - Days::new(7);
    }
    pub fn week(&self) -> IsoWeek {
        self.date.iso_week()
    }
    pub fn start_and_end(&self) -> (NaiveDate, NaiveDate) {
        (self.date.week(Weekday::Mon).first_day(), self.date.week(Weekday::Mon).last_day())
    }
}

impl Default for WeekPickerState {
    fn default() -> Self {
        WeekPickerState{
            date: Utc::now().date_naive()
        }
    }
}

pub struct WeekPicker {
}

impl StatefulWidget for WeekPicker {
    type State = WeekPickerState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let week = state.week();
        let (start, end) = state.start_and_end();
        let paragraph = Paragraph::new(format!("select week: ← {:4}/{:2} → ({} - {})", week.year(), week.week(), start, end));
        paragraph.render(area, buf)
    }
}

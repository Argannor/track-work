use crate::app_config::{AppConfig, Client, ProjectClient};
use crate::repository::model::WorkRecord;
use crate::SETTINGS;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Report {
    pub rows: Vec<Row>,
}

#[derive(Clone, Debug)]
pub struct Row {
    cells: Vec<Cell>,
}

#[derive(Clone, Debug)]
struct Cell {
    #[allow(dead_code)]
    title: String,
    value: String,
}

impl<'a> From<&Row> for tui::widgets::Row<'a> {
    fn from(value: &Row) -> Self {
        let cells: Vec<tui::widgets::Cell> = value.cells.iter().map(|x| x.into()).collect();
        tui::widgets::Row::new(cells)
    }
}

impl<'a> From<&Cell> for tui::widgets::Cell<'a> {
    fn from(value: &Cell) -> Self {
        tui::widgets::Cell::from(value.value.clone())
    }
}

impl Report {
    pub fn new_pct(records: &Vec<WorkRecord>) -> Report {
        let cfg = SETTINGS.read().expect("could not acquire lock");
        let projects: HashMap<&String, &Vec<ProjectClient>> =
            cfg.projects.iter().map(|p| (&p.name, &p.clients)).collect();
        let clients: HashMap<&String, &Client> = cfg.clients.iter().map(|c| (&c.name, c)).collect();
        let mut entries: HashMap<&String, f64> = cfg
            .clients
            .iter()
            .filter(|c| c.data.contains_key("psp"))
            .map(|c| (&c.data["psp"], 0.))
            .collect();

        for record in records {
            let project_clients = *projects.get(&record.name).unwrap_or_else(|| {
                panic!(
                    "no clients found for project {}, please revise configuration",
                    record.name
                )
            });
            if project_clients.is_empty() {
                panic!(
                    "no clients found for project {}, please revise configuration",
                    record.name
                )
            }
            if project_clients
                .iter()
                .map(|p| p.ratio)
                .reduce(|a, b| a + b)
                .unwrap_or(0.)
                != 1.
            {
                panic!(
                    "the ratios of the clients for project {} doesnt sum up to 1",
                    record.name
                )
            }
            let duration: f64 = record.calculate_duration().num_minutes() as f64;
            for p in project_clients {
                let client = clients
                    .get(&p.name)
                    .unwrap_or_else(|| panic!("client {} not found in clients", p.name));
                let ratio = p.ratio;
                let psp = &client.data["psp"];
                let old_value = entries[psp];
                let new_value: f64 = (ratio * duration) + old_value;
                entries.insert(psp, new_value);
            }
        }

        let mut rows: Vec<Row> = entries
            .iter()
            .map(|(psp, hours)| {
                vec![
                    Cell {
                        title: "psp".to_string(),
                        value: (*psp).clone(),
                    },
                    Cell {
                        title: "hours".to_string(),
                        value: format!("{:.2}h", hours / 60.),
                    },
                ]
            })
            .map(|cells| Row { cells })
            .collect();

        Report::sort_rows(&cfg, &mut rows);
        let sum: f64 = entries.into_values().reduce(|a, b| a + b).unwrap_or(0.);
        rows.push(Row { cells: vec![] });
        rows.push(Row {
            cells: vec![
                Cell {
                    title: "Total".to_string(),
                    value: "Total".to_string(),
                },
                Cell {
                    title: "hours".to_string(),
                    value: format!("{:.2}h", sum / 60.),
                },
            ],
        });
        Report { rows }
    }

    fn sort_rows(cfg: &AppConfig, rows: &mut [Row]) {
        let mut index: usize = 0;
        let mut sort_order: HashMap<&String, usize> = HashMap::new();
        for client in cfg.clients.iter() {
            if !client.data.contains_key("psp") {
                continue;
            }
            sort_order.insert(&client.data["psp"], index);
            index += 1;
        }
        rows.sort_by(|a, b| sort_order[&a.cells[0].value].cmp(&sort_order[&b.cells[0].value]));
    }
}

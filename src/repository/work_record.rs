use std::{fs, io};
use std::collections::HashMap;
use std::fs::{DirEntry, File};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use crate::log::log;
use crate::log::LOG;
use crate::repository::model::WorkRecord;

trait Entity {
    fn get_id(&self) -> String;
}

trait Repository<T> where T: Entity {
    fn get_by_id(&self, id: String) -> T;
    fn persist(&mut self, entity: T);
}

#[derive(Debug)]
pub struct WorkRecordRepository {
    subfolder: String,
}

impl Entity for WorkRecord {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}


impl WorkRecordRepository {
    pub fn new(path: &str) -> io::Result<WorkRecordRepository> {
        let root_path = Path::new(&path);
        create_dir_if_not_exists(root_path)?;
        let folder = root_path.join(Path::new("work_records/"));
        create_dir_if_not_exists(&folder)?;
        let subfolder = folder.into_os_string().into_string().expect("this path should always convert");
        Ok(WorkRecordRepository {
            subfolder,
        })
    }

    pub fn get_latest(&self) -> Option<WorkRecord> {
        self.load_latest_week().and_then(|entries| {
            let values: Vec<&WorkRecord> = entries.values().collect();
            let mut values: Vec<WorkRecord> = values.iter()
                .map(|project_reference: &&WorkRecord| (*project_reference).clone())
                .collect();
            values.sort_by_key(|x| x.start);
            if values.is_empty()  {
                return None
            }
            values.last().cloned()
        })
    }

    pub fn find_week(&self, start: DateTime<Utc>) -> io::Result<Vec<WorkRecord>> {
        let path = self.path_of_week(start);
        let records: Vec<WorkRecord> = WorkRecordRepository::get_all_of_file(&path)?
            .values()
            .cloned()
            .collect();
        Ok(records)
    }

    fn load_latest_week(&self) -> Option<HashMap<String, WorkRecord>> {
        let iterator: io::Result<Vec<DirEntry>> = fs::read_dir(&self.subfolder).and_then(Iterator::collect);
        match iterator {
            Err(err) => {
                log!("failed to list files in database directory ({}): {}", &self.subfolder, err);
                None
            }
            Ok(mut iterator) if !iterator.is_empty() => {
                iterator.sort_by_key(DirEntry::path);

                let latest: Option<&DirEntry> = iterator.last();
                if let Some(latest) = latest {
                    let result = WorkRecordRepository::get_all_of_file(&latest.path());
                    match result {
                        Ok(items) => Some(items),
                        Err(e) => {
                            log!("failed to load latest file ({:?}): {}", &latest.path(), e);
                            None
                        }
                    }
                } else {
                    None
                }
            },
            _ => None
        }
    }

    #[allow(dead_code)]
    pub fn get_by_id(&self, id: &str, date: DateTime<Utc>) -> Option<WorkRecord> {
        let path = self.path_of_week(date);

        if let Ok(entries) = WorkRecordRepository::get_all_of_file(&path) {
            entries.get(id).map(|x| (*x).clone())
        } else {
            None
        }
    }

    pub fn persist(&mut self, entity: &WorkRecord) -> io::Result<()> {
        let path = self.path_of_week(entity.start);

        let mut entries = WorkRecordRepository::get_all_of_file(&path)?;

        entries.insert(entity.id.clone(), entity.clone());

        let file = File::create(&path)?;
        match serde_json::to_writer(file, &entries) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into())
        }
    }

    fn get_all_of_file(path: &PathBuf) -> io::Result<HashMap<String, WorkRecord>> {
        if path.is_file() {
            let file = File::open(path)?;
            Ok(serde_json::from_reader(file).expect(""))
        } else {
            Ok(HashMap::new())
        }
    }

    fn path_of_week(&self, date: DateTime<Utc>) -> PathBuf {
        let filename = format!("{}.json", date.format("%Y-%U")); // 2023-52.json
        Path::new(&self.subfolder).join(filename)
    }
}

fn create_dir_if_not_exists(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        Ok(())
    } else {
        fs::create_dir(path)
    }
}
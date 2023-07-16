use std::collections::HashMap;
use std::fs::{DirEntry, File};
use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::log::log;
use crate::repository::model::WorkRecord;
use crate::repository::week::Week;

#[derive(Debug)]
pub struct WorkRecordRepository {
    subfolder: String,
}

impl WorkRecordRepository {
    pub fn new(path: &str) -> io::Result<WorkRecordRepository> {
        let root_path = Path::new(&path);
        create_dir_if_not_exists(root_path)?;
        let folder = root_path.join(Path::new("work_records/"));
        create_dir_if_not_exists(&folder)?;
        let subfolder = folder
            .into_os_string()
            .into_string()
            .expect("this path should always convert");
        Ok(WorkRecordRepository { subfolder })
    }

    pub fn get_latest(&self) -> Option<WorkRecord> {
        self.load_latest_week().and_then(|entries| {
            let mut values: Vec<WorkRecord> = entries.into_values().collect();
            if values.is_empty() {
                return None;
            }
            values.sort_by_key(|x| x.start);
            Some(values.remove(values.len() - 1))
        })
    }

    pub fn find_week(&self, start: &dyn Week) -> io::Result<Vec<WorkRecord>> {
        let path = self.path_of_week(start);
        let records: Vec<WorkRecord> = WorkRecordRepository::get_all_of_file(&path)?
            .into_values()
            .collect();
        Ok(records)
    }

    fn load_latest_week(&self) -> Option<HashMap<String, WorkRecord>> {
        let iterator: io::Result<Vec<DirEntry>> =
            fs::read_dir(&self.subfolder).and_then(Iterator::collect);
        match iterator {
            Err(err) => {
                log!(
                    "failed to list files in database directory ({}): {}",
                    &self.subfolder,
                    err
                );
                None
            }
            Ok(mut iterator) if !iterator.is_empty() => {
                iterator.sort_by_key(DirEntry::path);

                iterator
                    .last()
                    .map(|entry| (WorkRecordRepository::get_all_of_file(&entry.path()), entry))
                    .and_then(|(result, entry)| {
                        if let Err(e) = &result {
                            log!("failed to load latest file ({:?}): {}", entry.path(), e);
                        }
                        result.ok()
                    })
            }
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn get_by_id(&self, id: &str, date: &dyn Week) -> Option<WorkRecord> {
        let path = self.path_of_week(date);
        WorkRecordRepository::get_all_of_file(&path)
            .ok()
            .and_then(|mut entries| entries.remove(id))
    }

    pub fn persist(&mut self, entity: WorkRecord) -> io::Result<()> {
        let path = self.path_of_week(&entity.start);

        let mut entries = WorkRecordRepository::get_all_of_file(&path)?;

        entries.insert(entity.id.clone(), entity);

        let file = File::create(&path)?;
        match serde_json::to_writer(file, &entries) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
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

    fn path_of_week(&self, date: &dyn Week) -> PathBuf {
        let filename = format!("{}.json", date.to_week()); // 2023-52.json
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

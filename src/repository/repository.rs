use std::{fs, io};
use std::collections::HashMap;
use std::fs::{DirEntry, File};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use crate::app::ActiveProject;
use crate::log::log;
use crate::log::LOG;

trait Entity {
    fn get_id(&self) -> String;
}

trait Repository<T> where T: Entity {
    fn get_by_id(&self, id: String) -> T;
    fn persist(&mut self, entity: T);
}

#[derive(Debug)]
pub struct WorkRecordRepository {
    path: String,
    subfolder: String,
}

impl Entity for ActiveProject {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}


impl WorkRecordRepository {
    pub fn new(path: String) -> io::Result<WorkRecordRepository> {
        let root_path = Path::new(&path);
        create_dir_if_not_exists(&root_path)?;
        let folder = root_path.join(Path::new("work_records/"));
        create_dir_if_not_exists(&folder)?;
        let subfolder = folder.into_os_string().into_string().expect("this path should always convert");
        Ok(WorkRecordRepository {
            path,
            subfolder,
        })
    }

    pub fn get_latest(&self) -> Option<ActiveProject> {
        self.load_latest_week().and_then(|entries| {
            let values: Vec<&ActiveProject> = Vec::from_iter(entries.values());
            let mut values: Vec<ActiveProject> = values.iter()
                .map(|project_reference: &&ActiveProject| (*project_reference).clone())
                .collect();
            values.sort_by_key(|x| x.start);
            if values.len() == 0  {
                return None
            }
            values.get(values.len() - 1).map(|x: &ActiveProject| x.clone())
        })
    }

    fn load_latest_week(&self) -> Option<HashMap<String, ActiveProject>> {
        let mut iterator: io::Result<Vec<DirEntry>> = fs::read_dir(&self.subfolder).and_then(|it| it.collect());
        match iterator {
            Err(err) => {
                log!("failed to list files in database directory ({}): {}", &self.subfolder, err);
                None
            }
            Ok(mut iterator) if iterator.len() > 0 => {
                iterator.sort_by_key(|dir| dir.path());

                let latest: Option<&DirEntry> = iterator.get(iterator.len() - 1);
                if let Some(latest) = latest {
                    let result = self.get_all_of_file(&latest.path());
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

    pub fn get_by_id(&self, id: String, date: DateTime<Utc>) -> Option<ActiveProject> {
        let path = self.path_of_week(date);

        if let Ok(entries) = self.get_all_of_file(&path) {
            entries.get(&id).map(|x| (*x).clone())
        } else {
            None
        }
    }

    pub fn persist(&mut self, entity: &ActiveProject) -> io::Result<()> {
        let path = self.path_of_week(entity.start);

        let mut entries = self.get_all_of_file(&path)?;

        entries.insert(entity.id.clone(), entity.clone());

        let file = File::create(&path)?;
        match serde_json::to_writer(file, &entries) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into())
        }
    }

    fn get_all_of_file(&self, path: &PathBuf) -> io::Result<HashMap<String, ActiveProject>> {
        if path.is_file() {
            let file = File::open(&path)?;
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
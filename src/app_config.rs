use config::{Config, ConfigError, Value};

pub struct AppConfig {
    pub projects: Vec<ProjectConfig>
}


pub struct ProjectConfig {
    pub name: String
}

impl AppConfig {
    fn new(project: Vec<Value>) -> AppConfig {
        let projects: Vec<ProjectConfig> = project.iter().map(|val| val.try_into().expect("could not convert project entry to ProjectConfig")).collect();
        AppConfig {
            projects
        }
    }
}

impl TryFrom<Config> for AppConfig {
    type Error = String;

    fn try_from(cfg: Config) -> Result<Self, Self::Error> {
        match cfg.get_array("projects") {
            Ok(array) => Ok(AppConfig::new(array)),
            Err(e) => Err(e.to_string())
        }
    }
}

impl TryFrom<&Value> for ProjectConfig {
    type Error = ConfigError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let table = value.clone().into_table()?;
        if !table.contains_key("name") {
            return Err(ConfigError::NotFound("could not find project.name".to_string()))
        }

        Ok(ProjectConfig{
            name: table.get("name").expect("already tested before").clone().into_string().expect("project.name must be of type string")
        })
    }
}
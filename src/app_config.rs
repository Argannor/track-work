use config::{Config, ConfigError, Value};

#[derive(Clone)]
pub struct AppConfig {
    pub projects: Vec<ProjectConfig>,
    pub logging: LoggingConfig,
}

#[derive(Clone)]
pub struct ProjectConfig {
    pub name: String,
    pub windows: Vec<String>,
}

#[derive(Clone)]
pub struct LoggingConfig {
   pub window_change: bool
}

impl AppConfig {
    fn new(project: Vec<Value>, logging: LoggingConfig) -> AppConfig {
        let projects: Vec<ProjectConfig> = project.iter().map(|val| val.try_into().expect("could not convert project entry to ProjectConfig")).collect();
        AppConfig {
            projects,
            logging
        }
    }
}

impl TryFrom<Config> for AppConfig {
    type Error = String;

    fn try_from(cfg: Config) -> Result<Self, Self::Error> {
        let logging = if let Ok(logging) = cfg.get_table("logging") {
            let window_change = logging.get("windowChanges")
                .map(|v| v.clone().into_bool().expect("logging.windowChange has to be a boolean value"))
                .unwrap_or(false);
            LoggingConfig {
                window_change
            }
        } else {
            LoggingConfig::default()
        };
        match cfg.get_array("projects") {
            Ok(array) => Ok(AppConfig::new(array, logging)),
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
        let mut windows: Vec<String> = vec![];
        if let Some(window_value) = table.get("windows") {
            windows = window_value.clone().into_array().expect("project.windows has to be an array of strings")
                .iter()
                .map(|v| v.clone().into_string().expect("project.windows has to be an array of strings"))
                .collect();
        }

        Ok(ProjectConfig{
            name: table.get("name").expect("already tested before").clone().into_string().expect("project.name must be of type string"),
            windows
        })
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            window_change: false
        }
    }
}
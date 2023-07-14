use std::collections::HashMap;

use config::{Config, ConfigError, Value};

#[derive(Clone)]
pub struct AppConfig {
    pub projects: Vec<ProjectConfig>,
    pub clients: Vec<Client>,
    pub logging: LoggingConfig,
}

#[derive(Clone)]
pub struct ProjectConfig {
    pub name: String,
    pub windows: Vec<String>,
    pub clients: Vec<ProjectClient>,
}

#[derive(Clone)]
pub struct ProjectClient {
    pub name: String,
    pub ratio: f64,
}

#[derive(Clone)]
pub struct Client {
    pub name: String,
    pub data: HashMap<String, String>,
}

#[derive(Clone, Default)]
pub struct LoggingConfig {
    pub window_change: bool,
}

impl AppConfig {
    fn new(project: Vec<Value>, client: Option<Vec<Value>>, logging: LoggingConfig) -> AppConfig {
        let projects: Vec<ProjectConfig> = project.iter().map(|val| val.try_into().expect("could not convert project entry to ProjectConfig")).collect();
        let clients: Vec<Client> = if let Some(value) = client {
            value.iter().map(|val| val.try_into().expect("could not convert client entry to Client")).collect()
        } else {
            vec![]
        };
        AppConfig {
            projects,
            clients,
            logging,
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
        let clients = if let Ok(clients) = cfg.get_array("clients") {
            Some(clients)
        } else {
            None
        };
        match cfg.get_array("projects") {
            Ok(array) => Ok(AppConfig::new(array, clients, logging)),
            Err(e) => Err(e.to_string())
        }
    }
}

impl TryFrom<&Value> for ProjectConfig {
    type Error = ConfigError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let table = value.clone().into_table()?;
        if !table.contains_key("name") {
            return Err(ConfigError::NotFound("could not find project.name".to_string()));
        }
        let mut windows: Vec<String> = vec![];
        if let Some(window_value) = table.get("windows") {
            windows = window_value.clone().into_array().expect("project.windows has to be an array of strings")
                .iter()
                .map(|v| v.clone().into_string().expect("project.windows has to be an array of strings"))
                .collect();
        }
        let mut clients: Vec<ProjectClient> = vec![];
        if let Some(clients_value) = table.get("clients") {
            clients = clients_value.clone().into_array().expect("project.clients has to be an array of objects")
                .iter()
                .map(|v| v.try_into().expect("projects.clients is malformed"))
                .collect();
        }

        Ok(ProjectConfig {
            name: table.get("name").expect("already tested before").clone().into_string().expect("project.name must be of type string"),
            clients,
            windows,
        })
    }
}

impl TryFrom<&Value> for Client {
    type Error = ConfigError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let table = value.clone().into_table().expect("client must be an object");
        let name = table.get("name").expect("client must have a name attribute")
            .clone()
            .into_string()
            .expect("client name must be a string");
        let mut data: HashMap<String, String> = HashMap::new();
        if table.contains_key("data") {
            data = table.get("data").expect("presence was asserted above").clone().into_table()?.iter()
                .map(|(key, value)| (key.clone(), value.clone().into_string().expect("data must be a map of string->string")))
                .collect();
        }
        Ok(Client {
            name,
            data,
        })
    }
}


impl TryFrom<&Value> for ProjectClient {
    type Error = ConfigError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let table = value.clone().into_table()?;
        let name = table.get("name").expect("a ProjectClient needs a name").clone().into_string()?;
        let ratio = table.get("ratio")
            .map(|v| v.clone().into_float().expect("ratio needs to be a float"))
            .unwrap_or(1.);

        Ok(ProjectClient {
            name,
            ratio,
        })
    }
}


#[cfg(test)]
mod tests {
    use config::{File, FileFormat};

    use super::*;

    #[test]
    fn test_parse() {
        let cfg = config::Config::builder().add_source(File::from_str("
projects:
  - name: Xorcery
    windows:
      - Windows PowerShell
      - \"track-work\"
    clients:
      - name: XO
  - name: EKS
    windows:
      - \"@davidgiga1993 - Discord\"
    clients:
      - name: Innovation
        ratio: 0.75
      - name: Maintenance
        ratio: 0.25
  - name: Swag

clients:
  - name: XO
    data:
      psp: IT.1
  - name: Innovation
    data:
      psp: IT.2
  - name: Maintenance
    data:
      psp: IT.3

logging:
  windowChanges: true", FileFormat::Yaml)).build();
        let app_cfg: AppConfig = cfg.unwrap().try_into().expect("should be a valid config");
        assert_eq!(app_cfg.projects[0].name, "Xorcery");
        assert_eq!(app_cfg.projects[0].windows[0], "Windows PowerShell");
        assert_eq!(app_cfg.projects[0].clients[0].name, "XO");
        assert_eq!(app_cfg.projects[0].clients[0].ratio, 1.);
        assert_eq!(app_cfg.projects[1].clients[0].name, "Innovation");
        assert_eq!(app_cfg.projects[1].clients[0].ratio, 0.75);
        assert_eq!(app_cfg.clients[0].name, "XO");
        assert_eq!(app_cfg.clients[0].data["psp"], "IT.1");
    }
}
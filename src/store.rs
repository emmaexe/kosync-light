use crate::api::ProgressPutReq;
use crate::args::Arguments;
use serde::{Deserialize, Serialize};
use serde_json::Number;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::{fs, io};

#[derive(Serialize, Deserialize)]
pub struct DocumentData {
    pub device_id: String,
    pub percentage: Number,
    pub document: String,
    pub progress: String,
    pub device: String,
    pub timestamp: Number,
}

pub struct Store {
    path: PathBuf,
    noauth: bool,
}

impl Store {
    pub fn new(args: &Arguments) -> Result<Self, io::Error> {
        let path = PathBuf::from(&args.data_path);
        fs::create_dir_all(&path)?;

        if args.noauth {
            let noauth_dir = path.join("noauth");
            fs::create_dir_all(&noauth_dir)?;
            fs::create_dir_all(&noauth_dir.join("devices"))?;
        } else {
            fs::create_dir_all(&path.join("users"))?;
        }

        Ok(Store {
            path,
            noauth: args.noauth,
        })
    }

    pub fn user_exists(&self, username: &str) -> bool {
        if self.noauth {
            return true;
        } else {
            let user_path = self.path.join("users").join(username);
            return user_path.is_dir()
                && user_path.join("passwd").is_file()
                && user_path.join("devices").is_dir();
        }
    }

    pub fn user_create(&self, username: &str, password: &str) -> Result<(), std::io::Error> {
        if self.noauth {
            let noauth_dir = self.path.join("noauth");
            fs::create_dir_all(&noauth_dir)?;
            fs::create_dir_all(&noauth_dir.join("devices"))?;
        } else {
            if self.user_exists(username) {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "This user already exists.",
                ));
            }

            let user_dir = self.path.join("users").join(username);
            fs::create_dir_all(&user_dir)?;
            fs::create_dir_all(&user_dir.join("devices"))?;

            let mut hasher = Sha256::new();
            hasher.update(password);
            let hashed_password = format!("{:x}", hasher.finalize());

            let passwd_file = user_dir.join("passwd");
            fs::write(passwd_file, hashed_password)?;
        }

        return Ok(());
    }

    pub fn user_auth(&self, username: &str, password: &str) -> bool {
        if self.noauth {
            return true;
        }
        if self.user_exists(username) {
            let passwd_file = self.path.join("users").join(username).join("passwd");
            let stored_hash = match fs::read_to_string(passwd_file) {
                Ok(content) => content,
                Err(_) => return false,
            };

            let mut hasher = Sha256::new();
            hasher.update(password);
            let hash = format!("{:x}", hasher.finalize());
            return stored_hash == hash;
        }
        return false;
    }

    pub fn document_update(
        &self,
        username: &str,
        data: &ProgressPutReq,
        time: &Number,
    ) -> Result<(), io::Error> {
        if !self.user_exists(username) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "This user does not exist.",
            ));
        }

        let device_dir = if self.noauth {
            self.path
                .join("noauth")
                .join("devices")
                .join(&data.device_id)
        } else {
            self.path
                .join("users")
                .join(username)
                .join("devices")
                .join(&data.device_id)
        };
        fs::create_dir_all(&device_dir)?;

        let document_data = DocumentData {
            device_id: data.device_id.clone(),
            percentage: data.percentage.clone(),
            document: data.document.clone(),
            progress: data.progress.clone(),
            device: data.device.clone(),
            timestamp: time.clone(),
        };

        fs::write(
            device_dir.join(&data.document),
            serde_json::to_string(&document_data).unwrap(),
        )?;

        return Ok(());
    }

    pub fn document_read(
        &self,
        username: &str,
        document: &str,
    ) -> Result<Option<DocumentData>, io::Error> {
        if !self.user_exists(username) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "This user does not exist.",
            ));
        }

        let devices_dir = if self.noauth {
            self.path.join("noauth").join("devices")
        } else {
            self.path.join("users").join(username).join("devices")
        };
        let mut res: Option<DocumentData> = None;
        let mut timestamp: u64 = 0;

        if let Ok(entries) = fs::read_dir(devices_dir) {
            for entry in entries {
                if let Ok(device_dir) = entry.and_then(|e| Ok(e.path())) {
                    if device_dir.is_dir() {
                        let document_path = device_dir.join(document);
                        if document_path.exists() {
                            let content = fs::read_to_string(&document_path)?;
                            if let Ok(current) = serde_json::from_str::<DocumentData>(&content) {
                                if let Some(current_timestamp) = current.timestamp.as_u64() {
                                    if res.is_none() || current_timestamp > timestamp {
                                        timestamp = current_timestamp;
                                        res = Some(current);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        return Ok(res);
    }
}

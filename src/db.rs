use dorsal::query as sqlquery;
use dorsal::DefaultReturn;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppData {
    pub db: Database,
    pub http_client: awc::Client,
}

// ...
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum ReportType {
    Harassment,
    Abuse,
    Illegal,
    Harmful,
    Other,
}

impl Default for ReportType {
    fn default() -> Self {
        Self::Other
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum ReportStatus {
    /// Report is active and needs to be handled
    Active,
    /// Report has been handled
    Archived,
    /// Report has been flagged as spam
    Spam,
}

impl Default for ReportStatus {
    fn default() -> Self {
        Self::Active
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Report {
    /// The type of the report
    pub report_type: ReportType,
    /// The status of the report
    pub status: ReportStatus,
    /// The username of the user creating the report (can be empty)
    pub author: String,
    /// The report body content (written by user)
    pub content: String,
    /// The URL address that is getting reported
    pub address: String,
    /// When it was reported
    pub timestamp: u128,
}

// server
#[derive(Clone)]
pub struct Database {
    pub base: dorsal::StarterDatabase,
    pub auth: dorsal::AuthDatabase,
    pub logs: dorsal::LogDatabase,
    pub notifications: dorsal::NotificationDatabase,
}

impl Database {
    pub async fn new(opts: dorsal::DatabaseOpts) -> Database {
        let db = dorsal::StarterDatabase::new(opts).await;

        let auth = dorsal::AuthDatabase { base: db.clone() };
        let logs = dorsal::LogDatabase { base: db.clone() };

        Database {
            base: db.clone(),
            auth: auth.clone(),
            logs: logs.clone(),
            notifications: dorsal::NotificationDatabase {
                base: db,
                auth,
                logs,
            },
        }
    }

    pub async fn init(&self) {
        let c = &self.base.db.client;

        let _ = sqlquery(
            "CREATE TABLE IF NOT EXISTS \"de_reports\" (
                report_type VARCHAR(1000000),
                report_status VARCHAR(1000000),
                author VARCHAR(1000000),
                content VARCHAR(1000000),
                address VARCHAR(1000000),
                timestamp VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;

        // users and logs tables
        let _ = sqlquery(
            "CREATE TABLE IF NOT EXISTS \"Users\" (
                username VARCHAR(1000000),
                id_hashed VARCHAR(1000000),
                role VARCHAR(1000000),
                timestamp VARCHAR(1000000),
                metadata VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;

        let _ = sqlquery(
            "CREATE TABLE IF NOT EXISTS \"Logs\" (
                id VARCHAR(1000000),
                logtype VARCHAR(1000000),
                timestamp  VARCHAR(1000000),
                content VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;
    }

    // example

    // GET
    /// Get all [`Active`](ReportStatus) [`Report`]s (limited)
    ///
    /// # Arguments:
    /// * `offset` - optional value representing the SQL fetch offset
    pub async fn get_all_reports(&self, offset: Option<i32>) -> DefaultReturn<Option<Vec<Report>>> {
        let offset = if offset.is_some() { offset.unwrap() } else { 0 };

        // check in cache
        let cached = self
            .base
            .cachedb
            .get(format!("reports:offset{}", offset))
            .await;

        if cached.is_some() {
            // ...
            let reports = serde_json::from_str::<Vec<Report>>(cached.unwrap().as_str()).unwrap();

            // return
            return DefaultReturn {
                success: true,
                message: String::from("Found reports"),
                payload: Option::Some(reports),
            };
        }

        // ...
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"de_reports\" WHERE \"report_status\" = 'Active' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET ?"
        } else {
            "SELECT * FROM \"de_reports\" WHERE \"report_status\" = 'Active' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET $1"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query).bind(offset).fetch_all(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // build res
        let mut full_res: Vec<Report> = Vec::new();

        for row in res.unwrap() {
            let row = self.base.textify_row(row).data;
            full_res.push(Report {
                report_type: serde_json::from_str(row.get("report_type").unwrap()).unwrap(),
                status: serde_json::from_str(row.get("report_status").unwrap()).unwrap(),
                author: row.get("author").unwrap().to_string(),
                content: row.get("content").unwrap().to_string(),
                address: row.get("address").unwrap().to_string(),
                timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
            });
        }

        // store in cache
        self.base
            .cachedb
            .set(
                format!("reports:offset{}", offset),
                serde_json::to_string::<Vec<Report>>(&full_res).unwrap(),
            )
            .await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Found reports"),
            payload: Option::Some(full_res),
        };
    }

    /// Get a report by its id
    ///
    /// # Arguments:
    /// * `id` - `String` of the report's `id`
    pub async fn get_report_by_id(&self, id: String) -> DefaultReturn<Option<Report>> {
        // check in cache
        let cached = self.base.cachedb.get(format!("report:{}", id)).await;

        if cached.is_some() {
            // ...
            let report = serde_json::from_str::<Report>(cached.unwrap().as_str()).unwrap();

            // return
            return DefaultReturn {
                success: true,
                message: String::from("Report exists (cache)"),
                payload: Option::Some(report),
            };
        }

        // ...
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"de_reports\" WHERE \"id\" = ?"
        } else {
            "SELECT * FROM \"de_reports\" WHERE \"id\" = $1"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query).bind::<&String>(&id).fetch_one(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Report does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.base.textify_row(row).data;

        // store in cache
        let report = Report {
            report_type: serde_json::from_str(row.get("report_type").unwrap()).unwrap(),
            status: serde_json::from_str(row.get("report_status").unwrap()).unwrap(),
            author: row.get("author").unwrap().to_string(),
            content: row.get("content").unwrap().to_string(),
            address: row.get("address").unwrap().to_string(),
            timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
        };

        self.base
            .cachedb
            .set(
                format!("report:{}", id),
                serde_json::to_string::<Report>(&report).unwrap(),
            )
            .await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Report exists"),
            payload: Option::Some(report),
        };
    }

    // SET
    /// Create a new [`Report`] given various properties
    ///
    /// # Arguments:
    /// * `props` - [`Report`]
    pub async fn create_report(&self, props: &mut Report) -> DefaultReturn<Option<Report>> {
        // check content
        if (props.content.len() < 1) | (props.content.len() > 2_000) {
            return DefaultReturn {
                success: false,
                message: String::from("Content is invalid"),
                payload: Option::None,
            };
        }

        // create report
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "INSERT INTO \"de_reports\" VALUES (?, ?, ?, ?, ?, ?)"
        } else {
            "INSERT INTO \"de_reports\" VALUES ($1, $2, $3, $4, $5, $6)"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query)
            .bind::<&String>(&serde_json::to_string(&props.report_type).unwrap())
            .bind::<&String>(&serde_json::to_string(&props.status).unwrap())
            .bind::<&String>(&props.author)
            .bind::<&String>(&props.content)
            .bind::<&String>(&props.address)
            .bind::<&String>(&dorsal::utility::unix_epoch_timestamp().to_string())
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: res.err().unwrap().to_string(),
                payload: Option::None,
            };
        }

        // update cache
        self.base
            .cachedb
            .remove_starting_with("reports:offset*".to_string())
            .await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Content reported."),
            payload: Option::Some(props.to_owned()),
        };
    }

    /// Update a [`Report`]'s [`ReportStatus`] by its `id`
    pub async fn edit_report_status_by_id(
        &self,
        id: String,
        status: ReportStatus,
        edit_as: Option<dorsal::db::special::auth_db::FullUser<String>>,
    ) -> DefaultReturn<Option<String>> {
        // make sure report exists
        let existing = &self.get_report_by_id(id.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Report does not exist!"),
                payload: Option::None,
            };
        }

        // ...
        if edit_as.is_none() {
            return DefaultReturn {
                success: false,
                message: String::from("You are not allowed to do this."),
                payload: Option::None,
            };
        };

        // make sure we can do this
        match edit_as {
            Some(ua) => {
                if !ua.level.permissions.contains(&"StaffDashboard".to_string()) {
                    return DefaultReturn {
                        success: false,
                        message: String::from("You are not allowed to do this."),
                        payload: Option::None,
                    };
                }
            }
            None => {
                return DefaultReturn {
                    success: false,
                    message: String::from("You are not allowed to do this."),
                    payload: Option::None,
                }
            }
        }

        // update paste
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "UPDATE \"de_reports\" SET \"report_status\" = ? WHERE \"id\" = ?"
        } else {
            "UPDATE \"de_reports\" SET (\"report_status\") = ($1) WHERE \"id\" = $2"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query)
            .bind::<&String>(&serde_json::to_string(&status).unwrap())
            .bind::<&String>(&id)
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // update cache
        let existing_in_cache = self.base.cachedb.get(format!("report:{}", id)).await;

        if existing_in_cache.is_some() {
            let mut report = serde_json::from_str::<Report>(&existing_in_cache.unwrap()).unwrap();
            report.status = status;

            // update cache
            self.base
                .cachedb
                .update(
                    format!("report:{}", id),
                    serde_json::to_string::<Report>(&report).unwrap(),
                )
                .await;
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Report updated!"),
            payload: Option::Some(id),
        };
    }
}

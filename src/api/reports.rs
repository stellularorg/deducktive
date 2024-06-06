use crate::db::AppData;
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::pages::base;

// props
#[derive(Serialize, Deserialize)]
pub struct PCreateReport {
    pub report_type: crate::db::ReportType,
    pub content: String,
    pub address: String,
}

#[derive(Serialize, Deserialize)]
pub struct PEditStatus {
    pub status: crate::db::ReportStatus,
}

// ...
#[post("/api/v1/reports")]
/// Create a new report
pub async fn create_request(
    req: HttpRequest,
    body: web::Json<PCreateReport>,
    data: web::Data<AppData>,
) -> impl Responder {
    // ...
    let (set_cookie, _, token_user) = base::check_auth_status(req, data.clone()).await;

    // create report
    let res = data
        .db
        .create_report(&mut crate::db::Report {
            report_type: body.report_type.clone(),
            status: crate::db::ReportStatus::Active,
            author: if token_user.is_some() {
                token_user.unwrap().payload.unwrap().user.username
            } else {
                String::new()
            },
            content: body.content.clone(),
            address: body.address.clone(),
            timestamp: dorsal::utility::unix_epoch_timestamp(),
        })
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .append_header(("Set-Cookie", set_cookie))
        .body(serde_json::to_string(&res).unwrap());
}

#[post("/api/v1/reports/{id:.*}")]
/// Edit report status
pub async fn edit_status_request(
    req: HttpRequest,
    body: web::Json<PEditStatus>,
    data: web::Data<AppData>,
) -> impl Responder {
    let id = req.match_info().get("id").unwrap();

    // ...
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    // edit report
    let res = data
        .db
        .edit_report_status_by_id(
            id.to_string(),
            body.status.to_owned(),
            if token_user.is_some() {
                token_user.unwrap().payload
            } else {
                Option::None
            },
        )
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .append_header(("Set-Cookie", set_cookie))
        .body(serde_json::to_string(&res).unwrap());
}

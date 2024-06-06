use crate::db::Report;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};

use super::base;
use askama::Template;

#[derive(Default, PartialEq, serde::Deserialize)]
pub struct OffsetQueryProps {
    pub offset: Option<i32>,
}

#[derive(Template)]
#[template(path = "auth_picker.html")]
struct AuthPickerTemplate {
    // required fields (super::base)
    auth_state: bool,
    guppy: String,
    body_embed: String,
}

#[derive(Template)]
#[template(path = "homepage.html")]
struct HomeTemplate {
    reports: Vec<Report>,
    offset: i32,
    // required fields (super::base)
    auth_state: bool,
    guppy: String,
    body_embed: String,
}

#[derive(Template)]
#[template(path = "manage_report.html")]
struct ViewReportTemplate {
    report: Report,
    // required fields (super::base)
    auth_state: bool,
    guppy: String,
    body_embed: String,
}

#[get("/")]
pub async fn home_request(
    req: HttpRequest,
    data: web::Data<crate::db::AppData>,
    info: web::Query<OffsetQueryProps>,
) -> impl Responder {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req, data.clone()).await;

    match token_user.clone() {
        Some(ua) => match ua.payload {
            Some(ua) => {
                // check for permission
                if !ua.level.permissions.contains(&"StaffDashboard".to_string()) {
                    return auth_picker(token_user.is_some(), set_cookie).await;
                }
            }
            None => {
                return auth_picker(token_user.is_some(), set_cookie).await;
            }
        },
        None => {
            return auth_picker(token_user.is_some(), set_cookie).await;
        }
    }

    // ...
    // get reports
    let res = data.db.get_all_reports(info.offset).await;

    if res.success == false {
        return HttpResponse::NotAcceptable().body(res.message);
    }

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            HomeTemplate {
                reports: res.payload.unwrap(),
                offset: match info.offset {
                    Some(i) => i,
                    None => 0,
                },
                // required fields
                auth_state: base.auth_state,
                guppy: base.guppy,
                body_embed: base.body_embed,
            }
            .render()
            .unwrap(),
        );
}

/// Auth picker template response
pub async fn auth_picker(token_user_is_some: bool, set_cookie: String) -> HttpResponse {
    let base = base::get_base_values(token_user_is_some);
    return HttpResponse::NotAcceptable()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            AuthPickerTemplate {
                // required fields
                auth_state: base.auth_state,
                guppy: base.guppy,
                body_embed: base.body_embed,
            }
            .render()
            .unwrap(),
        );
}

#[get("/report/{id:.*}")]
pub async fn manage_report_request(
    req: HttpRequest,
    data: web::Data<crate::db::AppData>,
) -> impl Responder {
    let id = req.match_info().get("id").unwrap();

    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    match token_user.clone() {
        Some(ua) => match ua.payload {
            Some(ua) => {
                // check for permission
                if !ua.level.permissions.contains(&"StaffDashboard".to_string()) {
                    return auth_picker(token_user.is_some(), set_cookie).await;
                }
            }
            None => {
                return auth_picker(token_user.is_some(), set_cookie).await;
            }
        },
        None => {
            return auth_picker(token_user.is_some(), set_cookie).await;
        }
    }

    // ...
    // get report
    let res = data.db.get_report_by_id(id.to_string()).await;

    if res.success == false {
        return HttpResponse::NotAcceptable().body(res.message);
    }

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            ViewReportTemplate {
                report: res.payload.unwrap(),
                // required fields
                auth_state: base.auth_state,
                guppy: base.guppy,
                body_embed: base.body_embed,
            }
            .render()
            .unwrap(),
        );
}

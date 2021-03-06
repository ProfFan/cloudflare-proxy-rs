#![feature(proc_macro_hygiene, decl_macro)]

extern crate diesel;

#[macro_use]
extern crate rocket;

extern crate tera;

extern crate cloudflare;

extern crate regex;

use cloudflare_proxy::db::establish_connection;
use cloudflare_proxy::models::*;
use rocket_contrib::json::Json;
use rocket_contrib::templates::Template;

use rocket::fairing::AdHoc;

use diesel::*;

use cloudflare::{zones::dns::RecordType, Cloudflare};

use tera::Context;

struct CfCredentials {
    user: String,
    key: String,
}

#[get("/")]
fn index() -> Template {
    let mut context = Context::new();

    let test_vec = [User {
        name: "Nono".to_string(),
        id: 0,
        key: "fAk3_s3cr3t_kee".to_string(),
        disabled: false,
    }];

    context.insert("users", &test_vec);

    let test_vec_2: Vec<UserSitePrivilege> = Vec::new();
    context.insert("privs", &test_vec_2);

    Template::render("index", &context)
}

#[post("/update", format = "application/json", data = "<req>")]
fn update(req: Json<UpdateRequest>, cf_conf: rocket::State<CfCredentials>) -> Json<UpdateResult> {
    use cloudflare_proxy::schema::sites;
    use cloudflare_proxy::schema::user_site_privileges;
    use cloudflare_proxy::schema::users::dsl::*;

    let connection = establish_connection();
    let results = users
        .filter(disabled.eq(false))
        .filter(name.eq(&req.user))
        .filter(key.eq(&req.key))
        .load::<User>(&connection)
        .expect("Error loading users");

    let api_base = "https://api.cloudflare.com/client/v4/";

    if results.len() == 1 {
        let privs = UserSitePrivilege::belonging_to(&results[0])
            .inner_join(sites::dsl::sites)
            .select((
                sites::dsl::zone,
                user_site_privileges::pattern,
                user_site_privileges::dsl::superuser,
            ))
            .filter(sites::dsl::zone.eq(&req.zone))
            .load::<(String, String, bool)>(&connection)
            .expect("Error fetching privileges!");

        if privs.len() < 1 {
            return Json(UpdateResult {
                success: false,
                e: "ERR_NO_PRIV".to_string(),
            });
        }

        let mut allowed = false;
        for entry in privs {
            use regex::Regex;

            if entry.2 {
                allowed = true;
            }

            let re = Regex::new(&entry.1).unwrap();

            if re.is_match(&req.rec) {
                allowed = true;
            }
        }

        if !allowed {
            return Json(UpdateResult {
                success: false,
                e: "ERR_NO_PRIV".to_string(),
            });
        }

        let rectype: RecordType;
        let _rectype = req.rectype.to_uppercase();
        match _rectype.as_str() {
            "A" => {
                rectype = RecordType::A;
            }
            "AAAA" => {
                rectype = RecordType::AAAA;
            }
            "TXT" => {
                rectype = RecordType::TXT;
            }
            "SRV" => {
                rectype = RecordType::SRV;
            }
            "CNAME" => {
                rectype = RecordType::CNAME;
            }
            _ => {
                return Json(UpdateResult {
                    success: false,
                    e: "ERR_REC_TYPE".to_string(),
                });
            }
        }

        let cloudflare = Cloudflare::new(&cf_conf.key, &cf_conf.user, &api_base)
            .map_err(|err| {
                format!(
                    "Failed to initialize Cloudflare API client: {}",
                    format_error(err)
                )
            })
            .unwrap();

        match cloudflare::zones::get_zoneid(&cloudflare, &req.zone)
            .map_err(|err| format!("Failed to retreive zone ID: {}", format_error(err)))
        {
            Ok(zone_id) => {
                let current_rec_ =
                    cloudflare::zones::dns::list_dns_of_type(&cloudflare, &zone_id, rectype)
                        .map_err(|err| {
                            format!("Failed to list DNS A records: {}", format_error(err))
                        })
                        .and_then(|list| {
                            list.into_iter()
                                .find(|record| record.name == req.rec)
                                .ok_or_else(|| format!("Could not find A record for {}", req.rec))
                        });
                match current_rec_ {
                    Ok(current_rec) => {
                        use cloudflare::zones::dns::UpdateDnsRecord;

                        let update_result_ = cloudflare::zones::dns::update_dns_entry(
                            &cloudflare,
                            &zone_id,
                            &current_rec.id,
                            &UpdateDnsRecord {
                                record_type: current_rec.record_type,
                                name: current_rec.name.clone(),
                                content: req.value.clone(),
                                ttl: Some(current_rec.ttl),
                                proxied: Some(current_rec.proxied),
                            },
                        );

                        match update_result_ {
                            Ok(update_result) => {
                                return Json(UpdateResult {
                                    success: true,
                                    e: format!("{:?}", update_result),
                                });
                            }
                            Err(e) => {
                                return Json(UpdateResult {
                                    success: false,
                                    e: format_error(e),
                                });
                            }
                        }
                    }
                    Err(e) => {
                        return Json(UpdateResult { success: false, e });
                    }
                }
            }
            Err(e) => {
                return Json(UpdateResult {
                    success: false,
                    e: e.to_string(),
                });
            }
        }
    }

    Json(UpdateResult {
        success: false,
        e: "".to_string(),
    })
}

#[post("/add", format = "application/json", data = "<req>")]
fn add(req: Json<AddRequest>, cf_conf: rocket::State<CfCredentials>) -> Json<AddResult> {
    use cloudflare_proxy::schema::sites;
    use cloudflare_proxy::schema::user_site_privileges;
    use cloudflare_proxy::schema::users::dsl::*;

    let connection = establish_connection();
    let results = users
        .filter(disabled.eq(false))
        .filter(name.eq(&req.user))
        .filter(key.eq(&req.key))
        .load::<User>(&connection)
        .expect("Error loading users");

    let api_base = "https://api.cloudflare.com/client/v4/";

    if results.len() == 1 {
        let privs = UserSitePrivilege::belonging_to(&results[0])
            .inner_join(sites::dsl::sites)
            .select((
                sites::dsl::zone,
                user_site_privileges::pattern,
                user_site_privileges::dsl::superuser,
            ))
            .filter(sites::dsl::zone.eq(&req.zone))
            .load::<(String, String, bool)>(&connection)
            .expect("Error fetching privileges!");

        if privs.len() < 1 {
            return Json(AddResult {
                success: false,
                e: "ERR_NO_PRIV".to_string(),
            });
        }

        let mut allowed = false;
        for entry in privs {
            use regex::Regex;

            if entry.2 {
                allowed = true;
            }

            let re = Regex::new(&entry.1).unwrap();

            if re.is_match(&req.rec) {
                allowed = true;
            }
        }

        if !allowed {
            return Json(AddResult {
                success: false,
                e: "ERR_NO_PRIV".to_string(),
            });
        }

        let rectype: RecordType;
        let _rectype = req.rectype.to_uppercase();
        match _rectype.as_str() {
            "A" => {
                rectype = RecordType::A;
            }
            "AAAA" => {
                rectype = RecordType::AAAA;
            }
            "TXT" => {
                rectype = RecordType::TXT;
            }
            "SRV" => {
                rectype = RecordType::SRV;
            }
            "CNAME" => {
                rectype = RecordType::CNAME;
            }
            _ => {
                return Json(AddResult {
                    success: false,
                    e: "ERR_REC_TYPE".to_string(),
                });
            }
        }

        let cloudflare = Cloudflare::new(&cf_conf.key, &cf_conf.user, &api_base)
            .map_err(|err| {
                format!(
                    "Failed to initialize Cloudflare API client: {}",
                    format_error(err)
                )
            })
            .unwrap();

        match cloudflare::zones::get_zoneid(&cloudflare, &req.zone)
            .map_err(|err| format!("Failed to retreive zone ID: {}", format_error(err)))
        {
            Ok(zone_id) => {
                let create_result_ = cloudflare::zones::dns::create_dns_entry(
                    &cloudflare,
                    &zone_id,
                    rectype,
                    &req.rec,
                    &req.value,
                );

                match create_result_ {
                    Ok(create_result) => {
                        return Json(AddResult {
                            success: true,
                            e: format!("{:?}", create_result),
                        });
                    }
                    Err(e) => {
                        return Json(AddResult {
                            success: false,
                            e: format_error(e),
                        });
                    }
                }
            }
            Err(e) => {
                return Json(AddResult {
                    success: false,
                    e: e.to_string(),
                });
            }
        }
    }

    Json(AddResult {
        success: false,
        e: "".to_string(),
    })
}

#[post("/delete", format = "application/json", data = "<req>")]
fn delete(req: Json<DeleteRequest>, cf_conf: rocket::State<CfCredentials>) -> Json<DeleteResult> {
    use cloudflare_proxy::schema::sites;
    use cloudflare_proxy::schema::user_site_privileges;
    use cloudflare_proxy::schema::users::dsl::*;

    let connection = establish_connection();
    let results = users
        .filter(disabled.eq(false))
        .filter(name.eq(&req.user))
        .filter(key.eq(&req.key))
        .load::<User>(&connection)
        .expect("Error loading users");

    let api_base = "https://api.cloudflare.com/client/v4/";

    if results.len() != 1 {
        return Json(DeleteResult {
            success: false,
            e: "ERR_UNKNOWN".to_string(),
        })
    }

    let privs = UserSitePrivilege::belonging_to(&results[0])
        .inner_join(sites::dsl::sites)
        .select((
            sites::dsl::zone,
            user_site_privileges::pattern,
            user_site_privileges::dsl::superuser,
        ))
        .filter(sites::dsl::zone.eq(&req.zone))
        .load::<(String, String, bool)>(&connection)
        .expect("Error fetching privileges!");

    if privs.len() < 1 {
        return Json(DeleteResult {
            success: false,
            e: "ERR_NO_PRIV".to_string(),
        });
    }

    let mut allowed = false;
    for entry in privs {
        use regex::Regex;

        if entry.2 {
            allowed = true;
        }

        let re = Regex::new(&entry.1).unwrap();

        if re.is_match(&req.rec) {
            allowed = true;
        }
    }

    if !allowed {
        return Json(DeleteResult {
            success: false,
            e: "ERR_NO_PRIV".to_string(),
        });
    }

    let rectype: RecordType;
    let _rectype = req.rectype.to_uppercase();
    match _rectype.as_str() {
        "A" => {
            rectype = RecordType::A;
        }
        "AAAA" => {
            rectype = RecordType::AAAA;
        }
        "TXT" => {
            rectype = RecordType::TXT;
        }
        "SRV" => {
            rectype = RecordType::SRV;
        }
        "CNAME" => {
            rectype = RecordType::CNAME;
        }
        _ => {
            return Json(DeleteResult {
                success: false,
                e: "ERR_REC_TYPE".to_string(),
            });
        }
    }

    let cloudflare = Cloudflare::new(&cf_conf.key, &cf_conf.user, &api_base)
        .map_err(|err| {
            format!(
                "Failed to initialize Cloudflare API client: {}",
                format_error(err)
            )
        })
        .unwrap();

    match cloudflare::zones::get_zoneid(&cloudflare, &req.zone)
        .map_err(|err| format!("Failed to retreive zone ID: {}", format_error(err)))
    {
        Ok(zone_id) => {
            let current_rec_ =
                cloudflare::zones::dns::list_dns_of_type(&cloudflare, &zone_id, rectype)
                    .map_err(|err| format!("Failed to list DNS records: {}", format_error(err)))
                    .and_then(|list| {
                        list.into_iter()
                            .find(|record| {
                                (record.name == req.rec && record.content == req.value)
                            })
                            .ok_or_else(|| format!("Could not find record for {}", req.rec))
                    });
            match current_rec_ {
                Ok(current_rec) => {
                    let delete_result_ = cloudflare::zones::dns::delete_dns_entry(
                        &cloudflare,
                        &zone_id,
                        &current_rec.id,
                    );

                    match delete_result_ {
                        Ok(delete_result) => {
                            return Json(DeleteResult {
                                success: true,
                                e: format!("{:?}", delete_result),
                            });
                        }
                        Err(e) => {
                            return Json(DeleteResult {
                                success: true,
                                e: format_error(e),
                            });
                        }
                    }
                }
                Err(e) => {
                    return Json(DeleteResult { success: false, e });
                }
            }
        }
        Err(e) => {
            return Json(DeleteResult {
                success: false,
                e: e.to_string(),
            });
        }
    }
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index])
        .mount("/", routes![update])
        .mount("/", routes![add])
        .mount("/", routes![delete])
        .attach(Template::fairing())
        .attach(AdHoc::on_attach("cfconfig", |rocket| {
            let user = rocket.config().get_str("cfuser").unwrap().to_string();
            let key = rocket.config().get_str("cfkey").unwrap().to_string();
            Ok(rocket.manage(CfCredentials { user, key }))
        }))
        .launch();
}

fn format_error(error: cloudflare::Error) -> String {
    use cloudflare::Error;

    match error {
        Error::NoResultsReturned => "No results returned".into(),
        Error::InvalidOptions => "Invalid options".into(),
        Error::NotSuccess => "API request failed".into(),
        Error::Reqwest(cause) => format!("Network error: {}", cause),
        Error::Json(cause) => format!("JSON error: {}", cause),
        Error::Io(cause) => format!("IO error: {}", cause),
        Error::Url(cause) => format!("URL error: {}", cause),
    }
}

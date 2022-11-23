#![feature(proc_macro_hygiene, decl_macro)]

extern crate diesel;

#[macro_use]
extern crate rocket;

extern crate tera;

extern crate cloudflare;

extern crate regex;

use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use cloudflare::endpoints::dns::{DnsContent, ListDnsRecordsParams};
use cloudflare_proxy::db::establish_connection;
use cloudflare_proxy::models::*;
use rocket_contrib::json::Json;
use rocket_contrib::templates::Template;

use rocket::fairing::AdHoc;

use diesel::*;

use cloudflare::endpoints::{dns, zone};
use cloudflare::framework::{
    apiclient::ApiClient,
    auth::Credentials,
    mock::{MockApiClient, NoopEndpoint},
    response::ApiFailure,
    Environment, HttpApiClient, HttpApiClientConfig, OrderDirection,
};

use tera::Context;

struct CfCredentials {
    user: Option<String>,
    key: Option<String>,
    token: Option<String>,
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

    Template::render("index", context.into_json())
}

#[post("/update", format = "application/json", data = "<req>")]
fn update(req: Json<UpdateRequest>, cf_conf: rocket::State<CfCredentials>) -> Json<UpdateResult> {
    use cloudflare_proxy::schema::sites;
    use cloudflare_proxy::schema::user_site_privileges;
    use cloudflare_proxy::schema::users::dsl::*;

    let mut connection = establish_connection();
    let results = users
        .filter(disabled.eq(false))
        .filter(name.eq(&req.user))
        .filter(key.eq(&req.key))
        .load::<User>(&mut connection)
        .expect("Error loading users");

    if results.len() == 1 {
        let privs = UserSitePrivilege::belonging_to(&results[0])
            .inner_join(sites::dsl::sites)
            .select((
                sites::dsl::zone,
                user_site_privileges::pattern,
                user_site_privileges::dsl::superuser,
            ))
            .filter(sites::dsl::zone.eq(&req.zone))
            .load::<(String, String, bool)>(&mut connection)
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

        let record: dns::DnsContent;
        let _record = req.rectype.to_uppercase();
        match _record.as_str() {
            "A" => {
                record = DnsContent::A {
                    content: Ipv4Addr::from_str(&req.value).unwrap(),
                };
            }
            "AAAA" => {
                record = DnsContent::AAAA {
                    content: Ipv6Addr::from_str(&req.value).unwrap(),
                };
            }
            "TXT" => {
                record = DnsContent::TXT {
                    content: req.value.clone(),
                };
            }
            "SRV" => {
                record = DnsContent::SRV {
                    content: req.value.clone(),
                };
            }
            "CNAME" => {
                record = DnsContent::CNAME {
                    content: req.value.clone(),
                };
            }
            _ => {
                return Json(UpdateResult {
                    success: false,
                    e: "ERR_REC_TYPE".to_string(),
                });
            }
        }

        let credentials: Credentials = if let Some(cf_key) = &cf_conf.key {
            Credentials::UserAuthKey {
                email: cf_conf.user.clone().unwrap(),
                key: cf_key.clone(),
            }
        } else if let Some(token) = &cf_conf.token {
            Credentials::UserAuthToken {
                token: token.to_string(),
            }
        } else {
            panic!("Either API token or API key + email pair must be provided")
        };

        let api_client = HttpApiClient::new(
            credentials,
            HttpApiClientConfig::default(),
            Environment::Production,
        )
        .unwrap();

        let zone_id = match api_client.request(&zone::ListZones {
            params: zone::ListZonesParams {
                name: Some(req.zone.clone()),
                ..Default::default()
            },
        }) {
            Ok(resp) => resp.result.first().unwrap().id.clone(),
            Err(_) => {
                return Json(UpdateResult {
                    success: false,
                    e: "ERR_ZONE_NOT_FOUND".into(),
                })
            }
        };

        match api_client.request(&zone::ZoneDetails {
            identifier: &zone_id,
        }) {
            Ok(resp) => {
                let current_rec_ = api_client
                    .request(&dns::ListDnsRecords {
                        zone_identifier: &resp.result.id,
                        params: ListDnsRecordsParams {
                            record_type: Some(record.clone()),
                            direction: Some(OrderDirection::Ascending),
                            ..Default::default()
                        },
                    })
                    .map_err(|err| format!("Failed to list DNS A records: {}", format_error(err)))
                    .and_then(|recs| {
                        recs.result
                            .into_iter()
                            .find(|item| item.name == req.rec)
                            .ok_or_else(|| format!("Could not find A record for {}", req.rec))
                    });

                match current_rec_ {
                    Ok(current_rec) => {
                        let update_result_ = api_client.request(&dns::UpdateDnsRecord {
                            zone_identifier: &resp.result.id,
                            identifier: &current_rec.id,
                            params: dns::UpdateDnsRecordParams {
                                content: record,
                                name: &current_rec.name,
                                proxied: Some(current_rec.proxied),
                                ttl: Some(current_rec.ttl),
                            },
                        });

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
                    e: "ERR_NO_SUCH_ZONE".to_string(),
                });
            }
        };
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

    let mut connection = establish_connection();
    let results = users
        .filter(disabled.eq(false))
        .filter(name.eq(&req.user))
        .filter(key.eq(&req.key))
        .load::<User>(&mut connection)
        .expect("Error loading users");

    if results.len() == 1 {
        let privs = UserSitePrivilege::belonging_to(&results[0])
            .inner_join(sites::dsl::sites)
            .select((
                sites::dsl::zone,
                user_site_privileges::pattern,
                user_site_privileges::dsl::superuser,
            ))
            .filter(sites::dsl::zone.eq(&req.zone))
            .load::<(String, String, bool)>(&mut connection)
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

        let record: DnsContent;
        let _record_type = req.rectype.to_uppercase();
        match _record_type.as_str() {
            "A" => {
                record = DnsContent::A {
                    content: Ipv4Addr::from_str(&req.value).unwrap(),
                };
            }
            "AAAA" => {
                record = DnsContent::AAAA {
                    content: Ipv6Addr::from_str(&req.value).unwrap(),
                };
            }
            "TXT" => {
                record = DnsContent::TXT {
                    content: req.value.clone(),
                };
            }
            "SRV" => {
                record = DnsContent::SRV {
                    content: req.value.clone(),
                };
            }
            "CNAME" => {
                record = DnsContent::CNAME {
                    content: req.value.clone(),
                };
            }
            _ => {
                return Json(AddResult {
                    success: false,
                    e: "ERR_REC_TYPE".to_string(),
                });
            }
        }

        let credentials: Credentials = if let Some(cf_key) = &cf_conf.key {
            Credentials::UserAuthKey {
                email: cf_conf.user.clone().unwrap(),
                key: cf_key.clone(),
            }
        } else if let Some(token) = &cf_conf.token {
            Credentials::UserAuthToken {
                token: token.to_string(),
            }
        } else {
            panic!("Either API token or API key + email pair must be provided")
        };

        let api_client = HttpApiClient::new(
            credentials,
            HttpApiClientConfig::default(),
            Environment::Production,
        )
        .unwrap();

        let zone_id = match api_client.request(&zone::ListZones {
            params: zone::ListZonesParams {
                name: Some(req.zone.clone()),
                ..Default::default()
            },
        }) {
            Ok(resp) => resp.result.first().unwrap().id.clone(),
            Err(_) => {
                return Json(AddResult {
                    success: false,
                    e: "ERR_ZONE_NOT_FOUND".into(),
                })
            }
        };

        match api_client.request(&zone::ZoneDetails {
            identifier: &zone_id,
        }) {
            Ok(resp) => {
                let create_result_ = api_client.request(&dns::CreateDnsRecord {
                    zone_identifier: &resp.result.id,
                    params: dns::CreateDnsRecordParams {
                        content: record,
                        name: &req.rec,
                        proxied: Some(false),
                        ttl: None,
                        priority: None,
                    },
                });

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

    let mut connection = establish_connection();
    let results = users
        .filter(disabled.eq(false))
        .filter(name.eq(&req.user))
        .filter(key.eq(&req.key))
        .load::<User>(&mut connection)
        .expect("Error loading users");

    if results.len() != 1 {
        return Json(DeleteResult {
            success: false,
            e: "ERR_UNKNOWN".to_string(),
        });
    }

    let privs = UserSitePrivilege::belonging_to(&results[0])
        .inner_join(sites::dsl::sites)
        .select((
            sites::dsl::zone,
            user_site_privileges::pattern,
            user_site_privileges::dsl::superuser,
        ))
        .filter(sites::dsl::zone.eq(&req.zone))
        .load::<(String, String, bool)>(&mut connection)
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

    let record: DnsContent;
    let _record_type = req.rectype.to_uppercase();
    match _record_type.as_str() {
        "A" => {
            record = DnsContent::A {
                content: Ipv4Addr::from_str(&req.value).unwrap(),
            };
        }
        "AAAA" => {
            record = DnsContent::AAAA {
                content: Ipv6Addr::from_str(&req.value).unwrap(),
            };
        }
        "TXT" => {
            record = DnsContent::TXT {
                content: req.value.clone(),
            };
        }
        "SRV" => {
            record = DnsContent::SRV {
                content: req.value.clone(),
            };
        }
        "CNAME" => {
            record = DnsContent::CNAME {
                content: req.value.clone(),
            };
        }
        _ => {
            return Json(DeleteResult {
                success: false,
                e: "ERR_REC_TYPE".to_string(),
            });
        }
    }

    let credentials: Credentials = if let Some(cf_key) = &cf_conf.key {
        Credentials::UserAuthKey {
            email: cf_conf.user.clone().unwrap(),
            key: cf_key.clone(),
        }
    } else if let Some(token) = &cf_conf.token {
        Credentials::UserAuthToken {
            token: token.to_string(),
        }
    } else {
        panic!("Either API token or API key + email pair must be provided")
    };

    let api_client = HttpApiClient::new(
        credentials,
        HttpApiClientConfig::default(),
        Environment::Production,
    )
    .unwrap();

    let zone_id = match api_client.request(&zone::ListZones {
        params: zone::ListZonesParams {
            name: Some(req.zone.clone()),
            ..Default::default()
        },
    }) {
        Ok(resp) => resp.result.first().unwrap().id.clone(),
        Err(_) => {
            return Json(DeleteResult {
                success: false,
                e: "ERR_ZONE_NOT_FOUND".into(),
            })
        }
    };

    match api_client.request(&zone::ZoneDetails {
        identifier: &zone_id,
    }) {
        Ok(resp) => {
            let current_rec_ = api_client
                .request(&dns::ListDnsRecords {
                    zone_identifier: &resp.result.id,
                    params: ListDnsRecordsParams {
                        record_type: Some(record),
                        direction: Some(OrderDirection::Ascending),
                        ..Default::default()
                    },
                })
                .map_err(|err| format!("Failed to list DNS A records: {}", format_error(err)))
                .and_then(|recs| {
                    recs.result
                        .into_iter()
                        .find(|item| item.name == req.rec)
                        .ok_or_else(|| format!("Could not find A record for {}", req.rec))
                });
            match current_rec_ {
                Ok(current_rec) => {
                    let delete_result_ = api_client.request(&dns::DeleteDnsRecord {
                        zone_identifier: &resp.result.id,
                        identifier: &current_rec.id,
                    });

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
            let user = rocket
                .config()
                .get_str("cfuser")
                .ok()
                .and_then(|s| Some(s.to_string()));
            let key = rocket
                .config()
                .get_str("cfkey")
                .ok()
                .and_then(|s| Some(s.to_string()));
            let token = rocket
                .config()
                .get_str("cftoken")
                .ok()
                .and_then(|s| Some(s.to_string()));
            Ok(rocket.manage(CfCredentials {
                user: user,
                key: key,
                token: token,
            }))
        }))
        .launch();
}

fn format_error(error: ApiFailure) -> String {
    match error {
        ApiFailure::Error(status, errors) => {
            format!("{:?}", Json(errors.errors.into_iter().map(|e| { e.to_string() }).collect::<Vec<_>>()))
        }
        ApiFailure::Invalid(reqwest_err) => reqwest_err.to_string()
        // ApiFailure::NoResultsReturned => "No results returned".into(),
        // ApiFailure::InvalidOptions => "Invalid options".into(),
        // ApiFailure::NotSuccess => "API request failed".into(),
        // ApiFailure::Reqwest(cause) => format!("Network error: {}", cause),
        // ApiFailure::Json(cause) => format!("JSON error: {}", cause),
        // ApiFailure::Io(cause) => format!("IO error: {}", cause),
        // ApiFailure::Url(cause) => format!("URL error: {}", cause),
    }
}

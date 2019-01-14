use crate::schema::*;

#[derive(Queryable, Serialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub key: String,
    pub disabled: bool,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub key: &'a str,
}

#[derive(Queryable, Serialize)]
pub struct Site {
    pub id: i32,
    pub name: String,
    pub zone: String,
    pub disabled: bool,
}

#[derive(Queryable, Serialize, Insertable, PartialEq, Associations)]
#[belongs_to(User, foreign_key="user_id")]
pub struct UserSitePrivilege {
    pub id: i32,
    pub user_id: i32,
    pub site_id: i32,
    pub pattern: String,
    pub superuser: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateRequest {
    pub user: String,
    pub key: String,
    pub zone: String,
    pub rec: String,
    pub rectype: String,
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateResult {
    pub success: bool,
    pub e: String,
}

use crate::schema::*;

#[derive(Queryable, Serialize, Identifiable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub key: String,
    pub disabled: bool,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub key: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = sites)]
pub struct NewSite<'a> {
    pub name: &'a str,
    pub zone: &'a str,
}

#[derive(Queryable, Serialize, Identifiable)]
pub struct Site {
    pub id: i32,
    pub name: String,
    pub zone: String,
    pub disabled: bool,
}

#[derive(Insertable, Serialize)]
#[diesel(table_name = user_site_privileges)]
pub struct NewUserSitePrivilege<'a> {
    pub user_id: i32,
    pub site_id: i32,
    pub pattern: &'a str,
    pub superuser: bool,
}

#[derive(Queryable, Serialize, PartialEq, Associations, Debug, Identifiable)]
#[diesel(belongs_to(User, foreign_key = user_id))]
#[diesel(belongs_to(Site, foreign_key = site_id))]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct AddRequest {
    pub user: String,
    pub key: String,
    pub zone: String,
    pub rec: String,
    pub rectype: String,
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct AddResult {
    pub success: bool,
    pub e: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteRequest {
    pub user: String,
    pub key: String,
    pub zone: String,
    pub rec: String,
    pub rectype: String,
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteResult {
    pub success: bool,
    pub e: String,
}

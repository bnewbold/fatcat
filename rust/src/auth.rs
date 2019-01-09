//! Editor bearer token authentication

use data_encoding::BASE64;
use macaroon::{Format, Macaroon, Verifier};
use std::fmt;
use swagger::auth::{AuthData, Authorization, Scopes};

use crate::database_models::*;
use crate::database_schema::*;
use crate::errors::*;
use crate::identifiers::*;
use crate::server::*;
use chrono::prelude::*;
use diesel;
use diesel::prelude::*;
use std::collections::HashMap;
use std::env;
use std::str::FromStr;

// 32 bytes max (!)
static DUMMY_KEY: &[u8] = b"dummy-key-a-one-two-three-a-la";

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FatcatRole {
    Public,
    Editor,
    Bot,
    Human,
    Admin,
    Superuser,
}

#[derive(Clone)]
pub struct AuthContext {
    pub editor_id: FatCatId,
    editor_row: EditorRow,
}

impl AuthContext {
    pub fn has_role(&self, role: FatcatRole) -> bool {
        if !self.editor_row.is_active {
            // if account is disabled, only allow public role
            return role == FatcatRole::Public;
        }
        if self.editor_row.is_superuser {
            return true;
        }
        match role {
            FatcatRole::Public => true,
            FatcatRole::Editor => true,
            FatcatRole::Bot => self.editor_row.is_bot,
            FatcatRole::Human => !self.editor_row.is_bot,
            FatcatRole::Admin => self.editor_row.is_admin,
            FatcatRole::Superuser => self.editor_row.is_superuser,
        }
    }

    pub fn require_role(&self, role: FatcatRole) -> Result<()> {
        match self.has_role(role) {
            true => Ok(()),
            false => Err(ErrorKind::InsufficientPrivileges(format!(
                "doesn't have required role: {:?}",
                role
            ))
            .into()),
        }
    }

    pub fn require_editgroup(&self, conn: &DbConn, editgroup_id: FatCatId) -> Result<()> {
        if self.has_role(FatcatRole::Admin) {
            return Ok(());
        }
        let editgroup: EditgroupRow = editgroup::table
            .find(editgroup_id.to_uuid())
            .get_result(conn)?;
        match editgroup.editor_id == self.editor_id.to_uuid() {
            true => Ok(()),
            false => Err(ErrorKind::InsufficientPrivileges(
                "editor does not own this editgroup".to_string(),
            )
            .into()),
        }
    }
}

#[derive(Debug)]
pub struct AuthError {
    msg: String,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AuthError: {}", &self.msg)
    }
}

impl iron::Error for AuthError {
    fn description(&self) -> &str {
        &self.msg
    }
    fn cause(&self) -> Option<&iron::Error> {
        None
    }
}

fn new_auth_ironerror(m: &str) -> iron::error::IronError {
    iron::error::IronError::new(
        AuthError { msg: m.to_string() },
        (iron::status::BadRequest, m.to_string()),
    )
}

#[derive(Debug, Default)]
pub struct OpenAuthMiddleware;

impl OpenAuthMiddleware {
    /// Create a middleware that authorizes with the configured subject.
    pub fn new() -> OpenAuthMiddleware {
        OpenAuthMiddleware
    }
}

impl iron::middleware::BeforeMiddleware for OpenAuthMiddleware {
    fn before(&self, req: &mut iron::Request) -> iron::IronResult<()> {
        req.extensions.insert::<Authorization>(Authorization {
            subject: "undefined".to_string(),
            scopes: Scopes::All,
            issuer: None,
        });
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct MacaroonAuthMiddleware;

impl MacaroonAuthMiddleware {
    pub fn new() -> MacaroonAuthMiddleware {
        MacaroonAuthMiddleware
    }
}
impl iron::middleware::BeforeMiddleware for MacaroonAuthMiddleware {
    fn before(&self, req: &mut iron::Request) -> iron::IronResult<()> {
        // Structure here is sorta funky because we might some day actually want to parse token
        // here in some way
        let token: Option<String> = match req.extensions.get::<AuthData>() {
            Some(AuthData::ApiKey(header)) => {
                let header: Vec<String> =
                    header.split_whitespace().map(|s| s.to_string()).collect();
                if !(header.len() == 2 && header[0] == "Bearer") {
                    return Err(new_auth_ironerror("invalid bearer auth HTTP Header"));
                }
                Some(header[1].to_string())
            }
            None => None,
            _ => {
                return Err(new_auth_ironerror(
                    "auth HTTP Header should be empty or API token",
                ));
            }
        };
        if let Some(_token) = token {
            req.extensions.insert::<Authorization>(Authorization {
                // This is just a dummy; all actual authentication happens later
                subject: "undefined".to_string(),
                scopes: Scopes::All,
                issuer: None,
            });
        };
        Ok(())
    }
}

#[derive(Clone)]
pub struct AuthConfectionary {
    pub location: String,
    pub identifier: String,
    pub key: Vec<u8>,
    pub root_keys: HashMap<String, Vec<u8>>,
}

impl AuthConfectionary {
    pub fn new(
        location: String,
        identifier: String,
        key_base64: &str,
    ) -> Result<AuthConfectionary> {
        macaroon::initialize().unwrap();
        let key = BASE64.decode(key_base64.as_bytes())?;
        let mut root_keys = HashMap::new();
        root_keys.insert(identifier.clone(), key.clone());
        Ok(AuthConfectionary {
            location,
            identifier,
            key,
            root_keys,
        })
    }

    pub fn new_dummy() -> AuthConfectionary {
        AuthConfectionary::new(
            "test.fatcat.wiki".to_string(),
            "dummy".to_string(),
            &BASE64.encode(DUMMY_KEY),
        )
        .unwrap()
    }

    pub fn add_keypair(&mut self, identifier: String, key_base64: &str) -> Result<()> {
        let key = BASE64.decode(key_base64.as_bytes())?;
        self.root_keys.insert(identifier, key);
        Ok(())
    }

    pub fn create_token(
        &self,
        editor_id: FatCatId,
        duration: Option<chrono::Duration>,
    ) -> Result<String> {
        let mut mac = Macaroon::create(&self.location, &self.key, &self.identifier)
            .expect("Macaroon creation");
        mac.add_first_party_caveat(&format!("editor_id = {}", editor_id.to_string()));
        let now_utc = Utc::now();
        let now = now_utc.to_rfc3339_opts(SecondsFormat::Secs, true);
        mac.add_first_party_caveat(&format!("time > {}", now));
        if let Some(duration) = duration {
            let expires = now_utc + duration;
            mac.add_first_party_caveat(&format!(
                "time < {:?}",
                &expires.to_rfc3339_opts(SecondsFormat::Secs, true)
            ));
        };
        let raw = mac.serialize(Format::V2).expect("macaroon serialization");
        Ok(BASE64.encode(&raw))
    }

    pub fn parse_macaroon_token(
        &self,
        conn: &DbConn,
        s: &str,
        endpoint: Option<&str>,
    ) -> Result<EditorRow> {
        let raw = BASE64.decode(s.as_bytes())?;
        let mac = match Macaroon::deserialize(&raw) {
            Ok(m) => m,
            Err(e) => {
                // TODO: should be "chaining" here
                return Err(ErrorKind::InvalidCredentials(format!(
                    "macaroon deserialize error: {:?}",
                    e
                ))
                .into());
            }
        };
        let mac = match mac.validate() {
            Ok(m) => m,
            Err(e) => {
                // TODO: should be "chaining" here
                return Err(ErrorKind::InvalidCredentials(format!(
                    "macaroon validate error: {:?}",
                    e
                ))
                .into());
            }
        };
        let mut verifier = Verifier::new();
        let mut editor_id: Option<FatCatId> = None;
        for caveat in mac.first_party_caveats() {
            if caveat.predicate().starts_with("editor_id = ") {
                editor_id = Some(FatCatId::from_str(caveat.predicate().get(12..).unwrap())?);
                break;
            }
        }
        let editor_id = match editor_id {
            Some(id) => id,
            None => {
                return Err(ErrorKind::InvalidCredentials(
                    "expected an editor_id caveat".to_string(),
                )
                .into());
            }
        };
        verifier.satisfy_exact(&format!("editor_id = {}", editor_id.to_string()));
        if let Some(endpoint) = endpoint {
            // API endpoint
            verifier.satisfy_exact(&format!("endpoint = {}", endpoint));
        }
        let mut created: Option<DateTime<Utc>> = None;
        for caveat in mac.first_party_caveats() {
            if caveat.predicate().starts_with("time > ") {
                created = Some(
                    DateTime::parse_from_rfc3339(caveat.predicate().get(7..).unwrap())
                        .unwrap()
                        .with_timezone(&Utc),
                );
                break;
            }
        }
        let created = match created {
            Some(c) => c,
            None => {
                return Err(ErrorKind::InvalidCredentials(
                    "expected a 'created' (time >) caveat".to_string(),
                )
                .into());
            }
        };
        verifier.satisfy_exact(&format!(
            "time > {}",
            created.to_rfc3339_opts(SecondsFormat::Secs, true)
        ));
        let editor: EditorRow = editor::table.find(&editor_id.to_uuid()).get_result(conn)?;
        let auth_epoch = DateTime::<Utc>::from_utc(editor.auth_epoch, Utc);
        // allow a second of wiggle room for precision and, eg, tests
        if created < (auth_epoch - chrono::Duration::seconds(1)) {
            return Err(ErrorKind::InvalidCredentials(
                "token created before current auth_epoch (was probably revoked by editor)"
                    .to_string(),
            )
            .into());
        }
        verifier.satisfy_general(|p: &str| -> bool {
            // not expired (based on time)
            if p.starts_with("time < ") {
                let expires: DateTime<Utc> = DateTime::parse_from_rfc3339(p.get(7..).unwrap())
                    .unwrap()
                    .with_timezone(&Utc);
                expires < Utc::now()
            } else {
                false
            }
        });
        let verify_key = match self.root_keys.get(mac.identifier()) {
            Some(key) => key,
            None => {
                return Err(ErrorKind::InvalidCredentials(format!(
                    "no valid auth signing key for identifier: {}",
                    mac.identifier()
                ))
                .into());
            }
        };
        match mac.verify(verify_key, &mut verifier) {
            Ok(true) => (),
            Ok(false) => {
                return Err(ErrorKind::InvalidCredentials(
                    "auth token (macaroon) not valid (signature and/or caveats failed)".to_string(),
                )
                .into());
            }
            Err(e) => {
                // TODO: chain
                return Err(
                    ErrorKind::InvalidCredentials(format!("token parsing failed: {:?}", e)).into(),
                );
            }
        }
        Ok(editor)
    }

    pub fn parse_swagger(
        &self,
        conn: &DbConn,
        auth_data: &Option<AuthData>,
        endpoint: Option<&str>,
    ) -> Result<Option<AuthContext>> {
        let token: Option<String> = match auth_data {
            Some(AuthData::ApiKey(header)) => {
                let header: Vec<String> =
                    header.split_whitespace().map(|s| s.to_string()).collect();
                if !(header.len() == 2 && header[0] == "Bearer") {
                    return Err(ErrorKind::InvalidCredentials(
                        "invalid Bearer Auth HTTP header".to_string(),
                    )
                    .into());
                }
                Some(header[1].clone())
            }
            None => None,
            _ => {
                return Err(ErrorKind::InvalidCredentials(
                    "Authentication HTTP Header should either be empty or a Beaerer API key"
                        .to_string(),
                )
                .into());
            }
        };
        let token = match token {
            Some(t) => t,
            None => return Ok(None),
        };
        let editor_row = self.parse_macaroon_token(conn, &token, endpoint)?;
        Ok(Some(AuthContext {
            editor_id: FatCatId::from_uuid(&editor_row.id),
            editor_row,
        }))
    }

    pub fn require_auth(
        &self,
        conn: &DbConn,
        auth_data: &Option<AuthData>,
        endpoint: Option<&str>,
    ) -> Result<AuthContext> {
        match self.parse_swagger(conn, auth_data, endpoint)? {
            Some(auth) => Ok(auth),
            None => Err(ErrorKind::InvalidCredentials("no token supplied".to_string()).into()),
        }
    }

    // TODO: refactor out of this file?
    /// Only used from CLI tool
    pub fn inspect_token(&self, conn: &DbConn, token: &str) -> Result<()> {
        let raw = BASE64.decode(token.as_bytes())?;
        let mac = match Macaroon::deserialize(&raw) {
            Ok(m) => m,
            Err(e) => bail!("macaroon deserialize error: {:?}", e),
        };
        let now = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        println!("current time: {}", now);
        println!("domain (location): {:?}", mac.location());
        println!("signing key name (identifier): {}", mac.identifier());
        for caveat in mac.first_party_caveats() {
            println!("caveat: {}", caveat.predicate());
        }
        println!("verify: {:?}", self.parse_macaroon_token(conn, token, None));
        Ok(())
    }
}

pub fn create_key() -> String {
    let mut key: Vec<u8> = vec![0; 32];
    for v in key.iter_mut() {
        *v = rand::random()
    }
    BASE64.encode(&key)
}

pub fn revoke_tokens(conn: &DbConn, editor_id: FatCatId) -> Result<()> {
    diesel::update(editor::table.filter(editor::id.eq(&editor_id.to_uuid())))
        .set(editor::auth_epoch.eq(Utc::now()))
        .execute(conn)?;
    Ok(())
}

pub fn revoke_tokens_everyone(conn: &DbConn) -> Result<()> {
    diesel::update(editor::table)
        .set(editor::auth_epoch.eq(Utc::now()))
        .execute(conn)?;
    Ok(())
}

// TODO: refactor out of this file?
/// Only used from CLI tool
pub fn print_editors(conn: &DbConn) -> Result<()> {
    // iterate over all editors. format id, print flags, auth_epoch
    let all_editors: Vec<EditorRow> = editor::table.load(conn)?;
    println!("editor_id\t\t\tsuper/admin/bot\tauth_epoch\t\t\tusername\twrangler_id");
    for e in all_editors {
        println!(
            "{}\t{}/{}/{}\t{}\t{}\t{:?}",
            FatCatId::from_uuid(&e.id).to_string(),
            e.is_superuser,
            e.is_admin,
            e.is_bot,
            e.auth_epoch,
            e.username,
            e.wrangler_id,
        );
    }
    Ok(())
}

pub fn env_confectionary() -> Result<AuthConfectionary> {
    let auth_location = env::var("AUTH_LOCATION").expect("AUTH_LOCATION must be set");
    let auth_key = env::var("AUTH_SECRET_KEY").expect("AUTH_SECRET_KEY must be set");
    let auth_key_ident = env::var("AUTH_KEY_IDENT").expect("AUTH_KEY_IDENT must be set");
    info!("Loaded primary auth key: {}", auth_key_ident);
    let mut confectionary = AuthConfectionary::new(auth_location, auth_key_ident, &auth_key)?;
    if let Ok(var) = env::var("AUTH_ALT_KEYS") {
        for pair in var.split(',') {
            let pair: Vec<&str> = pair.split(':').collect();
            if pair.len() != 2 {
                println!("{:#?}", pair);
                bail!("couldn't parse keypair from AUTH_ALT_KEYS (expected 'ident:key' pairs separated by commas)");
            }
            info!("Loading alt auth key: {}", pair[0]);
            confectionary.add_keypair(pair[0].to_string(), pair[1])?;
        }
    };
    Ok(confectionary)
}

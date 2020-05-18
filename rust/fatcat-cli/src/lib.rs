
use fatcat_openapi;
use fatcat_openapi::ApiNoContext;
use fatcat_openapi::client::Client;
use fatcat_openapi::ContextWrapperExt;
use swagger::{AuthData, ContextBuilder, EmptyContext, Push, XSpanIdString};
use failure::{Error, format_err};
use log::{self,debug};
use lazy_static::lazy_static;
use std::str::FromStr;
use regex::Regex;
use hyper::client::ResponseFuture;
use tokio::runtime::current_thread::Runtime;

pub mod entities;

use entities::ApiEntityModel;

pub struct FatcatApiClient<'a> {
    pub api: fatcat_openapi::ContextWrapper<'a, Client<ResponseFuture>, swagger::make_context_ty!( ContextBuilder, EmptyContext, Option<AuthData>, XSpanIdString)>,
    pub rt: tokio::runtime::current_thread::Runtime,
}

impl<'a> FatcatApiClient<'a> {

    pub fn new(client: &'a fatcat_openapi::client::Client<ResponseFuture>) -> Self {

        let context: swagger::make_context_ty!(
            ContextBuilder,
            EmptyContext,
            Option<AuthData>,
            XSpanIdString
        ) = swagger::make_context!(
            ContextBuilder,
            EmptyContext,
            None as Option<AuthData>,
            XSpanIdString::default()
        );

        let wrapped_client: fatcat_openapi::ContextWrapper<Client<ResponseFuture>, swagger::make_context_ty!(
            ContextBuilder,
            EmptyContext,
            Option<AuthData>,
            XSpanIdString
        )> = client.with_context(context);
        let rt: Runtime = Runtime::new().unwrap();
        
        FatcatApiClient {
            api: wrapped_client,
            rt,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ReleaseLookupKey {
    DOI,
    PMCID,
    PMID,
    Arxiv,
    // TODO: the others
}

#[derive(Debug, PartialEq, Clone)]
pub enum ContainerLookupKey {
    ISSNL,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CreatorLookupKey {
    Orcid,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FileLookupKey {
    SHA1,
    SHA256,
    MD5,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Specifier {
    Release(String),
    ReleaseLookup(ReleaseLookupKey, String),
    Work(String),
    Container(String),
    ContainerLookup(ContainerLookupKey, String),
    Creator(String),
    CreatorLookup(CreatorLookupKey, String),
    File(String),
    FileLookup(FileLookupKey, String),
    FileSet(String),
    WebCapture(String),
    Editgroup(String),
    Editor(String),
    EditorUsername(String),
    Changelog(i64),
}

impl Specifier {

    /// If this Specifier is a lookup, call the API to do the lookup and return the resulting
    /// specific entity specifier (eg, with an FCID). If already specific, just pass through.
    // TODO: refactor to call self.get_from_api() for lookups, and pull out just identifiers
    pub fn into_entity_specifier(self, api_client: FatcatApiClient) -> Result<Specifier, Error> {
        use Specifier::*;
        match self {
            Release(_) | Work(_) | Creator(_) | Container(_) | File(_) | FileSet(_) | WebCapture(_) | Editgroup(_) | Editor(_) | Changelog(_) => Ok(self),
            ReleaseLookup(_, _) => Ok(Specifier::Release(self.get_from_api(api_client)?.fcid())),
            ContainerLookup(_, _) => Ok(Specifier::Container(self.get_from_api(api_client)?.fcid())),
            CreatorLookup(_, _) => Ok(Specifier::Creator(self.get_from_api(api_client)?.fcid())),
            FileLookup(_, _) => Ok(Specifier::File(self.get_from_api(api_client)?.fcid())),
            EditorUsername(_username) => {
                unimplemented!("editor lookup by username isn't implemented in fatcat-server API yet, sorry")
            },
        }
    }

    pub fn get_from_api(&self, mut api_client: FatcatApiClient) -> Result<Box<dyn ApiEntityModel>, Error> {
        use Specifier::*;
        match self {
            Release(fcid) =>
                match api_client.rt.block_on(api_client.api.get_release(fcid.to_string(), None, None)) {
                    Ok(fatcat_openapi::GetReleaseResponse::FoundEntity(model)) => Ok(Box::new(model)),
                    Ok(_) => unimplemented!("not found, etc"),
                    Err(_) => Err(format_err!("some API problem"))
                },
            ReleaseLookup(ext_id, key) => {
                use ReleaseLookupKey::*;
                let (doi, pmcid, pmid, arxiv) = (
                    if let DOI = ext_id { Some(key.to_string()) } else { None },
                    if let PMCID = ext_id { Some(key.to_string()) } else { None },
                    if let PMID = ext_id { Some(key.to_string()) } else { None },
                    if let Arxiv = ext_id { Some(key.to_string()) } else { None },
                );
                // doi, wikidata, isbn13, pmid, pmcid, core, arxiv, jstor, ark, mag
                let result = api_client.rt.block_on(
                    api_client.api.lookup_release(doi, None, None, pmid, pmcid, None, arxiv, None, None, None, None, None));
                match result {
                    Ok(fatcat_openapi::LookupReleaseResponse::FoundEntity(model)) => Ok(Box::new(model)),
                    Ok(_) => unimplemented!("not found, etc"),
                    Err(_) => Err(format_err!("some API problem"))
                }
            },
            Work(fcid) =>
                match api_client.rt.block_on(api_client.api.get_work(fcid.to_string(), None, None)) {
                    Ok(fatcat_openapi::GetWorkResponse::FoundEntity(model)) => Ok(Box::new(model)),
                    Ok(_) => unimplemented!("not found, etc"),
                    Err(_) => Err(format_err!("some API problem"))
                },
            Container(fcid) =>
                match api_client.rt.block_on(api_client.api.get_release(fcid.to_string(), None, None)) {
                    Ok(fatcat_openapi::GetReleaseResponse::FoundEntity(model)) => Ok(Box::new(model)),
                    Ok(_) => unimplemented!("not found, etc"),
                    Err(_) => Err(format_err!("some API problem"))
                },
            ContainerLookup(ext_id, key) => {
                let result = api_client.rt.block_on(match ext_id {
                    ContainerLookupKey::ISSNL => api_client.api.lookup_container(Some(key.to_string()), None, None, None),
                });
                match result {
                    Ok(fatcat_openapi::LookupContainerResponse::FoundEntity(model)) => Ok(Box::new(model)),
                    Ok(_) => unimplemented!("not found, etc"),
                    Err(_) => Err(format_err!("some API problem"))
                }
            },
            Creator(fcid) =>
                match api_client.rt.block_on(api_client.api.get_release(fcid.to_string(), None, None)) {
                    Ok(fatcat_openapi::GetReleaseResponse::FoundEntity(model)) => Ok(Box::new(model)),
                    Ok(_) => unimplemented!("not found, etc"),
                    Err(_) => Err(format_err!("some API problem"))
                },
            CreatorLookup(ext_id, key) => {
                let result = api_client.rt.block_on(match ext_id {
                    CreatorLookupKey::Orcid => api_client.api.lookup_creator(Some(key.to_string()), None, None, None),
                });
                match result {
                    Ok(fatcat_openapi::LookupCreatorResponse::FoundEntity(model)) => Ok(Box::new(model)),
                    Ok(_) => unimplemented!("not found, etc"),
                    Err(_) => Err(format_err!("some API problem"))
                }
            },
            File(fcid) =>
                match api_client.rt.block_on(api_client.api.get_file(fcid.to_string(), None, None)) {
                    Ok(fatcat_openapi::GetFileResponse::FoundEntity(model)) => Ok(Box::new(model)),
                    Ok(_) => unimplemented!("not found, etc"),
                    Err(_) => Err(format_err!("some API problem"))
                },
            FileLookup(hash, key) => {
                use FileLookupKey::*;
                let (sha1, sha256, md5) = (
                    if let SHA1 = hash { Some(key.to_string()) } else { None },
                    if let SHA256 = hash { Some(key.to_string()) } else { None },
                    if let MD5 = hash { Some(key.to_string()) } else { None },
                );
                let result = api_client.rt.block_on(
                    api_client.api.lookup_file(sha1, sha256, md5, None, None),
                );
                match result {
                    Ok(fatcat_openapi::LookupFileResponse::FoundEntity(model)) => Ok(Box::new(model)),
                    Ok(_) => unimplemented!("not found, etc"),
                    Err(_) => Err(format_err!("some API problem"))
                }
            },
            FileSet(fcid) =>
                match api_client.rt.block_on(api_client.api.get_fileset(fcid.to_string(), None, None)) {
                    Ok(fatcat_openapi::GetFilesetResponse::FoundEntity(model)) => Ok(Box::new(model)),
                    Ok(_) => unimplemented!("not found, etc"),
                    Err(_) => Err(format_err!("some API problem"))
                },
            WebCapture(fcid) =>
                match api_client.rt.block_on(api_client.api.get_webcapture(fcid.to_string(), None, None)) {
                    Ok(fatcat_openapi::GetWebcaptureResponse::FoundEntity(model)) => Ok(Box::new(model)),
                    Ok(_) => unimplemented!("not found, etc"),
                    Err(_) => Err(format_err!("some API problem"))
                },
            Editgroup(fcid) =>
                match api_client.rt.block_on(api_client.api.get_editgroup(fcid.to_string())) {
                    Ok(fatcat_openapi::GetEditgroupResponse::Found(model)) => Ok(Box::new(model)),
                    Ok(_) => unimplemented!("not found, etc"),
                    Err(_) => Err(format_err!("some API problem"))
                },
            Editor(fcid) =>
                match api_client.rt.block_on(api_client.api.get_editor(fcid.to_string())) {
                    Ok(fatcat_openapi::GetEditorResponse::Found(model)) => Ok(Box::new(model)),
                    Ok(_) => unimplemented!("not found, etc"),
                    Err(_) => Err(format_err!("some API problem"))
                },
            Changelog(_index) => {
                panic!("changelog doesn't have fcid, so doesn't match this trait")
            },
            EditorUsername(_username) => {
                unimplemented!("editor lookup by username isn't implemented in fatcat-server API yet, sorry")
            },
        }
    }
}

impl FromStr for Specifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // first try simple entity prefixes
        lazy_static! {
            static ref SPEC_ENTITY_RE: Regex = Regex::new(r"^(release|work|creator|container|file|fileset|webcapture|editgroup|editor)_([2-7a-z]{26})$").unwrap();
        }
        if let Some(caps) = SPEC_ENTITY_RE.captures(s) {
            return match (&caps[1], &caps[2]) {
                ("release", fcid) => Ok(Specifier::Release(fcid.to_string())),
                ("work", fcid) => Ok(Specifier::Work(fcid.to_string())),
                ("container", fcid) => Ok(Specifier::Container(fcid.to_string())),
                ("creator", fcid) => Ok(Specifier::Creator(fcid.to_string())),
                ("file", fcid) => Ok(Specifier::File(fcid.to_string())),
                ("fileset", fcid) => Ok(Specifier::FileSet(fcid.to_string())),
                ("webcapture", fcid) => Ok(Specifier::WebCapture(fcid.to_string())),
                ("editgroup", fcid) => Ok(Specifier::Editgroup(fcid.to_string())),
                ("editor", fcid) => Ok(Specifier::Editor(fcid.to_string())),
                _ => unimplemented!("unexpected fatcat FCID type: {}", &caps[1]),
            };
        }

        // then try lookup prefixes
        lazy_static! {
            static ref SPEC_LOOKUP_RE: Regex = Regex::new(r"^(doi|pmcid|pmid|arxiv|issnl|orcid|sha1|sha256|md5|username|changelog):(\S+)$").unwrap();
        }
        if let Some(caps) = SPEC_LOOKUP_RE.captures(s) {
            return match (&caps[1], &caps[2]) {
                ("doi", key) => Ok(Specifier::ReleaseLookup(ReleaseLookupKey::DOI, key.to_string())),
                ("pmcid", key) => Ok(Specifier::ReleaseLookup(ReleaseLookupKey::PMCID, key.to_string())),
                ("pmid", key) => Ok(Specifier::ReleaseLookup(ReleaseLookupKey::PMID, key.to_string())),
                ("arxiv", key) => Ok(Specifier::ReleaseLookup(ReleaseLookupKey::Arxiv, key.to_string())),
                ("issnl", key) => Ok(Specifier::ContainerLookup(ContainerLookupKey::ISSNL, key.to_string())),
                ("orcid", key) => Ok(Specifier::CreatorLookup(CreatorLookupKey::Orcid, key.to_string())),
                ("sha1", key) => Ok(Specifier::FileLookup(FileLookupKey::SHA1, key.to_string())),
                ("sha256", key) => Ok(Specifier::FileLookup(FileLookupKey::SHA256, key.to_string())),
                ("md5", key) => Ok(Specifier::FileLookup(FileLookupKey::MD5, key.to_string())),
                ("username", key) => Ok(Specifier::EditorUsername(key.to_string())),
                _ => unimplemented!("unexpected fatcat lookup key: {}", &caps[1]),
            };
        }
        // lastly, changelog entity lookup
        lazy_static! {
            static ref SPEC_CHANGELOG_RE: Regex = Regex::new(r"^changelog_(\d+)$").unwrap();
        };
        if let Some(caps) = SPEC_CHANGELOG_RE.captures(s) {
            return Ok(Specifier::Changelog(caps[1].parse::<i64>()?));
        }
        return Err(format_err!("expecting a specifier: entity identifier or key/value lookup: {}", s))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specifier_from_str() -> () {
        assert!(Specifier::from_str("release_asdf").is_err());
        assert_eq!(Specifier::from_str("creator_iimvc523xbhqlav6j3sbthuehu").unwrap(), Specifier::Creator("iimvc523xbhqlav6j3sbthuehu".to_string()));
        assert_eq!(Specifier::from_str("username:big-bot").unwrap(), Specifier::EditorUsername("big-bot".to_string()));
        assert_eq!(Specifier::from_str("doi:10.1234/a!s.df+-d").unwrap(), Specifier::ReleaseLookup(ReleaseLookupKey::DOI, "10.1234/a!s.df+-d".to_string()));
        assert!(Specifier::from_str("doi:").is_err());
        assert_eq!(Specifier::from_str("changelog_1234").unwrap(), Specifier::Changelog(1234));
        assert!(Specifier::from_str("changelog_12E4").is_err());
    }

}


use fatcat_openapi;
use serde_json;
use toml;
//use fatcat_openapi::Api;
use fatcat_openapi::ApiNoContext;
use fatcat_openapi::client::Client;
use fatcat_openapi::models::*;
use structopt::StructOpt;
use swagger::{AuthData, ContextBuilder, EmptyContext, Push, XSpanIdString};
use fatcat_openapi::ContextWrapperExt;
use failure::{Error, format_err};
use exitfailure::ExitFailure;
use atty::Stream;
use log::{self,info,debug};
use env_logger;
use lazy_static::lazy_static;
use std::str::FromStr;
use regex::Regex;
use tokio::runtime::current_thread::Runtime;
use hyper::client::ResponseFuture;


struct FatcatApiClient<'a> {
    api: fatcat_openapi::ContextWrapper<'a, Client<ResponseFuture>, swagger::make_context_ty!( ContextBuilder, EmptyContext, Option<AuthData>, XSpanIdString)>,
    rt: tokio::runtime::current_thread::Runtime,
}

#[derive(Debug, PartialEq, Clone)]
enum ReleaseLookupKey {
    DOI,
    PMCID,
    PMID,
    Arxiv,
    // TODO: the others
}

#[derive(Debug, PartialEq, Clone)]
enum ContainerLookupKey {
    ISSNL,
}

#[derive(Debug, PartialEq, Clone)]
enum CreatorLookupKey {
    Orcid,
}

#[derive(Debug, PartialEq, Clone)]
enum FileLookupKey {
    SHA1,
    SHA256,
    MD5,
}

#[derive(Debug, PartialEq, Clone)]
enum Specifier {
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

enum ApiModel {
    Release(ReleaseEntity),
    Work(WorkEntity),
    Container(ContainerEntity),
    Creator(CreatorEntity),
    File(FileEntity),
    FileSet(FilesetEntity),
    WebCapture(WebcaptureEntity),
    Editgroup(Editgroup),
    Editor(Editor),
    Changelog(ChangelogEntry),
}

impl ApiModel {

    fn to_json_string(&self) -> Result<String, Error> {
        use ApiModel::*;
        match self {
            Release(e) => Ok(serde_json::to_string(e)?),
            Work(e) => Ok(serde_json::to_string(e)?),
            Container(e) => Ok(serde_json::to_string(e)?),
            Creator(e) => Ok(serde_json::to_string(e)?),
            File(e) => Ok(serde_json::to_string(e)?),
            FileSet(e) => Ok(serde_json::to_string(e)?),
            WebCapture(e) => Ok(serde_json::to_string(e)?),
            Editgroup(e) => Ok(serde_json::to_string(e)?),
            Editor(e) => Ok(serde_json::to_string(e)?),
            Changelog(e) => Ok(serde_json::to_string(e)?),
        }
    }

    fn to_toml_string(&self) -> Result<String, Error> {
        use ApiModel::*;
        match self {
            Release(e) => Ok(toml::Value::try_from(e)?.to_string()),
            Work(e) => Ok(toml::Value::try_from(e)?.to_string()),
            Container(e) => Ok(toml::Value::try_from(e)?.to_string()),
            Creator(e) => Ok(serde_json::to_string(e)?),
            File(e) => Ok(serde_json::to_string(e)?),
            FileSet(e) => Ok(serde_json::to_string(e)?),
            WebCapture(e) => Ok(serde_json::to_string(e)?),
            Editgroup(e) => Ok(serde_json::to_string(e)?),
            Editor(e) => Ok(serde_json::to_string(e)?),
            Changelog(e) => Ok(serde_json::to_string(e)?),
        }
    }
}

impl Specifier {

    /// If this Specifier is a lookup, call the API to do the lookup and return the resulting
    /// specific entity specifier (eg, with an FCID). If already specific, just pass through.
    // TODO: refactor to call self.get_from_api() for lookups, and pull out just identifiers
    fn into_entity_specifier(self, mut api_client: FatcatApiClient) -> Result<Specifier, Error> {
        use Specifier::*;
        match self {
            Release(_) | Work(_) | Creator(_) | Container(_) | File(_) | FileSet(_) | WebCapture(_) | Editgroup(_) | Editor(_) | Changelog(_) => Ok(self),
            ReleaseLookup(_, _) => {
                if let ApiModel::Release(model) = self.get_from_api(api_client)? {
                    Ok(Specifier::Release(model.ident.unwrap()))
                } else {
                    panic!("wrong entity type");
                }
            },
            ContainerLookup(_, _) => {
                if let ApiModel::Container(model) = self.get_from_api(api_client)? {
                    Ok(Specifier::Container(model.ident.unwrap()))
                } else {
                    panic!("wrong entity type");
                }
            },
            CreatorLookup(_, _) => {
                if let ApiModel::Creator(model) = self.get_from_api(api_client)? {
                    Ok(Specifier::Creator(model.ident.unwrap()))
                } else {
                    panic!("wrong entity type");
                }
            },
            FileLookup(_, _) => {
                if let ApiModel::File(model) = self.get_from_api(api_client)? {
                    Ok(Specifier::File(model.ident.unwrap()))
                } else {
                    panic!("wrong entity type");
                }
            },
            EditorUsername(_username) => {
                unimplemented!("editor lookup by username isn't implemented in fatcat-server API yet, sorry")
            },
        }
    }

    fn get_from_api(&self, mut api_client: FatcatApiClient) -> Result<ApiModel, Error> {
        use Specifier::*;
        match self {
            Release(fcid) => {
                let result = api_client.rt.block_on(api_client.api.get_release(fcid.to_string(), None, None));
                if let Ok(fatcat_openapi::GetReleaseResponse::FoundEntity(model)) = result {
                    Ok(ApiModel::Release(model))
                } else {
                    Err(format_err!("some API problem"))
                }
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
                if let Ok(fatcat_openapi::LookupReleaseResponse::FoundEntity(model)) = result {
                    Ok(ApiModel::Release(model))
                } else {
                    Err(format_err!("some API problem"))
                }
            },
            Work(fcid) => {
                let result = api_client.rt.block_on(api_client.api.get_work(fcid.to_string(), None, None));
                if let Ok(fatcat_openapi::GetWorkResponse::FoundEntity(model)) = result {
                    Ok(ApiModel::Work(model))
                } else {
                    Err(format_err!("some API problem"))
                }
            },
            Container(fcid) => {
                let result = api_client.rt.block_on(api_client.api.get_container(fcid.to_string(), None, None));
                if let Ok(fatcat_openapi::GetContainerResponse::FoundEntity(model)) = result {
                    Ok(ApiModel::Container(model))
                } else {
                    Err(format_err!("some API problem"))
                }
            },
            ContainerLookup(ext_id, key) => {
                let result = api_client.rt.block_on(match ext_id {
                    ContainerLookupKey::ISSNL => api_client.api.lookup_container(Some(key.to_string()), None, None, None),
                });
                if let Ok(fatcat_openapi::LookupContainerResponse::FoundEntity(model)) = result {
                    Ok(ApiModel::Container(model))
                } else {
                    Err(format_err!("some API problem"))
                }
            },
            Creator(fcid) => {
                let result = api_client.rt.block_on(api_client.api.get_creator(fcid.to_string(), None, None));
                if let Ok(fatcat_openapi::GetCreatorResponse::FoundEntity(model)) = result {
                    Ok(ApiModel::Creator(model))
                } else {
                    Err(format_err!("some API problem"))
                }
            },
            CreatorLookup(ext_id, key) => {
                let result = api_client.rt.block_on(match ext_id {
                    CreatorLookupKey::Orcid => api_client.api.lookup_creator(Some(key.to_string()), None, None, None),
                });
                if let Ok(fatcat_openapi::LookupCreatorResponse::FoundEntity(model)) = result {
                    Ok(ApiModel::Creator(model))
                } else {
                    Err(format_err!("some API problem"))
                }
            },
            File(fcid) => {
                let result = api_client.rt.block_on(api_client.api.get_file(fcid.to_string(), None, None));
                if let Ok(fatcat_openapi::GetFileResponse::FoundEntity(model)) = result {
                    Ok(ApiModel::File(model))
                } else {
                    Err(format_err!("some API problem"))
                }
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
                if let Ok(fatcat_openapi::LookupFileResponse::FoundEntity(model)) = result {
                    Ok(ApiModel::File(model))
                } else {
                    Err(format_err!("some API problem"))
                }
            },
            FileSet(fcid) => {
                let result = api_client.rt.block_on(api_client.api.get_fileset(fcid.to_string(), None, None));
                if let Ok(fatcat_openapi::GetFilesetResponse::FoundEntity(model)) = result {
                    Ok(ApiModel::FileSet(model))
                } else {
                    Err(format_err!("some API problem"))
                }
            },
            WebCapture(fcid) => {
                let result = api_client.rt.block_on(api_client.api.get_webcapture(fcid.to_string(), None, None));
                if let Ok(fatcat_openapi::GetWebcaptureResponse::FoundEntity(model)) = result {
                    Ok(ApiModel::WebCapture(model))
                } else {
                    Err(format_err!("some API problem"))
                }
            },
            Editgroup(egid) => {
                let result = api_client.rt.block_on(api_client.api.get_editgroup(egid.to_string()));
                if let Ok(fatcat_openapi::GetEditgroupResponse::Found(eg)) = result {
                    Ok(ApiModel::Editgroup(eg))
                } else {
                    Err(format_err!("some API problem"))
                }
            },
            Editor(fcid) => {
                let result = api_client.rt.block_on(api_client.api.get_editor(fcid.to_string()));
                if let Ok(fatcat_openapi::GetEditorResponse::Found(eg)) = result {
                    Ok(ApiModel::Editor(eg))
                } else {
                    Err(format_err!("some API problem"))
                }
            },
            Changelog(index) => {
                let result = api_client.rt.block_on(api_client.api.get_changelog_entry(*index));
                if let Ok(fatcat_openapi::GetChangelogEntryResponse::FoundChangelogEntry(model)) = result {
                    Ok(ApiModel::Changelog(model))
                } else {
                    Err(format_err!("some API problem"))
                }
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

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case", about = "CLI interface to Fatcat API" )]
struct Opt {

    #[structopt(long, env = "FATCAT_API_HOST", default_value = "https://api.fatcat.wiki")]
    api_host: String,

    #[structopt(long, env = "FATCAT_API_AUTH_TOKEN", hide_env_values = true)]
    api_token: Option<String>,

    //#[structopt(long, env = "FATCAT_SEARCH_HOST", default_value = "https://search.fatcat.wiki")]
    //search_host: String,

    /// Pass many times for more log output
    ///
    /// By default, it'll only report errors. Passing `-v` one time also prints
    /// warnings, `-vv` enables info logging, `-vvv` debug, and `-vvvv` trace.
    #[structopt(long, short = "v", parse(from_occurrences))]
    verbose: i8,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    Status,
    Get {
        specifier: Specifier,

        #[structopt(long)]
        toml: bool,
    },
    //Update {},
    //Create {},
    //Delete {},
    //Edit
    //Editgroup
    //Download
    //Search
}

fn main() -> Result<(), ExitFailure> {
    let opt = Opt::from_args();

    let log_level = match opt.verbose {
        std::i8::MIN..=-1 => "none",
        0 => "error",
        1 => "warn",
        2 => "info",
        3 => "debug",
        4..=std::i8::MAX => "trace",
    };
    env_logger::from_env(env_logger::Env::default().default_filter_or(log_level))
        .format_timestamp(None)
        .init();

    debug!("Args parsed, starting up");


    let client = if opt.api_host.starts_with("https://") {
        // Using Simple HTTPS
        fatcat_openapi::client::Client::try_new_https(&opt.api_host).expect("Failed to create HTTPS client")
    } else if opt.api_host.starts_with("http://") {
        // Using HTTP
        fatcat_openapi::client::Client::try_new_http(&opt.api_host).expect("Failed to create HTTP client")
    } else {
        panic!("unsupported API Host prefix");
    };

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

    if atty::is(Stream::Stdout) {
        //println!("I'm a terminal");
    } else {
        //println!("I'm not");
    }

    let api_client = {

        let client: fatcat_openapi::ContextWrapper<Client<ResponseFuture>, swagger::make_context_ty!(
            ContextBuilder,
            EmptyContext,
            Option<AuthData>,
            XSpanIdString
        )> = client.with_context(context);
        let rt: Runtime = Runtime::new().unwrap();
        
        FatcatApiClient {
            api: client,
            rt,
        }
    };

    match opt.cmd {
        Command::Get {toml, specifier} => {
            let result = specifier.get_from_api(api_client)?;
            if toml {
                println!("{}", result.to_toml_string()?)
            } else {
                println!("{}", result.to_json_string()?)
            }
        },
        Command::Status => {
            println!("All good!");
        },
    }
    Ok(())
}

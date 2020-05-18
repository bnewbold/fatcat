
use toml;
use serde_json;
use serde;
use fatcat_openapi::models;
use failure::Error;

/*
 * Goal is to have traits around API entities. Things we'll want to do on concrete entities:
 *
 * - print, or pretty-print, as JSON or TOML
 * - get fcid (or, self-specifier)
 * - update (mutate or return copy) fields based on parameters
 * - update self to remote API
 *
 * Methods that might return trait objects:
 *
 * - get by specifier
 */

pub trait ApiEntityModel: ApiModelSer+ApiModelIdent {
}

impl ApiEntityModel for models::ReleaseEntity {}
impl ApiEntityModel for models::ContainerEntity {}
impl ApiEntityModel for models::CreatorEntity {}
impl ApiEntityModel for models::WorkEntity {}
impl ApiEntityModel for models::FileEntity {}
impl ApiEntityModel for models::FilesetEntity {}
impl ApiEntityModel for models::WebcaptureEntity {}
impl ApiEntityModel for models::Editor{}
impl ApiEntityModel for models::Editgroup{}

pub trait ApiModelSer {
    fn to_json_string(&self) -> Result<String, Error>;
    fn to_toml_string(&self) -> Result<String, Error>;
}

impl<T: serde::Serialize> ApiModelSer for T {

    fn to_json_string(&self) -> Result<String, Error> {
        Ok(serde_json::to_string(self)?)
    }

    fn to_toml_string(&self) -> Result<String, Error> {
        Ok(toml::Value::try_from(self)?.to_string())
    }
}

pub trait ApiModelIdent {
    fn fcid(&self) -> String;
}

macro_rules! generic_entity_fcid {
    () => {
        fn fcid(&self) -> String {
            if let Some(fcid) = &self.ident { fcid.to_string() } else { panic!("expected full entity") }
        }
    }
}

impl ApiModelIdent for models::ReleaseEntity { generic_entity_fcid!(); }
impl ApiModelIdent for models::ContainerEntity { generic_entity_fcid!(); }
impl ApiModelIdent for models::CreatorEntity { generic_entity_fcid!(); }
impl ApiModelIdent for models::WorkEntity { generic_entity_fcid!(); }
impl ApiModelIdent for models::FileEntity { generic_entity_fcid!(); }
impl ApiModelIdent for models::FilesetEntity { generic_entity_fcid!(); }
impl ApiModelIdent for models::WebcaptureEntity { generic_entity_fcid!(); }


impl ApiModelIdent for models::Editgroup {
    fn fcid(&self) -> String {
        if let Some(fcid) = &self.editgroup_id { fcid.to_string() } else { panic!("expected full entity") }
    }
}

impl ApiModelIdent for models::Editor {
    fn fcid(&self) -> String {
        if let Some(fcid) = &self.editor_id { fcid.to_string() } else { panic!("expected full entity") }
    }
}

#![allow(non_snake_case)]
#![allow(unused_macros)]
use extism_pdk::*;

#[allow(unused)]
fn panic_if_key_missing() -> ! {
    panic!("missing key");
}

pub(crate) mod internal {
    pub(crate) fn return_error(e: extism_pdk::Error) -> i32 {
        let err = format!("{:?}", e);
        let mem = extism_pdk::Memory::from_bytes(&err).unwrap();
        unsafe {
            extism_pdk::extism::error_set(mem.offset());
        }
        -1
    }
}

#[allow(unused)]
macro_rules! try_input {
    () => {{
        let x = extism_pdk::input();
        match x {
            Ok(x) => x,
            Err(e) => return internal::return_error(e),
        }
    }};
}

#[allow(unused)]
macro_rules! try_input_json {
    () => {{
        let x = extism_pdk::input();
        match x {
            Ok(extism_pdk::Json(x)) => x,
            Err(e) => return internal::return_error(e),
        }
    }};
}

use base64_serde::base64_serde_type;

base64_serde_type!(Base64Standard, base64::engine::general_purpose::STANDARD);

mod exports {
    use super::*;

    #[unsafe(no_mangle)]
    pub extern "C" fn call() -> i32 {
        let ret =
            crate::call(try_input_json!()).and_then(|x| extism_pdk::output(extism_pdk::Json(x)));

        match ret {
            Ok(()) => 0,
            Err(e) => internal::return_error(e),
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn describe() -> i32 {
        let ret = crate::describe().and_then(|x| extism_pdk::output(extism_pdk::Json(x)));

        match ret {
            Ok(()) => 0,
            Err(e) => internal::return_error(e),
        }
    }
}

pub mod types {
    use super::*;

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct BlobResourceContents {
        /// A base64-encoded string representing the binary data of the item.
        #[serde(rename = "blob")]
        pub blob: String,

        /// The MIME type of this resource, if known.
        #[serde(rename = "mimeType")]
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        pub mime_type: Option<String>,

        /// The URI of this resource.
        #[serde(rename = "uri")]
        pub uri: String,
    }

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct CallToolRequest {
        #[serde(rename = "method")]
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        pub method: Option<String>,

        #[serde(rename = "params")]
        pub params: types::Params,
    }

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct CallToolResult {
        #[serde(rename = "content")]
        pub content: Vec<types::Content>,

        /// Whether the tool call ended in an error.
        ///
        /// If not set, this is assumed to be false (the call was successful).
        #[serde(rename = "isError")]
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        pub is_error: Option<bool>,
    }

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct Content {
        #[serde(rename = "annotations")]
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        pub annotations: Option<types::TextAnnotation>,

        /// The base64-encoded image data.
        #[serde(rename = "data")]
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        pub data: Option<String>,

        /// The MIME type of the image. Different providers may support different image types.
        #[serde(rename = "mimeType")]
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        pub mime_type: Option<String>,

        /// The text content of the message.
        #[serde(rename = "text")]
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        pub text: Option<String>,

        #[serde(rename = "type")]
        pub r#type: types::ContentType,
    }

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub enum ContentType {
        #[default]
        #[serde(rename = "text")]
        Text,
        #[serde(rename = "image")]
        Image,
        #[serde(rename = "resource")]
        Resource,
    }

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct ListToolsResult {
        /// The list of ToolDescription objects provided by this servlet.
        #[serde(rename = "tools")]
        pub tools: Vec<types::ToolDescription>,
    }

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct Params {
        #[serde(rename = "arguments")]
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        pub arguments: Option<serde_json::Map<String, serde_json::Value>>,

        #[serde(rename = "name")]
        pub name: String,
    }

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub enum Role {
        #[default]
        #[serde(rename = "assistant")]
        Assistant,
        #[serde(rename = "user")]
        User,
    }

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct TextAnnotation {
        /// Describes who the intended customer of this object or data is.
        ///
        /// It can include multiple entries to indicate content useful for multiple audiences (e.g., `["user", "assistant"]`).
        #[serde(rename = "audience")]
        pub audience: Vec<types::Role>,

        /// Describes how important this data is for operating the server.
        ///
        /// A value of 1 means "most important," and indicates that the data is
        /// effectively required, while 0 means "least important," and indicates that
        /// the data is entirely optional.
        #[serde(rename = "priority")]
        pub priority: f32,
    }

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct TextResourceContents {
        /// The MIME type of this resource, if known.
        #[serde(rename = "mimeType")]
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        pub mime_type: Option<String>,

        /// The text of the item. This must only be set if the item can actually be represented as text (not binary data).
        #[serde(rename = "text")]
        pub text: String,

        /// The URI of this resource.
        #[serde(rename = "uri")]
        pub uri: String,
    }

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct ToolDescription {
        /// A description of the tool
        #[serde(rename = "description")]
        pub description: String,

        /// The JSON schema describing the argument input
        #[serde(rename = "inputSchema")]
        pub input_schema: serde_json::Map<String, serde_json::Value>,

        /// The name of the tool. It should match the plugin / binding name.
        #[serde(rename = "name")]
        pub name: String,
    }
}

mod raw_imports {
    use super::*;
    #[host_fn]
    extern "ExtismHost" {}
}

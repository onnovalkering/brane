use anyhow::Result;
use openapiv3::Schema;
use openapiv3::{
    Callback, Components, Example, Header, Link, Parameter, PathItem, ReferenceOr, RequestBody, Response,
    SecurityScheme,
};

///
///
///
pub fn resolve_path_item(item: &ReferenceOr<PathItem>) -> Result<PathItem> {
    match item {
        ReferenceOr::Item(item) => Ok(item.clone()),
        ReferenceOr::Reference { reference } => Err(anyhow!("Couldn't resolve callback reference: '{}'.", reference)),
    }
}

///
///
///
pub fn resolve_security_scheme(
    item: &ReferenceOr<SecurityScheme>,
    components: &Option<Components>,
) -> Result<SecurityScheme> {
    match item {
        ReferenceOr::Item(item) => Ok(item.clone()),
        ReferenceOr::Reference { reference } => {
            if let Some(components) = components {
                if let Some(reference) = reference.strip_prefix("#/components/") {
                    if let Some((_, name)) = reference.split_once('/') {
                        if let Some(schema) = components.security_schemes.get(name) {
                            if let ReferenceOr::Item(item) = schema {
                                return Ok(item.clone());
                            }
                        }
                    }
                }
            };

            Err(anyhow!("Couldn't resolve security scheme reference: '{}'.", reference))
        }
    }
}

///
///
///
pub fn resolve_response(
    item: &ReferenceOr<Response>,
    components: &Option<Components>,
) -> Result<Response> {
    match item {
        ReferenceOr::Item(item) => Ok(item.clone()),
        ReferenceOr::Reference { reference } => {
            if let Some(components) = components {
                if let Some(reference) = reference.strip_prefix("#/components/") {
                    if let Some((_, name)) = reference.split_once('/') {
                        if let Some(schema) = components.responses.get(name) {
                            if let ReferenceOr::Item(item) = schema {
                                return Ok(item.clone());
                            }
                        }
                    }
                }
            };

            Err(anyhow!("Couldn't resolve response reference: '{}'.", reference))
        }
    }
}

///
///
///
pub fn resolve_parameter(
    item: &ReferenceOr<Parameter>,
    components: &Option<Components>,
) -> Result<Parameter> {
    match item {
        ReferenceOr::Item(item) => Ok(item.clone()),
        ReferenceOr::Reference { reference } => {
            if let Some(components) = components {
                if let Some(reference) = reference.strip_prefix("#/components/") {
                    if let Some((_, name)) = reference.split_once('/') {
                        if let Some(schema) = components.parameters.get(name) {
                            if let ReferenceOr::Item(item) = schema {
                                return Ok(item.clone());
                            }
                        }
                    }
                }
            };

            Err(anyhow!("Couldn't resolve parameter reference: '{}'.", reference))
        }
    }
}

///
///
///
pub fn resolve_example(
    item: &ReferenceOr<Example>,
    components: &Option<Components>,
) -> Result<Example> {
    match item {
        ReferenceOr::Item(item) => Ok(item.clone()),
        ReferenceOr::Reference { reference } => {
            if let Some(components) = components {
                if let Some(reference) = reference.strip_prefix("#/components/") {
                    if let Some((_, name)) = reference.split_once('/') {
                        if let Some(schema) = components.examples.get(name) {
                            if let ReferenceOr::Item(item) = schema {
                                return Ok(item.clone());
                            }
                        }
                    }
                }
            };

            Err(anyhow!("Couldn't resolve example reference: '{}'.", reference))
        }
    }
}

///
///
///
pub fn resolve_request_body(
    item: &ReferenceOr<RequestBody>,
    components: &Option<Components>,
) -> Result<RequestBody> {
    match item {
        ReferenceOr::Item(item) => Ok(item.clone()),
        ReferenceOr::Reference { reference } => {
            if let Some(components) = components {
                if let Some(reference) = reference.strip_prefix("#/components/") {
                    if let Some((_, name)) = reference.split_once('/') {
                        if let Some(schema) = components.request_bodies.get(name) {
                            if let ReferenceOr::Item(item) = schema {
                                return Ok(item.clone());
                            }
                        }
                    }
                }
            };

            Err(anyhow!("Couldn't resolve request body reference: '{}'.", reference))
        }
    }
}

///
///
///
pub fn resolve_headers(
    item: &ReferenceOr<Header>,
    components: &Option<Components>,
) -> Result<Header> {
    match item {
        ReferenceOr::Item(item) => Ok(item.clone()),
        ReferenceOr::Reference { reference } => {
            if let Some(components) = components {
                if let Some(reference) = reference.strip_prefix("#/components/") {
                    if let Some((_, name)) = reference.split_once('/') {
                        if let Some(schema) = components.headers.get(name) {
                            if let ReferenceOr::Item(item) = schema {
                                return Ok(item.clone());
                            }
                        }
                    }
                }
            };

            Err(anyhow!("Couldn't resolve header reference: '{}'.", reference))
        }
    }
}

///
///
///
pub fn resolve_schema(
    item: &ReferenceOr<Schema>,
    components: &Option<Components>,
) -> Result<(Option<String>, Schema)> {
    match item {
        ReferenceOr::Item(item) => Ok((None, item.clone())),
        ReferenceOr::Reference { reference } => {
            if let Some(components) = components {
                if let Some(reference) = reference.strip_prefix("#/components/") {
                    if let Some((_, name)) = reference.split_once('/') {
                        if let Some(schema) = components.schemas.get(name) {
                            if let ReferenceOr::Item(item) = schema {
                                return Ok((Some(name.to_string()), item.clone()));
                            }
                        }
                    }
                }
            };

            Err(anyhow!("Couldn't resolve schema reference: '{}'.", reference))
        }
    }
}

///
///
///
pub fn resolve_links(
    item: &ReferenceOr<Link>,
    components: &Option<Components>,
) -> Result<Link> {
    match item {
        ReferenceOr::Item(item) => Ok(item.clone()),
        ReferenceOr::Reference { reference } => {
            if let Some(components) = components {
                if let Some(reference) = reference.strip_prefix("#/components/") {
                    if let Some((_, name)) = reference.split_once('/') {
                        if let Some(schema) = components.links.get(name) {
                            if let ReferenceOr::Item(item) = schema {
                                return Ok(item.clone());
                            }
                        }
                    }
                }
            };

            Err(anyhow!("Couldn't resolve link reference: '{}'.", reference))
        }
    }
}

///
///
///
pub fn resolve_callback(
    item: &ReferenceOr<Callback>,
    components: &Option<Components>,
) -> Result<Callback> {
    match item {
        ReferenceOr::Item(item) => Ok(item.clone()),
        ReferenceOr::Reference { reference } => {
            if let Some(components) = components {
                if let Some(reference) = reference.strip_prefix("#/components/") {
                    if let Some((_, name)) = reference.split_once('/') {
                        if let Some(schema) = components.callbacks.get(name) {
                            if let ReferenceOr::Item(item) = schema {
                                return Ok(item.clone());
                            }
                        }
                    }
                }
            };

            Err(anyhow!("Couldn't resolve callback reference: '{}'.", reference))
        }
    }
}

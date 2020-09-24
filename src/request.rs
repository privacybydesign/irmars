use itertools::Itertools;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::cmp::Ordering;
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum Error {
    NotAnAttributeTypeIdentifier,
}

/// An IRMA AttributeType identifies an attribute
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct AttributeType {
    pub scheme: String,
    pub issuer: String,
    pub credential: String,
    pub attribute: String,
}

/// An instance of an IRMA attribute, a type and optionally a value
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd)]
pub struct Attribute {
    #[serde(rename = "type")]
    pub atype: AttributeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

impl Ord for Attribute {
    fn cmp(&self, other: &Self) -> Ordering {
        self.atype.cmp(&other.atype)
    }
}

impl Attribute {
    pub fn new(atype: String, value: Option<String>) -> Result<Self, Error> {
        match atype
            .split('.')
            .collect_tuple()
            .map(|(scheme, issuer, credential, attribute)| AttributeType {
                scheme: scheme.to_string(),
                issuer: issuer.to_string(),
                credential: credential.to_string(),
                attribute: attribute.to_string(),
            }) {
            None => Err(Error::NotAnAttributeTypeIdentifier),
            Some(attr_type) => Ok(Attribute {
                atype: attr_type,
                value: value,
            }),
        }
    }
}

/// An AttributeRequest asks for an instance of an attribute type,
/// possibly requiring it to have a specified value, in a session request.
#[derive(Serialize, Deserialize, Eq, PartialOrd, PartialEq, Ord)]
pub struct AttributeRequest {
    pub attribute: Attribute,
    pub not_null: bool,
}

/// A conjunction of attribute requests, only satisfied
/// when all of its containing attribute requests are satisfied.
#[derive(Serialize, Deserialize)]
pub struct AttributeCon(pub Vec<AttributeRequest>);

/// A disjunction of conjunction of attribute requests, only satisfied
/// when at least one of its containing attribute request conjunctions is satisfied.
#[derive(Serialize, Deserialize)]
pub struct AttributeDisCon(pub Vec<AttributeCon>);

/// AttributeConDisCon is only satisfied if all of the containing AttributeDisCon are satisfied.
#[derive(Serialize, Deserialize)]
pub struct AttributeConDisCon(pub Vec<AttributeDisCon>);

/// A DisclosureRequest is a request to disclose certain attributes.
#[derive(Deserialize)]
pub struct DisclosureRequest {
    pub disclose: AttributeConDisCon,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<BTreeMap<usize, BTreeMap<String, String>>>,
}

impl Serialize for DisclosureRequest {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let len = 2 + if self.labels.is_some() { 1 } else { 0 };
        let mut dr = serializer.serialize_struct("DisclosureRequest", len)?;
        dr.serialize_field("@context", "https://irma.app/ld/request/disclosure/v2")?;
        dr.serialize_field("disclose", &self.disclose)?;

        if self.labels.is_some() {
            dr.serialize_field("labels", &self.labels)?;
        }

        dr.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort() -> Result<(), Error> {
        let attr1 = Attribute::new(String::from("pbdf.pbdf.email.email"), None)?;
        let attr2 = Attribute::new(String::from("pbdf.pbdf.email.domain"), None)?;
        let attr3 = Attribute::new(String::from("pbdf.pbdf.fmail.email"), None)?;
        let attr4 = Attribute::new(String::from("1.2.3.4.5"), None);

        assert_eq!(attr4.is_err(), true);
        assert_eq!(Ordering::Greater, attr1.cmp(&attr2));
        assert_eq!(Ordering::Equal, attr1.cmp(&attr1));
        assert_eq!(Ordering::Less, attr2.cmp(&attr3));

        Ok(())
    }
}

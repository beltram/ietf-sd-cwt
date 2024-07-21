use std::collections::VecDeque;
use std::str::FromStr;

use itertools::Itertools;
use serde::{Deserialize, Deserializer};
use serde_yaml::Value as YamlValue;

use crate::error::SdCwtError;

#[derive(Debug, Clone)]
pub struct InputClaims {
    pub raw: YamlValue,
    pub disclosable_paths: Vec<String>,
}

impl FromStr for InputClaims {
    type Err = SdCwtError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(serde_yaml::from_str::<Self>(s)?)
    }
}

impl<'de> Deserialize<'de> for InputClaims {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut value = YamlValue::deserialize(deserializer)?;
        let mut path = Vec::new();
        let mut tagged_paths = Vec::new();
        collect_tagged_keys(&mut value, &mut path, &mut tagged_paths).map_err(|e| {
            serde::de::Error::custom(format!("Error parsing YAML at path {:?}: {:?}", path, e))
        })?;

        Ok(Self {
            raw: value,
            disclosable_paths: tagged_paths,
        })
    }
}

fn collect_tagged_keys(
    node: &mut YamlValue,
    path: &mut Vec<String>,
    paths: &mut Vec<String>,
) -> Result<(), SdCwtError> {
    let sd_tag = serde_yaml::value::Tag::new("!sd");
    match node {
        YamlValue::Mapping(obj) => {
            for (key, value) in obj {
                match &key {
                    YamlValue::Tagged(tag) => {
                        let tag = tag.as_ref();
                        if tag.tag == sd_tag {
                            let new_val = tag
                                .value
                                .as_str()
                                .ok_or_else(|| SdCwtError::InvalidYamlInput(tag.tag.clone()))?;
                            let full_path = build_full_path(path, new_val);
                            paths.push(full_path);
                        }
                    }
                    YamlValue::String(key) => {
                        path.push(key.to_string());
                        collect_tagged_keys(value, path, paths)?;
                        path.pop();
                    }
                    _ => {}
                }
            }
        }
        YamlValue::Sequence(seq) => {
            for (index, value) in seq.iter_mut().enumerate() {
                path.push(index.to_string());
                collect_tagged_keys(value, path, paths)?;
                // Ugly hack to remove tag from sequence
                if let YamlValue::Tagged(tag) = &value {
                    let tag = tag.as_ref();
                    if tag.tag == sd_tag {
                        let new_val = tag
                            .value
                            .as_str()
                            .ok_or_else(|| SdCwtError::InvalidYamlInput(tag.tag.clone()))?;
                        *value = YamlValue::String(new_val.to_string());
                    }
                }
                path.pop();
            }
        }
        YamlValue::Tagged(tag) => {
            let tag = tag.as_ref();
            if tag.tag == sd_tag {
                let mut full_path = String::new();
                for (index, path_fragment) in path.iter().enumerate() {
                    if index == 0 {
                        full_path = format!("/{}", path_fragment);
                    } else {
                        full_path = format!("{}/{}", full_path, path_fragment)
                    }
                }
                paths.push(full_path);
            }
        }
        _ => {}
    }

    Ok(())
}

fn build_full_path(path: &Vec<String>, additional_segment: &str) -> String {
    let full_path = path.iter().join("/");
    format!("{}/{}", full_path, additional_segment)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse() {
        let yaml = r#"
            sub: user_42
            !sd given_name: John
        "#;

        let input = yaml.parse::<InputClaims>().unwrap();

        println!("{:?}", input.raw);
        println!("{:?}", input.disclosable_paths);
        
        /*assert_eq!(
            json,
            serde_json::json!({
                "sub": "user_42",
                "given_name": "John",
            })
        );

        assert_eq!(tagged_paths, vec!["/given_name",]);*/
    }

    /*#[test]
    fn test_parse_yaml1() {
        let yaml_str = r#"
            sub: user_42
            !sd given_name: John
            !sd family_name: Doe
            email: "johndoe@example.com"
            phone_number: "+1-202-555-0101"
            phone_number_verified: true
            address:
                !sd street_address: "123 Main St"
                !sd locality: Anytown
                region: Anystate
                country: US
            birthdate: "1940-01-01"
            updated_at: 1570000000
            !sd nationalities:
                - US
                - DE
            "#;

        let (json, tagged_paths) = crate::parser::parse_yaml(yaml_str).unwrap();
        println!("{:?}", json);
        println!("{:?}", tagged_paths);

        assert_eq!(
            json,
            serde_json::json!({
                "sub": "user_42",
                "given_name": "John",
                "family_name": "Doe",
                "email": "johndoe@example.com",
                "phone_number": "+1-202-555-0101",
                "phone_number_verified": true,
                "address": {
                    "street_address": "123 Main St",
                    "locality": "Anytown",
                    "region": "Anystate",
                    "country": "US"
                },
                "birthdate": "1940-01-01",
                "updated_at": 1570000000,
                "nationalities": [
                    "US",
                    "DE"
                ]
            })
        );

        assert_eq!(
            tagged_paths,
            vec![
                "/given_name",
                "/family_name",
                "/address/street_address",
                "/address/locality",
                "/nationalities",
            ]
        );
    }

    #[test]
    fn test_parse_yaml2() {
        let yaml_str = r#"
            sub: user_42
            !sd given_name: John
            !sd family_name: Doe
            email: "johndoe@example.com"
            phone_number: "+1-202-555-0101"
            phone_number_verified: true
            !sd address:
                street_address: "123 Main St"
                locality: Anytown
                region: Anystate
                country: US
            birthdate: "1940-01-01"
            updated_at: 1570000000
            nationalities:
                - !sd US
                - !sd DE
                - PL
            "#;

        let (json, tagged_paths) = crate::parser::parse_yaml(yaml_str).unwrap();
        println!("{:?}", json);
        println!("{:?}", tagged_paths);

        assert_eq!(
            json,
            serde_json::json!({
                "sub": "user_42",
                "given_name": "John",
                "family_name": "Doe",
                "email": "johndoe@example.com",
                "phone_number": "+1-202-555-0101",
                "phone_number_verified": true,
                "address": {
                    "street_address": "123 Main St",
                    "locality": "Anytown",
                    "region": "Anystate",
                    "country": "US"
                },
                "birthdate": "1940-01-01",
                "updated_at": 1570000000,
                "nationalities": [
                    "US",
                    "DE",
                    "PL"
                ]
            })
        );

        assert_eq!(
            tagged_paths,
            vec![
                "/given_name",
                "/family_name",
                "/address",
                "/nationalities/0",
                "/nationalities/1",
            ]
        );
    }*/
}

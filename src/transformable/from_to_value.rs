use super::TransformableList;
use crate::utils::result::{AppError, AppResult};
use serde_yaml::{Mapping, Sequence, Value};

static OPERATIONS_KEY: &str = "_transform";

fn value_convert(val: &evalexpr::Value) -> AppResult<serde_yaml::Value> {
    match val {
        evalexpr::Value::String(s) => return Ok(Value::String(s.clone())),
        evalexpr::Value::Float(n) => {
            let n_int: Option<i64> = {
                let is_int = n.fract() == 0.0;
                let is_int = is_int && n < &(i64::MAX as f64);
                let is_int = is_int && n > &(i64::MIN as f64);
                match is_int {
                    true => Some(n.clone() as i64),
                    false => None,
                }
            };

            match n_int {
                Some(i_int) => return Ok(Value::Number(serde_yaml::Number::from(i_int))),
                None => return Ok(Value::Number(serde_yaml::Number::from(n.clone()))),
            }
        }
        evalexpr::Value::Int(n) => return Ok(Value::Number(serde_yaml::Number::from(n.clone()))),
        evalexpr::Value::Boolean(b) => return Ok(Value::Bool(b.clone())),
        evalexpr::Value::Empty => return Ok(Value::Null),
        _ => Err(AppError::ApplyFormula(format!(
            "Can't convert {val:?} to yml",
        )))?,
    }
}

impl TryInto<Value> for TransformableList {
    type Error = AppError;

    fn try_into(self) -> Result<Value, Self::Error> {
        if self.is_empty() {
            return Ok(Value::Null);
        }

        let (first_key, first_value) = self.iter().next().unwrap();

        if first_key == "" {
            return value_convert(first_value);
        }

        let first_part = first_key
            .split('.')
            .next()
            .ok_or_else(|| AppError::ApplyFormula(format!("No key segment found")))?;

        let first_part_as_number = first_part.parse::<usize>();
        let first_part_as_string = first_part.parse::<String>();

        let mut yml = match (first_part_as_number, first_part_as_string) {
            (Ok(_), _) => Value::Sequence(Sequence::new()),
            (_, Ok(_)) => Value::Mapping(Mapping::new()),
            _ => Err(AppError::ApplyFormula(format!(
                "{first_part:?} is not a valid key",
            )))?,
        };

        for (key, value) in &*self {
            let mut current = &mut yml;
            let mut parts = key.split('.').peekable();
            let value = value_convert(value)?;

            while let Some(part) = parts.next() {
                enum NextPart {
                    Number(usize),
                    String(String),
                    None,
                }
                impl NextPart {
                    fn try_new(next_part: Option<&str>) -> AppResult<Self> {
                        match next_part {
                            Some(part) => match (part.parse::<usize>(), part.parse::<String>()) {
                                (Ok(i), _) => Ok(NextPart::Number(i)),
                                (_, Ok(s)) => Ok(NextPart::String(s)),
                                _ => Err(AppError::ApplyFormula(format!(
                                    "{next_part:?} is not a valid key",
                                )))?,
                            },
                            None => Ok(NextPart::None),
                        }
                    }

                    fn to_next_container_or_value(&self, val: &Value) -> Value {
                        match self {
                            NextPart::Number(_) => Value::Sequence(Sequence::new()),
                            NextPart::String(_) => Value::Mapping(Mapping::new()),
                            NextPart::None => val.clone(),
                        }
                    }
                }

                let next_part = NextPart::try_new(parts.peek().map(|x| *x))?;

                match current {
                    Value::Sequence(seq) => {
                        let index = part.parse::<usize>().map_err(|_| {
                            AppError::ApplyFormula(format!("Expected a number, got {part:?}"))
                        })?;
                        if index >= seq.len() {
                            seq.resize_with(index + 1, || Value::Null);
                        }
                        let entry = seq.get_mut(index).ok_or_else(|| {
                            AppError::ApplyFormula(format!("Nothing at {part:?}"))
                        })?;
                        match entry {
                            Value::Null => {
                                *entry = next_part.to_next_container_or_value(&value);
                            }
                            _ => {}
                        }
                        current = seq.get_mut(index).ok_or_else(|| {
                            AppError::ApplyFormula(format!("Can't get mutable at {part:?}"))
                        })?;
                    }
                    Value::Mapping(map) => {
                        let key = part.parse::<String>().map_err(|_| {
                            AppError::ApplyFormula(format!("Expected a string, got {part:?}"))
                        })?;
                        let entry = map.get(&key);
                        if let None = entry {
                            map.insert(
                                Value::String(key.clone()),
                                next_part.to_next_container_or_value(&value),
                            );
                        }
                        current = map.get_mut(key).unwrap();
                    }
                    _ => Err(AppError::ApplyFormula(format!(
                        "Can only insert something in mapping or sequence, got {current:?}"
                    )))?,
                }
            }
        }

        Ok(yml)
    }
}

impl TryFrom<Value> for TransformableList {
    type Error = AppError;

    fn try_from(value: Value) -> AppResult<Self> {
        fn visit(val: &Value, parent_key: &str) -> AppResult<TransformableList> {
            let mut transformable_list = TransformableList::new(None);
            match val {
                Value::String(s) => {
                    transformable_list
                        .set(format!("{parent_key}"), evalexpr::Value::String(s.clone()));
                }
                Value::Bool(s) => {
                    transformable_list
                        .set(format!("{parent_key}"), evalexpr::Value::Boolean(s.clone()));
                }
                Value::Number(s) => {
                    transformable_list.set(
                        format!("{parent_key}"),
                        evalexpr::Value::Float(s.as_f64().ok_or_else(|| {
                            AppError::ApplyFormula(format!("Your numbers must be f64"))
                        })?),
                    );
                }
                Value::Null => {
                    transformable_list.set(format!("{parent_key}"), evalexpr::Value::Empty);
                }
                Value::Mapping(m) => {
                    for (key, v) in m {
                        let k = match key {
                            Value::String(str) => Ok(str),
                            _ => {
                                let key = key.clone();
                                Err(AppError::ApplyFormula(format!(
                                    "Mapping keys is not a string: {key:?}",
                                )))
                            }
                        }?;

                        let new_key = match parent_key {
                            "" => format!("{k}"),
                            _ => format!("{parent_key}.{k}"),
                        };
                        let child_flat_yml = visit(v, &new_key)?;
                        transformable_list
                            .extend(child_flat_yml.iter().map(|(k, v)| (k.clone(), v.clone())));
                    }
                }
                Value::Sequence(seq) => {
                    for (i, v) in seq.iter().enumerate() {
                        let new_key = match parent_key {
                            "" => format!("{i}"),
                            _ => format!("{parent_key}.{i}"),
                        };
                        let child_flat_yml = visit(v, &new_key)?;
                        transformable_list
                            .extend(child_flat_yml.iter().map(|(k, v)| (k.clone(), v.clone())));
                    }
                }
                Value::Tagged(t) => {
                    let tag = &t.tag;
                    Err(AppError::ApplyFormula(format!("Unhandle tag: {tag:?}",)))?;
                }
            }

            Ok(transformable_list)
        }

        let mut value = value.clone();

        let operations = match &mut value {
            Value::Mapping(m) => {
                let entry = m.remove(OPERATIONS_KEY);
                match entry {
                    Some(Value::String(s)) => Some(vec![s.clone()]),
                    Some(Value::Sequence(s)) => {
                        let transformations = s.iter().try_fold(vec![], |mut acc, v| match v {
                            Value::String(s) => {
                                acc.push(s.clone());
                                Ok(acc)
                            }
                            Value::Sequence(seq) => {
                                let seq: Vec<String> = serde_yaml::from_value(Value::Sequence(
                                    seq.clone(),
                                ))
                                .map_err(|_| {
                                    AppError::ApplyFormula(format!(
                                        "_transform should be a list of string"
                                    ))
                                })?;
                                acc.extend(seq);
                                Ok(acc)
                            }
                            _ => Err(AppError::ApplyFormula(format!(
                                "_transform should be composed of strings or of lists of string"
                            ))),
                        })?;

                        Some(transformations)
                    }
                    Some(Value::Mapping(map)) => {
                        let mut keys = map
                            .keys()
                            .cloned()
                            .map(|k| match k {
                                Value::String(s) => Ok(s.clone()),
                                _ => Err(AppError::ApplyFormula(format!(
                                    "_transform should be a mapping of string"
                                ))),
                            })
                            .collect::<AppResult<Vec<_>>>()?;
                        keys.sort();

                        let transformations: AppResult<Vec<String>> =
                            keys.iter().try_fold(vec![], |mut acc, k| {
                                let v = match map.get(k) {
                                    Some(Value::String(s)) => Ok(vec![s.clone()]),
                                    Some(Value::Sequence(seq)) => {
                                        let seq: Vec<String> =
                                            serde_yaml::from_value(Value::Sequence(seq.clone()))
                                                .map_err(|_| {
                                                    AppError::ApplyFormula(format!(
                                                        "_transform should be a list of string"
                                                    ))
                                                })?;
                                        Ok(seq)
                                    }
                                    _ => Err(AppError::ApplyFormula(format!(
                                        "_transform should be a mapping of string"
                                    ))),
                                }?;

                                acc.extend(v);
                                Ok(acc)
                            });

                        Some(transformations?)
                    }
                    None => None,
                    _ => {
                        return Err(AppError::ApplyFormula(
                            "_transform should be a string or a list of string".to_string(),
                        ))?
                    }
                }
            }
            _ => None,
        };

        let transformable_list = visit(&value, "")?;

        Ok(Self {
            operations,
            ..transformable_list
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    struct TestStruct<'a> {
        structure: SubStruct<'a>,
        entry: &'a str,
        content: Vec<&'a str>,
        flag: bool,
    }

    #[derive(Debug, Serialize)]
    struct SubStruct<'a> {
        sub_entry: &'a str,
        sub_content: Vec<&'a str>,
        sub_flag: bool,
    }

    #[derive(Debug, Serialize)]
    struct NumberStruct {
        entry_int: u8,
        entry_float: f64,
        _transform: Vec<String>,
    }

    #[test]
    pub fn it_should_flatten_keys_on_try_to() {
        let test_struct = TestStruct {
            structure: SubStruct {
                sub_entry: "I'm a sub entry",
                sub_content: vec![
                    "I'm a sub content 0",
                    "I'm a sub content 1",
                    "I'm a sub content 2",
                ],
                sub_flag: false,
            },
            entry: "I'm an entry",
            content: vec!["I'm a content 0"],
            flag: true,
        };

        let test_yml = serde_yaml::to_value(&test_struct).unwrap();
        let trans_list = TransformableList::try_from(test_yml).unwrap();

        assert_eq!(trans_list.len(), 8);
        assert_eq!(
            trans_list.get("structure.sub_entry").unwrap(),
            &evalexpr::Value::String("I'm a sub entry".to_string())
        );
        assert_eq!(
            trans_list.get("structure.sub_content.0").unwrap(),
            &evalexpr::Value::String("I'm a sub content 0".to_string())
        );
        assert_eq!(
            trans_list.get("structure.sub_content.1").unwrap(),
            &evalexpr::Value::String("I'm a sub content 1".to_string())
        );
        assert_eq!(
            trans_list.get("structure.sub_content.2").unwrap(),
            &evalexpr::Value::String("I'm a sub content 2".to_string())
        );
        assert_eq!(
            trans_list.get("structure.sub_flag").unwrap(),
            &evalexpr::Value::Boolean(false)
        );
        assert_eq!(
            trans_list.get("entry").unwrap(),
            &evalexpr::Value::String("I'm an entry".to_string())
        );
        assert_eq!(
            trans_list.get("content.0").unwrap(),
            &evalexpr::Value::String("I'm a content 0".to_string())
        );
        assert_eq!(
            trans_list.get("flag").unwrap(),
            &evalexpr::Value::Boolean(true)
        );
    }

    #[test]
    fn it_should_unflattend_in_try_from() {
        let mut trans_list = TransformableList::new(None);

        trans_list.set(
            "structure.sub_entry".to_string(),
            evalexpr::Value::String("I'm a sub entry".to_string()),
        );
        trans_list.set(
            "structure.sub_content.0".to_string(),
            evalexpr::Value::String("I'm a sub content 0".to_string()),
        );
        trans_list.set(
            "structure.sub_content.1".to_string(),
            evalexpr::Value::String("I'm a sub content 1".to_string()),
        );
        trans_list.set(
            "structure.sub_content.2".to_string(),
            evalexpr::Value::String("I'm a sub content 2".to_string()),
        );
        trans_list.set(
            "structure.sub_flag".to_string(),
            evalexpr::Value::Boolean(false),
        );
        trans_list.set(
            "entry".to_string(),
            evalexpr::Value::String("I'm an entry".to_string()),
        );
        trans_list.set(
            "content.0".to_string(),
            evalexpr::Value::String("I'm a content 0".to_string()),
        );
        trans_list.set("flag".to_string(), evalexpr::Value::Boolean(true));

        let yml: Value = trans_list.try_into().unwrap();

        let test_struct = TestStruct {
            structure: SubStruct {
                sub_entry: "I'm a sub entry",
                sub_content: vec![
                    "I'm a sub content 0",
                    "I'm a sub content 1",
                    "I'm a sub content 2",
                ],
                sub_flag: false,
            },
            entry: "I'm an entry",
            content: vec!["I'm a content 0"],
            flag: true,
        };

        let test_yml = serde_yaml::to_value(&test_struct).unwrap();

        assert_eq!(yml, test_yml);
    }

    #[test]
    fn it_should_handle_input_int_as_float() {
        let test_struct = NumberStruct {
            entry_int: 3,
            entry_float: 1.0,
            _transform: vec!["ceiled_int = ceil(entry_int / 2)".to_string()],
        };

        let yml = serde_yaml::to_value(&test_struct).unwrap();
        let mut trans_list = TransformableList::try_from(yml).unwrap();

        assert_eq!(
            trans_list.get("entry_int").unwrap(),
            &evalexpr::Value::Float(3.0)
        );
        trans_list.transform().unwrap();
        assert_eq!(
            trans_list.get("ceiled_int").unwrap(),
            &evalexpr::Value::Float(2.0)
        );
    }

    #[test]
    fn it_should_output_int_when_no_decimals() {
        let test_struct = NumberStruct {
            entry_int: 1,
            entry_float: 1.0,
            _transform: vec![
                "entry_int = entry_int + 1.2".to_string(),
                "entry_float = entry_float + 1".to_string(),
            ],
        };

        let yml = serde_yaml::to_value(&test_struct).unwrap();
        let mut trans_list = TransformableList::try_from(yml).unwrap();
        trans_list.transform().unwrap();
        let yml: Value = trans_list.try_into().unwrap();

        let map = match yml {
            Value::Mapping(m) => m,
            _ => panic!("Should be a mapping"),
        };

        match map.get(&Value::String("entry_int".to_string())) {
            Some(Value::Number(n)) => assert_eq!(n.as_f64().unwrap(), 2.2),

            _ => panic!("Should be a number"),
        };

        match map.get(&Value::String("entry_float".to_string())) {
            Some(Value::Number(n)) => assert_eq!(n.as_u64().unwrap(), 2),

            _ => panic!("Should be a number"),
        };
    }
}

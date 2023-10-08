use super::Variables;
use crate::utils::result::{AppError, AppResult};
use evalexpr::eval;
use regex::Regex;
use serde_yaml::{
    value::{Tag, TaggedValue},
    Mapping, Number, Value,
};
use std::str::FromStr;

impl Variables {
    pub fn inject(&self, val: &Value) -> AppResult<Value> {
        match val {
            Value::Tagged(t) => self.on_tag(t),
            Value::Mapping(map) => self.on_mapping(map),
            Value::Sequence(seq) => self.on_sequence(seq),
            Value::String(str) => self.on_string(str),
            x => Ok(x.clone()),
        }
    }

    fn on_tag(&self, val: &TaggedValue) -> AppResult<Value> {
        let tag_label = &val.tag.to_string();
        let tag = self.on_string(tag_label)?;

        let tag = match tag {
            Value::String(tag) => Ok(Tag::new(tag)),
            _ => Err(AppError::ParseYml(format!(
                "{tag_label} can't be used as tag"
            ))),
        }?;

        let value = self.inject(&val.value)?;
        Ok(Value::Tagged(Box::new(TaggedValue { tag, value })))
    }

    fn on_sequence(&self, val: &Vec<Value>) -> AppResult<Value> {
        let mut new_seq: Vec<Value> = vec![];
        for value in val {
            let yml = self.inject(&value)?;
            new_seq.push(yml)
        }
        Ok(Value::Sequence(new_seq))
    }

    fn on_mapping(&self, val: &Mapping) -> AppResult<Value> {
        let mut new_map = Mapping::new();
        for (key, value) in val {
            let new_key = match key {
                Value::String(str) => self.on_string(str)?,
                _ => key.clone(),
            };

            let new_key = match new_key {
                Value::String(str) => str,
                Value::Number(i) => i.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => Err(AppError::ParseYml(format!(
                    "{:?} can't be used as mapping key",
                    key
                )))?,
            };

            let yml = self.inject(&value)?;
            new_map.insert(Value::String(new_key), yml);
        }
        Ok(Value::Mapping(new_map))
    }

    fn on_string(&self, str: &str) -> AppResult<Value> {
        let mut val = Value::String(str.to_string());
        let mut is_replacing = true;

        while is_replacing {
            let folded: AppResult<(Value, bool)> = self.iter().try_fold(
                (val.clone(), false),
                |(acc, is_replacing), (var_key, var_value)| {
                    let variable_identifier = format!("${var_key}");
                    let is_standalone = acc == Value::String(variable_identifier.clone());

                    let new_acc = match (is_standalone, acc.clone(), var_value) {
                        (is_standalone, _, var_value) if is_standalone == true => {
                            Ok(var_value.clone())
                        }
                        (_, Value::String(acc_string), var_value) => {
                            let var_value = match var_value {
                                Value::String(str) => str.to_string(),
                                Value::Number(number) => number.to_string(),
                                Value::Bool(boolean) => boolean.to_string(),
                                _ => "".to_string(),
                            };
                            let regex = Regex::new(&format!(r#"\{variable_identifier}\b"#))
                                .map_err(|e| {
                                    AppError::ParseYml(format!(
                                        "{var_key} can't be used as variable identifier: {}",
                                        e.to_string()
                                    ))
                                })?;
                            let acc_string: String =
                                regex.replace_all(&acc_string, var_value).to_string();
                            let acc_string = self.evaluate_string(&acc_string)?;
                            Ok(acc_string)
                        }
                        (_, acc, _) => self.inject(&acc),
                    }?;

                    if new_acc == acc {
                        return Ok((new_acc, is_replacing));
                    }

                    Ok((new_acc, true))
                },
            );

            let folded = folded?;
            val = folded.0;
            is_replacing = folded.1;
        }

        Ok(val)
    }

    fn evaluate_string(&self, str: &str) -> AppResult<Value> {
        let str = str.to_string();
        fn contains_multibyte(s: &str) -> bool {
            for c in s.chars() {
                if c.len_utf8() > 1 {
                    return true;
                }
            }
            false
        }

        if contains_multibyte(&str) {
            return Ok(Value::String(str));
        }

        let result = match eval(&str) {
            Ok(evaluated) => match evaluated {
                evalexpr::Value::Boolean(b) => Value::Bool(b),
                evalexpr::Value::Int(n) => {
                    let number = Number::from_str(&n.to_string());
                    match number {
                        Ok(number) => Value::Number(number),
                        Err(_) => Value::String(str),
                    }
                }
                evalexpr::Value::Float(n) => {
                    let number = Number::from_str(&n.to_string());
                    match number {
                        Ok(number) => Value::Number(number),
                        Err(_) => Value::String(str),
                    }
                }
                _ => Value::String(str),
            },
            Err(_) => Value::String(str),
        };
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_yml_variables() -> &'static str {
        let yml_part = r#"
            test: 1
            test2: 10.1
            a: Something
            b:
                - toto
                - 3
            c:
                foo: foo_string
                bar: false
            d: null
        "#;
        yml_part
    }

    #[test]
    fn it_should_apply_variables_and_evaluate() {
        let variables: Value = serde_yaml::from_str(get_yml_variables()).unwrap();
        let variables: Variables = variables.try_into().unwrap();

        let yml_part: Value = serde_yaml::from_str(
            r#"
            - $test + $test2
            - I am $a
            - $b
            - $c
        "#,
        )
        .unwrap();
        let yml = variables.inject(&yml_part).unwrap();

        let expected_yml: Value = serde_yaml::from_str(
            r#"
            - 11.1
            - I am Something
            - 
                - toto
                - 3
            - 
                foo: foo_string
                bar: false 
                "#,
        )
        .unwrap();

        assert_eq!(yml, expected_yml);
    }

    #[test]
    fn it_should_inject_null_variables() {
        let variables: Value = serde_yaml::from_str(get_yml_variables()).unwrap();
        let variables: Variables = variables.try_into().unwrap();

        let yml_sequence: Value = serde_yaml::from_str(
            r#"
            - $d
            - I am $d
            - I am not null
        "#,
        )
        .unwrap();
        let yml_sequence = variables.inject(&yml_sequence).unwrap();

        match yml_sequence {
            Value::Sequence(seq) => {
                assert_eq!(seq.len(), 3);
                assert_eq!(seq[0], Value::Null);
                assert_eq!(seq[1], Value::String("I am ".to_string()));
                assert_eq!(seq[2], Value::String("I am not null".to_string()));
            }
            _ => panic!("yml should be a sequence"),
        };

        let yml_mapping: Value = serde_yaml::from_str(
            r#"
            a: $d
            b: I am $d
            c: I am not null
        "#,
        )
        .unwrap();
        let yml_mapping = variables.inject(&yml_mapping).unwrap();

        match yml_mapping {
            Value::Mapping(map) => {
                assert_eq!(map.len(), 3);
                assert_eq!(map["a"], Value::Null);
                assert_eq!(map["b"], Value::String("I am ".to_string()));
                assert_eq!(map["c"], Value::String("I am not null".to_string()));
            }
            _ => panic!("yml should be a mapping"),
        };
    }
}

use std::sync::Arc;

use crate::{adapters, mixins::MixIns, utils::result::AppResult, variables::Variables};
use serde_yaml::{
    value::{Tag, TaggedValue},
    Mapping, Value,
};

pub struct YmlAggregator {
    reader: Arc<dyn adapters::PartReaderPort>,
    pub mixins: MixIns,
}

impl YmlAggregator {
    const INCLUDE_TAG_PREFIX: &'static str = "!inc::";

    pub fn new(reader: Arc<dyn adapters::PartReaderPort>) -> Self {
        YmlAggregator {
            reader,
            mixins: MixIns::new(),
        }
    }

    pub fn load(&mut self, identifier: &str, variables: &Variables) -> AppResult<Value> {
        let yml = self.reader.get_value(identifier)?;
        let (yml, mixins) = parse_yml_part(yml, &variables)?;

        let mixins = mixins
            .iter()
            .map(|(key, values)| {
                let sub_mixins = values
                    .iter()
                    .map(|value| {
                        let mut aggregator = YmlAggregator::new(Arc::clone(&self.reader));
                        let value = aggregator.visit(value, &variables)?;
                        let mut mixins = aggregator.mixins;
                        mixins.add(key.clone(), vec![value]);
                        Ok(mixins)
                    })
                    .collect::<AppResult<Vec<MixIns>>>()?;

                let sub_mixins = sub_mixins.iter().fold(MixIns::new(), |mut acc, mix| {
                    acc.merge(mix);
                    acc
                });

                Ok(sub_mixins)
            })
            .collect::<AppResult<Vec<MixIns>>>()?;

        let mixins = mixins.iter().fold(MixIns::new(), |mut acc, mix| {
            acc.merge(mix);
            acc
        });

        mixins.iter().for_each(|(key, value)| {
            self.mixins.add(key.clone(), value.clone());
        });

        let yml = self.visit(&yml, &variables)?;
        Ok(yml)
    }

    pub fn visit(&mut self, val: &Value, variables: &Variables) -> AppResult<Value> {
        match val {
            Value::Tagged(t) => self.on_tag(t, variables),
            Value::Mapping(map) => self.on_mapping(map, variables),
            Value::Sequence(seq) => self.on_sequence(seq, variables),
            x => Ok(x.clone()),
        }
    }

    fn on_tag(&mut self, val: &TaggedValue, variables: &Variables) -> AppResult<Value> {
        let tag = val.tag.to_string();
        let value = &val.value;

        match tag.starts_with(Self::INCLUDE_TAG_PREFIX) {
            true => {
                let file = tag.trim_start_matches(Self::INCLUDE_TAG_PREFIX);
                let new_variables: Variables = value.clone().try_into()?;
                let mut variables = variables.clone();

                for (key, value) in new_variables.iter() {
                    variables.insert(key.clone(), value.clone());
                }

                let yml = self.load(file, &variables)?;
                let yml = self.visit(&yml, &variables)?;
                Ok(yml)
            }
            false => {
                let yml = self.visit(value, variables)?;
                Ok(Value::Tagged(Box::new(TaggedValue {
                    tag: Tag::new(tag),
                    value: yml,
                })))
            }
        }
    }

    fn on_mapping(&mut self, val: &Mapping, variables: &Variables) -> AppResult<Value> {
        let mut new_map = Mapping::new();
        for (key, value) in val {
            let yml = self.visit(&value, variables)?;
            if let Value::Null = yml {
                continue;
            }
            new_map.insert(key.clone(), yml);
        }
        if new_map.is_empty() {
            return Ok(Value::Null);
        }
        Ok(Value::Mapping(new_map))
    }

    fn on_sequence(&mut self, val: &Vec<Value>, variables: &Variables) -> AppResult<Value> {
        let mut new_seq: Vec<Value> = vec![];
        for value in val {
            let yml = self.visit(&value, variables)?;
            if let Value::Null = yml {
                continue;
            }
            new_seq.push(yml)
        }
        if new_seq.is_empty() {
            return Ok(Value::Null);
        }
        Ok(Value::Sequence(new_seq))
    }
}

fn parse_yml_part(part: Value, variables: &Variables) -> AppResult<(Value, MixIns)> {
    let part = variables.inject(&part)?;
    let mut mixin = MixIns::new();
    let part = mixin.trim(&part)?;

    return Ok((part, mixin));
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_yml_part() -> &'static str {
        let yml_part = r#"
            foo:
                - $test
                - $test2
            bar: !mix
                - $test is $test2
        "#;
        yml_part
    }

    #[test]
    fn it_should_inject_variables_and_get_mixins() {
        let yml_part = serde_yaml::from_str(get_yml_part()).unwrap();
        let mut variables = Variables::new();
        variables.insert("test".to_string(), Value::String("test_value".to_string()));
        variables.insert(
            "test2".to_string(),
            Value::String("test_value2".to_string()),
        );

        let expected_yml: Value = serde_yaml::from_str(
            r#"
            foo:
                - test_value
                - test_value2
            "#,
        )
        .unwrap();

        let expected_mixins: Value = serde_yaml::from_str(
            r#"
                - test_value is test_value2
            "#,
        )
        .unwrap();

        let (yml, mixins) = parse_yml_part(yml_part, &variables).unwrap();

        assert_eq!(yml, expected_yml);
        let bar_mixin = mixins.get("bar").unwrap();
        assert_eq!(bar_mixin, &vec![expected_mixins]);
    }
}
